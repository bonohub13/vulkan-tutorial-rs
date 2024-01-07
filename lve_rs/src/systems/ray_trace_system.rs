use anyhow::Result;
use ash::vk;

pub struct RayTraceSystem {
    pipeline: Box<crate::ComputePipeline>,
    pipeline_layout: vk::PipelineLayout,
}

impl RayTraceSystem {
    pub fn new(device: &crate::Device, set_layout: &vk::DescriptorSetLayout) -> Result<Self> {
        let pipeline_layout = Self::create_pipeline_layout(device, set_layout)?;
        let pipeline = Box::new(crate::ComputePipeline::new(
            device,
            &pipeline_layout,
            "./shaders/ray_trace.comp.spv",
        )?);

        Ok(Self {
            pipeline,
            pipeline_layout,
        })
    }

    pub unsafe fn destroy(&mut self, device: &crate::Device) {
        let device_ref = device.device();

        self.pipeline.destroy(device);
        device_ref.destroy_pipeline_layout(self.pipeline_layout, None);
    }

    pub unsafe fn dispatch(&self, device: &crate::Device, frame_info: &crate::FrameInfo) {
        let device_ref = device.device();

        self.pipeline.bind(device, &frame_info.command_buffer);
        device_ref.cmd_bind_descriptor_sets(
            frame_info.command_buffer,
            vk::PipelineBindPoint::COMPUTE,
            self.pipeline_layout,
            0,
            std::slice::from_ref(
                &frame_info.descriptor_sets[&crate::PipelineIdentifier::COMPUTE]
                    [frame_info.frame_index],
            ),
            &[],
        );
        self.pipeline
            .dispatch(device, &frame_info.command_buffer, frame_info.screen_size)
    }

    fn create_pipeline_layout(
        device: &crate::Device,
        set_layout: &vk::DescriptorSetLayout,
    ) -> Result<vk::PipelineLayout> {
        let device = device.device();
        let set_layouts = [*set_layout];
        let create_info = vk::PipelineLayoutCreateInfo::builder().set_layouts(&set_layouts);
        let pipeline_layout = unsafe { device.create_pipeline_layout(&create_info, None) }?;

        Ok(pipeline_layout)
    }
}
