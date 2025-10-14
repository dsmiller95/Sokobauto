use crate::state_graph::models::{Edge, StateGraph};
use std::collections::{HashSet, VecDeque};
use crate::state_graph::UniqueNode;

impl StateGraph {
    pub fn new() -> Self {
        StateGraph {
            nodes: bimap::BiMap::new(),
            edges: HashSet::new(),
            unvisited: HashSet::new(),
            next_unvisted: VecDeque::new(),
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
            self.next_unvisted.push_back(id);
            id
        }
    }

    pub fn get_state(&self, id: usize) -> Option<&UniqueNode> {
        self.nodes.get_by_right(&id)
    }

    pub fn add_edge(&mut self, edge: Edge) {
        self.edges.insert(edge);
    }

    pub fn take_and_visit_unvisited_node(&mut self) -> Option<usize> {
        while let Some(node_id) = self.next_unvisted.pop_front() {
            if self.unvisited.remove(&node_id) {
                return Some(node_id);
            }
        }
        None
    }

    pub fn assert_all_visited(&self) {
        assert!(self.unvisited.is_empty());
        assert!(self.next_unvisted.is_empty());
    }
}
