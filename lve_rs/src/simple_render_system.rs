use anyhow::Result;
use ash::vk;
use std::mem::size_of;

#[derive(Default)]
#[repr(C, align(16))]
pub struct SimplePushConstantData {
    model_matrix: glm::Mat4,
    normal_matrix: glm::Mat4,
}

pub struct SimpleRenderSystem {
    pipeline: Box<crate::Pipeline>,
    pipeline_layout: vk::PipelineLayout,
}

impl SimpleRenderSystem {
    pub fn new(
        device: &crate::Device,
        render_pass: &vk::RenderPass,
        global_set_layout: &vk::DescriptorSetLayout,
    ) -> Result<Self> {
        let pipeline_layout = Self::create_pipeline_layout(device, global_set_layout)?;
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
        frame_info: &mut crate::FrameInfo,
    ) {
        let device_ref = device.device();

        self.pipeline.bind(device, &frame_info.command_buffer);
        device_ref.cmd_bind_descriptor_sets(
            frame_info.command_buffer,
            vk::PipelineBindPoint::GRAPHICS,
            self.pipeline_layout,
            0,
            std::slice::from_ref(&frame_info.global_descriptor_set),
            &[],
        );
        for key in frame_info.game_objects.keys() {
            let push = SimplePushConstantData {
                model_matrix: frame_info.game_objects[key].transform.mat4(),
                normal_matrix: frame_info.game_objects[key].transform.normal_matrix(),
            };
            let offsets = {
                let transform = bytemuck::offset_of!(SimplePushConstantData, model_matrix) as u32;
                let color = bytemuck::offset_of!(SimplePushConstantData, normal_matrix) as u32;
                let aligned_offset = |offset: u32| {
                    if offset % 16 == 0 {
                        offset
                    } else {
                        (offset / 16 + 1) * 16
                    }
                };

                [aligned_offset(transform), aligned_offset(color)]
            };

            device_ref.cmd_push_constants(
                frame_info.command_buffer,
                self.pipeline_layout,
                vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT,
                0,
                bytemuck::cast_slice(push.model_matrix.as_slice()),
            );
            device_ref.cmd_push_constants(
                frame_info.command_buffer,
                self.pipeline_layout,
                vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT,
                offsets[1],
                bytemuck::cast_slice(push.normal_matrix.as_slice()),
            );
            frame_info.game_objects[key]
                .model
                .borrow()
                .bind(device, &frame_info.command_buffer);
            frame_info.game_objects[key]
                .model
                .borrow()
                .draw(device, &frame_info.command_buffer);
        }
    }

    fn create_pipeline_layout(
        device: &crate::Device,
        global_set_layout: &vk::DescriptorSetLayout,
    ) -> Result<vk::PipelineLayout> {
        let push_constant_range = vk::PushConstantRange::builder()
            .stage_flags(vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT)
            .offset(0)
            .size(size_of::<SimplePushConstantData>() as u32)
            .build();
        let descriptor_set_layouts = vec![*global_set_layout];
        let create_info = vk::PipelineLayoutCreateInfo::builder()
            .set_layouts(&descriptor_set_layouts)
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
