use std::collections::HashMap;

use crate::terrain::{NodeId, node::{Node, OutputPortRef}};


pub struct Graph {
    nodes: HashMap<NodeId, Box<dyn Node>>,
    counter: usize,
    children: HashMap<NodeId, Vec<NodeId>>,
    parents: HashMap<NodeId, Vec<NodeId>>,
}

impl Graph {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            counter: 0,
            children: HashMap::new(),
            parents: HashMap::new()
        }
    }

    pub fn add_node(&mut self, node: Box<dyn Node>) -> NodeId {
        let node_id = NodeId(self.counter);
        self.counter += 1;
        self.nodes.insert(node_id, node);
        node_id
    }

    pub fn get_node(&self, id: NodeId) -> Option<&Box<dyn Node>> {
        self.nodes.get(&id)
    }

    pub fn get_node_mut(&mut self, id: NodeId) -> Option<&mut Box<dyn Node>> {
        self.nodes.get_mut(&id)
    }

    pub fn connect(&mut self, from: NodeId, from_port: String, to: NodeId, to_port: String) {
        if let Some(node) = self.nodes.get_mut(&to) {
            if let Some(port) = node.inputs_mut().get_mut(&to_port) {
                port.connection = Some(OutputPortRef {
                    node_id: from,
                    port_name: from_port.to_string(),
                });
            }
        }
        self.children.entry(from).or_default().push(to);
        self.parents.entry(to).or_default().push(from);
    }

    pub fn mark_dirty_recursive(&mut self, node_id: NodeId) {
        if let Some(node) = self.nodes.get_mut(&node_id) {
            node.set_dirty();
        }
        if let Some(children) = self.children.get(&node_id).cloned() {
            for child_id in children {
                self.mark_dirty_recursive(child_id);
            }
        }
    }

    pub fn disconnect(&mut self, from: NodeId, from_port: String, to: NodeId, to_port: String) {
        if let Some(node) = self.nodes.get_mut(&to) {
            if let Some(port) = node.inputs_mut().get_mut(&to_port) {
                port.connection = None;
            }
        }
        if let Some(children) = self.children.get_mut(&from) {
            children.retain(|&child| child != to);
        }
        if let Some(parents) = self.parents.get_mut(&to) {
            parents.retain(|&parent| parent != from);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::terrain::node::{InputPort, OutputPort, ValueType};
    use crate::terrain::resource_registry::ResourceRegistry;
    use crate::gpu::context::GpuContext;

    struct TestNode {
        inputs: HashMap<String, InputPort>,
        outputs: HashMap<String, OutputPort>,
        dirty: bool,
    }

    impl TestNode {
        fn new() -> Self {
            Self { inputs: HashMap::new(), outputs: HashMap::new(), dirty: false }
        }

        fn with_input(mut self, name: &str) -> Self {
            self.inputs.insert(name.to_string(), InputPort {
                name: name.to_string(),
                value_type: ValueType::Float(0.0),
                connection: None,
            });
            self
        }
    }

    impl Node for TestNode {
        fn encode(&self, _: NodeId, _: &mut wgpu::CommandEncoder, _: &mut ResourceRegistry, _: &GpuContext) {}
        fn inputs(&self) -> &HashMap<String, InputPort> { &self.inputs }
        fn inputs_mut(&mut self) -> &mut HashMap<String, InputPort> { &mut self.inputs }
        fn outputs(&self) -> &HashMap<String, OutputPort> { &self.outputs }
        fn is_dirty(&self) -> bool { self.dirty }
        fn set_dirty(&mut self) { self.dirty = true; }
        fn set_clean(&mut self) { self.dirty = false; }
    }

    #[test]
    fn test_connect_sets_adjacency_and_port() {
        let mut graph = Graph::new();
        let a = graph.add_node(Box::new(TestNode::new()));
        let b = graph.add_node(Box::new(TestNode::new().with_input("In")));

        graph.connect(a, "Out".to_string(), b, "In".to_string());

        assert!(graph.children[&a].contains(&b));
        assert!(graph.parents[&b].contains(&a));
        assert_eq!(graph.get_node(b).unwrap().inputs()["In"].connection.as_ref().unwrap().node_id, a);
    }

    #[test]
    fn test_disconnect_clears_adjacency_and_port() {
        let mut graph = Graph::new();
        let a = graph.add_node(Box::new(TestNode::new()));
        let b = graph.add_node(Box::new(TestNode::new().with_input("In")));

        graph.connect(a, "Out".to_string(), b, "In".to_string());
        graph.disconnect(a, "Out".to_string(), b, "In".to_string());

        assert!(graph.children.get(&a).map_or(true, |v| !v.contains(&b)));
        assert!(graph.parents.get(&b).map_or(true, |v| !v.contains(&a)));
        assert!(graph.get_node(b).unwrap().inputs()["In"].connection.is_none());
    }

    #[test]
    fn test_mark_dirty_recursive() {
        let mut graph = Graph::new();
        let a = graph.add_node(Box::new(TestNode::new()));
        let b = graph.add_node(Box::new(TestNode::new().with_input("In")));
        let c = graph.add_node(Box::new(TestNode::new().with_input("In")));

        graph.connect(a, "Out".to_string(), b, "In".to_string());
        graph.connect(b, "Out".to_string(), c, "In".to_string());

        graph.get_node_mut(a).unwrap().set_clean();
        graph.get_node_mut(b).unwrap().set_clean();
        graph.get_node_mut(c).unwrap().set_clean();

        graph.mark_dirty_recursive(a);

        assert!(graph.get_node(a).unwrap().is_dirty());
        assert!(graph.get_node(b).unwrap().is_dirty());
        assert!(graph.get_node(c).unwrap().is_dirty());
    }
}
