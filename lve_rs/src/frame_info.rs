use ash::vk;
use std::collections::HashMap;

pub const MAX_LIGHT: usize = 10;

#[derive(Debug, Clone, Copy)]
#[repr(C, align(16))]
pub struct PointLight {
    pub position: glm::Vec4,
    pub color: glm::Vec4,
}

#[derive(Debug, Clone, Copy)]
#[repr(C, align(16))]
pub struct GlobalUbo {
    pub projection: glm::Mat4,
    pub view: glm::Mat4,
    pub inverse_view: glm::Mat4,
    pub ambient_light_color: glm::Vec4,
    pub point_lights: [PointLight; MAX_LIGHT],
    pub num_lights: i32,
}

pub struct FrameInfo<'a> {
    pub frame_index: usize,
    pub frame_time: f32,
    pub command_buffer: vk::CommandBuffer,
    pub camera: &'a crate::Camera,
    pub descriptor_sets: &'a HashMap<crate::PipelineIdentifier, Vec<vk::DescriptorSet>>,
    pub screen_size: &'a vk::Extent2D,
    pub game_objects: &'a mut crate::Map,
}

impl Default for PointLight {
    fn default() -> Self {
        Self {
            position: glm::Vec4::default(),
            color: glm::Vec4::default(),
        }
    }
}

impl Default for GlobalUbo {
    fn default() -> Self {
        Self {
            projection: glm::Mat4::identity(),
            view: glm::Mat4::identity(),
            inverse_view: glm::Mat4::identity(),
            ambient_light_color: glm::vec4(1.0, 1.0, 1.0, 0.02),
            point_lights: [
                PointLight::default(),
                PointLight::default(),
                PointLight::default(),
                PointLight::default(),
                PointLight::default(),
                PointLight::default(),
                PointLight::default(),
                PointLight::default(),
                PointLight::default(),
                PointLight::default(),
            ],
            num_lights: 0,
        }
    }
}
