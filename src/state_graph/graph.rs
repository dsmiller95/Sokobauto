use std::collections::{HashMap, HashSet};
use crate::core::{GameState, UserAction};
use crate::state_graph::models::{Edge, NodeMeta, StateGraph};

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

    pub fn add_edge(&mut self, from: usize, to: usize, action: UserAction) {
        self.edges.insert(Edge {
            from, to, action
        });
    }

    pub fn set_node_meta(&mut self, node_id: usize, meta: NodeMeta) {
        self.metadata.insert(node_id, meta);
    }

    pub fn get_node_meta(&self, node_id: usize) -> NodeMeta {
        self.metadata.get(&node_id)
            .map(|x| x.clone())
            .unwrap_or(NodeMeta::default())
    }

    pub fn get_node_meta_mut(&mut self, node_id: usize) -> &mut NodeMeta {
        self.metadata.entry(node_id).or_insert_with(NodeMeta::default)
    }
}