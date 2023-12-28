use anyhow::Result;
use ash::vk;
use std::mem::size_of;

pub struct PointLightSystem {
    pipeline: Box<crate::Pipeline>,
    pipeline_layout: vk::PipelineLayout,
}

impl PointLightSystem {
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

    pub unsafe fn render(&self, device: &crate::Device, frame_info: &crate::FrameInfo) {
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
        device_ref.cmd_draw(frame_info.command_buffer, 6, 1, 0, 0);
    }

    fn create_pipeline_layout(
        device: &crate::Device,
        global_set_layout: &vk::DescriptorSetLayout,
    ) -> Result<vk::PipelineLayout> {
        // let push_constant_range = vk::PushConstantRange::builder()
        //     .stage_flags(vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT)
        //     .offset(0)
        //     .size(sizeof(SimplePushConstantData))
        //     .build();
        let descriptor_set_layouts = vec![*global_set_layout];
        let create_info =
            vk::PipelineLayoutCreateInfo::builder().set_layouts(&descriptor_set_layouts);
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

        config_info.binding_descriptions.clear();
        config_info.attribute_descriptions.clear();
        config_info.render_pass = *render_pass;
        config_info.pipeline_layout = *pipeline_layout;

        Ok(Box::new(crate::Pipeline::new(
            &device,
            "./shaders/point_light.vert.spv",
            "./shaders/point_light.frag.spv",
            &config_info,
        )?))
    }
}
