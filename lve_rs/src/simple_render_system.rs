use anyhow::Result;
use ash::vk;
use std::mem::size_of;

#[derive(Default)]
#[repr(C, align(16))]
pub struct SimplePushConstantData {
    transform: glm::Mat4,
    color: glm::Vec3,
}

pub struct SimpleRenderSystem {
    pipeline: Box<crate::Pipeline>,
    pipeline_layout: vk::PipelineLayout,
}

impl SimpleRenderSystem {
    pub fn new(device: &crate::Device, render_pass: &vk::RenderPass) -> Result<Self> {
        let pipeline_layout = Self::create_pipeline_layout(device)?;
        let pipeline = Self::create_pipeline(device, &pipeline_layout, render_pass)?;
        Ok(Self {
            pipeline_layout,
            pipeline,
        })
    }

    pub unsafe fn destroy(&mut self, device: &crate::Device) {
        self.pipeline.destroy(device);
        device
            .device()
            .destroy_pipeline_layout(self.pipeline_layout, None);
    }

    pub unsafe fn render_game_objects(
        &self,
        device: &crate::Device,
        command_buffer: vk::CommandBuffer,
        game_objects: &mut Vec<crate::GameObject>,
    ) {
        for game_object in game_objects.iter_mut() {
            game_object.transform.rotation.y = glm::modf(
                game_object.transform.rotation.y + 0.01,
                2.0 * std::f32::consts::PI,
            );
            game_object.transform.rotation.x = glm::modf(
                game_object.transform.rotation.x + 0.005,
                2.0 * std::f32::consts::PI,
            );
        }

        self.pipeline.bind(device, &command_buffer);
        for game_object in game_objects.iter_mut() {
            let push = SimplePushConstantData {
                transform: game_object.transform.mat4(),
                color: game_object.color,
            };
            let offsets = {
                let transform = bytemuck::offset_of!(SimplePushConstantData, transform) as u32;
                let color = bytemuck::offset_of!(SimplePushConstantData, color) as u32;
                let aligned_offset = |offset: u32| {
                    if offset % 16 == 0 {
                        offset
                    } else {
                        (offset / 16 + 1) * 16
                    }
                };

                [aligned_offset(transform), aligned_offset(color)]
            };

            device.device().cmd_push_constants(
                command_buffer,
                self.pipeline_layout,
                vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT,
                0,
                bytemuck::cast_slice(push.transform.as_slice()),
            );
            device.device().cmd_push_constants(
                command_buffer,
                self.pipeline_layout,
                vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT,
                offsets[1],
                bytemuck::cast_slice(push.color.as_slice()),
            );
            game_object.model.borrow().bind(device, &command_buffer);
            game_object.model.borrow().draw(device, &command_buffer);
        }
    }

    fn create_pipeline_layout(device: &crate::Device) -> Result<vk::PipelineLayout> {
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
        device: &crate::Device,
        pipeline_layout: &vk::PipelineLayout,
        render_pass: &vk::RenderPass,
    ) -> Result<Box<crate::Pipeline>> {
        assert!(
            *pipeline_layout != vk::PipelineLayout::null(),
            "Cannot create pipeline before pipeline layout"
        );

        let mut config_info = crate::Pipeline::default_pipeline_config_info();

        config_info.render_pass = *render_pass;
        config_info.pipeline_layout = *pipeline_layout;

        Ok(Box::new(crate::Pipeline::new(
            &device,
            "./shaders/simple_shader.vert.spv",
            "./shaders/simple_shader.frag.spv",
            &config_info,
        )?))
    }
}
