use anyhow::Result;
use std::mem::ManuallyDrop;
use winit::{event_loop::EventLoop, window::Window};

pub struct App {
    window: lve_rs::Window,
    device: lve_rs::Device,
    pipeline: lve_rs::Pipeline,
}

impl App {
    pub const WIDTH: i32 = 800;
    pub const HEIGHT: i32 = 600;

    pub fn new<T>(event_loop: &EventLoop<T>) -> Result<Self> {
        let window = lve_rs::Window::new(event_loop, Self::WIDTH, Self::HEIGHT, "Hello Vulkan!")?;
        let device = lve_rs::Device::new(&window, &lve_rs::ApplicationInfo::default())?;
        let pipeline = lve_rs::Pipeline::new(
            &device,
            "./shaders/simple_shader.vert.spv",
            "./shaders/simple_shader.frag.spv",
            &lve_rs::Pipeline::default_pipeline_config_info(
                Self::WIDTH as u32,
                Self::HEIGHT as u32,
            ),
        )?;

        Ok(Self {
            window,
            device,
            pipeline,
        })
    }

    pub fn window(&self) -> &Window {
        &self.window.window()
    }

    pub fn run(&self) {}
}

impl Drop for App {
    fn drop(&mut self) {
        unsafe {
            self.pipeline.destroy(&self.device);
        }
        let _ = ManuallyDrop::new(&mut self.device);
    }
}
