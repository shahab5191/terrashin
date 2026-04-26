use std::collections::HashMap;

use glam::{Vec2, Vec3, Vec4};

use crate::{gpu::context::GpuContext, terrain::{NodeId, resource_registry::{ResourceRegistry}}};

pub mod checker;
pub mod perlin_noise;
pub mod solid_color;

pub use checker::CheckerNode;
pub use perlin_noise::PerlinNoiseNode;
pub use solid_color::SolidColorNode;

pub trait Node: Send {
    fn encode(
        &self,
        node_id: NodeId,
        encoder: &mut wgpu::CommandEncoder,
        resource: &mut ResourceRegistry,
        gpu_context: &GpuContext,
    );

    fn inputs(&self) -> &HashMap<String, InputPort>;
    fn inputs_mut(&mut self) -> &mut HashMap<String, InputPort>;
    fn outputs(&self) -> &HashMap<String, OutputPort>;
    fn is_dirty(&self) -> bool;
    fn set_dirty(&mut self);
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

pub(super) fn create_fallback_texture(color: Vec4, gpu: &GpuContext) -> (wgpu::Texture, wgpu::TextureView) {
    let texture = gpu.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("fallback_texture"),
        size: wgpu::Extent3d { width: 1, height: 1, depth_or_array_layers: 1 },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });
    let bytes = [
        (color.x.clamp(0.0, 1.0) * 255.0) as u8,
        (color.y.clamp(0.0, 1.0) * 255.0) as u8,
        (color.z.clamp(0.0, 1.0) * 255.0) as u8,
        (color.w.clamp(0.0, 1.0) * 255.0) as u8,
    ];
    gpu.queue.write_texture(
        wgpu::TexelCopyTextureInfo {
            texture: &texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        &bytes,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(4),
            rows_per_image: None,
        },
        wgpu::Extent3d { width: 1, height: 1, depth_or_array_layers: 1 },
    );
    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    (texture, view)
}
