use anyhow::Result;
use ash::vk;
use std::mem::size_of;
use std::{cell::RefCell, rc::Rc};
use winit::{
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};

extern crate nalgebra_glm as glm;

#[derive(Default)]
#[repr(C, align(16))]
pub struct SimplePushConstantData {
    transform: glm::Mat2,
    offset: glm::Vec2,
    color: glm::Vec3,
}

pub struct App {
    window: lve_rs::Window,
    device: lve_rs::Device,
    renderer: lve_rs::Renderer,
    pipeline: Box<lve_rs::Pipeline>,
    pipeline_layout: vk::PipelineLayout,
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

        let pipeline_layout = Self::create_pipeline_layout(&device)?;
        let pipeline = Self::create_pipeline(&device, &renderer, &pipeline_layout)?;

        Ok(Self {
            window,
            device,
            renderer,
            pipeline,
            pipeline_layout,
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
                self.render_game_objects(command_buffer);
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
        let vertices = lve_rs::Vertex::serpinski(
            &lve_rs::Vertex::new(&[0.0f32, -0.5f32], &[1.0, 0., 0.]),
            &lve_rs::Vertex::new(&[0.5f32, 0.5f32], &[0., 1., 0.]),
            &lve_rs::Vertex::new(&[-0.5f32, 0.5f32], &[0., 0., 1.]),
            0,
        );
        let model = lve_rs::Model::new(device, &vertices)?;
        let mut triangle =
            unsafe { lve_rs::GameObject::create_game_object(Rc::new(RefCell::new(model))) };

        triangle.color = glm::vec3(0.1, 0.8, 0.1);
        triangle.transform_2d.translation.x = 0.2;
        triangle.transform_2d.scale = glm::vec2(2.0, 0.5);
        triangle.transform_2d.rotation = 0.5 * std::f32::consts::PI;

        *game_objects = vec![triangle];

        Ok(())
    }

    fn create_pipeline_layout(device: &lve_rs::Device) -> Result<vk::PipelineLayout> {
        let push_constant_range = vk::PushConstantRange::builder()
            .stage_flags(vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT)
            .offset(0)
            .size(size_of::<SimplePushConstantData>() as u32)
            .build();
        let create_info = vk::PipelineLayoutCreateInfo::builder()
            .set_layouts(&[])
            .push_constant_ranges(std::slice::from_ref(&push_constant_range));
        let pipeline_layout =
            unsafe { device.device().create_pipeline_layout(&create_info, None) }?;

        Ok(pipeline_layout)
    }

    fn create_pipeline(
        device: &lve_rs::Device,
        renderer: &lve_rs::Renderer,
        pipeline_layout: &vk::PipelineLayout,
    ) -> Result<Box<lve_rs::Pipeline>> {
        assert!(
            *pipeline_layout != vk::PipelineLayout::null(),
            "Cannot create pipeline before pipeline layout"
        );

        let mut config_info = lve_rs::Pipeline::default_pipeline_config_info(
            renderer.swap_chain().width(),
            renderer.swap_chain().height(),
        );

        config_info.render_pass = *renderer.swap_chain_render_pass();
        config_info.pipeline_layout = *pipeline_layout;

        Ok(Box::new(lve_rs::Pipeline::new(
            &device,
            "./shaders/simple_shader.vert.spv",
            "./shaders/simple_shader.frag.spv",
            &config_info,
        )?))
    }

    unsafe fn render_game_objects(&mut self, command_buffer: vk::CommandBuffer) {
        for (i, game_object) in self.game_objects.iter_mut().enumerate() {
            game_object.transform_2d.rotation = glm::modf(
                game_object.transform_2d.rotation + 0.001 * (i + 1) as f32,
                2.0 * std::f32::consts::PI,
            );
        }

        self.pipeline.bind(&self.device, &command_buffer);
        for game_object in self.game_objects.iter_mut() {
            let push = SimplePushConstantData {
                transform: game_object.transform_2d.mat2(),
                offset: game_object.transform_2d.translation,
                color: game_object.color,
            };
            let offsets = {
                let transform = bytemuck::offset_of!(SimplePushConstantData, transform) as u32;
                let offset = bytemuck::offset_of!(SimplePushConstantData, offset) as u32;
                let color = bytemuck::offset_of!(SimplePushConstantData, color) as u32;
                let aligned_offset = |offset: u32| {
                    if offset % 16 == 0 {
                        offset
                    } else {
                        (offset / 16 + 1) * 16
                    }
                };

                [
                    aligned_offset(transform),
                    aligned_offset(offset),
                    aligned_offset(color),
                ]
            };

            self.device.device().cmd_push_constants(
                command_buffer,
                self.pipeline_layout,
                vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT,
                0,
                bytemuck::cast_slice(push.transform.as_slice()),
            );
            self.device.device().cmd_push_constants(
                command_buffer,
                self.pipeline_layout,
                vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT,
                offsets[1],
                bytemuck::cast_slice(push.offset.as_slice()),
            );
            self.device.device().cmd_push_constants(
                command_buffer,
                self.pipeline_layout,
                vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT,
                offsets[2],
                bytemuck::cast_slice(push.color.as_slice()),
            );
            game_object
                .model
                .borrow()
                .bind(&self.device, &command_buffer);
            game_object
                .model
                .borrow()
                .draw(&self.device, &command_buffer);
        }
    }
}

impl Drop for App {
    fn drop(&mut self) {
        unsafe {
            for game_object in self.game_objects.iter() {
                game_object.model.borrow_mut().destroy(&self.device);
            }
            self.device
                .device()
                .destroy_pipeline_layout(self.pipeline_layout, None);
            self.pipeline.destroy(&self.device);
            self.renderer.destroy(&self.device);
            self.device.destroy();
        }
    }
}
