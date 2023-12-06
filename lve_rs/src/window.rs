use anyhow::Result;
use winit::{
    dpi::LogicalSize,
    event_loop::EventLoop,
    window::{self, WindowBuilder},
};

pub struct Window {
    window: window::Window,
    window_name: Box<str>,
    width: i32,
    height: i32,
}

impl Window {
    pub fn new<T>(event_loop: &EventLoop<T>, width: i32, height: i32, name: &str) -> Result<Self> {
        let window = Self::init_window(event_loop, width, height, name)?;

        Ok(Self {
            window,
            window_name: Box::from(name),
            width,
            height,
        })
    }

    pub fn window(&self) -> &window::Window {
        &self.window
    }

    /* --- Helper functions --- */
    fn init_window<T>(
        event_loop: &EventLoop<T>,
        width: i32,
        height: i32,
        name: &str,
    ) -> Result<window::Window> {
        let window = WindowBuilder::new()
            .with_resizable(false)
            .with_inner_size(LogicalSize::new(width, height))
            .with_title(name)
            .build(event_loop)?;

        Ok(window)
    }
}
