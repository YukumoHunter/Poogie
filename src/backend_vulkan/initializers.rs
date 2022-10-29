use super::shader::{ShaderSource, ShaderStage};
use ash::vk;

#[inline(always)]
pub fn pipeline_shader_stage_create_info(
    shader_module: vk::ShaderModule,
    shader_source: &ShaderSource,
) -> vk::PipelineShaderStageCreateInfoBuilder {
    vk::PipelineShaderStageCreateInfo::builder()
        .stage(match shader_source.stage {
            ShaderStage::Vertex => vk::ShaderStageFlags::VERTEX,
            ShaderStage::Fragment => vk::ShaderStageFlags::FRAGMENT,
            ShaderStage::Compute => vk::ShaderStageFlags::COMPUTE,
        })
        .module(shader_module)
}

#[inline(always)]
pub fn pipeline_vertex_input_state_create_info<'a>(
) -> vk::PipelineVertexInputStateCreateInfoBuilder<'a> {
    // no vertex bindings or attributes
    vk::PipelineVertexInputStateCreateInfo::builder()
        .vertex_attribute_descriptions(&[])
        .vertex_binding_descriptions(&[])
}

#[inline(always)]
pub fn pipeline_input_assembly_create_info<'a>(
    topology: vk::PrimitiveTopology,
) -> vk::PipelineInputAssemblyStateCreateInfoBuilder<'a> {
    vk::PipelineInputAssemblyStateCreateInfo::builder()
        .topology(topology)
        .primitive_restart_enable(false)
}

#[inline(always)]
pub fn pipeline_rasterization_state_create_info<'a>(
    polygon_mode: vk::PolygonMode,
    cull_mode: vk::CullModeFlags,
) -> vk::PipelineRasterizationStateCreateInfoBuilder<'a> {
    vk::PipelineRasterizationStateCreateInfo::builder()
        .depth_clamp_enable(false)
        .rasterizer_discard_enable(false)
        .polygon_mode(polygon_mode)
        .line_width(1.0)
        .cull_mode(cull_mode)
        .front_face(vk::FrontFace::CLOCKWISE)
        .depth_bias_enable(false)
        .depth_bias_constant_factor(0.0)
        .depth_bias_clamp(0.0)
        .depth_bias_slope_factor(0.0)
}

#[inline(always)]
pub fn pipeline_multisampling_state_create_info<'a>(
) -> vk::PipelineMultisampleStateCreateInfoBuilder<'a> {
    vk::PipelineMultisampleStateCreateInfo::builder()
        .sample_shading_enable(false)
        // no multisampling (1 sample/pixel)
        .rasterization_samples(vk::SampleCountFlags::TYPE_1)
        .min_sample_shading(1.0)
        .sample_mask(&[])
        .alpha_to_coverage_enable(false)
        .alpha_to_one_enable(false)
}

#[inline(always)]
pub fn pipeline_color_blend_attachment_state<'a>(
) -> vk::PipelineColorBlendAttachmentStateBuilder<'a> {
    vk::PipelineColorBlendAttachmentState::builder()
        .color_write_mask(vk::ColorComponentFlags::RGBA)
        .blend_enable(false)
}
