use anyhow::Result;
use ash::vk;
use offset::offset_of;
use std::mem::{align_of, size_of, size_of_val};

#[derive(Clone, Copy)]
pub struct Vertex {
    pub position: glm::Vec3,
    pub color: glm::Vec3,
}

pub struct ModelBuilder {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}

pub struct Model {
    vertex_buffer: vk::Buffer,
    vertex_buffer_memory: vk::DeviceMemory,
    vertex_count: u32,
    index_buffer: vk::Buffer,
    index_buffer_memory: vk::DeviceMemory,
    index_count: u32,
    has_index_buffer: bool,
}

impl Vertex {
    pub fn new(position: &[f32; 3], color: &[f32; 3]) -> Self {
        Self {
            position: glm::Vec3::from_row_slice(position),
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
                .format(vk::Format::R32G32B32_SFLOAT)
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

impl ModelBuilder {
    pub fn new() -> Self {
        ModelBuilder {
            vertices: vec![],
            indices: vec![],
        }
    }

    pub fn vertices(&self, vertices: &[Vertex]) -> Self {
        Self {
            vertices: vertices.to_vec(),
            indices: self.indices.clone(),
        }
    }

    pub fn indices(&self, indices: &[u32]) -> Self {
        Self {
            vertices: self.vertices.clone(),
            indices: indices.to_vec(),
        }
    }

    pub fn build(&self, device: &crate::Device) -> Result<Model> {
        Model::new(device, &self.vertices, &self.indices)
    }
}

impl Model {
    pub fn new(device: &crate::Device, vertices: &[Vertex], indices: &[u32]) -> Result<Self> {
        let (vertex_buffer, vertex_buffer_memory, vertex_count) =
            Self::create_vertex_buffers(device, vertices)?;
        let (index_buffer, index_buffer_memory, index_count, has_index_buffer) =
            Self::create_index_buffers(device, indices)?;

        Ok(Self {
            vertex_buffer,
            vertex_buffer_memory,
            vertex_count,
            index_buffer,
            index_buffer_memory,
            index_count,
            has_index_buffer,
        })
    }

    pub fn builder() -> ModelBuilder {
        ModelBuilder {
            vertices: vec![],
            indices: vec![],
        }
    }

    pub unsafe fn destroy(&mut self, device: &crate::Device) {
        let device_ref = device.device();

        if self.vertex_buffer != vk::Buffer::null() {
            device_ref.destroy_buffer(self.vertex_buffer, None);
        }
        if self.vertex_buffer_memory != vk::DeviceMemory::null() {
            device_ref.free_memory(self.vertex_buffer_memory, None);
        }

        if self.has_index_buffer {
            if self.index_buffer != vk::Buffer::null() {
                device_ref.destroy_buffer(self.index_buffer, None);
            }
            if self.index_buffer_memory != vk::DeviceMemory::null() {
                device_ref.free_memory(self.index_buffer_memory, None);
            }
        }
    }

    #[inline]
    pub unsafe fn bind(&self, device: &crate::Device, command_buffer: &vk::CommandBuffer) {
        let device_ref = device.device();
        let buffers = [self.vertex_buffer];
        let offsets = [0];

        device_ref.cmd_bind_vertex_buffers(*command_buffer, 0, &buffers, &offsets);
        if self.has_index_buffer {
            device_ref.cmd_bind_index_buffer(
                *command_buffer,
                self.index_buffer,
                0,
                vk::IndexType::UINT32,
            )
        }
    }

    #[inline]
    pub unsafe fn draw(&self, device: &crate::Device, command_buffer: &vk::CommandBuffer) {
        let device_ref = device.device();

        if self.has_index_buffer {
            device_ref.cmd_draw_indexed(*command_buffer, self.index_count, 1, 0, 0, 0);
        } else {
            device_ref.cmd_draw(*command_buffer, self.vertex_count, 1, 0, 0)
        }
    }

    fn create_vertex_buffers(
        device: &crate::Device,
        vertices: &[Vertex],
    ) -> Result<(vk::Buffer, vk::DeviceMemory, u32)> {
        let vertex_count = vertices.len();

        assert!(vertex_count >= 3, "Vertex count must be at least 3");

        let device_ref = device.device();
        let buffer_size = size_of_val(&vertices[0]) * vertex_count;
        let (staging_buffer, staging_buffer_memory) = device.create_buffer(
            buffer_size as u64,
            vk::BufferUsageFlags::TRANSFER_SRC,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
        )?;
        let (vertex_buffer, vertex_buffer_memory) = device.create_buffer(
            buffer_size as u64,
            vk::BufferUsageFlags::VERTEX_BUFFER | vk::BufferUsageFlags::TRANSFER_DST,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
        )?;
        let data = unsafe {
            device_ref.map_memory(
                staging_buffer_memory,
                0,
                buffer_size as u64,
                vk::MemoryMapFlags::empty(),
            )
        }?;
        let mut align = unsafe {
            let mem_size = device_ref.get_buffer_memory_requirements(staging_buffer);

            ash::util::Align::<Vertex>::new(data, align_of::<Vertex>() as u64, mem_size.size)
        };

        align.copy_from_slice(vertices);
        unsafe { device_ref.unmap_memory(staging_buffer_memory) };
        unsafe { device.copy_buffer(&staging_buffer, &vertex_buffer, buffer_size as u64) }?;
        unsafe {
            device_ref.destroy_buffer(staging_buffer, None);
            device_ref.free_memory(staging_buffer_memory, None);
        }

        Ok((vertex_buffer, vertex_buffer_memory, vertex_count as u32))
    }

    fn create_index_buffers(
        device: &crate::Device,
        indices: &[u32],
    ) -> Result<(vk::Buffer, vk::DeviceMemory, u32, bool)> {
        let index_count = indices.len();
        let has_index_buffer = index_count > 0;

        if !has_index_buffer {
            return Ok((
                vk::Buffer::null(),
                vk::DeviceMemory::null(),
                0,
                has_index_buffer,
            ));
        }

        let device_ref = device.device();
        let buffer_size = size_of_val(&indices[0]) * index_count;
        let (staging_buffer, staging_buffer_memory) = device.create_buffer(
            buffer_size as u64,
            vk::BufferUsageFlags::TRANSFER_SRC,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
        )?;
        let (index_buffer, index_buffer_memory) = device.create_buffer(
            buffer_size as u64,
            vk::BufferUsageFlags::INDEX_BUFFER | vk::BufferUsageFlags::TRANSFER_DST,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
        )?;
        let data = unsafe {
            device_ref.map_memory(
                staging_buffer_memory,
                0,
                buffer_size as u64,
                vk::MemoryMapFlags::empty(),
            )
        }?;
        let mut align = unsafe {
            let mem_size = device_ref.get_buffer_memory_requirements(staging_buffer);

            ash::util::Align::<u32>::new(data, align_of::<u32>() as u64, mem_size.size)
        };

        align.copy_from_slice(indices);
        unsafe { device_ref.unmap_memory(staging_buffer_memory) };
        unsafe { device.copy_buffer(&staging_buffer, &index_buffer, buffer_size as u64) }?;
        unsafe {
            device_ref.destroy_buffer(staging_buffer, None);
            device_ref.free_memory(staging_buffer_memory, None)
        }

        Ok((
            index_buffer,
            index_buffer_memory,
            index_count as u32,
            has_index_buffer,
        ))
    }
}
