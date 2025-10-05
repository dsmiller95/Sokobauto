use std::collections::{HashMap, HashSet};
use crate::core::SharedGameState;
use crate::state_graph::StateGraph;

#[derive(Debug)]
pub struct TrimStats {
    pub nodes_before: usize,
    pub nodes_after: usize,
    pub edges_before: usize,
    pub edges_after: usize,
}

impl TrimStats {
    pub fn nodes_removed(&self) -> usize {
        self.nodes_before - self.nodes_after
    }

    pub fn nodes_removed_percentage(&self) -> f64 {
        if self.nodes_before == 0 {
            0.0
        } else {
            (self.nodes_removed() as f64 / self.nodes_before as f64) * 100.0
        }
    }

    pub fn edges_removed(&self) -> usize {
        self.edges_before - self.edges_after
    }

    pub fn edges_removed_percentage(&self) -> f64 {
        if self.edges_before == 0 {
            0.0
        } else {
            (self.edges_removed() as f64 / self.edges_before as f64) * 100.0
        }
    }
}

// TODO: populate metadata related to how many unwinnable states are adjacent to every winnable state
//  that is, how many moves are available which will softlock the level
pub fn trim_unwinnable(graph: &mut StateGraph, shared: &SharedGameState) -> TrimStats {
    let win_checker = shared.get_won_check_helper();
    let initial_winning_states: Vec<usize> = graph
        .nodes
        .iter()
        .filter_map(|(node, &node_id)| {
            if win_checker.is_won(&node.environment) {
                Some(node_id)
            } else {
                None
            }
        })
        .collect();

    let mut edge_map_successor_to_predecessors: HashMap<usize, HashSet<usize>> = HashMap::new();
    for edge in &graph.edges {
        edge_map_successor_to_predecessors.entry(edge.to).or_default().insert(edge.from);
    }
    let mut winning_states = HashSet::new();

    let mut stack = initial_winning_states.clone();
    while let Some(next) = stack.pop() {
        if !winning_states.insert(next) {
            continue;
        }

        if let Some(predecessors) = edge_map_successor_to_predecessors.get(&next) {
            for &pred in predecessors {
                stack.push(pred);
            }
        }
    }

    let total_nodes = graph.nodes.len();
    let total_edges = graph.edges.len();
    graph.nodes.retain(|_, node_id| winning_states.contains(node_id));
    graph.edges.retain(|edge| winning_states.contains(&edge.from) && winning_states.contains(&edge.to));

    TrimStats {
        nodes_before: total_nodes,
        nodes_after: graph.nodes.len(),
        edges_before: total_edges,
        edges_after: graph.edges.len(),
    }
}