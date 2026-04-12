use crate::{gpu::context::GpuContext, terrain::{NodeId, graph::Graph, resource_registry::{ResourceRegistry, ResourceKey}}};

pub struct Executor {}

impl Executor {
    pub fn new() -> Self {
        Self {}
    }
    pub fn evaluate(
        &self,
        node_id: NodeId,
        graph: &mut Graph,
        resource_registry: &mut ResourceRegistry,
        gpu_context: &GpuContext,
    ) {
        let mut encoder =
            gpu_context
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("terrain_executor_encoder"),
                });

        self.evaluate_node(node_id, graph, resource_registry, gpu_context, &mut encoder);

        gpu_context.queue.submit(std::iter::once(encoder.finish()));
    }

    fn evaluate_node(
        &self,
        node_id: NodeId,
        graph: &mut Graph,
        resource_registry: &mut ResourceRegistry,
        gpu_context: &GpuContext,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        let node_inputs = {
            let node = graph.get_node(node_id).unwrap();
            node.inputs().clone()
        };

        for (input_name, input_port) in node_inputs.iter() {
            if let Some(connection) = &input_port.connection {
                // First ensure the dependency is evaluated
                self.evaluate_node(connection.node_id, graph, resource_registry, gpu_context, encoder);

                // Bridge the output view of the source node to the input slot of this node
                if let Some(view) = resource_registry.views.get(&ResourceKey::output(connection.node_id, connection.port_name.clone())) {
                    // Clone the view handle into the local input slot
                    let view_clone = view.clone();
                    resource_registry.views.insert(ResourceKey::input(node_id, input_name.clone()), view_clone);
                }
            }
        }

        let node = graph.get_node_mut(node_id).unwrap();
        if node.is_dirty() {
            node.encode(node_id, encoder, resource_registry, gpu_context);
            node.set_clean();
        }
    }
}
