use std::collections::HashSet;
use crate::core::{GameUpdate, UserAction, step, SharedGameState, GameState, Vec2};
use crate::state_graph::Edge;
use crate::state_graph::models::{PopulateResult, StateGraph, UniqueNode};

pub fn get_all_adjacent_nodes(from_node: &UniqueNode, shared: &SharedGameState) -> Vec<UniqueNode> {
    let reachable_positions = shared.reachable_positions(&GameState {
        player: from_node.minimum_reachable_player_position,
        environment: from_node.environment.clone(),
    });
    let actions = from_node.environment.boxes.iter()
        .flat_map(UserAction::all_push_actions_around)
        .filter(|(box_pos, _)| reachable_positions.contains(&box_pos))
        .collect::<Vec<_>>();

    let next_states: Vec<UniqueNode> = actions.into_iter()
        .filter_map(|(pos, action)| {
            let from_state = GameState {
                player: pos,
                environment: from_node.environment.clone(),
            };
            let update = step(shared, &from_state, action);
            let GameUpdate::NextState(new_state, change_type) = update else {
                return None;
            };
            if !change_type.did_box_move() {
                return None;
            }

            let min_reachable_position = shared.min_reachable_position(&new_state);
            let new_node = UniqueNode {
                environment: new_state.environment,
                minimum_reachable_player_position: min_reachable_position,
            };
            Some(new_node)
        })
        .collect();

    next_states
}

pub fn populate_node(graph: &mut StateGraph, from_id: usize, shared: &SharedGameState) {
    let Some(source_node) = graph.get_state(from_id) else {
        return;
    };
    let source_node = source_node.clone();

    let adjacent_nodes = get_all_adjacent_nodes(&source_node, shared);
    for node in adjacent_nodes {
        let to_id = graph.upsert_state(node);
        let edge = Edge {
            from: from_id,
            to: to_id,
        };
        graph.add_edge(edge);
    }

    graph.mark_visited(from_id);
}

pub fn populate_step(graph: &mut StateGraph, shared: &SharedGameState) -> PopulateResult {
    let picked_node = graph.get_unvisited_node();

    let Some(node_id) = picked_node else {
        graph.assert_all_visited();
        return PopulateResult::AllVisited;
    };
    populate_node(graph, node_id, shared);
    PopulateResult::Populated
}
