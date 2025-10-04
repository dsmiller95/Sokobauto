use std::collections::{HashMap, HashSet};
use crate::core::{GameChangeType, GameState, UserAction};

pub struct StateGraph {
    // map from game state to node id
    pub nodes: bimap::BiMap<GameState, usize>,
    pub metadata: HashMap<usize, NodeMeta>,
    pub edges: HashSet<Edge>,
}

#[derive(Default, Clone)]
pub struct NodeMeta {
    pub state: NodeState,
}

#[derive(Eq, PartialEq, Default, Clone)]
pub enum NodeState {
    #[default]
    Unvisited,
    Visited,
}

#[derive(Hash, Eq, PartialEq, Clone)]
pub struct Edge {
    pub from: usize,
    pub to: usize,
    pub action: UserAction,
    pub game_change_type: GameChangeType,
}

pub enum PopulateResult {
    AllVisited,
    Populated(usize),
}
