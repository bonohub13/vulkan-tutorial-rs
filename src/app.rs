use anyhow::Result;
use ash::vk;
use std::{cell::RefCell, collections::HashMap, rc::Rc};
use winit::{
    event::VirtualKeyCode,
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};

extern crate nalgebra_glm as glm;

pub struct App {
    window: lve_rs::Window,
    device: lve_rs::Device,
    renderer: lve_rs::Renderer,
    ray_tracer_system: lve_rs::RayTraceSystem,
    simple_render_system: lve_rs::SimpleRenderSystem,
    camera: lve_rs::Camera,
    camera_controller: lve_rs::controller::keyboard::KeyboardMovementController,
    viewer_object: lve_rs::GameObject,
    descriptor_pools: HashMap<lve_rs::PipelineIdentifier, Box<lve_rs::DescriptorPool>>,
    game_objects: lve_rs::Map,
    descriptor_sets: HashMap<lve_rs::PipelineIdentifier, Vec<vk::DescriptorSet>>,
    set_layouts: HashMap<lve_rs::PipelineIdentifier, Box<lve_rs::DescriptorSetLayout>>,
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
        let descriptor_pools = {
            let mut pools = HashMap::new();

            pools.insert(
                lve_rs::PipelineIdentifier::GRAPHICS,
                lve_rs::DescriptorPool::builder()
                    .set_max_sets(lve_rs::SwapChain::MAX_FRAMES_IN_FLIGHT as u32)
                    .add_pool_size(
                        vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
                        lve_rs::SwapChain::MAX_FRAMES_IN_FLIGHT as u32,
                    )
                    .build(&device)?,
            );
            pools.insert(
                lve_rs::PipelineIdentifier::COMPUTE,
                lve_rs::DescriptorPool::builder()
                    .set_max_sets(lve_rs::SwapChain::MAX_FRAMES_IN_FLIGHT as u32)
                    .add_pool_size(
                        vk::DescriptorType::STORAGE_IMAGE,
                        lve_rs::SwapChain::MAX_FRAMES_IN_FLIGHT as u32,
                    )
                    .build(&device)?,
            );

            pools
        };
        let mut game_objects = lve_rs::Map::new();

        Self::load_game_object(&mut game_objects, &device)?;

        let mut camera = lve_rs::Camera::new();
        let camera_controller =
            lve_rs::controller::keyboard::KeyboardMovementController::new(9.0 * 2.0, 4.25 * 2.0);
        let mut viewer_object = { unsafe { lve_rs::GameObject::create_game_object(None) } };
        let set_layouts = {
            let mut set_layouts = HashMap::new();

            set_layouts.insert(
                lve_rs::PipelineIdentifier::GRAPHICS,
                lve_rs::DescriptorSetLayout::builder()
                    .add_binding(
                        0,
                        vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
                        vk::ShaderStageFlags::ALL_GRAPHICS,
                        None,
                    )
                    .build(&device)?,
            );
            set_layouts.insert(
                lve_rs::PipelineIdentifier::COMPUTE,
                lve_rs::DescriptorSetLayout::builder()
                    .add_binding(
                        0,
                        vk::DescriptorType::STORAGE_IMAGE,
                        vk::ShaderStageFlags::COMPUTE,
                        None,
                    )
                    .build(&device)?,
            );

            set_layouts
        };
        let ray_tracer_system = lve_rs::RayTraceSystem::new(
            &device,
            &set_layouts[&lve_rs::PipelineIdentifier::COMPUTE].descriptor_set_layout(),
        )?;
        let simple_render_system = lve_rs::SimpleRenderSystem::new(
            &device,
            renderer.swap_chain_render_pass(),
            &set_layouts[&lve_rs::PipelineIdentifier::GRAPHICS].descriptor_set_layout(),
        )?;
        let mut descriptor_sets = HashMap::new();
        let mut graphics_descriptor_sets = vec![];
        let mut compute_descriptor_sets = vec![];

        for i in 0..lve_rs::SwapChain::MAX_FRAMES_IN_FLIGHT as usize {
            compute_descriptor_sets.push(
                unsafe {
                    lve_rs::DescriptorWriter::new(
                        &set_layouts[&lve_rs::PipelineIdentifier::COMPUTE],
                        &descriptor_pools[&lve_rs::PipelineIdentifier::COMPUTE],
                    )
                    .write_image(0, &renderer.swap_chain().write_descriptors[i])
                    .build(&device)
                }
                .0,
            );
            graphics_descriptor_sets.push(
                unsafe {
                    lve_rs::DescriptorWriter::new(
                        &set_layouts[&lve_rs::PipelineIdentifier::GRAPHICS],
                        &descriptor_pools[&lve_rs::PipelineIdentifier::GRAPHICS],
                    )
                    .write_image(0, &renderer.swap_chain().read_descriptors[i])
                    .build(&device)
                }
                .0,
            );
        }
        descriptor_sets.insert(lve_rs::PipelineIdentifier::COMPUTE, compute_descriptor_sets);
        descriptor_sets.insert(
            lve_rs::PipelineIdentifier::GRAPHICS,
            graphics_descriptor_sets,
        );

        viewer_object.transform.translation.z = -2.5;
        camera.set_view_target(&[-1.0, -2.0, 2.0], &[0.0, 0.0, 2.5], None);

        Ok(Self {
            window,
            device,
            renderer,
            ray_tracer_system,
            simple_render_system,
            camera,
            camera_controller,
            viewer_object,
            descriptor_pools,
            game_objects,
            descriptor_sets,
            set_layouts,
        })
    }

    pub fn draw_frame(
        &mut self,
        mut control_flow: Option<&mut ControlFlow>,
        delta_time: f32,
        keys: &[Option<VirtualKeyCode>],
    ) -> Result<()> {
        let aspect = self.renderer.aspect_ratio();
        let write_descriptors = [
            lve_rs::DescriptorWriter::new(
                &self.set_layouts[&lve_rs::PipelineIdentifier::COMPUTE],
                &self.descriptor_pools[&lve_rs::PipelineIdentifier::COMPUTE],
            ),
            lve_rs::DescriptorWriter::new(
                &self.set_layouts[&lve_rs::PipelineIdentifier::GRAPHICS],
                &self.descriptor_pools[&lve_rs::PipelineIdentifier::GRAPHICS],
            ),
        ];

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
            |device, swap_chain, frame_index| unsafe {
                write_descriptors[0]
                    .write_image(0, &swap_chain.write_descriptors[frame_index])
                    .overwrite(
                        device,
                        &self.descriptor_sets[&lve_rs::PipelineIdentifier::COMPUTE][frame_index],
                    );
                write_descriptors[1]
                    .write_image(0, &swap_chain.read_descriptors[frame_index])
                    .overwrite(
                        device,
                        &self.descriptor_sets[&lve_rs::PipelineIdentifier::GRAPHICS][frame_index],
                    );
            },
        )?;

        if command_buffer != vk::CommandBuffer::null() {
            let frame_index = self.renderer.frame_index();
            let mut frame_info = lve_rs::FrameInfo {
                frame_index,
                frame_time: delta_time,
                command_buffer,
                camera: &self.camera,
                descriptor_sets: &self.descriptor_sets,
                screen_size: &self.renderer.swap_chain().swap_chain_extent(),
                game_objects: &mut self.game_objects,
            };
            // Compute
            unsafe {
                self.renderer
                    .prepare_to_trace_barrier(&self.device, &command_buffer);
                self.ray_tracer_system.dispatch(&self.device, &frame_info);
                self.renderer.enforce_barrier(&self.device, &command_buffer);
            }
            // Graphics
            //  render
            unsafe {
                self.renderer
                    .begin_swap_chain_render_pass(&self.device, &command_buffer);
                self.simple_render_system
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
        let mut smooth_vase = {
            let model = lve_rs::Model::create_model_from_file(device, "models/smooth_vase.obj")?;

            unsafe { lve_rs::GameObject::create_game_object(Some(Rc::new(RefCell::new(*model)))) }
        };
        let mut flat_vase = {
            let model = lve_rs::Model::create_model_from_file(device, "models/flat_vase.obj")?;

            unsafe { lve_rs::GameObject::create_game_object(Some(Rc::new(RefCell::new(*model)))) }
        };
        let mut floor = {
            let model = lve_rs::Model::create_model_from_file(device, "models/quad.obj")?;

            unsafe { lve_rs::GameObject::create_game_object(Some(Rc::new(RefCell::new(*model)))) }
        };
        let light_colors = [
            glm::vec3(1.0, 0.1, 0.1),
            glm::vec3(0.1, 0.1, 1.0),
            glm::vec3(0.1, 1.0, 0.1),
            glm::vec3(1.0, 1.0, 0.1),
            glm::vec3(0.1, 1.0, 1.0),
            glm::vec3(1.0, 1.0, 1.0),
        ];
        for (i, light_color) in light_colors.iter().enumerate() {
            let mut point_light = unsafe {
                lve_rs::GameObject::make_point_light(Some(0.2), None, Some(*light_color))
            };
            let rotate_light = glm::rotate(
                &glm::Mat4::identity(),
                (i as f32 * 2.0 * std::f32::consts::PI) / light_colors.len() as f32,
                &glm::vec3(0.0, -1.0, 0.0),
            );

            point_light.transform.translation =
                (rotate_light * glm::vec4(-1.0, -1.0, -1.0, 1.0)).xyz();
            game_objects.insert(point_light.id(), point_light);
        }

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

impl Drop for App {
    fn drop(&mut self) {
        unsafe {
            self.set_layouts
                .iter_mut()
                .for_each(|(_, set_layout)| set_layout.destroy(&self.device));
            for key in self.game_objects.keys() {
                if let Some(model) = &self.game_objects[key].model {
                    model.borrow_mut().destroy(&self.device);
                }
            }
            self.game_objects.clear();
            self.descriptor_pools
                .iter_mut()
                .for_each(|(_, pool)| pool.destroy(&self.device));
            self.simple_render_system.destroy(&self.device);
            self.ray_tracer_system.destroy(&self.device);
            self.renderer.destroy(&self.device);
            self.device.destroy();
        }
    }
}
