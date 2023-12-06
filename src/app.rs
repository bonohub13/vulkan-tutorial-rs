use anyhow::Result;
use winit::{event_loop::EventLoop, window::Window};

pub struct App {
    window: lve_rs::Window,
    pipeline: lve_rs::Pipeline,
}

impl App {
    pub const WIDTH: i32 = 800;
    pub const HEIGHT: i32 = 600;

    pub fn new<T>(event_loop: &EventLoop<T>) -> Result<Self> {
        let window = lve_rs::Window::new(event_loop, Self::WIDTH, Self::HEIGHT, "Hello Vulkan!")?;
        let pipeline = lve_rs::Pipeline::new(
            "./shaders/simple_shader.vert.spv",
            "./shaders/simple_shader.frag.spv",
        )?;

        Ok(Self { window, pipeline })
    }

    pub fn window(&self) -> &Window {
        &self.window.window()
    }

    pub fn run(&self) {}
}
