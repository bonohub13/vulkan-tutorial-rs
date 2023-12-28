use anyhow::Result;
use ash::vk;
use std::mem::size_of;

#[repr(C, align(16))]
pub struct PointLightPushConstants {
    pub position: glm::Vec4,
    pub color: glm::Vec4,
    pub radius: f32,
}

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

    pub fn update(&self, frame_info: &mut crate::FrameInfo, ubo: &mut crate::GlobalUbo) {
        let rotate_light = glm::rotate(
            &glm::Mat4::identity(),
            20.0 * frame_info.frame_time,
            &glm::vec3(0.0, -1.0, 0.0),
        );
        let mut light_index = 0;

        for kv in frame_info.game_objects.iter_mut() {
            let obj = kv.1;

            if let Some(point_light) = obj.point_light {
                assert!(
                    light_index < crate::frame_info::MAX_LIGHT,
                    "Point lights exceed maximum specified"
                );

                obj.transform.translation = (rotate_light
                    * glm::vec4(
                        obj.transform.translation.x,
                        obj.transform.translation.y,
                        obj.transform.translation.z,
                        1.0,
                    ))
                .xyz();
                ubo.point_lights[light_index].position = glm::vec4(
                    obj.transform.translation.x,
                    obj.transform.translation.y,
                    obj.transform.translation.z,
                    1.0,
                );
                ubo.point_lights[light_index].color = glm::vec4(
                    obj.color.x,
                    obj.color.y,
                    obj.color.z,
                    point_light.light_intensity,
                );
                light_index += 1;
            }
        }
        ubo.num_lights = light_index as i32;
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

        for kv in frame_info.game_objects.iter() {
            let obj = &kv.1;

            if let Some(point_light) = &obj.point_light {
                let push = PointLightPushConstants {
                    position: glm::vec4(
                        obj.transform.translation.x,
                        obj.transform.translation.y,
                        obj.transform.translation.z,
                        1.0,
                    ),
                    color: glm::vec4(
                        obj.color.x,
                        obj.color.y,
                        obj.color.z,
                        point_light.light_intensity,
                    ),
                    radius: obj.transform.scale.x,
                };
                let push_constant_offsets = {
                    let color = bytemuck::offset_of!(push, PointLightPushConstants, color) as u32;
                    let radius = bytemuck::offset_of!(push, PointLightPushConstants, radius) as u32;
                    let aligned_offset = |offset: u32| {
                        if offset % 16 == 0 {
                            offset
                        } else {
                            (offset / 16 + 1) * 16
                        }
                    };

                    [0, aligned_offset(color), aligned_offset(radius)]
                };

                device_ref.cmd_push_constants(
                    frame_info.command_buffer,
                    self.pipeline_layout,
                    vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT,
                    push_constant_offsets[0],
                    bytemuck::cast_slice(push.position.as_slice()),
                );
                device_ref.cmd_push_constants(
                    frame_info.command_buffer,
                    self.pipeline_layout,
                    vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT,
                    push_constant_offsets[1],
                    bytemuck::cast_slice(push.color.as_slice()),
                );
                device_ref.cmd_push_constants(
                    frame_info.command_buffer,
                    self.pipeline_layout,
                    vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT,
                    push_constant_offsets[2],
                    bytemuck::cast_slice(std::slice::from_ref(&push.radius)),
                );
            }
            device_ref.cmd_draw(frame_info.command_buffer, 6, 1, 0, 0);
        }
    }

    fn create_pipeline_layout(
        device: &crate::Device,
        global_set_layout: &vk::DescriptorSetLayout,
    ) -> Result<vk::PipelineLayout> {
        let push_constant_range = vk::PushConstantRange::builder()
            .stage_flags(vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT)
            .offset(0)
            .size(size_of::<PointLightPushConstants>() as u32)
            .build();
        let descriptor_set_layouts = vec![*global_set_layout];
        let create_info = vk::PipelineLayoutCreateInfo::builder()
            .push_constant_ranges(&std::slice::from_ref(&push_constant_range))
            .set_layouts(&descriptor_set_layouts);
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
