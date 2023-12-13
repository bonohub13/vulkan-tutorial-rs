use anyhow::{bail, Result};
use ash::vk;
use winit::{
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};

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
        let (swap_chain, pipeline) =
            Self::recreate_swap_chain(&window, &device, &pipeline_layout, None)?;
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
                    unsafe { self.device_wait_idle() }.unwrap();
                    unsafe {
                        self.pipeline.destroy(&self.device);
                        self.swap_chain.destroy(&self.device);
                    }
                    (self.swap_chain, self.pipeline) = Self::recreate_swap_chain(
                        &self.window,
                        &self.device,
                        &self.pipeline_layout,
                        control_flow,
                    )?;

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
                    unsafe { self.device_wait_idle() }.unwrap();
                    unsafe {
                        self.pipeline.destroy(&self.device);
                        self.swap_chain.destroy(&self.device);
                    }
                    self.window.reset_window_resized_flag();
                    (self.swap_chain, self.pipeline) = Self::recreate_swap_chain(
                        &self.window,
                        &self.device,
                        &self.pipeline_layout,
                        control_flow,
                    )?;

                    return Ok(());
                }
            }

            Err(_) => {
                if self.window.was_window_resized() {
                    unsafe { self.device_wait_idle() }.unwrap();
                    unsafe {
                        self.pipeline.destroy(&self.device);
                        self.swap_chain.destroy(&self.device);
                    }
                    self.window.reset_window_resized_flag();
                    (self.swap_chain, self.pipeline) = Self::recreate_swap_chain(
                        &self.window,
                        &self.device,
                        &self.pipeline_layout,
                        control_flow,
                    )?;

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
            &lve_rs::Vertex::new(&[0.0f32, -0.95f32], &[1.0, 0., 0.]),
            &lve_rs::Vertex::new(&[0.95f32, 0.95f32], &[0., 1., 0.]),
            &lve_rs::Vertex::new(&[-0.95f32, 0.95f32], &[0., 0., 1.]),
            8,
        );

        Ok(Box::new(lve_rs::Model::new(device, &vertices)?))
    }

    fn create_pipeline_layout(device: &lve_rs::Device) -> Result<vk::PipelineLayout> {
        let create_info = vk::PipelineLayoutCreateInfo::builder()
            .set_layouts(&[])
            .push_constant_ranges(&[]);
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

        let swap_chain = lve_rs::SwapChain::new(device, extent)?;
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

    fn record_command_buffer(&self, image_index: usize) -> Result<()> {
        let begin_info = vk::CommandBufferBeginInfo::builder();
        let clear_values = [
            vk::ClearValue {
                color: vk::ClearColorValue {
                    float32: [0.1f32, 0.1f32, 0.1f32, 1.0f32],
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
            self.pipeline
                .bind(&self.device, &self.command_buffers[image_index]);
            self.model
                .bind(&self.device, &self.command_buffers[image_index]);
            self.model
                .draw(&self.device, &self.command_buffers[image_index]);

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
