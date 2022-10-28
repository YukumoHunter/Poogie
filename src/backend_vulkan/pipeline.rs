use super::{
    device::Device,
    initializers::{
        self, pipeline_color_blend_attachment_state, pipeline_input_assembly_create_info,
        pipeline_rasterization_state_create_info,
    },
    shader::ShaderDesc,
    swapchain::Swapchain,
};
use anyhow::Result;
use ash::vk;

pub struct GraphicsPipeline {
    pub pipeline: vk::Pipeline,
    pub layout: vk::PipelineLayout,
}

impl GraphicsPipeline {
    pub fn create_pipeline(
        device: &Device,
        swapchain: &Swapchain,
        shaders: &[ShaderDesc],
    ) -> Result<Self> {
        let viewports = [vk::Viewport::builder()
            .x(0.0)
            .y(0.0)
            .width(swapchain.desc.extent.width as f32)
            .height(swapchain.desc.extent.height as f32)
            .min_depth(0.0)
            .max_depth(1.0)
            .build()];

        let scissors = [vk::Rect2D::builder()
            .offset(vk::Offset2D::builder().x(0).y(0).build())
            .extent(swapchain.desc.extent)
            .build()];

        let viewport_state = vk::PipelineViewportStateCreateInfo::builder()
            .viewport_count(1)
            .viewports(&viewports)
            .scissor_count(1)
            .scissors(&scissors);

        let stages = shaders
            .iter()
            .map(|desc| {
                let module = desc
                    .clone()
                    .create_shader()
                    .expect("Error creating shader")
                    .create_module(device)
                    .unwrap();

                initializers::pipeline_shader_stage_create_info(module, desc).build()
            })
            .collect::<Vec<vk::PipelineShaderStageCreateInfo>>();

        let vertex_input_state = initializers::pipeline_vertex_input_state_create_info();
        let input_assembly_state =
            pipeline_input_assembly_create_info(vk::PrimitiveTopology::TRIANGLE_LIST);
        let rasterizer = pipeline_rasterization_state_create_info(
            vk::PolygonMode::FILL,
            vk::CullModeFlags::NONE,
        );
        let multisampling = initializers::pipeline_multisampling_state_create_info();

        let color_blend_attachments = [pipeline_color_blend_attachment_state().build()];

        let color_blend_state = vk::PipelineColorBlendStateCreateInfo::builder()
            .logic_op_enable(false)
            .logic_op(vk::LogicOp::COPY)
            .attachments(&color_blend_attachments);

        let layout_create_info = vk::PipelineLayoutCreateInfo::builder();

        let layout = unsafe {
            device
                .raw
                .create_pipeline_layout(&layout_create_info, None)?
        };

        let mut rendering_info = vk::PipelineRenderingCreateInfo::builder()
            .color_attachment_formats(&[swapchain.desc.surface_format.format])
            .build();

        let pipeline_create_info = vk::GraphicsPipelineCreateInfo::builder()
            .stages(&stages)
            .vertex_input_state(&vertex_input_state)
            .input_assembly_state(&input_assembly_state)
            .rasterization_state(&rasterizer)
            .multisample_state(&multisampling)
            .viewport_state(&viewport_state)
            .color_blend_state(&color_blend_state)
            .layout(layout)
            .push_next(&mut rendering_info);

        let pipeline = unsafe {
            device
                .raw
                .create_graphics_pipelines(
                    vk::PipelineCache::null(),
                    &[pipeline_create_info.build()],
                    None,
                )
                .map_err(|e| e.1)?[0]
        };

        Ok(GraphicsPipeline { pipeline, layout })
    }
}
