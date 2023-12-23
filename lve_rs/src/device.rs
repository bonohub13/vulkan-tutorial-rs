use crate::utils as lve_utils;
use anyhow::{bail, Context, Result};
use ash::{extensions::khr as vk_khr, vk};
use raw_window_handle::HasRawDisplayHandle;
use std::{collections::HashSet, ffi::CStr};

pub struct QueryFamilyIndices {
    pub graphics_family: Option<u32>,
    pub present_family: Option<u32>,
}

pub struct Device {
    entry: ash::Entry,
    instance: ash::Instance,
    debug_messenger: crate::DebugUtilsMessenger,
    surface: crate::Surface,
    physical_device: vk::PhysicalDevice,
    device: ash::Device,
    graphics_queue: vk::Queue,
    present_queue: vk::Queue,
    command_pool: vk::CommandPool,
}

impl QueryFamilyIndices {
    pub fn none() -> Self {
        Self {
            graphics_family: None,
            present_family: None,
        }
    }

    pub fn unique_queue_families(&self) -> Result<HashSet<u32>> {
        Ok(HashSet::from([
            self.graphics_family
                .context("Graphics queue family missing")?,
            self.present_family
                .context("Present queue family missing")?,
        ]))
    }

    pub fn is_complete(&self) -> bool {
        self.graphics_family.is_some() && self.present_family.is_some()
    }
}

impl Device {
    const VALIDATION_LAYERS: [*const i8; 1] =
        [
            unsafe { CStr::from_bytes_with_nul_unchecked(b"VK_LAYER_KHRONOS_validation\0") }
                .as_ptr(),
        ];
    const DEVICE_EXTENSIONS: [*const i8; 1] = [vk_khr::Swapchain::name().as_ptr()];

    pub fn new(window: &crate::Window, app_info: &crate::ApplicationInfo) -> Result<Self> {
        let entry = unsafe { ash::Entry::load() }?;
        let instance = Self::create_instance(window, &entry, app_info)?;
        let debug_messenger = if lve_utils::is_debug_build() {
            crate::DebugUtilsMessenger::new(&entry, &instance)?
        } else {
            crate::DebugUtilsMessenger::null(&entry, &instance)
        };
        let surface = window.create_surface(&entry, &instance)?;
        let physical_device = Self::pick_physical_device(&instance, &surface)?;
        let (device, graphics_queue, present_queue) =
            Self::create_device(&instance, &surface, &physical_device)?;
        let command_pool =
            Self::create_command_pool(&instance, &surface, &physical_device, &device)?;

        Ok(Self {
            entry,
            instance,
            debug_messenger,
            surface,
            physical_device,
            device,
            graphics_queue,
            present_queue,
            command_pool,
        })
    }

    pub unsafe fn destroy(&self) {
        self.device.destroy_command_pool(self.command_pool, None);
        self.device.destroy_device(None);
        self.surface.destroy_surface();

        if lve_utils::is_debug_build() {
            self.debug_messenger.destroy_debug_utils_messenger();
        }

        self.instance.destroy_instance(None);
    }

    #[inline]
    pub fn entry(&self) -> &ash::Entry {
        &self.entry
    }

    #[inline]
    pub fn instance(&self) -> &ash::Instance {
        &self.instance
    }

    #[inline]
    pub fn device(&self) -> &ash::Device {
        &self.device
    }

    #[inline]
    pub fn surface(&self) -> &crate::Surface {
        &self.surface
    }

    #[inline]
    pub fn graphics_queue(&self) -> &vk::Queue {
        &self.graphics_queue
    }

    #[inline]
    pub fn present_queue(&self) -> &vk::Queue {
        &self.present_queue
    }

    #[inline]
    pub fn command_pool(&self) -> &vk::CommandPool {
        &self.command_pool
    }

    #[inline]
    pub unsafe fn swap_chain_support(&self) -> Result<crate::SwapChainSupportDetails> {
        self.surface.query_swap_chain_support(&self.physical_device)
    }

    pub fn find_memory_type(
        &self,
        type_filter: u32,
        properties: vk::MemoryPropertyFlags,
    ) -> Result<u32> {
        let mem_properties = unsafe {
            self.instance
                .get_physical_device_memory_properties(self.physical_device)
        };

        for i in 0..mem_properties.memory_type_count {
            if (type_filter & (1 << i)) != 0
                && (mem_properties.memory_types[i as usize].property_flags & properties)
                    == properties
            {
                return Ok(i);
            }
        }

        bail!("Failed to find suitable memory type");
    }

    #[inline]
    pub fn find_physical_queue_families(&self) -> Result<QueryFamilyIndices> {
        Self::find_queue_families(&self.instance, &self.surface, &self.physical_device)
    }

    pub fn find_supported_format(
        &self,
        candidates: &[vk::Format],
        tiling: vk::ImageTiling,
        features: vk::FormatFeatureFlags,
    ) -> Result<vk::Format> {
        let format = candidates
            .iter()
            .filter(|format| {
                let properties = unsafe {
                    self.instance
                        .get_physical_device_format_properties(self.physical_device, **format)
                };

                match tiling {
                    vk::ImageTiling::LINEAR => {
                        (properties.linear_tiling_features & features) == features
                    }
                    vk::ImageTiling::OPTIMAL => {
                        (properties.optimal_tiling_features & features) == features
                    }
                    _ => false,
                }
            })
            .map(|format| *format)
            .next()
            .context("Failed to find supported format")?;

        Ok(format)
    }

    pub fn create_buffer(
        &self,
        size: vk::DeviceSize,
        usage: vk::BufferUsageFlags,
        properties: vk::MemoryPropertyFlags,
    ) -> Result<(vk::Buffer, vk::DeviceMemory)> {
        let buffer = {
            let create_info = vk::BufferCreateInfo::builder()
                .size(size)
                .usage(usage)
                .sharing_mode(vk::SharingMode::EXCLUSIVE);

            unsafe { self.device.create_buffer(&create_info, None) }?
        };
        let buffer_memory = {
            let mem_requirements = unsafe { self.device.get_buffer_memory_requirements(buffer) };
            let allocation_info = vk::MemoryAllocateInfo::builder()
                .allocation_size(mem_requirements.size)
                .memory_type_index(
                    self.find_memory_type(mem_requirements.memory_type_bits, properties)?,
                );

            unsafe { self.device.allocate_memory(&allocation_info, None) }?
        };

        unsafe { self.device.bind_buffer_memory(buffer, buffer_memory, 0) }?;

        Ok((buffer, buffer_memory))
    }

    pub fn begin_single_time_commands(&self) -> Result<vk::CommandBuffer> {
        let command_buffer = {
            let allocate_info = vk::CommandBufferAllocateInfo::builder()
                .level(vk::CommandBufferLevel::PRIMARY)
                .command_pool(self.command_pool)
                .command_buffer_count(1);

            unsafe { self.device.allocate_command_buffers(&allocate_info) }?
                .into_iter()
                .next()
                .context("Failed to allocate single use command buffer")?
        };
        let begin_info = vk::CommandBufferBeginInfo::builder()
            .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);

        unsafe {
            self.device
                .begin_command_buffer(command_buffer, &begin_info)
        }?;

        Ok(command_buffer)
    }

    pub unsafe fn end_single_time_commands(
        &self,
        command_buffer: &vk::CommandBuffer,
    ) -> Result<()> {
        self.device.end_command_buffer(*command_buffer)?;

        let submit_info =
            vk::SubmitInfo::builder().command_buffers(std::slice::from_ref(command_buffer));

        self.device.queue_submit(
            self.graphics_queue,
            std::slice::from_ref(&submit_info),
            vk::Fence::null(),
        )?;
        self.device.queue_wait_idle(self.graphics_queue)?;
        self.device
            .free_command_buffers(self.command_pool, std::slice::from_ref(command_buffer));

        Ok(())
    }

    pub unsafe fn copy_buffer(
        &self,
        src_buffer: &vk::Buffer,
        dst_buffer: &vk::Buffer,
        size: vk::DeviceSize,
    ) -> Result<()> {
        let command_buffer = self.begin_single_time_commands()?;
        let copy_region = vk::BufferCopy::builder()
            .src_offset(0)
            .dst_offset(0)
            .size(size);

        self.device.cmd_copy_buffer(
            command_buffer,
            *src_buffer,
            *dst_buffer,
            std::slice::from_ref(&copy_region),
        );
        self.end_single_time_commands(&command_buffer)?;

        Ok(())
    }

    pub fn copy_buffer_to_image(
        &self,
        buffer: &vk::Buffer,
        image: &vk::Image,
        width: u32,
        height: u32,
        layer_count: u32,
    ) -> Result<()> {
        let command_buffer = self.begin_single_time_commands()?;
        let region = vk::BufferImageCopy::builder()
            .buffer_offset(0)
            .buffer_row_length(0)
            .buffer_image_height(0)
            .image_subresource(
                vk::ImageSubresourceLayers::builder()
                    .aspect_mask(vk::ImageAspectFlags::COLOR)
                    .mip_level(0)
                    .base_array_layer(0)
                    .layer_count(layer_count)
                    .build(),
            )
            .image_offset(vk::Offset3D { x: 0, y: 0, z: 0 })
            .image_extent(vk::Extent3D {
                width,
                height,
                depth: 1,
            });

        unsafe {
            self.device.cmd_copy_buffer_to_image(
                command_buffer,
                *buffer,
                *image,
                vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                std::slice::from_ref(&region),
            );
            self.end_single_time_commands(&command_buffer)
        }?;

        Ok(())
    }

    pub fn create_image_with_info(
        &self,
        image_info: &vk::ImageCreateInfo,
        properties: vk::MemoryPropertyFlags,
    ) -> Result<(vk::Image, vk::DeviceMemory)> {
        let image = unsafe { self.device.create_image(image_info, None) }?;
        let mem_requirements = unsafe { self.device.get_image_memory_requirements(image) };
        let image_memory = {
            let allocate_info = vk::MemoryAllocateInfo::builder()
                .allocation_size(mem_requirements.size)
                .memory_type_index(
                    self.find_memory_type(mem_requirements.memory_type_bits, properties)?,
                );

            unsafe { self.device.allocate_memory(&allocate_info, None) }?
        };

        unsafe { self.device.bind_image_memory(image, image_memory, 0) }?;

        Ok((image, image_memory))
    }

    fn create_instance(
        window: &crate::Window,
        entry: &ash::Entry,
        app_info: &crate::ApplicationInfo,
    ) -> Result<ash::Instance> {
        if lve_utils::is_debug_build() && !Self::check_validation_layer_support(entry)? {
            bail!("Requested validation layers not available");
        }

        let instance = {
            let app_info = vk::ApplicationInfo::builder()
                .application_name(app_info.name)
                .application_version(app_info.version)
                .engine_name(app_info.engine_name)
                .engine_version(app_info.engine_version)
                .api_version(app_info.api_version);
            let extensions = Self::get_required_extensions(window)?;
            let layers = Self::VALIDATION_LAYERS.to_vec();
            let mut debug_create_info =
                crate::DebugUtilsMessenger::populate_debug_message_create_info();

            let create_info = if lve_utils::is_debug_build() {
                vk::InstanceCreateInfo::builder()
                    .application_info(&app_info)
                    .enabled_extension_names(&extensions)
                    .enabled_layer_names(&layers)
                    .push_next(&mut debug_create_info)
            } else {
                vk::InstanceCreateInfo::builder()
                    .application_info(&app_info)
                    .enabled_extension_names(&extensions)
            };

            unsafe { entry.create_instance(&create_info, None) }?
        };

        Self::has_required_instance_extensions(window, entry)?;

        Ok(instance)
    }

    fn pick_physical_device(
        instance: &ash::Instance,
        surface: &crate::Surface,
    ) -> Result<vk::PhysicalDevice> {
        let physical_devices = unsafe { instance.enumerate_physical_devices() }?;

        if physical_devices.is_empty() {
            bail!("Failed to find a suitable GPU");
        }

        println!("Device count: {}", physical_devices.len());

        let physical_device = {
            let mut physical_device = None;
            for device in physical_devices.iter() {
                if Self::is_device_suitable(instance, surface, device)? {
                    physical_device = Some(*device);
                    break;
                }
            }

            physical_device
        }
        .context("Failed to find a suitable GPU")?;
        let properties = unsafe { instance.get_physical_device_properties(physical_device) };

        println!("physical device: {:?}", unsafe {
            CStr::from_ptr(properties.device_name.as_ptr())
        });

        Ok(physical_device)
    }

    fn create_device(
        instance: &ash::Instance,
        surface: &crate::Surface,
        physical_device: &vk::PhysicalDevice,
    ) -> Result<(ash::Device, vk::Queue, vk::Queue)> {
        let indices = Self::find_queue_families(instance, surface, physical_device)?;
        let queue_create_infos = {
            let queue_priority = 1.0f32;

            indices
                .unique_queue_families()?
                .iter()
                .map(|queue_family| {
                    vk::DeviceQueueCreateInfo::builder()
                        .queue_family_index(*queue_family)
                        .queue_priorities(std::slice::from_ref(&queue_priority))
                        .build()
                })
                .collect::<Vec<_>>()
        };
        let device = {
            let create_info = vk::DeviceCreateInfo::builder()
                .queue_create_infos(&queue_create_infos)
                .enabled_extension_names(&Self::DEVICE_EXTENSIONS);

            unsafe { instance.create_device(*physical_device, &create_info, None) }?
        };
        let graphics_queue = unsafe {
            device.get_device_queue(
                indices
                    .graphics_family
                    .context("Failed to get graphics queue")?,
                0,
            )
        };
        let present_queue = unsafe {
            device.get_device_queue(
                indices
                    .present_family
                    .context("Failed to get present queue")?,
                0,
            )
        };

        Ok((device, graphics_queue, present_queue))
    }

    fn create_command_pool(
        instance: &ash::Instance,
        surface: &crate::Surface,
        physical_device: &vk::PhysicalDevice,
        device: &ash::Device,
    ) -> Result<vk::CommandPool> {
        let queue_family_indices = Self::find_queue_families(instance, surface, physical_device)?;
        let command_pool = {
            let create_info = vk::CommandPoolCreateInfo::builder()
                .queue_family_index(
                    queue_family_indices
                        .graphics_family
                        .context("Failed to get graphics queue family")?,
                )
                .flags(
                    vk::CommandPoolCreateFlags::TRANSIENT
                        | vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER,
                );

            unsafe { device.create_command_pool(&create_info, None) }?
        };

        Ok(command_pool)
    }

    /* Helper functions */
    fn is_device_suitable(
        instance: &ash::Instance,
        surface: &crate::Surface,
        physical_device: &vk::PhysicalDevice,
    ) -> Result<bool> {
        let indices = Self::find_queue_families(instance, surface, physical_device)?;
        let extensions_supported = Self::check_device_extension_support(instance, physical_device)?;
        let swap_chain_adequate = if extensions_supported {
            let swap_chain_support_details =
                unsafe { surface.query_swap_chain_support(physical_device) }?;

            !swap_chain_support_details.formats.is_empty()
                && !swap_chain_support_details.present_modes.is_empty()
        } else {
            false
        };
        let supported_features = unsafe { instance.get_physical_device_features(*physical_device) };

        Ok(indices.is_complete()
            && extensions_supported
            && swap_chain_adequate
            && supported_features.sampler_anisotropy != 0)
    }

    fn get_required_extensions(window: &crate::Window) -> Result<Vec<*const i8>> {
        let mut extensions =
            ash_window::enumerate_required_extensions(window.window().raw_display_handle())?
                .to_vec();

        if lve_utils::is_debug_build() {
            extensions.push(crate::DebugUtilsMessenger::extension_name().as_ptr());
        }

        Ok(extensions)
    }

    fn check_validation_layer_support(entry: &ash::Entry) -> Result<bool> {
        println!("Requested validation layers:");
        let validation_layers = Self::VALIDATION_LAYERS
            .iter()
            .map(|layer| {
                let layer_name = unsafe { CStr::from_ptr(*layer) };

                println!("\t{:?}", layer_name);

                layer_name
            })
            .collect::<Vec<_>>();
        println!("Available layers:");
        let layers_found = entry
            .enumerate_instance_layer_properties()?
            .iter()
            .filter(|layer| {
                let layer_name = unsafe { CStr::from_ptr((**layer).layer_name.as_ptr()) };
                println!("\t{:?}", layer_name);

                validation_layers.contains(&layer_name)
            })
            .count();

        Ok(layers_found == validation_layers.len())
    }

    fn find_queue_families(
        instance: &ash::Instance,
        surface: &crate::Surface,
        physical_device: &vk::PhysicalDevice,
    ) -> Result<QueryFamilyIndices> {
        let queue_families =
            unsafe { instance.get_physical_device_queue_family_properties(*physical_device) };
        let mut indices = QueryFamilyIndices::none();

        for (idx, queue_family) in queue_families.iter().enumerate() {
            let present_support = unsafe {
                surface.get_physical_device_surface_support(physical_device, idx as u32)
            }?;

            if queue_family.queue_count > 0 {
                if (queue_family.queue_flags & vk::QueueFlags::GRAPHICS) == vk::QueueFlags::GRAPHICS
                {
                    indices.graphics_family = Some(idx as u32);
                }
                if present_support {
                    indices.present_family = Some(idx as u32);
                }
            }

            if indices.is_complete() {
                break;
            }
        }

        Ok(indices)
    }

    fn has_required_instance_extensions(window: &crate::Window, entry: &ash::Entry) -> Result<()> {
        println!("Available extensions:");
        let available = entry
            .enumerate_instance_extension_properties(None)?
            .iter()
            .map(|extension| {
                let extension_name = unsafe { CStr::from_ptr(extension.extension_name.as_ptr()) };

                println!("\t{:?}", extension_name);

                extension_name
            })
            .collect::<Vec<_>>();

        println!("Required extensions:");
        let required_extensions = Self::get_required_extensions(window)?;
        let contained_required_extensions = required_extensions
            .iter()
            .filter(|extension| {
                let extension_name = unsafe { CStr::from_ptr(**extension) };

                println!("\t{:?}", extension_name);

                available.contains(&extension_name)
            })
            .count();

        if contained_required_extensions != required_extensions.len() {
            bail!("Missing required extension");
        }

        Ok(())
    }

    fn check_device_extension_support(
        instance: &ash::Instance,
        physical_device: &vk::PhysicalDevice,
    ) -> Result<bool> {
        let required_extensions = Self::DEVICE_EXTENSIONS
            .iter()
            .map(|extension| unsafe { CStr::from_ptr(*extension) })
            .collect::<Vec<_>>();
        let required_extensions_available =
            unsafe { instance.enumerate_device_extension_properties(*physical_device) }?
                .iter()
                .filter(|extension| {
                    let extension_name =
                        unsafe { CStr::from_ptr(extension.extension_name.as_ptr()) };

                    required_extensions.contains(&extension_name)
                })
                .count();

        Ok(required_extensions_available == required_extensions.len())
    }
}
