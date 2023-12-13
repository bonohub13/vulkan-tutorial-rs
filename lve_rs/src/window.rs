use anyhow::Result;
use ash::vk;
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
    framebuffer_resized: bool,
}

impl Window {
    pub fn new<T>(event_loop: &EventLoop<T>, width: i32, height: i32, name: &str) -> Result<Self> {
        let window = Self::init_window(event_loop, width, height, name)?;

        Ok(Self {
            window,
            window_name: Box::from(name),
            width,
            height,
            framebuffer_resized: false,
        })
    }

    #[inline]
    pub fn window(&self) -> &window::Window {
        &self.window
    }

    #[inline]
    pub fn extent(&self) -> Result<vk::Extent2D> {
        Ok(vk::Extent2D {
            width: self.width.try_into()?,
            height: self.height.try_into()?,
        })
    }

    #[inline]
    pub fn was_window_resized(&self) -> bool {
        self.framebuffer_resized
    }

    #[inline]
    pub fn reset_window_resized_flag(&mut self) {
        self.framebuffer_resized = false
    }

    pub fn create_surface(
        &self,
        entry: &ash::Entry,
        instance: &ash::Instance,
    ) -> Result<crate::Surface> {
        crate::Surface::new(self, entry, instance)
    }

    pub fn framebuffer_resized(&mut self, width: i32, height: i32) {
        self.framebuffer_resized = true;
        self.width = width;
        self.height = height;
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
