use crate::{gpu::context::GpuContext, terrain::{executor::Executor, resource_registry::{ResourceRegistry, ResourceKey}}};

mod executor;
pub mod node;
pub mod resource_registry;
mod graph;


#[derive(Eq, Hash, PartialEq, Clone, Copy, Debug)]
pub struct NodeId(usize);


pub struct TerrainSystem {
    pub graph: graph::Graph,
    executor: Executor,
    resources: ResourceRegistry,
}

impl TerrainSystem {
    pub fn new() -> Self {
        Self {
            graph: graph::Graph::new(),
            executor: Executor::new(),
            resources: ResourceRegistry::new(),
        }
    }
    pub fn evaluate_node(
        &mut self,
        node_id: NodeId,
        gpu: &GpuContext,
    ) {
        if let Some(node) = self.graph.get_node(node_id) {
            if !node.is_dirty() {
                return; // No need to re-evaluate clean nodes
            }
        } else {
            panic!("Node with ID {:?} does not exist in the graph", node_id);
        }
        self.executor.evaluate(node_id, &mut self.graph, &mut self.resources, gpu);
    }

    pub fn get_output(&self, key: ResourceKey) -> Option<&wgpu::TextureView> {
        self.resources.views.get(&key)
    }
}
