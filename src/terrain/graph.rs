use std::collections::HashMap;

use crate::terrain::{NodeId, node::Node};


pub struct Graph {
    nodes: HashMap<NodeId, Box<dyn Node>>,
    counter: usize,
}

impl Graph {
    pub fn new() -> Self {
        Self { nodes: HashMap::new(), counter: 0}
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
}
