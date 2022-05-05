#[allow(clippy::too_many_arguments)]
pub(crate) fn create_render_pipeline(
    device: &wgpu::Device,
    pipeline_layout_label: &str,
    pipeline_label: &str,
    bind_group_layouts: &[&wgpu::BindGroupLayout],
    shader_desc: &wgpu::ShaderModuleDescriptor,
    vertex_buffer_layouts: &[wgpu::VertexBufferLayout],
    depth_format: wgpu::TextureFormat,
    sample_count: u32,
) -> wgpu::RenderPipeline {
    let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some(pipeline_layout_label),
        bind_group_layouts,
        push_constant_ranges: &[],
    });

    let shader = device.create_shader_module(shader_desc);
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some(pipeline_label),
        layout: Some(&render_pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: "vs_main",
            buffers: vertex_buffer_layouts,
        },
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: None, // TODO: what's the correct value here?
            unclipped_depth: false,
            polygon_mode: wgpu::PolygonMode::Fill,
            conservative: false,
        },
        depth_stencil: Some(wgpu::DepthStencilState {
            format: depth_format,
            depth_write_enabled: true,
            depth_compare: wgpu::CompareFunction::Less,
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        }),
        multisample: wgpu::MultisampleState {
            count: sample_count,
            ..Default::default()
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: "fs_main",
            targets: &[wgpu::ColorTargetState {
                format: wgpu::TextureFormat::Rgba8Unorm,
                blend: Some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING),
                write_mask: wgpu::ColorWrites::all(),
            }],
        }),
        multiview: None,
    })
}
