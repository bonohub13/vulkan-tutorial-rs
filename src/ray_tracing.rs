use anyhow::Result;
use ash::vk;
use std::{cell::RefCell, mem::size_of, rc::Rc};
use winit::{
    event::VirtualKeyCode,
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};

extern crate nalgebra_glm as glm;

pub struct RayTracing {
    window: lve_rs::Window,
    device: lve_rs::Device,
    renderer: lve_rs::Renderer,
    world_render_system: lve_rs::WorldRenderSystem,
    camera: lve_rs::Camera,
    camera_controller: lve_rs::controller::keyboard::KeyboardMovementController,
    viewer_object: lve_rs::GameObject,
    global_pool: Box<lve_rs::DescriptorPool>,
    game_objects: lve_rs::Map,
    global_descriptor_sets: Vec<vk::DescriptorSet>,
    global_set_layout: Box<lve_rs::DescriptorSetLayout>,
    ubo_buffers: Vec<Box<lve_rs::Buffer>>,
}

impl RayTracing {
    pub const WIDTH: i32 = 1280;
    pub const HEIGHT: i32 = 800;

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
        let window = lve_rs::Window::new(
            event_loop,
            width,
            height,
            "Ray Tracing in One Weekend (CPU)",
        )?;
        let device = lve_rs::Device::new(&window, &lve_rs::ApplicationInfo::default())?;
        let renderer = lve_rs::Renderer::new(&window, &device)?;
        let global_pool = lve_rs::DescriptorPool::builder()
            .set_max_sets(lve_rs::SwapChain::MAX_FRAMES_IN_FLIGHT as u32)
            .add_pool_size(
                vk::DescriptorType::UNIFORM_BUFFER,
                lve_rs::SwapChain::MAX_FRAMES_IN_FLIGHT as u32,
            )
            .build(&device)?;
        let mut game_objects = lve_rs::Map::new();

        Self::load_game_object(&mut game_objects, &device)?;

        let mut camera = lve_rs::Camera::new();
        let camera_controller =
            lve_rs::controller::keyboard::KeyboardMovementController::new(9.0 * 4.0, 4.25 * 4.0);
        let mut viewer_object = { unsafe { lve_rs::GameObject::create_game_object(None) } };
        let global_set_layout = lve_rs::DescriptorSetLayout::builder()
            .add_binding(
                0,
                vk::DescriptorType::UNIFORM_BUFFER,
                vk::ShaderStageFlags::ALL_GRAPHICS,
                None,
            )
            .build(&device)?;
        let world = Self::load_world(&device)?;
        let world_render_system = lve_rs::WorldRenderSystem::new(
            &device,
            renderer.swap_chain_render_pass(),
            &global_set_layout.descriptor_set_layout(),
            world,
        )?;
        let mut ubo_buffers = Vec::with_capacity(lve_rs::SwapChain::MAX_FRAMES_IN_FLIGHT as usize);
        let mut global_descriptor_sets = vec![];

        for i in 0..lve_rs::SwapChain::MAX_FRAMES_IN_FLIGHT as usize {
            ubo_buffers.push(Box::new(lve_rs::Buffer::new(
                &device,
                size_of::<lve_rs::GlobalUbo>() as u64,
                1,
                vk::BufferUsageFlags::UNIFORM_BUFFER,
                vk::MemoryPropertyFlags::HOST_VISIBLE,
                Some(device.properties.limits.min_uniform_buffer_offset_alignment),
            )?));
            unsafe { ubo_buffers[i].map(&device, None, None) }?;
            let buffer_info = ubo_buffers[i].descriptor_info(None, None);
            global_descriptor_sets.push(
                unsafe {
                    lve_rs::DescriptorWriter::new(&global_set_layout, &global_pool)
                        .write_buffer(0, &buffer_info)
                        .build(&device)
                }
                .0,
            );
        }

        viewer_object.transform.translation.z = -2.5;
        camera.set_view_target(&[-1.0, -2.0, 2.0], &[0.0, 0.0, 2.5], None);

        Ok(Self {
            window,
            device,
            renderer,
            world_render_system,
            camera,
            camera_controller,
            viewer_object,
            global_pool,
            game_objects,
            global_descriptor_sets,
            global_set_layout,
            ubo_buffers,
        })
    }

    pub fn draw_frame(
        &mut self,
        mut control_flow: Option<&mut ControlFlow>,
        delta_time: f32,
        keys: &[Option<VirtualKeyCode>],
    ) -> Result<()> {
        let aspect = self.renderer.aspect_ratio();

        self.camera_controller
            .move_in_plane_xz(delta_time, &mut self.viewer_object, keys);
        self.camera.set_view_xyz(
            &[
                self.viewer_object.transform.translation.x,
                self.viewer_object.transform.translation.y,
                self.viewer_object.transform.translation.z,
            ],
            &[
                self.viewer_object.transform.rotation.x,
                self.viewer_object.transform.rotation.y,
                self.viewer_object.transform.rotation.z,
            ],
        );
        self.camera
            .set_perspective_projection(f32::to_radians(50.0), aspect, 0.1, 100.0);

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
            let frame_index = self.renderer.frame_index();
            let mut frame_info = lve_rs::FrameInfo {
                frame_index,
                frame_time: delta_time,
                command_buffer,
                camera: &self.camera,
                global_descriptor_set: self.global_descriptor_sets[frame_index],
                game_objects: &mut self.game_objects,
            };
            // update
            let ubo = lve_rs::GlobalUbo {
                projection: *self.camera.projection(),
                view: *self.camera.view(),
                inverse_view: *self.camera.inverse_view(),
                ..Default::default()
            };
            self.world_render_system.update(&frame_info);
            unsafe {
                self.ubo_buffers[frame_index].write_to_buffer(
                    &self.device,
                    std::slice::from_ref(&ubo),
                    None,
                    None,
                );
                self.ubo_buffers[frame_index].flush(&self.device, None, None)
            }?;
            // render
            unsafe {
                self.renderer
                    .begin_swap_chain_render_pass(&self.device, &command_buffer);
                self.world_render_system
                    .render(&self.device, &mut frame_info);
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

    fn load_game_object(game_objects: &mut lve_rs::Map, device: &lve_rs::Device) -> Result<()> {
        Ok(())
    }

    fn load_world(device: &lve_rs::Device) -> Result<Box<lve_rs::GameObject>> {
        let mut world = {
            let world = lve_rs::Model::create_model_from_file(device, "models/sphere.obj")?;

            unsafe { lve_rs::GameObject::create_game_object(Some(Rc::new(RefCell::new(*world)))) }
        };

        world.transform.scale = 75.0f32 * glm::vec3(1.0, 1.0, 1.0);
        world.transform.translation.y = 0.0;

        Ok(Box::new(world))
    }
}

impl Drop for RayTracing {
    fn drop(&mut self) {
        unsafe {
            self.global_set_layout.destroy(&self.device);
            self.ubo_buffers
                .iter_mut()
                .for_each(|ubo_buffer| ubo_buffer.destroy(&self.device));
            self.ubo_buffers.clear();
            for key in self.game_objects.keys() {
                if let Some(model) = &self.game_objects[key].model {
                    model.borrow_mut().destroy(&self.device);
                }
            }
            self.game_objects.clear();
            self.global_pool.destroy(&self.device);
            self.world_render_system.destroy(&self.device);
            self.renderer.destroy(&self.device);
            self.device.destroy();
        }
    }
}
