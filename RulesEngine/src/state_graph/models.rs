use std::collections::{HashSet, VecDeque};
use crate::state_graph::unique_node::UniqueNode;

#[derive(Clone)]
pub struct StateGraph {
    // map from game state to node id
    pub nodes: bimap::BiMap<UniqueNode, usize>,
    pub edges: HashSet<Edge>,
    pub unvisited: HashSet<usize>,
    pub next_unvisted: VecDeque<usize>,
    pub next_id: usize,
}

#[derive(Hash, Eq, PartialEq, Clone)]
pub struct Edge {
    pub from: usize,
    pub to: usize,
}

pub enum PopulateResult {
    AllVisited,
    Populated,
}
