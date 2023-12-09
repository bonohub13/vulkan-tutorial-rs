use crate as lve_rs;
use anyhow::Result;
use ash::vk;
use std::fs::File;

/* MEMO
 *  In the Vulkan Tutorial video, a reference to lve_rs::Device is passed but
 *  DO NOT do this.
 *  Instead, require the functions to have it passed as an argument
 *  if the function requires it.
 */

pub struct PipelineConfigInfo {}
pub struct Pipeline {}

impl Pipeline {
    pub fn new(
        device: &lve_rs::Device,
        vert_file_path: &str,
        frag_file_path: &str,
        config_info: &PipelineConfigInfo,
    ) -> Result<Self> {
        Self::create_graphics_pipeline(device, vert_file_path, frag_file_path, config_info)?;

        Ok(Self {})
    }

    pub fn default_pipeline_config_info(width: u32, height: u32) -> PipelineConfigInfo {
        PipelineConfigInfo {}
    }

    fn read_file(file_path: &str) -> Result<File> {
        Ok(File::open(file_path)?)
    }

    /* --- Helper functions --- */
    fn create_graphics_pipeline(
        device: &lve_rs::Device,
        vert_file_path: &str,
        frag_file_path: &str,
        config_info: &PipelineConfigInfo,
    ) -> Result<()> {
        let vert_code = Self::read_file(vert_file_path)?;
        let frag_code = Self::read_file(frag_file_path)?;

        println!("Vertex code size: {}", vert_code.metadata()?.len());
        println!("Fragment code size: {}", frag_code.metadata()?.len());

        Ok(())
    }

    fn create_shader_module(device: &lve_rs::Device, code: &mut File) -> Result<vk::ShaderModule> {
        let spv_code = ash::util::read_spv(code)?;
        let create_info = vk::ShaderModuleCreateInfo::builder().code(&spv_code);
        let shader_module = unsafe { device.device().create_shader_module(&create_info, None) }?;

        Ok(shader_module)
    }
}
