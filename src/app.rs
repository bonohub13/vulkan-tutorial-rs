use anyhow::Result;
use ash::vk;
use std::{cell::RefCell, mem::size_of, rc::Rc};
use winit::{
    event::VirtualKeyCode,
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};

extern crate nalgebra_glm as glm;

#[derive(Debug, Clone, Copy)]
pub struct GlobalUbo {
    pub projection: glm::Mat4,
    pub view: glm::Mat4,
    pub ambient_light_color: glm::Vec4,
    pub light_position: glm::Vec4, // Does not work with glm::Vec3
    pub light_color: glm::Vec4,
}

pub struct App {
    window: lve_rs::Window,
    device: lve_rs::Device,
    renderer: lve_rs::Renderer,
    simple_render_system: lve_rs::SimpleRenderSystem,
    point_light_system: lve_rs::PointLightSystem,
    camera: lve_rs::Camera,
    camera_controller: lve_rs::controller::keyboard::KeyboardMovementController,
    viewer_object: lve_rs::GameObject,
    global_pool: Box<lve_rs::DescriptorPool>,
    game_objects: lve_rs::Map,
    global_descriptor_sets: Vec<vk::DescriptorSet>,
    global_set_layout: Box<lve_rs::DescriptorSetLayout>,
    ubo_buffers: Vec<Box<lve_rs::Buffer>>,
}

impl App {
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
        let window = lve_rs::Window::new(event_loop, width, height, "Hello Vulkan!")?;
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
            lve_rs::controller::keyboard::KeyboardMovementController::new(9.0 * 2.0, 4.25 * 2.0);
        let mut viewer_object = {
            let viewer = lve_rs::Model::builder()
                .vertices(&[
                    lve_rs::Vertex::new(&[0.0, 0.0, 0.0], &[0.0, 0.0, 0.0]),
                    lve_rs::Vertex::new(&[0.0, 0.0, 0.0], &[0.0, 0.0, 0.0]),
                    lve_rs::Vertex::new(&[0.0, 0.0, 0.0], &[0.0, 0.0, 0.0]),
                ])
                .build(&device)?;

            unsafe { lve_rs::GameObject::create_game_object(Rc::new(RefCell::new(viewer))) }
        };
        let global_set_layout = lve_rs::DescriptorSetLayout::builder()
            .add_binding(
                0,
                vk::DescriptorType::UNIFORM_BUFFER,
                vk::ShaderStageFlags::ALL_GRAPHICS,
                None,
            )
            .build(&device)?;
        let simple_render_system = lve_rs::SimpleRenderSystem::new(
            &device,
            renderer.swap_chain_render_pass(),
            &global_set_layout.descriptor_set_layout(),
        )?;
        let point_light_system = lve_rs::PointLightSystem::new(
            &device,
            renderer.swap_chain_render_pass(),
            &global_set_layout.descriptor_set_layout(),
        )?;
        let mut ubo_buffers = Vec::with_capacity(lve_rs::SwapChain::MAX_FRAMES_IN_FLIGHT as usize);
        let mut global_descriptor_sets = vec![];

        for i in 0..lve_rs::SwapChain::MAX_FRAMES_IN_FLIGHT as usize {
            ubo_buffers.push(Box::new(lve_rs::Buffer::new(
                &device,
                size_of::<GlobalUbo>() as u64,
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
            simple_render_system,
            point_light_system,
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
                game_objects: &self.game_objects,
            };
            // update
            let ubo = GlobalUbo {
                projection: *self.camera.projection(),
                view: *self.camera.view(),
                ..Default::default()
            };

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
                self.simple_render_system
                    .render_game_objects(&self.device, &mut frame_info);
                self.point_light_system.render(&self.device, &frame_info);
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
        let mut smooth_vase = {
            let model = lve_rs::Model::create_model_from_file(device, "models/smooth_vase.obj")?;

            unsafe { lve_rs::GameObject::create_game_object(Rc::new(RefCell::new(*model))) }
        };
        let mut flat_vase = {
            let model = lve_rs::Model::create_model_from_file(device, "models/flat_vase.obj")?;

            unsafe { lve_rs::GameObject::create_game_object(Rc::new(RefCell::new(*model))) }
        };
        let mut floor = {
            let model = lve_rs::Model::create_model_from_file(device, "models/quad.obj")?;

            unsafe { lve_rs::GameObject::create_game_object(Rc::new(RefCell::new(*model))) }
        };

        smooth_vase.transform.translation = glm::vec3(0.5, 0.5, 0.0);
        smooth_vase.transform.scale = 3.0f32 * glm::vec3(1.0, 0.5, 1.0);
        flat_vase.transform.translation = glm::vec3(-0.5, 0.5, 0.0);
        flat_vase.transform.scale = 3.0f32 * glm::vec3(1.0, 0.5, 1.0);
        floor.transform.translation = glm::vec3(0., 0.5, 0.);
        floor.transform.scale = glm::vec3(3.0, 1.0, 3.0);

        game_objects.insert(smooth_vase.id(), smooth_vase);
        game_objects.insert(flat_vase.id(), flat_vase);
        game_objects.insert(floor.id(), floor);

        Ok(())
    }
}

impl Default for GlobalUbo {
    fn default() -> Self {
        Self {
            projection: glm::Mat4::identity(),
            view: glm::Mat4::identity(),
            ambient_light_color: glm::vec4(1.0, 1.0, 1.0, 0.2),
            light_position: glm::vec4(-1.0, -1.0, -1.0, -1.0),
            light_color: glm::vec4(1.0, 1.0, 1.0, 1.0),
        }
    }
}

impl Drop for App {
    fn drop(&mut self) {
        unsafe {
            self.global_set_layout.destroy(&self.device);
            self.ubo_buffers
                .iter_mut()
                .for_each(|ubo_buffer| ubo_buffer.destroy(&self.device));
            self.ubo_buffers.clear();
            for key in self.game_objects.keys() {
                self.game_objects[key]
                    .model
                    .borrow_mut()
                    .destroy(&self.device);
            }
            self.game_objects.clear();
            self.global_pool.destroy(&self.device);
            self.viewer_object.model.borrow_mut().destroy(&self.device);
            self.point_light_system.destroy(&self.device);
            self.simple_render_system.destroy(&self.device);
            self.renderer.destroy(&self.device);
            self.device.destroy();
        }
    }
}
