use crate::core::{GameState};
use crate::state_graph::models::{Edge, NodeMeta, StateGraph};
use std::collections::{HashMap, HashSet};

impl StateGraph {
    pub fn new() -> Self {
        StateGraph {
            nodes: bimap::BiMap::new(),
            metadata: HashMap::new(),
            edges: HashSet::new(),
        }
    }

    pub fn upsert_state(&mut self, state: GameState) -> usize {
        if let Some(&id) = self.nodes.get_by_left(&state) {
            id
        } else {
            let id = self.nodes.len();
            self.nodes.insert(state, id);
            self.metadata.insert(id, NodeMeta::default());
            id
        }
    }

    pub fn get_state(&self, id: usize) -> Option<&GameState> {
        self.nodes.get_by_right(&id)
    }

    pub fn add_edge(&mut self, edge: Edge) {
        self.edges.insert(edge);
    }

    pub fn get_node_meta_mut(&mut self, node_id: usize) -> &mut NodeMeta {
        self.metadata
            .entry(node_id)
            .or_insert_with(NodeMeta::default)
    }
}
