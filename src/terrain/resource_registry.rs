use std::collections::HashMap;
use crate::terrain::NodeId;

#[derive(Hash, PartialEq, Eq, Clone, Copy, Debug)]
pub enum ResourceKind {
    Input,
    Output,
    Internal,
}

#[derive(Hash, PartialEq, Eq, Clone, Debug)]
pub struct ResourceKey {
    pub node_id: NodeId,
    pub kind: ResourceKind,
    pub port_name: String,
}

impl ResourceKey {
    pub fn output(node_id: NodeId, port_name: String) -> Self {
        Self {
            node_id,
            kind: ResourceKind::Output,
            port_name,
        }
    }

    pub fn input(node_id: NodeId, port_name: String) -> Self {
        Self {
            node_id,
            kind: ResourceKind::Input,
            port_name,
        }
    }

    pub fn internal(node_id: NodeId, port_name: String) -> Self {
        Self {
            node_id,
            kind: ResourceKind::Internal,
            port_name,
        }
    }
}

pub struct ResourceRegistry {
    // Note: wgpu::Texture is not Clone, so we usually only store it for Output/Internal (the "owners")
    pub textures: HashMap<ResourceKey, wgpu::Texture>,
    // TextureView is Clone, so it can be stored for Input, Output, and Internal slots.
    pub views: HashMap<ResourceKey, wgpu::TextureView>,
}

impl ResourceRegistry {
    pub fn new() -> Self {
        Self {
            textures: HashMap::new(),
            views: HashMap::new(),
        }
    }
}
