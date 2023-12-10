use anyhow::Result;
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
        let swap_chain = lve_rs::SwapChain::new(&device, window.extent())?;
        let pipeline_layout = Self::create_pipeline_layout(&device)?;
        let pipeline = Self::create_pipeline(&device, &swap_chain, &pipeline_layout)?;

        Ok(Self {
            window,
            device,
            swap_chain,
            pipeline,
            pipeline_layout,
            command_buffers: vec![],
        })
    }

    #[inline]
    pub fn window(&self) -> &Window {
        &self.window.window()
    }

    pub fn run(&self) {}

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

    fn create_command_buffers() {}

    fn draw_frame() {}
}

impl Drop for App {
    fn drop(&mut self) {
        unsafe {
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
