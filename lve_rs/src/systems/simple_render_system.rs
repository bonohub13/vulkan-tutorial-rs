use anyhow::Result;
use ash::vk;

#[derive(Default)]
#[repr(C, align(16))]
pub struct SimplePushConstantData {
    model_matrix: glm::Mat4,
    normal_matrix: glm::Mat4,
}

pub struct SimpleRenderSystem {
    pipeline: Box<crate::GraphicsPipeline>,
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

    pub unsafe fn render(&self, device: &crate::Device, frame_info: &mut crate::FrameInfo) {
        let device_ref = device.device();

        self.pipeline.bind(device, &frame_info.command_buffer);
        device_ref.cmd_bind_descriptor_sets(
            frame_info.command_buffer,
            vk::PipelineBindPoint::GRAPHICS,
            self.pipeline_layout,
            0,
            std::slice::from_ref(
                &frame_info.descriptor_sets[&crate::PipelineIdentifier::GRAPHICS]
                    [frame_info.frame_index],
            ),
            &[],
        );
        device_ref.cmd_draw(frame_info.command_buffer, 6, 1, 0, 0);
    }

    fn create_pipeline_layout(
        device: &crate::Device,
        global_set_layout: &vk::DescriptorSetLayout,
    ) -> Result<vk::PipelineLayout> {
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
    ) -> Result<Box<crate::GraphicsPipeline>> {
        assert!(
            *pipeline_layout != vk::PipelineLayout::null(),
            "Cannot create pipeline before pipeline layout"
        );

        let mut config_info = crate::GraphicsPipeline::default_pipeline_config_info();

        config_info.binding_descriptions.clear();
        config_info.attribute_descriptions.clear();
        config_info.render_pass = *render_pass;
        config_info.pipeline_layout = *pipeline_layout;

        Ok(Box::new(crate::GraphicsPipeline::new(
            &device,
            "./shaders/simple_shader.vert.spv",
            "./shaders/simple_shader.frag.spv",
            &config_info,
        )?))
    }
}
