mod camera;
mod debug;
mod device;
pub mod extras;
mod game_objects;
mod model;
mod pipeline;
mod renderer;
mod simple_render_system;
mod surface;
mod swap_chain;
mod window;

pub use camera::Camera;
pub use debug::DebugUtilsMessenger;
pub use device::{Device, QueryFamilyIndices};
pub use game_objects::{GameObject, ObjectId, TransformComponent};
pub use model::{Model, Vertex};
pub use pipeline::Pipeline;
pub use renderer::Renderer;
pub use simple_render_system::{SimplePushConstantData, SimpleRenderSystem};
pub use surface::{Surface, SwapChainSupportDetails};
pub use swap_chain::SwapChain;
pub use utils::create_cube_model;
pub use window::Window;

extern crate nalgebra_glm as glm;

use ash::vk;
use std::ffi::CStr;

mod utils {
    use anyhow::Result;

    #[inline]
    pub fn is_debug_build() -> bool {
        cfg!(debug_assertions)
    }

    pub fn create_cube_model(
        device: &crate::Device,
        offset: &[f32; 3],
    ) -> Result<Box<crate::Model>> {
        let offset = glm::Vec3::from_row_slice(offset);
        let vertices = [
            // left face
            crate::Vertex::new(&[-0.5f32, -0.5f32, -0.5f32], &[0.9, 0.9, 0.9]),
            crate::Vertex::new(&[-0.5f32, 0.5f32, 0.5f32], &[0.9, 0.9, 0.9]),
            crate::Vertex::new(&[-0.5f32, -0.5f32, 0.5f32], &[0.9, 0.9, 0.9]),
            crate::Vertex::new(&[-0.5f32, -0.5f32, -0.5f32], &[0.9, 0.9, 0.9]),
            crate::Vertex::new(&[-0.5f32, 0.5f32, -0.5f32], &[0.9, 0.9, 0.9]),
            crate::Vertex::new(&[-0.5f32, 0.5f32, 0.5f32], &[0.9, 0.9, 0.9]),
            // right face
            crate::Vertex::new(&[0.5f32, -0.5f32, -0.5f32], &[0.8, 0.8, 0.1]),
            crate::Vertex::new(&[0.5f32, 0.5f32, 0.5f32], &[0.8, 0.8, 0.1]),
            crate::Vertex::new(&[0.5f32, -0.5f32, 0.5f32], &[0.8, 0.8, 0.1]),
            crate::Vertex::new(&[0.5f32, -0.5f32, -0.5f32], &[0.8, 0.8, 0.1]),
            crate::Vertex::new(&[0.5f32, 0.5f32, -0.5f32], &[0.8, 0.8, 0.1]),
            crate::Vertex::new(&[0.5f32, 0.5f32, 0.5f32], &[0.8, 0.8, 0.1]),
            // top face
            crate::Vertex::new(&[-0.5f32, -0.5f32, -0.5f32], &[0.9, 0.6, 0.1]),
            crate::Vertex::new(&[0.5f32, -0.5f32, 0.5f32], &[0.9, 0.6, 0.1]),
            crate::Vertex::new(&[-0.5f32, -0.5f32, 0.5f32], &[0.9, 0.6, 0.1]),
            crate::Vertex::new(&[-0.5f32, -0.5f32, -0.5f32], &[0.9, 0.6, 0.1]),
            crate::Vertex::new(&[0.5f32, -0.5f32, -0.5f32], &[0.9, 0.6, 0.1]),
            crate::Vertex::new(&[0.5f32, -0.5f32, 0.5f32], &[0.9, 0.6, 0.1]),
            // bottom face
            crate::Vertex::new(&[-0.5f32, 0.5f32, -0.5f32], &[0.8, 0.1, 0.1]),
            crate::Vertex::new(&[0.5f32, 0.5f32, 0.5f32], &[0.8, 0.1, 0.1]),
            crate::Vertex::new(&[-0.5f32, 0.5f32, 0.5f32], &[0.8, 0.1, 0.1]),
            crate::Vertex::new(&[-0.5f32, 0.5f32, -0.5f32], &[0.8, 0.1, 0.1]),
            crate::Vertex::new(&[0.5f32, 0.5f32, -0.5f32], &[0.8, 0.1, 0.1]),
            crate::Vertex::new(&[0.5f32, 0.5f32, 0.5f32], &[0.8, 0.1, 0.1]),
            // front face
            crate::Vertex::new(&[-0.5f32, -0.5f32, 0.5f32], &[0.1, 0.1, 0.8]),
            crate::Vertex::new(&[0.5f32, 0.5f32, 0.5f32], &[0.1, 0.1, 0.8]),
            crate::Vertex::new(&[-0.5f32, 0.5f32, 0.5f32], &[0.1, 0.1, 0.8]),
            crate::Vertex::new(&[-0.5f32, -0.5f32, 0.5f32], &[0.1, 0.1, 0.8]),
            crate::Vertex::new(&[0.5f32, -0.5f32, 0.5f32], &[0.1, 0.1, 0.8]),
            crate::Vertex::new(&[0.5f32, 0.5f32, 0.5f32], &[0.1, 0.1, 0.8]),
            // back face
            crate::Vertex::new(&[-0.5f32, -0.5f32, -0.5f32], &[0.1, 0.8, 0.1]),
            crate::Vertex::new(&[0.5f32, 0.5f32, -0.5f32], &[0.1, 0.8, 0.1]),
            crate::Vertex::new(&[-0.5f32, 0.5f32, -0.5f32], &[0.1, 0.8, 0.1]),
            crate::Vertex::new(&[-0.5f32, -0.5f32, -0.5f32], &[0.1, 0.8, 0.1]),
            crate::Vertex::new(&[0.5f32, -0.5f32, -0.5f32], &[0.1, 0.8, 0.1]),
            crate::Vertex::new(&[0.5f32, 0.5f32, -0.5f32], &[0.1, 0.8, 0.1]),
        ]
        .iter_mut()
        .map(|v| {
            v.position += offset;

            v.clone()
        })
        .collect::<Vec<_>>();

        Ok(Box::new(crate::Model::new(device, &vertices)?))
    }
}

pub struct ApplicationInfo<'a> {
    pub name: &'a CStr,
    pub version: u32,
    pub engine_name: &'a CStr,
    pub engine_version: u32,
    pub api_version: u32,
}

impl ApplicationInfo<'_> {
    pub fn new(
        name: &str,
        version: u32,
        engine_name: &str,
        engine_version: u32,
        api_version: u32,
    ) -> Self {
        Self {
            name: unsafe { CStr::from_ptr(name.as_ptr() as *const i8) },
            version,
            engine_name: unsafe { CStr::from_ptr(engine_name.as_ptr() as *const i8) },
            engine_version,
            api_version,
        }
    }

    pub fn with_api_version_1_0(
        name: &str,
        version: u32,
        engine_name: &str,
        engine_version: u32,
    ) -> Self {
        Self::new(
            name,
            version,
            engine_name,
            engine_version,
            vk::API_VERSION_1_0,
        )
    }

    pub fn with_api_version_1_1(
        name: &str,
        version: u32,
        engine_name: &str,
        engine_version: u32,
    ) -> Self {
        Self::new(
            name,
            version,
            engine_name,
            engine_version,
            vk::API_VERSION_1_1,
        )
    }

    pub fn with_api_version_1_2(
        name: &str,
        version: u32,
        engine_name: &str,
        engine_version: u32,
    ) -> Self {
        Self::new(
            name,
            version,
            engine_name,
            engine_version,
            vk::API_VERSION_1_2,
        )
    }

    pub fn with_api_version_1_3(
        name: &str,
        version: u32,
        engine_name: &str,
        engine_version: u32,
    ) -> Self {
        Self::new(
            name,
            version,
            engine_name,
            engine_version,
            vk::API_VERSION_1_3,
        )
    }
}

impl Default for ApplicationInfo<'_> {
    fn default() -> Self {
        Self::with_api_version_1_3(
            "Vulkan Engine",
            vk::make_api_version(0, 1, 0, 0),
            "No Engine",
            vk::make_api_version(0, 1, 0, 0),
        )
    }
}
