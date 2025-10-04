use crate::core::{step, GameState, GameUpdate, UserAction};
use crate::state_graph::models::{NodeState, PopulateResult, StateGraph};

pub fn populate_node(graph: &mut StateGraph, from_id: usize) {
    let Some(from_state) = graph.get_state(from_id) else {
        return;
    };
    let from_state = from_state.clone();
    
    let actions = UserAction::all_actions();
    for action in actions {
        let update = step(&from_state, action.clone());
        if let GameUpdate::NextState(new_state) = update {
            let to_id = graph.get_id(new_state);
            graph.add_edge(from_id, to_id, action);
        }
    }

    let meta = graph.get_node_meta_mut(from_id);
    meta.state = NodeState::Visited;
}

pub fn populate_step(graph: &mut StateGraph) -> PopulateResult {
    let unvisited_nodes: Vec<usize> = graph
        .metadata
        .iter()
        .filter_map(|(&id, meta)| if meta.state == NodeState::Unvisited { Some(id) } else { None })
        .collect();
    
    let picked_node = unvisited_nodes.first().cloned();
    let Some(node_id) = picked_node else {
        return PopulateResult::AllVisited;
    };
    populate_node(graph, node_id);
    PopulateResult::Populated(node_id)
}