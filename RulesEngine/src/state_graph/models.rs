use crate::core::{GameStateEnvironment};
use std::collections::{HashSet};
use bevy::math::IVec2;

#[derive(Clone)]
pub struct StateGraph {
    // map from game state to node id
    pub nodes: bimap::BiMap<UniqueNode, usize>,
    pub edges: HashSet<Edge>,
    pub unvisited: HashSet<usize>,
    pub next_id: usize,
}

#[derive(Hash, Eq, PartialEq, Clone, Debug)]
pub struct UniqueNode {
    pub environment: GameStateEnvironment,
    pub minimum_reachable_player_position: IVec2,
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
