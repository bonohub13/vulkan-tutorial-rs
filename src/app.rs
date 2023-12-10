use anyhow::{bail, Result};
use ash::vk;
use winit::{event_loop::EventLoop, window::Window};

pub struct App {
    window: lve_rs::Window,
    device: lve_rs::Device,
    swap_chain: lve_rs::SwapChain,
    pipeline: Box<lve_rs::Pipeline>,
    pipeline_layout: vk::PipelineLayout,
    command_buffers: Vec<vk::CommandBuffer>,
}

impl App {
    pub const WIDTH: i32 = 800;
    pub const HEIGHT: i32 = 600;

    pub fn new<T>(event_loop: &EventLoop<T>) -> Result<Self> {
        let window = lve_rs::Window::new(event_loop, Self::WIDTH, Self::HEIGHT, "Hello Vulkan!")?;
        let device = lve_rs::Device::new(&window, &lve_rs::ApplicationInfo::default())?;
        let swap_chain = lve_rs::SwapChain::new(&device, window.extent()?)?;
        let pipeline_layout = Self::create_pipeline_layout(&device)?;
        let pipeline = Self::create_pipeline(&device, &swap_chain, &pipeline_layout)?;
        let command_buffers = Self::create_command_buffers(&device, &swap_chain, &pipeline)?;

        Ok(Self {
            window,
            device,
            swap_chain,
            pipeline,
            pipeline_layout,
            command_buffers,
        })
    }

    #[inline]
    pub fn window(&self) -> &Window {
        &self.window.window()
    }

    pub fn draw_frame(&mut self) -> Result<()> {
        let (image_index, result) = self.swap_chain.acquire_next_image(&self.device)?;

        if result {
            bail!("Failed to acquire swap chain image!");
        }

        let result = self.swap_chain.submit_command_buffers(
            &self.device,
            &self.command_buffers[image_index],
            image_index,
        )?;

        if result {
            bail!("Failed to present swap chain image!")
        }

        Ok(())
    }

    #[inline]
    pub unsafe fn device_wait_idle(&self) -> Result<()> {
        println!("Device waiting at idle");

        Ok(self.device.device().device_wait_idle()?)
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

    fn create_command_buffers(
        device: &lve_rs::Device,
        swap_chain: &lve_rs::SwapChain,
        pipeline: &lve_rs::Pipeline,
    ) -> Result<Vec<vk::CommandBuffer>> {
        let allocate_info = vk::CommandBufferAllocateInfo::builder()
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_pool(*device.command_pool())
            .command_buffer_count(swap_chain.image_count().try_into()?);
        let command_buffers = unsafe { device.device().allocate_command_buffers(&allocate_info) }?;
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

        for (index, command_buffer) in command_buffers.iter().enumerate() {
            let render_pass_info = vk::RenderPassBeginInfo::builder()
                .render_pass(*swap_chain.render_pass())
                .framebuffer(*swap_chain.framebuffer(index))
                .render_area(vk::Rect2D {
                    offset: vk::Offset2D { x: 0, y: 0 },
                    extent: swap_chain.swap_chain_extent(),
                })
                .clear_values(&clear_values);

            unsafe {
                device
                    .device()
                    .begin_command_buffer(*command_buffer, &begin_info)
            }?;
            unsafe {
                device.device().cmd_begin_render_pass(
                    *command_buffer,
                    &render_pass_info,
                    vk::SubpassContents::INLINE,
                );
                pipeline.bind(device, command_buffer);
                device.device().cmd_draw(*command_buffer, 3, 1, 0, 0);
                device.device().cmd_end_render_pass(*command_buffer);
            }
            unsafe { device.device().end_command_buffer(*command_buffer) }?;
        }

        Ok(command_buffers)
    }
}

impl Drop for App {
    fn drop(&mut self) {
        println!("Drop called in App");
        unsafe {
            println!("Dropping CommandBuffers");
            if self.command_buffers.len() > 0 {
                self.device
                    .device()
                    .free_command_buffers(*self.device.command_pool(), &self.command_buffers);
            }
            println!("Dropped CommandBuffers");
            println!("Dropping PipelineLayout");
            self.device
                .device()
                .destroy_pipeline_layout(self.pipeline_layout, None);
            println!("Dropped PipelineLayout");
            println!("Dropping Pipeline");
            self.pipeline.destroy(&self.device);
            println!("Dropped Pipeline");
            println!("Dropping SwapChain");
            self.swap_chain.destroy(&self.device);
            println!("Dropped SwapChain");
            println!("Dropping Device");
            self.device.destroy();
            println!("Dropped Device");
        }
    }
}
