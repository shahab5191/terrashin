use std::collections::HashMap;

use glam::{Vec2, Vec3, Vec4};

use crate::{gpu::context::GpuContext, terrain::{NodeId, resource_registry::{ResourceRegistry, ResourceKey}}};

pub trait Node {
    fn encode(
        &self,
        node_id: NodeId,
        encoder: &mut wgpu::CommandEncoder,
        resource: &mut ResourceRegistry,
        gpu_context: &GpuContext,
    );

    fn inputs(&self) -> &HashMap<String, InputPort>;
    fn outputs(&self) -> &HashMap<String, OutputPort>;
    fn is_dirty(&self) -> bool;
    fn set_clean(&mut self);
}


#[derive(Debug, Clone, PartialEq)]
pub enum ValueType {
    Float(f32),
    Vec2(Vec2),
    Vec3(Vec3),
    Vec4(Vec4),
    Bool(bool),
    Texture,
}

#[derive(Debug, Clone)]
pub struct InputPort {
    pub name: String,
    pub value_type: ValueType,
    pub connection: Option<OutputPortRef>,
}

#[derive(Debug, Clone)]
pub struct OutputPort {
    pub name: String,
    pub value_type: ValueType,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct OutputPortRef {
    pub node_id: NodeId,
    pub port_name: String,
}

pub struct NoiseNode {
    pub inputs: HashMap<String, InputPort>,
    pub outputs: HashMap<String, OutputPort>,
    pub dirty: bool,
}

impl NoiseNode {
    pub fn new() -> Self {
        let output_ports = HashMap::from([(
            "Output".to_string(),
            OutputPort {
                name: "Output".to_string(),
                value_type: ValueType::Texture,
            },
        )]);
        Self {
            inputs: HashMap::new(),
            outputs: output_ports,
            dirty: true,
        }
    }
}

impl Node for NoiseNode {
    fn encode(
        &self,
        node_id: NodeId,
        _encoder: &mut wgpu::CommandEncoder,
        resource: &mut ResourceRegistry,
        gpu_context: &GpuContext,
    ) {
        // for testing we only generate a red texture:
        let texture = gpu_context.device.create_texture(
            &wgpu::TextureDescriptor {
                label: Some("noise_node_texture"),
                size: wgpu::Extent3d {
                    width: 512,
                    height: 512,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[],
            }
        );
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        // Store in registry
        resource.textures.insert(ResourceKey::output(node_id, "Output".to_string()), texture);
        resource.views.insert(ResourceKey::output(node_id, "Output".to_string()), view);
    }

    fn inputs(&self) -> &HashMap<String, InputPort> {
        &self.inputs
    }

    fn outputs(&self) -> &HashMap<String, OutputPort> {
        &self.outputs
    }

    fn is_dirty(&self) -> bool {
        self.dirty
    }

    fn set_clean(&mut self) {
        self.dirty = false;
    }
}

pub struct SolidColorNode {
    pub inputs: HashMap<String, InputPort>,
    pub outputs: HashMap<String, OutputPort>,
    pub color: Vec4,
    pub dirty: bool,
}

impl SolidColorNode {
    pub fn new(color: Vec4) -> Self {
        let output_ports = HashMap::from([(
            "Output".to_string(),
            OutputPort {
                name: "Output".to_string(),
                value_type: ValueType::Texture,
            },
        )]);
        Self {
            inputs: HashMap::new(),
            outputs: output_ports,
            color,
            dirty: true,
        }
    }
}

impl Node for SolidColorNode {
    fn encode(
        &self,
        node_id: NodeId,
        encoder: &mut wgpu::CommandEncoder,
        resource: &mut ResourceRegistry,
        gpu_context: &GpuContext,
    ) {
        let texture = gpu_context.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("solid_color_texture"),
            size: wgpu::Extent3d {
                width: 512,
                height: 512,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        // Use a render pass to clear the texture with the solid color
        {
            let _rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("solid_color_clear"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: self.color.x as f64,
                            g: self.color.y as f64,
                            b: self.color.z as f64,
                            a: self.color.w as f64,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
        }

        // Store the resources in the registry
        resource.textures.insert(ResourceKey::output(node_id, "Output".to_string()), texture);
        resource.views.insert(ResourceKey::output(node_id, "Output".to_string()), view);
    }

    fn inputs(&self) -> &HashMap<String, InputPort> {
        &self.inputs
    }

    fn outputs(&self) -> &HashMap<String, OutputPort> {
        &self.outputs
    }

    fn is_dirty(&self) -> bool {
        self.dirty
    }

    fn set_clean(&mut self) {
        self.dirty = false;
    }
}
