use anyhow::Result;
use ash::vk;
use offset::offset_of;
use ordered_float::OrderedFloat;
use std::{
    collections::HashMap,
    hash::{Hash, Hasher},
    mem::{size_of, size_of_val},
};

#[derive(Clone, Copy)]
pub struct Vertex {
    pub position: glm::Vec3,
    pub color: glm::Vec3,
    pub normal: glm::Vec3,
    pub uv: glm::Vec2,
}

pub struct ModelBuilder {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}

pub struct Model {
    vertex_buffer: Box<crate::Buffer>,
    vertex_count: u32,
    index_buffer: Box<crate::Buffer>,
    index_count: u32,
    has_index_buffer: bool,
}

impl Vertex {
    pub fn new(position: &[f32; 3], color: &[f32; 3]) -> Self {
        Self {
            position: glm::Vec3::from_row_slice(position),
            color: glm::Vec3::from_row_slice(color),
            normal: glm::Vec3::zeros(),
            uv: glm::Vec2::zeros(),
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
            vk::VertexInputAttributeDescription::builder()
                .location(2)
                .binding(0)
                .format(vk::Format::R32G32B32_SFLOAT)
                .offset(offset_of!(Vertex::normal).into())
                .build(),
            vk::VertexInputAttributeDescription::builder()
                .location(3)
                .binding(0)
                .format(vk::Format::R32G32_SFLOAT)
                .offset(offset_of!(Vertex::uv).into())
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

    pub fn load_model(&self, filepath: &str) -> Self {
        let (shapes, _) = match tobj::load_obj(filepath, &tobj::LoadOptions::default()) {
            Ok((shapes, materials)) => match materials {
                Ok(materials) => (shapes, materials),
                Err(err) => {
                    eprintln!("Materials not available: {}", err);
                    (shapes, vec![])
                }
            },
            Err(err) => panic!("{}\n\tFailed to load file: {}", err, filepath),
        };

        let mut unique_vertices = HashMap::new();
        let mut indices = vec![];
        let mut vertices = vec![];
        for shape in shapes.iter() {
            for ((vertex_index, normal_index), tex_coord_index) in shape
                .mesh
                .indices
                .iter()
                .zip(shape.mesh.normal_indices.iter())
                .zip(shape.mesh.texcoord_indices.iter())
            {
                let vertex = Vertex {
                    position: if !shape.mesh.positions.is_empty() {
                        glm::vec3(
                            shape.mesh.positions[3 * (*vertex_index) as usize + 0],
                            shape.mesh.positions[3 * (*vertex_index) as usize + 1],
                            shape.mesh.positions[3 * (*vertex_index) as usize + 2],
                        )
                    } else {
                        glm::Vec3::zeros()
                    },
                    normal: if !shape.mesh.normals.is_empty() {
                        glm::vec3(
                            shape.mesh.normals[3 * (*normal_index) as usize + 0],
                            shape.mesh.normals[3 * (*normal_index) as usize + 1],
                            shape.mesh.normals[3 * (*normal_index) as usize + 2],
                        )
                    } else {
                        glm::Vec3::zeros()
                    },
                    uv: if !shape.mesh.texcoords.is_empty() {
                        glm::vec2(
                            shape.mesh.texcoords[2 * (*tex_coord_index) as usize + 0],
                            shape.mesh.texcoords[2 * (*tex_coord_index) as usize + 1],
                        )
                    } else {
                        glm::Vec2::zeros()
                    },
                    color: if !shape.mesh.vertex_color.is_empty() {
                        glm::vec3(
                            shape.mesh.vertex_color[3 * (*vertex_index) as usize + 0],
                            shape.mesh.vertex_color[3 * (*vertex_index) as usize + 1],
                            shape.mesh.vertex_color[3 * (*vertex_index) as usize + 2],
                        )
                    } else {
                        glm::vec3(1.0, 1.0, 1.0)
                    },
                };

                if !unique_vertices.contains_key(&vertex) {
                    unique_vertices.insert(vertex, vertices.len());
                    vertices.push(vertex);
                }
                indices.push(unique_vertices[&vertex] as u32);
            }
        }

        Self { vertices, indices }
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
        let (vertex_buffer, vertex_count) = Self::create_vertex_buffers(device, vertices)?;
        let (index_buffer, index_count, has_index_buffer) =
            Self::create_index_buffers(device, indices)?;

        Ok(Self {
            vertex_buffer,
            vertex_count,
            index_buffer,
            index_count,
            has_index_buffer,
        })
    }

    pub fn create_model_from_file(device: &crate::Device, filepath: &str) -> Result<Box<Self>> {
        let builder = Self::builder().load_model(filepath);

        println!("Vertex count: {}", builder.vertices.len());

        Ok(Box::new(builder.build(device)?))
    }

    pub fn builder() -> ModelBuilder {
        ModelBuilder {
            vertices: vec![],
            indices: vec![],
        }
    }

    pub unsafe fn destroy(&mut self, device: &crate::Device) {
        self.vertex_buffer.destroy(device);

        if self.has_index_buffer {
            self.index_buffer.destroy(device);
        }
    }

    #[inline]
    pub unsafe fn bind(&self, device: &crate::Device, command_buffer: &vk::CommandBuffer) {
        let device_ref = device.device();
        let offsets = [0];

        device_ref.cmd_bind_vertex_buffers(
            *command_buffer,
            0,
            std::slice::from_ref(self.vertex_buffer.buffer()),
            &offsets,
        );
        if self.has_index_buffer {
            device_ref.cmd_bind_index_buffer(
                *command_buffer,
                *self.index_buffer.buffer(),
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
    ) -> Result<(Box<crate::Buffer>, u32)> {
        let vertex_count = vertices.len();

        assert!(vertex_count >= 3, "Vertex count must be at least 3");

        let buffer_size = size_of_val(&vertices[0]) * vertex_count;
        let vertex_size = size_of_val(&vertices[0]);
        let mut staging_buffer = crate::Buffer::new(
            device,
            vertex_size as u64,
            vertex_count,
            vk::BufferUsageFlags::TRANSFER_SRC,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
            None,
        )?;
        let vertex_buffer = Box::new(crate::Buffer::new(
            device,
            vertex_size as u64,
            vertex_count,
            vk::BufferUsageFlags::VERTEX_BUFFER | vk::BufferUsageFlags::TRANSFER_DST,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
            None,
        )?);
        unsafe { staging_buffer.map(device, None, None) }?;
        unsafe { staging_buffer.write_to_buffer(device, vertices, None, None) }
        unsafe {
            device.copy_buffer(
                staging_buffer.buffer(),
                vertex_buffer.buffer(),
                buffer_size as u64,
            )
        }?;
        unsafe { staging_buffer.destroy(device) };

        Ok((vertex_buffer, vertex_count as u32))
    }

    fn create_index_buffers(
        device: &crate::Device,
        indices: &[u32],
    ) -> Result<(Box<crate::Buffer>, u32, bool)> {
        let index_count = indices.len();
        let has_index_buffer = index_count > 0;

        if !has_index_buffer {
            return Ok((Box::new(crate::Buffer::null()), 0, has_index_buffer));
        }

        let buffer_size = size_of_val(&indices[0]) * index_count;
        let index_size = size_of_val(&indices[0]);
        let mut staging_buffer = crate::Buffer::new(
            device,
            index_size as u64,
            index_count,
            vk::BufferUsageFlags::TRANSFER_SRC,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
            None,
        )?;
        let index_buffer = Box::new(crate::Buffer::new(
            device,
            index_size as u64,
            index_count,
            vk::BufferUsageFlags::INDEX_BUFFER | vk::BufferUsageFlags::TRANSFER_DST,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
            None,
        )?);

        unsafe { staging_buffer.map(device, None, None) }?;
        unsafe { staging_buffer.write_to_buffer(device, indices, None, None) }
        unsafe {
            device.copy_buffer(
                staging_buffer.buffer(),
                index_buffer.buffer(),
                buffer_size as u64,
            )
        }?;
        unsafe { staging_buffer.destroy(device) }

        Ok((index_buffer, index_count as u32, has_index_buffer))
    }
}

impl Default for Vertex {
    fn default() -> Self {
        Self {
            position: glm::Vec3::default(),
            normal: glm::Vec3::default(),
            uv: glm::Vec2::default(),
            color: glm::vec3(1., 1., 1.),
        }
    }
}

impl PartialEq for Vertex {
    fn eq(&self, other: &Self) -> bool {
        self.position == other.position
            && self.color == other.color
            && self.normal == other.normal
            && self.uv == other.uv
    }
}

impl Eq for Vertex {}

impl Hash for Vertex {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.position
            .iter()
            .for_each(|pos| OrderedFloat(*pos).hash(state));
        self.color
            .iter()
            .for_each(|rgb| OrderedFloat(*rgb).hash(state));
        self.normal
            .iter()
            .for_each(|normal| OrderedFloat(*normal).hash(state));
        self.uv.iter().for_each(|uv| OrderedFloat(*uv).hash(state));
    }
}
