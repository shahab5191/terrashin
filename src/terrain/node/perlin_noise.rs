use std::collections::HashMap;

use crate::{gpu::context::GpuContext, terrain::{NodeId, resource_registry::{ResourceRegistry, ResourceKey}}};

use super::{InputPort, Node, OutputPort, ValueType};

pub struct PerlinNoiseNode {
    pub inputs: HashMap<String, InputPort>,
    pub outputs: HashMap<String, OutputPort>,
    pub scale: f32,
    pub dirty: bool,
    render_pipeline: wgpu::RenderPipeline,
    bind_group_layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
    uniform_buffer: wgpu::Buffer,
}

impl PerlinNoiseNode {
    pub fn new(gpu: &GpuContext, scale: f32) -> Self {
        let shader = gpu.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Perlin Noise Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../../shaders/perlin_noise.wgsl").into()),
        });

        let bind_group_layout = gpu.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("perlin_noise_bind_group_layout"),
            entries: &[
                // Sampler for noise texture
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                // Uniform buffer for scale
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        let pipeline_layout = gpu.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Perlin Noise Pipeline Layout"),
            bind_group_layouts: &[Some(&bind_group_layout)],
            immediate_size: 0,
        });

        let render_pipeline = gpu.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Perlin Noise Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::R32Float,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState { count: 1, mask: !0, alpha_to_coverage_enabled: false },
            multiview_mask: None,
            cache: None,
        });

        let sampler = gpu.device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        });

        let uniform_buffer = gpu.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("perlin_noise_uniforms"),
            size: 16,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let inputs = HashMap::from([
            ("Scale".to_string(), InputPort {
                name: "Scale".to_string(),
                value_type: ValueType::Float(scale),
                connection: None,
            }),
        ]);

        let outputs = HashMap::from([("Output".to_string(), OutputPort {
            name: "Output".to_string(),
            value_type: ValueType::Texture,
        })]);

        Self {
            inputs,
            outputs,
            scale,
            dirty: true,
            render_pipeline,
            bind_group_layout,
            sampler,
            uniform_buffer,
        }
    }
}

impl Node for PerlinNoiseNode {
    fn inputs(&self) -> &HashMap<String, InputPort> { &self.inputs }
    fn inputs_mut(&mut self) -> &mut HashMap<String, InputPort> { &mut self.inputs }
    fn outputs(&self) -> &HashMap<String, OutputPort> { &self.outputs }
    fn is_dirty(&self) -> bool { self.dirty }
    fn set_dirty(&mut self) { self.dirty = true; }
    fn set_clean(&mut self) { self.dirty = false; }

    fn encode(
        &self,
        node_id: NodeId,
        encoder: &mut wgpu::CommandEncoder,
        resource: &mut ResourceRegistry,
        gpu_context: &GpuContext,
    ) {
        let scale_bytes: [u8; 16] = {
            let b = self.scale.to_le_bytes();
            [b[0], b[1], b[2], b[3], 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]
        };
        gpu_context.queue.write_buffer(&self.uniform_buffer, 0, &scale_bytes);

        let bind_group = gpu_context.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("perlin_noise_bind_group"),
            layout: &self.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::Sampler(&self.sampler) },
                wgpu::BindGroupEntry { binding: 1, resource: self.uniform_buffer.as_entire_binding() },
            ],
        });

        let output_texture = gpu_context.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("perlin_noise_output"),
            size: wgpu::Extent3d { width: 512, height: 512, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::R32Float,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });
        let output_view = output_texture.create_view(&wgpu::TextureViewDescriptor::default());

        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("perlin_noise_pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &output_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            rpass.set_pipeline(&self.render_pipeline);
            rpass.set_bind_group(0, &bind_group, &[]);
            rpass.draw(0..3, 0..1);
        }

        resource.textures.insert(ResourceKey::output(node_id, "Output".to_string()), output_texture);
        resource.views.insert(ResourceKey::output(node_id, "Output".to_string()), output_view);
    }
}

