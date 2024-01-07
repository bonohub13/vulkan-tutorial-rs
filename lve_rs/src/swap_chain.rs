use anyhow::{bail, Context, Result};
use ash::{extensions::khr as vk_khr, vk};

pub struct SwapChain {
    swap_chain_image_format: vk::Format,
    swap_chain_depth_format: vk::Format,
    swap_chain_extent: vk::Extent2D,
    swap_chain_framebuffers: Vec<vk::Framebuffer>,
    render_pass: vk::RenderPass,
    storage_images: Vec<vk::Image>,
    storage_image_views: Vec<vk::ImageView>,
    storage_image_memory: Vec<vk::DeviceMemory>,
    storage_samplers: Vec<vk::Sampler>,
    depth_images: Vec<vk::Image>,
    depth_image_memory: Vec<vk::DeviceMemory>,
    depth_image_views: Vec<vk::ImageView>,
    swap_chain_images: Vec<vk::Image>,
    swap_chain_image_views: Vec<vk::ImageView>,
    window_extent: vk::Extent2D,
    extension: vk_khr::Swapchain,
    swap_chain: vk::SwapchainKHR,
    image_available_semaphores: Vec<vk::Semaphore>,
    render_finished_semaphores: Vec<vk::Semaphore>,
    in_flight_fences: Vec<vk::Fence>,
    images_in_flight: Vec<vk::Fence>,
    current_frame: usize,
    pub write_descriptors: Vec<vk::DescriptorImageInfo>,
    pub read_descriptors: Vec<vk::DescriptorImageInfo>,
}

impl SwapChain {
    pub const MAX_FRAMES_IN_FLIGHT: i32 = 2;

    pub fn new(device: &crate::Device, extent: vk::Extent2D) -> Result<Self> {
        Self::init(device, extent, &vk::SwapchainKHR::null())
    }

    pub fn null(device: &crate::Device) -> Self {
        Self {
            swap_chain_image_format: vk::Format::default(),
            swap_chain_depth_format: vk::Format::default(),
            swap_chain_extent: vk::Extent2D::default(),
            swap_chain_framebuffers: vec![],
            render_pass: vk::RenderPass::null(),
            storage_images: vec![],
            storage_image_views: vec![],
            storage_image_memory: vec![],
            storage_samplers: vec![],
            depth_images: vec![],
            depth_image_memory: vec![],
            depth_image_views: vec![],
            swap_chain_images: vec![],
            swap_chain_image_views: vec![],
            window_extent: vk::Extent2D::default(),
            extension: vk_khr::Swapchain::new(device.instance(), device.device()),
            swap_chain: vk::SwapchainKHR::null(),
            image_available_semaphores: vec![],
            render_finished_semaphores: vec![],
            in_flight_fences: vec![],
            images_in_flight: vec![],
            current_frame: 0,
            write_descriptors: vec![],
            read_descriptors: vec![],
        }
    }

    pub fn with_previous_swap_chain(
        device: &crate::Device,
        extent: vk::Extent2D,
        previous_swap_chain: &vk::SwapchainKHR,
    ) -> Result<Self> {
        let swap_chain = Self::init(device, extent, previous_swap_chain)?;

        Ok(swap_chain)
    }

    pub unsafe fn destroy(&mut self, device: &crate::Device) {
        let device_ref = device.device();
        self.swap_chain_image_views.iter().for_each(|image_view| {
            device_ref.destroy_image_view(*image_view, None);
        });
        self.swap_chain_image_views.clear();

        if self.swap_chain != vk::SwapchainKHR::null() {
            self.extension.destroy_swapchain(self.swap_chain, None);
        }

        (0..self.storage_images.len())
            .into_iter()
            .for_each(|index| {
                device_ref.destroy_sampler(self.storage_samplers[index], None);
                device_ref.destroy_image_view(self.storage_image_views[index], None);
                device_ref.destroy_image(self.storage_images[index], None);
                device_ref.free_memory(self.storage_image_memory[index], None);
            });
        (0..self.depth_images.len()).into_iter().for_each(|index| {
            device_ref.destroy_image_view(self.depth_image_views[index], None);
            device_ref.destroy_image(self.depth_images[index], None);
            device_ref.free_memory(self.depth_image_memory[index], None);
        });

        self.swap_chain_framebuffers.iter().for_each(|framebuffer| {
            device_ref.destroy_framebuffer(*framebuffer, None);
        });

        device_ref.destroy_render_pass(self.render_pass, None);

        (0..Self::MAX_FRAMES_IN_FLIGHT as usize)
            .into_iter()
            .for_each(|index| {
                device_ref.destroy_semaphore(self.render_finished_semaphores[index], None);
                device_ref.destroy_semaphore(self.image_available_semaphores[index], None);
                device_ref.destroy_fence(self.in_flight_fences[index], None);
            });
    }

    #[inline]
    pub fn swap_chain(&self) -> &vk::SwapchainKHR {
        &self.swap_chain
    }

    #[inline]
    pub fn framebuffer(&self, index: usize) -> &vk::Framebuffer {
        &self.swap_chain_framebuffers[index]
    }

    #[inline]
    pub const fn render_pass(&self) -> &vk::RenderPass {
        &self.render_pass
    }

    #[inline]
    pub fn image_view(&self, index: usize) -> &vk::ImageView {
        &self.swap_chain_image_views[index]
    }

    #[inline]
    pub fn image_count(&self) -> usize {
        self.swap_chain_images.len()
    }

    #[inline]
    pub fn swap_chain_image_format(&self) -> vk::Format {
        self.swap_chain_image_format
    }

    #[inline]
    pub fn swap_chain_extent(&self) -> vk::Extent2D {
        self.swap_chain_extent
    }

    #[inline]
    pub fn width(&self) -> u32 {
        self.swap_chain_extent.width
    }

    #[inline]
    pub fn height(&self) -> u32 {
        self.swap_chain_extent.height
    }

    #[inline]
    pub fn extent_aspect_ratio(&self) -> f64 {
        self.swap_chain_extent.width as f64 / self.swap_chain_extent.height as f64
    }

    #[inline]
    pub fn find_depth_format(&self, device: &crate::Device) -> Result<vk::Format> {
        Ok(Self::find_depth_format_from_device(device)?)
    }

    pub fn acquire_next_image(&self, device: &crate::Device) -> Result<(usize, bool)> {
        if self.in_flight_fences[self.current_frame] == vk::Fence::null() {
            bail!("in_flight_fences[{}] is NULL (invalid)", self.current_frame);
        }

        let fences = [self.in_flight_fences[self.current_frame]];
        unsafe { device.device().wait_for_fences(&fences, true, u64::MAX) }?;

        let (image_index, result) = match unsafe {
            self.extension.acquire_next_image(
                self.swap_chain,
                u64::MAX,
                self.image_available_semaphores[self.current_frame],
                vk::Fence::null(),
            )
        } {
            Ok((image_index, _)) => Ok((image_index, false)),
            Err(err) => {
                if err == vk::Result::ERROR_OUT_OF_DATE_KHR {
                    Ok((0, true))
                } else {
                    Err(err)
                }
            }
        }?;

        Ok((image_index as usize, result))
    }

    pub fn submit_command_buffers(
        &mut self,
        device: &crate::Device,
        buffer: &vk::CommandBuffer,
        image_index: usize,
    ) -> Result<bool> {
        if self.images_in_flight[image_index] != vk::Fence::null() {
            unsafe {
                device.device().wait_for_fences(
                    std::slice::from_ref(&self.images_in_flight[image_index]),
                    true,
                    u64::MAX,
                )
            }?;
        }
        self.images_in_flight[image_index] = self.in_flight_fences[self.current_frame];

        let signal_semaphores = [self.render_finished_semaphores[self.current_frame]];
        let wait_semaphores = [self.image_available_semaphores[self.current_frame]];
        let wait_stages = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
        let submit_info = vk::SubmitInfo::builder()
            .wait_semaphores(&wait_semaphores)
            .wait_dst_stage_mask(&wait_stages)
            .command_buffers(std::slice::from_ref(buffer))
            .signal_semaphores(&signal_semaphores)
            .build();

        unsafe {
            device.device().reset_fences(std::slice::from_ref(
                &self.in_flight_fences[self.current_frame],
            ))
        }?;
        unsafe {
            device.device().queue_submit(
                *device.graphics_queue(),
                std::slice::from_ref(&submit_info),
                self.in_flight_fences[self.current_frame],
            )
        }?;

        let image_indices = [image_index as u32];
        let present_info = {
            vk::PresentInfoKHR::builder()
                .wait_semaphores(&signal_semaphores)
                .swapchains(std::slice::from_ref(&self.swap_chain))
                .image_indices(&image_indices)
        };

        let result = match unsafe {
            self.extension
                .queue_present(*device.present_queue(), &present_info)
        } {
            Ok(_) => Ok(false),
            Err(err) => {
                if err == vk::Result::ERROR_OUT_OF_DATE_KHR || err == vk::Result::SUBOPTIMAL_KHR {
                    return Ok(true);
                } else {
                    Err(err)
                }
            }
        }?;

        self.current_frame = (self.current_frame + 1) % Self::MAX_FRAMES_IN_FLIGHT as usize;

        Ok(result)
    }

    pub fn compare_swap_formats(&self, swap_chain: &Self) -> bool {
        self.swap_chain_image_format == swap_chain.swap_chain_image_format
            && self.swap_chain_depth_format == swap_chain.swap_chain_depth_format
    }

    pub unsafe fn prepare_to_trace_barrier(
        &self,
        device: &crate::Device,
        command_buffer: &vk::CommandBuffer,
        frame_index: usize,
    ) {
        let device = device.device();
        let barrier = vk::ImageMemoryBarrier::builder()
            .old_layout(vk::ImageLayout::UNDEFINED)
            .new_layout(vk::ImageLayout::GENERAL)
            .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
            .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
            .image(self.swap_chain_images[frame_index])
            .subresource_range(vk::ImageSubresourceRange {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                base_mip_level: 0,
                level_count: 1,
                base_array_layer: 0,
                layer_count: 1,
            })
            .src_access_mask(vk::AccessFlags::NONE_KHR)
            .dst_access_mask(vk::AccessFlags::MEMORY_WRITE);

        device.cmd_pipeline_barrier(
            *command_buffer,
            vk::PipelineStageFlags::TOP_OF_PIPE,
            vk::PipelineStageFlags::COMPUTE_SHADER,
            vk::DependencyFlags::default(),
            &[],
            &[],
            std::slice::from_ref(&barrier),
        )
    }

    pub unsafe fn enforce_barrier(
        &self,
        device: &crate::Device,
        command_buffer: &vk::CommandBuffer,
        frame_index: usize,
    ) {
        let device = device.device();
        let barrier = vk::ImageMemoryBarrier::builder()
            .old_layout(vk::ImageLayout::GENERAL)
            .new_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
            .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
            .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
            .image(self.swap_chain_images[frame_index])
            .subresource_range(vk::ImageSubresourceRange {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                base_mip_level: 0,
                level_count: 1,
                base_array_layer: 0,
                layer_count: 1,
            })
            .src_access_mask(vk::AccessFlags::MEMORY_WRITE)
            .dst_access_mask(vk::AccessFlags::SHADER_READ);

        device.cmd_pipeline_barrier(
            *command_buffer,
            vk::PipelineStageFlags::COMPUTE_SHADER,
            vk::PipelineStageFlags::FRAGMENT_SHADER,
            vk::DependencyFlags::default(),
            &[],
            &[],
            std::slice::from_ref(&barrier),
        )
    }

    fn init(
        device: &crate::Device,
        extent: vk::Extent2D,
        previous_swap_chain: &vk::SwapchainKHR,
    ) -> Result<Self> {
        let (extension, swap_chain, swap_chain_images, swap_chain_image_format, swap_chain_extent) =
            Self::create_swap_chain(device, &extent, previous_swap_chain)?;
        let swap_chain_image_views =
            Self::create_image_views(device, &swap_chain_images, swap_chain_image_format)?;
        let render_pass = Self::create_render_pass(device, swap_chain_image_format)?;
        let (depth_images, depth_image_memory, depth_image_views, swap_chain_depth_format) =
            Self::create_depth_resources(device, &swap_chain_extent, &swap_chain_images)?;
        let (
            storage_images,
            storage_image_memory,
            storage_image_views,
            storage_samplers,
            write_descriptors,
            read_descriptors,
        ) = Self::create_storage_resources(device, &swap_chain_extent, &swap_chain_images)?;
        let swap_chain_framebuffers = Self::create_framebuffers(
            device,
            &swap_chain_extent,
            &swap_chain_images,
            &swap_chain_image_views,
            &depth_image_views,
            &render_pass,
        )?;
        let (
            image_available_semaphores,
            render_finished_semaphores,
            in_flight_fences,
            images_in_flight,
        ) = Self::create_sync_objects(&device, &swap_chain_images)?;

        Ok(Self {
            swap_chain_image_format,
            swap_chain_depth_format,
            swap_chain_extent,
            swap_chain_framebuffers,
            render_pass,
            depth_images,
            depth_image_memory,
            depth_image_views,
            storage_images,
            storage_image_memory,
            storage_image_views,
            storage_samplers,
            swap_chain_images,
            swap_chain_image_views,
            window_extent: extent,
            extension,
            swap_chain,
            image_available_semaphores,
            render_finished_semaphores,
            in_flight_fences,
            images_in_flight,
            current_frame: 0,
            write_descriptors,
            read_descriptors,
        })
    }

    fn create_swap_chain(
        device: &crate::Device,
        window_extent: &vk::Extent2D,
        previous_swap_chain: &vk::SwapchainKHR,
    ) -> Result<(
        vk_khr::Swapchain,
        vk::SwapchainKHR,
        Vec<vk::Image>,
        vk::Format,
        vk::Extent2D,
    )> {
        let swap_chain_support = unsafe { device.swap_chain_support() }?;
        let surface_format = Self::choose_swap_surface_format(&swap_chain_support.formats)?;
        let present_mode = Self::choose_swap_present_mode(&swap_chain_support.present_modes);
        let extent = Self::choose_swap_extent(window_extent, &swap_chain_support.capabilities);
        let image_count = if swap_chain_support.capabilities.max_image_count > 0
            && (swap_chain_support.capabilities.min_image_count + 1
                > swap_chain_support.capabilities.max_image_count)
        {
            swap_chain_support.capabilities.max_image_count
        } else {
            swap_chain_support.capabilities.min_image_count + 1
        };
        let indices = device.find_physical_queue_families()?;
        let queue_family_indices = [
            indices
                .graphics_family
                .context("Graphics queue family is not available")?,
            indices
                .present_family
                .context("Present queue family is not available")?,
        ];
        let queue_family_matches = queue_family_indices[0] == queue_family_indices[1];
        let extension = vk_khr::Swapchain::new(device.instance(), device.device());

        let swap_chain = {
            let create_info = vk::SwapchainCreateInfoKHR::builder()
                .surface(*device.surface().surface())
                .min_image_count(image_count)
                .image_format(surface_format.format)
                .image_color_space(surface_format.color_space)
                .image_extent(extent)
                .image_array_layers(1)
                .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
                .image_sharing_mode(if queue_family_matches {
                    vk::SharingMode::EXCLUSIVE
                } else {
                    vk::SharingMode::CONCURRENT
                })
                .queue_family_indices(if queue_family_matches {
                    &[]
                } else {
                    &queue_family_indices
                })
                .pre_transform(swap_chain_support.capabilities.current_transform)
                .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
                .present_mode(present_mode)
                .clipped(true)
                .old_swapchain(if *previous_swap_chain != vk::SwapchainKHR::null() {
                    *previous_swap_chain
                } else {
                    vk::SwapchainKHR::null()
                });

            unsafe { extension.create_swapchain(&create_info, None) }?
        };

        let swap_chain_images = unsafe { extension.get_swapchain_images(swap_chain) }?;

        Ok((
            extension,
            swap_chain,
            swap_chain_images,
            surface_format.format,
            extent,
        ))
    }

    fn create_image_views(
        device: &crate::Device,
        swap_chain_images: &[vk::Image],
        swap_chain_image_format: vk::Format,
    ) -> Result<Vec<vk::ImageView>> {
        let mut swap_chain_image_views = (0..swap_chain_images.len())
            .into_iter()
            .map(|_| vk::ImageView::null())
            .collect::<Vec<_>>();

        for index in 0..swap_chain_images.len() {
            let view_info = vk::ImageViewCreateInfo::builder()
                .image(swap_chain_images[index])
                .view_type(vk::ImageViewType::TYPE_2D)
                .format(swap_chain_image_format)
                .subresource_range(vk::ImageSubresourceRange {
                    aspect_mask: vk::ImageAspectFlags::COLOR,
                    base_mip_level: 0,
                    level_count: 1,
                    base_array_layer: 0,
                    layer_count: 1,
                })
                .build();

            swap_chain_image_views[index] =
                unsafe { device.device().create_image_view(&view_info, None) }?;
        }

        Ok(swap_chain_image_views)
    }

    fn create_resources<F: Fn(&[vk::Image]) -> Result<()>>(
        device: &crate::Device,
        swap_chain_extent: &vk::Extent2D,
        swap_chain_images: &[vk::Image],
        format: vk::Format,
        usage: vk::ImageUsageFlags,
        subresource_range: vk::ImageSubresourceRange,
        transition_func: F,
    ) -> Result<(Vec<vk::Image>, Vec<vk::DeviceMemory>, Vec<vk::ImageView>)> {
        let image_count = swap_chain_images.len();
        let image_info = vk::ImageCreateInfo::builder()
            .image_type(vk::ImageType::TYPE_2D)
            .extent(vk::Extent3D {
                width: swap_chain_extent.width,
                height: swap_chain_extent.height,
                depth: 1,
            })
            .mip_levels(1)
            .array_layers(1)
            .format(format)
            .tiling(vk::ImageTiling::OPTIMAL)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .usage(usage)
            .samples(vk::SampleCountFlags::TYPE_1)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);

        let mut images = (0..image_count)
            .into_iter()
            .map(|_| vk::Image::null())
            .collect::<Vec<_>>();
        let mut image_memory = (0..image_count)
            .into_iter()
            .map(|_| vk::DeviceMemory::null())
            .collect::<Vec<_>>();
        let mut image_views = (0..image_count)
            .into_iter()
            .map(|_| vk::ImageView::null())
            .collect::<Vec<_>>();

        for (image, image_memory) in images.iter_mut().zip(image_memory.iter_mut()) {
            (*image, *image_memory) = device
                .create_image_with_info(&image_info, vk::MemoryPropertyFlags::DEVICE_LOCAL)?;
        }
        transition_func(&images)?;
        for (image, image_view) in images.iter().zip(image_views.iter_mut()) {
            *image_view = {
                let create_info = vk::ImageViewCreateInfo::builder()
                    .image(*image)
                    .view_type(vk::ImageViewType::TYPE_2D)
                    .format(format)
                    .subresource_range(subresource_range);

                unsafe { device.device().create_image_view(&create_info, None) }?
            };
        }

        Ok((images, image_memory, image_views))
    }

    unsafe fn transition_image_layout(
        device: &crate::Device,
        images: &[vk::Image],
        old_layout: vk::ImageLayout,
        new_layout: vk::ImageLayout,
    ) -> Result<()> {
        let device_ref = device.device();
        let (src_access_mask, dst_access_mask, source_stage, destination_stage) =
            match (old_layout, new_layout) {
                (vk::ImageLayout::UNDEFINED, vk::ImageLayout::TRANSFER_DST_OPTIMAL) => (
                    vk::AccessFlags::NONE_KHR,
                    vk::AccessFlags::TRANSFER_WRITE,
                    vk::PipelineStageFlags::TOP_OF_PIPE,
                    vk::PipelineStageFlags::TRANSFER,
                ),
                (vk::ImageLayout::UNDEFINED, vk::ImageLayout::GENERAL) => (
                    vk::AccessFlags::NONE_KHR,
                    vk::AccessFlags::MEMORY_WRITE,
                    vk::PipelineStageFlags::TOP_OF_PIPE,
                    vk::PipelineStageFlags::COMPUTE_SHADER,
                ),
                _ => (
                    vk::AccessFlags::TRANSFER_WRITE,
                    vk::AccessFlags::SHADER_READ,
                    vk::PipelineStageFlags::TRANSFER,
                    vk::PipelineStageFlags::FRAGMENT_SHADER,
                ),
            };
        let mut barrier = vk::ImageMemoryBarrier::builder()
            .old_layout(old_layout)
            .new_layout(new_layout)
            .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
            .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
            .subresource_range(vk::ImageSubresourceRange {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                base_mip_level: 0,
                level_count: 1,
                base_array_layer: 0,
                layer_count: 1,
            })
            .src_access_mask(src_access_mask)
            .dst_access_mask(dst_access_mask);

        for image in images {
            let command_buffer = device.begin_single_time_commands()?;

            barrier.image = *image;
            device_ref.cmd_pipeline_barrier(
                command_buffer,
                source_stage,
                destination_stage,
                vk::DependencyFlags::default(),
                &[],
                &[],
                std::slice::from_ref(&barrier),
            );

            device.end_single_time_commands(&command_buffer)?;
        }

        Ok(())
    }

    fn create_depth_resources(
        device: &crate::Device,
        swap_chain_extent: &vk::Extent2D,
        swap_chain_images: &[vk::Image],
    ) -> Result<(
        Vec<vk::Image>,
        Vec<vk::DeviceMemory>,
        Vec<vk::ImageView>,
        vk::Format,
    )> {
        let depth_format = Self::find_depth_format_from_device(device)?;
        let subresource_range = vk::ImageSubresourceRange {
            aspect_mask: vk::ImageAspectFlags::DEPTH,
            base_mip_level: 0,
            level_count: 1,
            base_array_layer: 0,
            layer_count: 1,
        };
        let (depth_images, depth_image_memory, depth_image_views) = Self::create_resources(
            device,
            swap_chain_extent,
            swap_chain_images,
            depth_format,
            vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT,
            subresource_range,
            |_| Ok(()),
        )?;

        Ok((
            depth_images,
            depth_image_memory,
            depth_image_views,
            depth_format,
        ))
    }

    fn create_storage_resources(
        device: &crate::Device,
        swap_chain_extent: &vk::Extent2D,
        swap_chain_images: &[vk::Image],
    ) -> Result<(
        Vec<vk::Image>,
        Vec<vk::DeviceMemory>,
        Vec<vk::ImageView>,
        Vec<vk::Sampler>,
        Vec<vk::DescriptorImageInfo>,
        Vec<vk::DescriptorImageInfo>,
    )> {
        let format = vk::Format::R8G8B8A8_UNORM;
        let usage = vk::ImageUsageFlags::STORAGE | vk::ImageUsageFlags::SAMPLED;
        let subresource_range = vk::ImageSubresourceRange {
            aspect_mask: vk::ImageAspectFlags::COLOR,
            base_mip_level: 0,
            level_count: 1,
            base_array_layer: 0,
            layer_count: 1,
        };
        let image_count = swap_chain_images.len();
        let (storage_images, storage_image_memory, storage_image_views) = Self::create_resources(
            device,
            swap_chain_extent,
            swap_chain_images,
            format,
            usage,
            subresource_range,
            |images| unsafe {
                Self::transition_image_layout(
                    device,
                    images,
                    vk::ImageLayout::UNDEFINED,
                    vk::ImageLayout::GENERAL,
                )
            },
        )?;
        let mut sampler = (0..image_count)
            .into_iter()
            .map(|_| vk::Sampler::null())
            .collect::<Vec<_>>();
        let sampler_create_info = vk::SamplerCreateInfo::builder()
            .min_filter(vk::Filter::NEAREST)
            .mag_filter(vk::Filter::LINEAR)
            .address_mode_u(vk::SamplerAddressMode::REPEAT)
            .address_mode_v(vk::SamplerAddressMode::REPEAT)
            .address_mode_w(vk::SamplerAddressMode::REPEAT)
            .anisotropy_enable(false)
            .max_anisotropy(1.0)
            .border_color(vk::BorderColor::INT_OPAQUE_BLACK)
            .unnormalized_coordinates(false)
            .compare_enable(false)
            .compare_op(vk::CompareOp::ALWAYS)
            .mipmap_mode(vk::SamplerMipmapMode::LINEAR)
            .mip_lod_bias(0.0)
            .min_lod(0.0)
            .max_lod(0.0);
        let mut write_descriptors = (0..image_count)
            .into_iter()
            .map(|_| {
                vk::DescriptorImageInfo::builder()
                    .image_layout(vk::ImageLayout::GENERAL)
                    .build()
            })
            .collect::<Vec<_>>();
        let mut read_descriptors = write_descriptors.clone();

        for sampler in sampler.iter_mut() {
            *sampler = unsafe { device.device().create_sampler(&sampler_create_info, None) }?;
        }

        storage_image_views
            .iter()
            .zip(
                sampler.iter().zip(
                    read_descriptors
                        .iter_mut()
                        .zip(write_descriptors.iter_mut()),
                ),
            )
            .for_each(
                |(image_view, (sampler, (read_descriptor, write_descriptor)))| {
                    write_descriptor.image_view = *image_view;
                    read_descriptor.image_view = *image_view;
                    read_descriptor.sampler = *sampler;
                },
            );

        Ok((
            storage_images,
            storage_image_memory,
            storage_image_views,
            sampler,
            write_descriptors,
            read_descriptors,
        ))
    }

    fn create_render_pass(
        device: &crate::Device,
        swap_chain_image_format: vk::Format,
    ) -> Result<vk::RenderPass> {
        let attachment = [
            vk::AttachmentDescription::builder()
                .format(swap_chain_image_format)
                .samples(vk::SampleCountFlags::TYPE_1)
                .load_op(vk::AttachmentLoadOp::LOAD)
                .store_op(vk::AttachmentStoreOp::STORE)
                .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
                .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
                .initial_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
                .final_layout(vk::ImageLayout::PRESENT_SRC_KHR)
                .build(),
            vk::AttachmentDescription::builder()
                .format(Self::find_depth_format_from_device(device)?)
                .samples(vk::SampleCountFlags::TYPE_1)
                .load_op(vk::AttachmentLoadOp::CLEAR)
                .store_op(vk::AttachmentStoreOp::DONT_CARE)
                .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
                .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
                .initial_layout(vk::ImageLayout::UNDEFINED)
                .final_layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL)
                .build(),
        ];
        let color_attachment = vk::AttachmentReference::builder()
            .attachment(0)
            .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
            .build();
        let depth_stencil_attachment = vk::AttachmentReference::builder()
            .attachment(1)
            .layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL)
            .build();
        let subpass = {
            vk::SubpassDescription::builder()
                .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
                .color_attachments(std::slice::from_ref(&color_attachment))
                .depth_stencil_attachment(&depth_stencil_attachment)
        };
        let dependency = vk::SubpassDependency::builder()
            .src_subpass(vk::SUBPASS_EXTERNAL)
            .src_stage_mask(
                vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT
                    | vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS,
            )
            .src_access_mask(vk::AccessFlags::empty())
            .dst_subpass(0)
            .dst_stage_mask(
                vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT
                    | vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS,
            )
            .dst_access_mask(
                vk::AccessFlags::COLOR_ATTACHMENT_WRITE
                    | vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE,
            )
            .build();
        let create_info = vk::RenderPassCreateInfo::builder()
            .attachments(&attachment)
            .subpasses(std::slice::from_ref(&subpass))
            .dependencies(std::slice::from_ref(&dependency))
            .build();

        Ok(unsafe { device.device().create_render_pass(&create_info, None) }?)
    }

    fn create_framebuffers(
        device: &crate::Device,
        swap_chain_extent: &vk::Extent2D,
        swap_chain_images: &[vk::Image],
        swap_chain_image_views: &[vk::ImageView],
        depth_image_views: &[vk::ImageView],
        render_pass: &vk::RenderPass,
    ) -> Result<Vec<vk::Framebuffer>> {
        let image_count = swap_chain_images.len();
        let mut swap_chain_framebuffers = Vec::from_iter(
            (0..image_count)
                .into_iter()
                .map(|_| vk::Framebuffer::null()),
        );

        for index in 0..image_count {
            let attachments = [swap_chain_image_views[index], depth_image_views[index]];
            let create_info = vk::FramebufferCreateInfo::builder()
                .render_pass(*render_pass)
                .attachments(&attachments)
                .width(swap_chain_extent.width)
                .height(swap_chain_extent.height)
                .layers(1)
                .build();

            swap_chain_framebuffers[index] =
                unsafe { device.device().create_framebuffer(&create_info, None) }?;
        }

        Ok(swap_chain_framebuffers)
    }

    fn create_sync_objects(
        device: &crate::Device,
        swap_chain_images: &[vk::Image],
    ) -> Result<(
        Vec<vk::Semaphore>,
        Vec<vk::Semaphore>,
        Vec<vk::Fence>,
        Vec<vk::Fence>,
    )> {
        let mut image_available_semaphores = Vec::from_iter(
            (0..Self::MAX_FRAMES_IN_FLIGHT)
                .into_iter()
                .map(|_| vk::Semaphore::null()),
        );
        let mut render_finished_semaphores = Vec::from_iter(
            (0..Self::MAX_FRAMES_IN_FLIGHT)
                .into_iter()
                .map(|_| vk::Semaphore::null()),
        );
        let mut in_flight_fences = Vec::from_iter(
            (0..Self::MAX_FRAMES_IN_FLIGHT)
                .into_iter()
                .map(|_| vk::Fence::null()),
        );
        let image_count = swap_chain_images.len();
        let images_in_flight = (0..image_count)
            .into_iter()
            .map(|_| vk::Fence::null())
            .collect::<Vec<_>>();
        let semaphore_info = vk::SemaphoreCreateInfo::builder();
        let fence_info = vk::FenceCreateInfo::builder().flags(vk::FenceCreateFlags::SIGNALED);

        for index in 0..Self::MAX_FRAMES_IN_FLIGHT as usize {
            image_available_semaphores[index] =
                unsafe { device.device().create_semaphore(&semaphore_info, None) }?;
            render_finished_semaphores[index] =
                unsafe { device.device().create_semaphore(&semaphore_info, None) }?;
            in_flight_fences[index] = unsafe { device.device().create_fence(&fence_info, None) }?;
        }

        Ok((
            image_available_semaphores,
            render_finished_semaphores,
            in_flight_fences,
            images_in_flight,
        ))
    }

    /* --- Helper functions --- */

    fn choose_swap_surface_format(
        available_formats: &Vec<vk::SurfaceFormatKHR>,
    ) -> Result<vk::SurfaceFormatKHR> {
        for available_format in available_formats {
            if available_format.format == vk::Format::B8G8R8A8_SRGB
                && available_format.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR
            {
                return Ok(*available_format);
            }
        }

        Ok(*available_formats
            .iter()
            .next()
            .context("No format was available")?)
    }

    fn choose_swap_present_mode(
        available_present_modes: &Vec<vk::PresentModeKHR>,
    ) -> vk::PresentModeKHR {
        for availbale_present_mode in available_present_modes {
            if *availbale_present_mode == vk::PresentModeKHR::MAILBOX {
                println!("Present mode: Mailbox");

                return *availbale_present_mode;
            }
        }

        println!("Present mode: V-Sync");

        vk::PresentModeKHR::FIFO
    }

    fn choose_swap_extent(
        window_extent: &vk::Extent2D,
        capabilities: &vk::SurfaceCapabilitiesKHR,
    ) -> vk::Extent2D {
        if capabilities.current_extent.width != u32::MAX {
            capabilities.current_extent
        } else {
            vk::Extent2D::builder()
                .width(
                    capabilities
                        .min_image_extent
                        .width
                        .max(capabilities.max_image_extent.width.min(window_extent.width)),
                )
                .height(
                    capabilities.min_image_extent.height.max(
                        capabilities
                            .max_image_extent
                            .height
                            .min(window_extent.height),
                    ),
                )
                .build()
        }
    }

    fn find_depth_format_from_device(device: &crate::Device) -> Result<vk::Format> {
        Ok(device.find_supported_format(
            &[
                vk::Format::D32_SFLOAT,
                vk::Format::D32_SFLOAT_S8_UINT,
                vk::Format::D24_UNORM_S8_UINT,
            ],
            vk::ImageTiling::OPTIMAL,
            vk::FormatFeatureFlags::DEPTH_STENCIL_ATTACHMENT,
        )?)
    }
}
