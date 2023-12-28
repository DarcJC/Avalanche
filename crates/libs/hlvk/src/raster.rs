mod vertex_stream;

use std::sync::Arc;
use ash::vk;
use anyhow::Result;
use derive_builder::Builder;
use crate::{Context, Device, StagedShader};

pub use vertex_stream::*;
use crate::layout::PipelineLayout;

pub struct RasterPipeline {
    device: Arc<Device>,
    pub inner: vk::Pipeline,
}

#[derive(Builder, Clone, Copy)]
pub struct RasterPipelineCreateInfo<'a> {
    pub shaders: &'a [StagedShader],
    pub primitive_topology: vk::PrimitiveTopology,
    pub vertex_stream: &'a VertexStreamSet,
    pub viewport: Option<vk::Viewport>,
    pub scissor: Option<vk::Rect2D>,
    pub color_attachment_format: vk::Format,
    pub color_attachment_blend: Option<vk::PipelineColorBlendAttachmentState>,
    pub dynamic_states: Option<&'a [vk::DynamicState]>,
    pub polygon_mode: vk::PolygonMode,
    pub front_face: vk::FrontFace,
    pub cull_mode: vk::CullModeFlags,
}

impl RasterPipeline {
    pub fn new(
        device: Arc<Device>,
        layout: &PipelineLayout,
        create_info: RasterPipelineCreateInfo,
    ) -> Result<Self> {
        let _shader_modules = create_info.shaders.iter().map(|s| s.module.clone()).collect::<Vec<_>>();
        let shader_stages_info = create_info
            .shaders
            .iter()
            .map(|s| vk::PipelineShaderStageCreateInfo::builder()
                .stage(s.stage)
                .module(s.module.inner)
                .name(&s.entry_point_name)
                .build())
            .collect::<Vec<_>>();

        let (vertex_bindings, vertex_attributes) = create_info.vertex_stream.generate_description();
        let vertex_input_info = vk::PipelineVertexInputStateCreateInfo::builder()
            .vertex_binding_descriptions(&vertex_bindings)
            .vertex_attribute_descriptions(&vertex_attributes)
            .build();

        let  input_assembly_info = vk::PipelineInputAssemblyStateCreateInfo::builder()
            .topology(create_info.primitive_topology)
            .primitive_restart_enable(false);

        let viewports = create_info.viewport.map( |v| vec![v]).unwrap_or_default();
        let scissors = create_info.scissor.map(|s| vec![s]).unwrap_or_default();

        let viewport_info = vk::PipelineViewportStateCreateInfo::builder()
            .viewports(&viewports)
            .viewport_count(1)
            .scissors(&scissors)
            .scissor_count(1);

        let rasterizer_info = vk::PipelineRasterizationStateCreateInfo::builder()
            .depth_clamp_enable(false)
            .rasterizer_discard_enable(false)
            .line_width(1.0)
            .polygon_mode(create_info.polygon_mode)
            .front_face(create_info.front_face)
            .cull_mode(create_info.cull_mode)
            .depth_bias_enable(false)
            .depth_bias_constant_factor(0.0)
            .depth_bias_clamp(0.0)
            .depth_bias_slope_factor(0.0);

        let multisampling_info = vk::PipelineMultisampleStateCreateInfo::builder()
            .sample_shading_enable(false)
            .rasterization_samples(vk::SampleCountFlags::TYPE_1)
            .min_sample_shading(1.0)
            .alpha_to_coverage_enable(false)
            .alpha_to_one_enable(false);

        let color_blend_attachment = create_info
            .color_attachment_blend
            .unwrap_or(vk::PipelineColorBlendAttachmentState::builder()
                .color_write_mask(vk::ColorComponentFlags::RGBA)
                .build());
        let color_blend_attachments = [color_blend_attachment];
        let color_blending_info = vk::PipelineColorBlendStateCreateInfo::builder()
            .logic_op_enable(false)
            .logic_op(vk::LogicOp::COPY)
            .attachments(&color_blend_attachments)
            .blend_constants([0.0, 0.0, 0.0, 0.0]);

        let dynamic_state_info = vk::PipelineDynamicStateCreateInfo::builder()
            .dynamic_states(create_info.dynamic_states.unwrap_or(&[]));

        let color_attachment_formats = [create_info.color_attachment_format];
        let mut rendering_info = vk::PipelineRenderingCreateInfo::builder()
            .color_attachment_formats(&color_attachment_formats);

        let pipeline_info = vk::GraphicsPipelineCreateInfo::builder()
            .stages(&shader_stages_info)
            .vertex_input_state(&vertex_input_info)
            .input_assembly_state(&input_assembly_info)
            .viewport_state(&viewport_info)
            .rasterization_state(&rasterizer_info)
            .multisample_state(&multisampling_info)
            .color_blend_state(&color_blending_info)
            .dynamic_state(&dynamic_state_info)
            .layout(layout.inner)
            .push_next(&mut rendering_info);

        let inner = unsafe {
            device
                .inner
                .create_graphics_pipelines(
                    vk::PipelineCache::null(),
                    std::slice::from_ref(&pipeline_info),
                    None,
                )
                .map_err(|e| e.1)?[0]
        };

        Ok ( Self { device, inner } )
    }
}

impl Context {
    pub fn create_graphics_pipeline(&self, layout: &PipelineLayout, create_info: RasterPipelineCreateInfo) -> Result<RasterPipeline> {
        RasterPipeline::new(self.device.clone(), layout, create_info)
    }
}

impl Drop for RasterPipeline {
    fn drop(&mut self) {
        unsafe {
            self.device.inner.destroy_pipeline(self.inner, None)
        };
    }
}
