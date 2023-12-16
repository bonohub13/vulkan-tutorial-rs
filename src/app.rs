use anyhow::{bail, Result};
use ash::vk;
use std::mem::size_of;
use winit::{
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};
extern crate nalgebra_glm as glm;

#[derive(Default)]
#[repr(C, align(16))]
pub struct SimplePushConstantData {
    offset: glm::Vec2,
    color: glm::Vec3,
}

pub struct App {
    window: lve_rs::Window,
    device: lve_rs::Device,
    swap_chain: Box<lve_rs::SwapChain>,
    pipeline: Box<lve_rs::Pipeline>,
    pipeline_layout: vk::PipelineLayout,
    command_buffers: Vec<vk::CommandBuffer>,
    model: Box<lve_rs::Model>,
}

impl App {
    pub const WIDTH: i32 = 1280;
    pub const HEIGHT: i32 = 800;
    pub const FRAMES_PER_SECOND_LIMIT: u64 = 144;
    pub const MICROSECONDS_IN_SECOND: u64 = 1_000_000;
    pub const MILLISECONDS_PER_FRAME: u64 =
        ((Self::MICROSECONDS_IN_SECOND / Self::FRAMES_PER_SECOND_LIMIT) / 100 + 1) * 100;

    pub fn new<T>(
        event_loop: &EventLoop<T>,
        width: Option<i32>,
        height: Option<i32>,
    ) -> Result<Self> {
        let width = if let Some(width) = width {
            width
        } else {
            Self::WIDTH
        };
        let height = if let Some(height) = height {
            height
        } else {
            Self::HEIGHT
        };
        let window = lve_rs::Window::new(event_loop, width, height, "Hello Vulkan!")?;
        let device = lve_rs::Device::new(&window, &lve_rs::ApplicationInfo::default())?;
        let model = Self::load_models(&device)?;
        let pipeline_layout = Self::create_pipeline_layout(&device)?;
        let (swap_chain, pipeline) = Self::recreate_swap_chain(
            &window,
            &device,
            &pipeline_layout,
            &vk::SwapchainKHR::null(),
            &mut vec![],
            None,
        )?;
        let command_buffers = Self::create_command_buffers(&device, &swap_chain)?;

        Ok(Self {
            window,
            device,
            swap_chain,
            pipeline,
            pipeline_layout,
            command_buffers,
            model,
        })
    }

    #[inline]
    pub fn window(&self) -> &Window {
        &self.window.window()
    }

    #[inline]
    pub fn window_resized(&mut self, width: i32, height: i32) {
        self.window.framebuffer_resized(width, height);
    }

    pub fn draw_frame(&mut self, control_flow: Option<&mut ControlFlow>) -> Result<()> {
        let (image_index, _) = match self.swap_chain.acquire_next_image(&self.device) {
            Ok((image_index, result)) => {
                if result {
                    let (swap_chain, pipeline) = Self::recreate_swap_chain(
                        &self.window,
                        &self.device,
                        &self.pipeline_layout,
                        self.swap_chain.swap_chain(),
                        &mut self.command_buffers,
                        control_flow,
                    )?;
                    unsafe { self.device_wait_idle() }.unwrap();
                    unsafe {
                        self.pipeline.destroy(&self.device);
                        self.swap_chain.destroy(&self.device);
                    }
                    (self.swap_chain, self.pipeline) = (swap_chain, pipeline);

                    return Ok(());
                }

                Ok((image_index, result)) as Result<(usize, bool)>
            }
            Err(_) => bail!("Failed to acquire swap chain image!"),
        }?;

        self.record_command_buffer(image_index)?;

        match self.swap_chain.submit_command_buffers(
            &self.device,
            &self.command_buffers[image_index],
            image_index,
        ) {
            Ok(window_resized) => {
                if window_resized || self.window.was_window_resized() {
                    self.window.reset_window_resized_flag();
                    let (swap_chain, pipeline) = Self::recreate_swap_chain(
                        &self.window,
                        &self.device,
                        &self.pipeline_layout,
                        self.swap_chain.swap_chain(),
                        &mut self.command_buffers,
                        control_flow,
                    )?;
                    unsafe { self.device_wait_idle() }.unwrap();
                    unsafe {
                        self.pipeline.destroy(&self.device);
                        self.swap_chain.destroy(&self.device);
                    }
                    (self.swap_chain, self.pipeline) = (swap_chain, pipeline);

                    return Ok(());
                }
            }

            Err(_) => {
                if self.window.was_window_resized() {
                    self.window.reset_window_resized_flag();
                    let (swap_chain, pipeline) = Self::recreate_swap_chain(
                        &self.window,
                        &self.device,
                        &self.pipeline_layout,
                        self.swap_chain.swap_chain(),
                        &mut self.command_buffers,
                        control_flow,
                    )?;
                    unsafe { self.device_wait_idle() }.unwrap();
                    unsafe {
                        self.pipeline.destroy(&self.device);
                        self.swap_chain.destroy(&self.device);
                    }
                    (self.swap_chain, self.pipeline) = (swap_chain, pipeline);

                    return Ok(());
                } else {
                    bail!("Failed to present swap chain image!")
                }
            }
        };

        Ok(())
    }

    #[inline]
    pub unsafe fn device_wait_idle(&self) -> Result<()> {
        Ok(self.device.device().device_wait_idle()?)
    }

    fn load_models(device: &lve_rs::Device) -> Result<Box<lve_rs::Model>> {
        let mut vertices = vec![];

        lve_rs::Vertex::serpinski(
            &mut vertices,
            &lve_rs::Vertex::new(&[0.0f32, -0.5f32], &[1.0, 0., 0.]),
            &lve_rs::Vertex::new(&[0.5f32, 0.5f32], &[0., 1., 0.]),
            &lve_rs::Vertex::new(&[-0.5f32, 0.5f32], &[0., 0., 1.]),
            0,
        );

        Ok(Box::new(lve_rs::Model::new(device, &vertices)?))
    }

    fn create_pipeline_layout(device: &lve_rs::Device) -> Result<vk::PipelineLayout> {
        let push_constant_range = vk::PushConstantRange::builder()
            .stage_flags(vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT)
            .offset(0)
            .size(size_of::<SimplePushConstantData>() as u32)
            .build();
        let create_info = vk::PipelineLayoutCreateInfo::builder()
            .set_layouts(&[])
            .push_constant_ranges(std::slice::from_ref(&push_constant_range));
        let pipeline_layout =
            unsafe { device.device().create_pipeline_layout(&create_info, None) }?;

        Ok(pipeline_layout)
    }

    fn create_pipeline(
        device: &lve_rs::Device,
        swap_chain: &lve_rs::SwapChain,
        pipeline_layout: &vk::PipelineLayout,
    ) -> Result<Box<lve_rs::Pipeline>> {
        let mut config_info =
            lve_rs::Pipeline::default_pipeline_config_info(swap_chain.width(), swap_chain.height());

        config_info.render_pass = *swap_chain.render_pass();
        config_info.pipeline_layout = *pipeline_layout;

        Ok(Box::new(lve_rs::Pipeline::new(
            &device,
            "./shaders/simple_shader.vert.spv",
            "./shaders/simple_shader.frag.spv",
            &config_info,
        )?))
    }

    fn recreate_swap_chain(
        window: &lve_rs::Window,
        device: &lve_rs::Device,
        pipeline_layout: &vk::PipelineLayout,
        swap_chain: &vk::SwapchainKHR,
        command_buffers: &mut Vec<vk::CommandBuffer>,
        mut control_flow: Option<&mut ControlFlow>,
    ) -> Result<(Box<lve_rs::SwapChain>, Box<lve_rs::Pipeline>)> {
        let device_ref = device.device();
        let mut extent = window.extent()?;

        while extent.width == 0 || extent.height == 0 {
            extent = window.extent()?;
            if let Some(ref mut control_flow_mut_ref) = control_flow {
                **control_flow_mut_ref = ControlFlow::Wait;
            }
        }
        // Wait until current swap chain is out of use
        unsafe { device_ref.device_wait_idle() }?;

        let swap_chain = if *swap_chain != vk::SwapchainKHR::null() {
            let swap_chain =
                lve_rs::SwapChain::with_previous_swap_chain(device, extent, swap_chain)?;

            if swap_chain.image_count() != command_buffers.len() {
                unsafe { Self::free_command_buffers(device, command_buffers) }

                *command_buffers = Self::create_command_buffers(device, &swap_chain)?;
            }

            swap_chain
        } else {
            lve_rs::SwapChain::new(device, extent)?
        };

        let pipeline = Self::create_pipeline(device, &swap_chain, pipeline_layout)?;

        Ok((Box::new(swap_chain), pipeline))
    }

    fn create_command_buffers(
        device: &lve_rs::Device,
        swap_chain: &lve_rs::SwapChain,
    ) -> Result<Vec<vk::CommandBuffer>> {
        let allocate_info = vk::CommandBufferAllocateInfo::builder()
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_pool(*device.command_pool())
            .command_buffer_count(swap_chain.image_count().try_into()?);
        let command_buffers = unsafe { device.device().allocate_command_buffers(&allocate_info) }?;

        Ok(command_buffers)
    }

    #[inline]
    unsafe fn free_command_buffers(
        device: &lve_rs::Device,
        command_buffers: &mut Vec<vk::CommandBuffer>,
    ) {
        device
            .device()
            .free_command_buffers(*device.command_pool(), command_buffers);
        command_buffers.clear()
    }

    fn record_command_buffer(&self, image_index: usize) -> Result<()> {
        static mut FRAME: i32 = 0;

        unsafe {
            FRAME = (FRAME + 1) % 1000;
        }
        let begin_info = vk::CommandBufferBeginInfo::builder();
        let clear_values = [
            vk::ClearValue {
                color: vk::ClearColorValue {
                    float32: [0.01f32, 0.01f32, 0.01f32, 1.0f32],
                },
            },
            vk::ClearValue {
                depth_stencil: vk::ClearDepthStencilValue::builder()
                    .depth(1.0f32)
                    .stencil(0)
                    .build(),
            },
        ];
        let render_pass_info = vk::RenderPassBeginInfo::builder()
            .render_pass(*self.swap_chain.render_pass())
            .framebuffer(*self.swap_chain.framebuffer(image_index))
            .render_area(vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0 },
                extent: self.swap_chain.swap_chain_extent(),
            })
            .clear_values(&clear_values);
        let viewport = vk::Viewport {
            x: 0.0,
            y: 0.0,
            width: self.swap_chain.width() as f32,
            height: self.swap_chain.height() as f32,
            min_depth: 0.0,
            max_depth: 1.0,
        };
        let scissor = vk::Rect2D {
            extent: self.swap_chain.swap_chain_extent(),
            offset: vk::Offset2D { x: 0, y: 0 },
        };
        let color_offset: u32 =
            (bytemuck::offset_of!(SimplePushConstantData, color) as u32 / 16 + 1) * 16;

        unsafe {
            self.device
                .device()
                .begin_command_buffer(self.command_buffers[image_index], &begin_info)
        }?;
        unsafe {
            self.device.device().cmd_begin_render_pass(
                self.command_buffers[image_index],
                &render_pass_info,
                vk::SubpassContents::INLINE,
            );
            self.device.device().cmd_set_viewport(
                self.command_buffers[image_index],
                0,
                std::slice::from_ref(&viewport),
            );
            self.device.device().cmd_set_scissor(
                self.command_buffers[image_index],
                0,
                std::slice::from_ref(&scissor),
            );
            self.pipeline
                .bind(&self.device, &self.command_buffers[image_index]);
            self.model
                .bind(&self.device, &self.command_buffers[image_index]);

            for j in 0..4 {
                let push = SimplePushConstantData {
                    offset: glm::vec2(-0.5 + FRAME as f32 * 0.002, -0.4 + j as f32 * 0.25),
                    color: glm::vec3(0.0, 0.0, 0.2 + 0.2 * j as f32),
                };

                self.device.device().cmd_push_constants(
                    self.command_buffers[image_index],
                    self.pipeline_layout,
                    vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT,
                    bytemuck::offset_of!(SimplePushConstantData, offset) as u32,
                    bytemuck::cast_slice(push.offset.as_slice()),
                );
                self.device.device().cmd_push_constants(
                    self.command_buffers[image_index],
                    self.pipeline_layout,
                    vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT,
                    color_offset,
                    bytemuck::cast_slice(push.color.as_slice()),
                );
                self.model
                    .draw(&self.device, &self.command_buffers[image_index]);
            }

            self.device
                .device()
                .cmd_end_render_pass(self.command_buffers[image_index]);
        }
        unsafe {
            self.device
                .device()
                .end_command_buffer(self.command_buffers[image_index])
        }?;

        Ok(())
    }
}

impl Drop for App {
    fn drop(&mut self) {
        unsafe {
            self.model.destroy(&self.device);
            if self.command_buffers.len() > 0 {
                self.device
                    .device()
                    .free_command_buffers(*self.device.command_pool(), &self.command_buffers);
            }
            self.device
                .device()
                .destroy_pipeline_layout(self.pipeline_layout, None);
            self.pipeline.destroy(&self.device);
            self.swap_chain.destroy(&self.device);
            self.device.destroy();
        }
    }
}
