use crate::core::{GameUpdate, UserAction, step};
use crate::state_graph::Edge;
use crate::state_graph::models::{NodeState, PopulateResult, StateGraph};

pub fn populate_node(graph: &mut StateGraph, from_id: usize) {
    let Some(from_state) = graph.get_state(from_id) else {
        return;
    };
    let from_state = from_state.clone();

    let actions = UserAction::all_actions();
    for action in actions {
        let update = step(&from_state, action.clone());
        if let GameUpdate::NextState(new_state, change_type) = update {
            let to_id = graph.upsert_state(new_state);
            let edge = Edge {
                from: from_id,
                to: to_id,
                action: action.clone(),
                game_change_type: change_type,
            };
            graph.add_edge(edge);
        }
    }

    let meta = graph.get_node_meta_mut(from_id);
    meta.state = NodeState::Visited;
}

pub fn populate_step(graph: &mut StateGraph) -> PopulateResult {
    let picked_node = graph
        .metadata
        .iter()
        .filter_map(|(&id, meta)| {
            if meta.state == NodeState::Unvisited {
                Some(id)
            } else {
                None
            }
        })
        .next();

    let Some(node_id) = picked_node else {
        return PopulateResult::AllVisited;
    };
    populate_node(graph, node_id);
    PopulateResult::Populated
}
