use crate::core::{GameState};
use crate::state_graph::models::{Edge, NodeMeta, NodeState, StateGraph};
use std::collections::{HashMap, HashSet};

impl StateGraph {
    pub fn new() -> Self {
        StateGraph {
            nodes: bimap::BiMap::new(),
            metadata: HashMap::new(),
            edges: HashSet::new(),
            unvisited: HashSet::new(),
        }
    }

    pub fn upsert_state(&mut self, state: GameState) -> usize {
        if let Some(&id) = self.nodes.get_by_left(&state) {
            id
        } else {
            let id = self.nodes.len();
            self.nodes.insert(state, id);
            self.metadata.insert(id, NodeMeta::default());
            self.unvisited.insert(id);
            id
        }
    }

    pub fn get_state(&self, id: usize) -> Option<&GameState> {
        self.nodes.get_by_right(&id)
    }

    pub fn add_edge(&mut self, edge: Edge) {
        self.edges.insert(edge);
    }

    pub fn mark_visited(&mut self, node_id: usize) {
        if let Some(meta) = self.metadata.get_mut(&node_id) {
            meta.state = NodeState::Visited;
        }
        self.unvisited.remove(&node_id);
    }

    pub fn get_unvisited_node(&self) -> Option<usize> {
        self.unvisited.iter().next().map(|id| *id)
    }

    pub fn assert_all_visited(&self) {
        let unvisited_meta = self.metadata
            .iter()
            .filter(|(_, meta)| {
                if meta.state == NodeState::Unvisited {
                    true
                } else {
                    false
                }
            })
            .count();

        assert!(self.unvisited.is_empty());
        assert_eq!(unvisited_meta, 0);
    }
}
