use crate::state_graph::models::{Edge, StateGraph, UniqueNode};
use std::collections::{HashSet};

impl StateGraph {
    pub fn new() -> Self {
        StateGraph {
            nodes: bimap::BiMap::new(),
            edges: HashSet::new(),
            unvisited: HashSet::new(),
            next_id: 0,
        }
    }

    pub fn upsert_state(&mut self, state: UniqueNode) -> usize {
        if let Some(&id) = self.nodes.get_by_left(&state) {
            id
        } else {
            let id = self.next_id;
            self.next_id += 1;

            // we know that this insertion is unique, because id is unique, and we just checked to ensure that te state is unique
            self.nodes.insert_no_overwrite(state, id).unwrap();
            self.unvisited.insert(id);
            id
        }
    }

    pub fn get_state(&self, id: usize) -> Option<&UniqueNode> {
        self.nodes.get_by_right(&id)
    }

    pub fn add_edge(&mut self, edge: Edge) {
        self.edges.insert(edge);
    }

    pub fn mark_visited(&mut self, node_id: usize) {
        self.unvisited.remove(&node_id);
    }

    pub fn get_unvisited_node(&self) -> Option<usize> {
        self.unvisited.iter().next().map(|id| *id)
    }

    pub fn assert_all_visited(&self) {
        assert!(self.unvisited.is_empty());
    }
}
