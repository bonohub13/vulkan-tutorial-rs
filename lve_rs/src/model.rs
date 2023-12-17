use anyhow::Result;
use ash::vk;
use offset::offset_of;
use std::mem::{align_of, size_of, size_of_val};

#[derive(Clone, Copy)]
pub struct Vertex {
    pub position: glm::Vec2,
    pub color: glm::Vec3,
}

pub struct Model {
    vertex_buffer: vk::Buffer,
    vertex_buffer_memory: vk::DeviceMemory,
    vertex_count: u32,
}

impl Vertex {
    pub fn new(position: &[f32; 2], color: &[f32; 3]) -> Self {
        Self {
            position: glm::Vec2::from_row_slice(position),
            color: glm::Vec3::from_row_slice(color),
        }
    }

    pub fn binding_descriptions() -> Vec<vk::VertexInputBindingDescription> {
        vec![vk::VertexInputBindingDescription::builder()
            .binding(0)
            .stride(size_of::<Self>() as u32)
            .input_rate(vk::VertexInputRate::VERTEX)
            .build()]
    }

    pub fn attribute_descriptions() -> Vec<vk::VertexInputAttributeDescription> {
        vec![
            vk::VertexInputAttributeDescription::builder()
                .location(0)
                .binding(0)
                .format(vk::Format::R32G32_SFLOAT)
                .offset(offset_of!(Vertex::position).into())
                .build(),
            vk::VertexInputAttributeDescription::builder()
                .location(1)
                .binding(0)
                .format(vk::Format::R32G32B32_SFLOAT)
                .offset(offset_of!(Vertex::color).into())
                .build(),
        ]
    }
}

impl Model {
    pub fn new(device: &crate::Device, vertices: &[Vertex]) -> Result<Self> {
        let (vertex_buffer, vertex_buffer_memory, vertex_count) =
            Self::create_vertex_buffers(device, vertices)?;

        Ok(Self {
            vertex_buffer,
            vertex_buffer_memory,
            vertex_count,
        })
    }

    pub unsafe fn destroy(&mut self, device: &crate::Device) {
        let device_ref = device.device();

        if self.vertex_buffer != vk::Buffer::null() {
            device_ref.destroy_buffer(self.vertex_buffer, None);
        }
        if self.vertex_buffer_memory != vk::DeviceMemory::null() {
            device_ref.free_memory(self.vertex_buffer_memory, None);
        }
    }

    #[inline]
    pub unsafe fn bind(&self, device: &crate::Device, command_buffer: &vk::CommandBuffer) {
        let device_ref = device.device();
        let buffers = [self.vertex_buffer];
        let offsets = [0];

        device_ref.cmd_bind_vertex_buffers(*command_buffer, 0, &buffers, &offsets)
    }

    #[inline]
    pub unsafe fn draw(&self, device: &crate::Device, command_buffer: &vk::CommandBuffer) {
        let device_ref = device.device();

        device_ref.cmd_draw(*command_buffer, self.vertex_count, 1, 0, 0)
    }

    fn create_vertex_buffers(
        device: &crate::Device,
        vertices: &[Vertex],
    ) -> Result<(vk::Buffer, vk::DeviceMemory, u32)> {
        let vertex_count = vertices.len();

        assert!(vertex_count >= 3, "Vertex count must be at least 3");

        let device_ref = device.device();
        let buffer_size = size_of_val(&vertices[0]) * vertex_count;
        let (vertex_buffer, vertex_buffer_memory) = device.create_buffer(
            buffer_size as u64,
            vk::BufferUsageFlags::VERTEX_BUFFER,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
        )?;
        let data = unsafe {
            device_ref.map_memory(
                vertex_buffer_memory,
                0,
                buffer_size as u64,
                vk::MemoryMapFlags::empty(),
            )
        }?;
        let mut align = unsafe {
            let mem_size = device_ref.get_buffer_memory_requirements(vertex_buffer);

            ash::util::Align::<Vertex>::new(data, align_of::<Vertex>() as u64, mem_size.size)
        };

        align.copy_from_slice(vertices);
        unsafe { device_ref.unmap_memory(vertex_buffer_memory) };

        Ok((vertex_buffer, vertex_buffer_memory, vertex_count as u32))
    }
}
