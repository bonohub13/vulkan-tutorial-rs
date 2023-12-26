use ash::vk;

pub struct FrameInfo<'a> {
    pub frame_index: usize,
    pub frame_time: f32,
    pub command_buffer: vk::CommandBuffer,
    pub camera: &'a crate::Camera,
}
