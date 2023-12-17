mod debug;
mod device;
mod game_objects;
mod model;
mod pipeline;
mod renderer;
mod surface;
mod swap_chain;
mod window;

pub use debug::DebugUtilsMessenger;
pub use device::{Device, QueryFamilyIndices};
pub use game_objects::{GameObject, ObjectId, TransformComponent2D};
pub use model::{Model, Vertex};
pub use pipeline::Pipeline;
pub use renderer::Renderer;
pub use surface::{Surface, SwapChainSupportDetails};
pub use swap_chain::SwapChain;
pub use window::Window;

extern crate nalgebra_glm as glm;

use ash::vk;
use std::ffi::CStr;

mod utils {
    #[inline]
    pub fn is_debug_build() -> bool {
        cfg!(debug_assertions)
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
