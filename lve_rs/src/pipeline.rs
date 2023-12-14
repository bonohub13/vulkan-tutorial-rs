use crate as lve_rs;
use anyhow::{Context, Result};
use ash::vk;
use std::{ffi::CStr, fs::File};

/* MEMO
 *  In the Vulkan Tutorial video, a reference to lve_rs::Device is passed but
 *  DO NOT do this.
 *  Instead, require the functions to have it passed as an argument
 *  if the function requires it.
 */

pub struct PipelineConfigInfo {
    /*
    pub viewport: vk::Viewport,
    pub scissor: vk::Rect2D,
    */
    pub viewport_info: vk::PipelineViewportStateCreateInfo,
    pub input_assembly_info: vk::PipelineInputAssemblyStateCreateInfo,
    pub rasterization_info: vk::PipelineRasterizationStateCreateInfo,
    pub multisample_info: vk::PipelineMultisampleStateCreateInfo,
    pub color_blend_attachment: vk::PipelineColorBlendAttachmentState,
    pub depth_stencil_info: vk::PipelineDepthStencilStateCreateInfo,
    pub dynamic_state_enables: Vec<vk::DynamicState>,
    pub pipeline_layout: vk::PipelineLayout,
    pub render_pass: vk::RenderPass,
    pub subpass: u32,
}

pub struct Pipeline {
    graphics_pipeline: vk::Pipeline,
    vert_shader_module: vk::ShaderModule,
    frag_shader_module: vk::ShaderModule,
}

impl Pipeline {
    pub fn new(
        device: &lve_rs::Device,
        vert_file_path: &str,
        frag_file_path: &str,
        config_info: &PipelineConfigInfo,
    ) -> Result<Self> {
        let (graphics_pipeline, vert_shader_module, frag_shader_module) =
            Self::create_graphics_pipeline(device, vert_file_path, frag_file_path, config_info)?;

        Ok(Self {
            graphics_pipeline,
            vert_shader_module,
            frag_shader_module,
        })
    }

    #[inline]
    pub unsafe fn bind(&self, device: &lve_rs::Device, command_buffer: &vk::CommandBuffer) {
        device.device().cmd_bind_pipeline(
            *command_buffer,
            vk::PipelineBindPoint::GRAPHICS,
            self.graphics_pipeline,
        )
    }

    pub fn default_pipeline_config_info(width: u32, height: u32) -> PipelineConfigInfo {
        PipelineConfigInfo {
            /*
            viewport: vk::Viewport {
                x: 0.0f32,
                y: 0.0f32,
                width: width as f32,
                height: height as f32,
                min_depth: 0.0f32,
                max_depth: 1.0f32,
            },
            scissor: vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0 },
                extent: vk::Extent2D { width, height },
            },
            */
            viewport_info: vk::PipelineViewportStateCreateInfo::builder()
                .viewports(&[])
                .scissors(&[])
                .build(),
            input_assembly_info: vk::PipelineInputAssemblyStateCreateInfo::builder()
                .topology(vk::PrimitiveTopology::TRIANGLE_LIST)
                .primitive_restart_enable(false)
                .build(),
            rasterization_info: vk::PipelineRasterizationStateCreateInfo::builder()
                .depth_clamp_enable(false)
                .rasterizer_discard_enable(false)
                .polygon_mode(vk::PolygonMode::FILL)
                .line_width(1.0f32)
                .cull_mode(vk::CullModeFlags::NONE)
                .front_face(vk::FrontFace::CLOCKWISE)
                .depth_bias_enable(false)
                .depth_bias_constant_factor(0.0)
                .depth_bias_clamp(0.0)
                .depth_bias_slope_factor(0.0)
                .build(),
            multisample_info: vk::PipelineMultisampleStateCreateInfo::builder()
                .sample_shading_enable(false)
                .rasterization_samples(vk::SampleCountFlags::TYPE_1)
                .min_sample_shading(1.0)
                .sample_mask(&[])
                .alpha_to_coverage_enable(false)
                .alpha_to_one_enable(false)
                .build(),
            color_blend_attachment: vk::PipelineColorBlendAttachmentState::builder()
                .color_write_mask(vk::ColorComponentFlags::RGBA)
                .blend_enable(false)
                .src_color_blend_factor(vk::BlendFactor::ONE)
                .dst_color_blend_factor(vk::BlendFactor::ZERO)
                .color_blend_op(vk::BlendOp::ADD)
                .src_alpha_blend_factor(vk::BlendFactor::ONE)
                .dst_alpha_blend_factor(vk::BlendFactor::ZERO)
                .alpha_blend_op(vk::BlendOp::ADD)
                .build(),
            depth_stencil_info: vk::PipelineDepthStencilStateCreateInfo::builder()
                .depth_test_enable(true)
                .depth_write_enable(true)
                .depth_compare_op(vk::CompareOp::LESS)
                .depth_bounds_test_enable(false)
                .min_depth_bounds(0.0)
                .max_depth_bounds(1.0)
                .stencil_test_enable(false)
                .build(),
            dynamic_state_enables: vec![
                vk::DynamicState::VIEWPORT_WITH_COUNT,
                vk::DynamicState::SCISSOR_WITH_COUNT,
            ],
            pipeline_layout: vk::PipelineLayout::null(),
            render_pass: vk::RenderPass::null(),
            subpass: 0,
        }
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
    ) -> Result<(vk::Pipeline, vk::ShaderModule, vk::ShaderModule)> {
        assert!(
            config_info.pipeline_layout != vk::PipelineLayout::null(),
            "Cannot create graphics pipeline: No pipeline_layout provided in config_info"
        );
        assert!(
            config_info.render_pass != vk::RenderPass::null(),
            "Cannot create graphics pipeline: No render_pass provided in config_info"
        );

        let vert_shader_module = {
            let mut vert_code = Self::read_file(vert_file_path)?;

            Self::create_shader_module(device, &mut vert_code)?
        };
        let frag_shader_module = {
            let mut frag_code = Self::read_file(frag_file_path)?;

            Self::create_shader_module(device, &mut frag_code)?
        };
        let graphics_pipeline = {
            let shader_stages = [
                vk::PipelineShaderStageCreateInfo::builder()
                    .stage(vk::ShaderStageFlags::VERTEX)
                    .module(vert_shader_module)
                    .name(unsafe { CStr::from_bytes_with_nul_unchecked(b"main\0") })
                    .build(),
                vk::PipelineShaderStageCreateInfo::builder()
                    .stage(vk::ShaderStageFlags::FRAGMENT)
                    .module(frag_shader_module)
                    .name(unsafe { CStr::from_bytes_with_nul_unchecked(b"main\0") })
                    .build(),
            ];
            let binding_descriptions = crate::Vertex::binding_descriptions();
            let attribute_descriptions = crate::Vertex::attribute_descriptions();
            let vertex_input_info = vk::PipelineVertexInputStateCreateInfo::builder()
                .vertex_attribute_descriptions(&attribute_descriptions)
                .vertex_binding_descriptions(&binding_descriptions);
            /*
            let viewport_info = vk::PipelineViewportStateCreateInfo::builder()
                .viewports(std::slice::from_ref(&config_info.viewport))
                .scissors(std::slice::from_ref(&config_info.scissor));
            */
            let color_blend_info = vk::PipelineColorBlendStateCreateInfo::builder()
                .logic_op_enable(false)
                .logic_op(vk::LogicOp::COPY)
                .attachments(std::slice::from_ref(&config_info.color_blend_attachment))
                .blend_constants([0.0f32, 0.0f32, 0.0f32, 0.0f32]);
            let dynamic_state_info = vk::PipelineDynamicStateCreateInfo::builder()
                .dynamic_states(&config_info.dynamic_state_enables)
                .flags(vk::PipelineDynamicStateCreateFlags::empty())
                .build();
            let create_info = vk::GraphicsPipelineCreateInfo::builder()
                .stages(&shader_stages)
                .vertex_input_state(&vertex_input_info)
                .input_assembly_state(&config_info.input_assembly_info)
                .viewport_state(&config_info.viewport_info)
                .rasterization_state(&config_info.rasterization_info)
                .multisample_state(&config_info.multisample_info)
                .color_blend_state(&color_blend_info)
                .depth_stencil_state(&config_info.depth_stencil_info)
                .dynamic_state(&dynamic_state_info)
                .layout(config_info.pipeline_layout)
                .render_pass(config_info.render_pass)
                .subpass(config_info.subpass)
                .base_pipeline_index(-1)
                .base_pipeline_handle(vk::Pipeline::null());

            match unsafe {
                device.device().create_graphics_pipelines(
                    vk::PipelineCache::null(),
                    std::slice::from_ref(&create_info),
                    None,
                )
            } {
                Ok(pipelines) => Ok(pipelines),
                Err((_, e)) => Err(e),
            }?
        }
        .into_iter()
        .next()
        .context("Failed to create graphics pipeline")?;

        Ok((graphics_pipeline, vert_shader_module, frag_shader_module))
    }

    pub unsafe fn destroy(&mut self, device: &crate::Device) {
        device
            .device()
            .destroy_shader_module(self.frag_shader_module, None);
        device
            .device()
            .destroy_shader_module(self.vert_shader_module, None);
        device
            .device()
            .destroy_pipeline(self.graphics_pipeline, None);
    }

    fn create_shader_module(device: &lve_rs::Device, code: &mut File) -> Result<vk::ShaderModule> {
        let spv_code = ash::util::read_spv(code)?;
        let create_info = vk::ShaderModuleCreateInfo::builder().code(&spv_code);
        let shader_module = unsafe { device.device().create_shader_module(&create_info, None) }?;

        Ok(shader_module)
    }
}
