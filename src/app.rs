use anyhow::Result;
use ash::vk;
use std::{cell::RefCell, rc::Rc};
use winit::{
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};

extern crate nalgebra_glm as glm;

pub struct App {
    window: lve_rs::Window,
    device: lve_rs::Device,
    renderer: lve_rs::Renderer,
    simple_render_system: lve_rs::SimpleRenderSystem,
    game_objects: Vec<lve_rs::GameObject>,
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
        let renderer = lve_rs::Renderer::new(&window, &device)?;
        let mut game_objects = vec![];

        Self::load_game_object(&mut game_objects, &device)?;

        let simple_render_system =
            lve_rs::SimpleRenderSystem::new(&device, renderer.swap_chain_render_pass())?;

        Ok(Self {
            window,
            device,
            renderer,
            simple_render_system,
            game_objects,
        })
    }

    pub fn draw_frame(&mut self, mut control_flow: Option<&mut ControlFlow>) -> Result<()> {
        let command_buffer = self.renderer.begin_frame(
            &self.window,
            &self.device,
            if let Some(ref mut cf_mut_ref) = control_flow {
                Some(*cf_mut_ref)
            } else {
                None
            },
        )?;

        if command_buffer != vk::CommandBuffer::null() {
            unsafe {
                self.renderer
                    .begin_swap_chain_render_pass(&self.device, &command_buffer);
                self.simple_render_system.render_game_objects(
                    &self.device,
                    command_buffer,
                    &mut self.game_objects,
                );
                self.renderer
                    .end_swap_chain_render_pass(&self.device, &command_buffer);
                self.renderer.end_frame(
                    &mut self.window,
                    &self.device,
                    if let Some(ref mut cf_mut_ref) = control_flow {
                        Some(*cf_mut_ref)
                    } else {
                        None
                    },
                )?;
            }
        }

        Ok(())
    }

    #[inline]
    pub fn window(&self) -> &Window {
        &self.window.window()
    }

    #[inline]
    pub fn window_resized(&mut self, width: i32, height: i32) {
        self.window.framebuffer_resized(width, height);
    }

    #[inline]
    pub unsafe fn device_wait_idle(&self) -> Result<()> {
        Ok(self.device.device().device_wait_idle()?)
    }

    fn load_game_object(
        game_objects: &mut Vec<lve_rs::GameObject>,
        device: &lve_rs::Device,
    ) -> Result<()> {
        let model = lve_rs::create_cube_model(device, &[0., 0., 0.])?;
        let mut cube =
            unsafe { lve_rs::GameObject::create_game_object(Rc::new(RefCell::new(*model))) };

        cube.transform.translation = glm::vec3(0., 0., 0.5);
        cube.transform.scale = glm::vec3(0.5, 0.5, 0.5);

        *game_objects = vec![cube];

        Ok(())
    }
}

impl Drop for App {
    fn drop(&mut self) {
        unsafe {
            for game_object in self.game_objects.iter() {
                game_object.model.borrow_mut().destroy(&self.device);
            }
            self.simple_render_system.destroy(&self.device);
            self.renderer.destroy(&self.device);
            self.device.destroy();
        }
    }
}
