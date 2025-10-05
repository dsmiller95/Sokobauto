use crate::core::{Cell, SharedGameState, UserAction};
use crate::state_graph::StateGraph;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
struct JsonData {
    nodes: Vec<JsonNode>,
    edges: Vec<JsonEdge>,
}

#[derive(Serialize, Deserialize, Debug)]
struct JsonNode {
    id: usize,
    on_targets: usize,
}

#[derive(Serialize, Deserialize, Debug)]
struct JsonEdge {
    source: usize,
    target: usize,
    dir: JsonDirection,
    change_type: JsonEdgeType,
}

#[derive(Serialize, Deserialize, Debug)]
enum JsonDirection {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Serialize, Deserialize, Debug)]
enum JsonEdgeType {
    PlayerMove,
    PlayerAndBoxMove,
}

impl From<crate::core::Direction> for JsonDirection {
    fn from(dir: crate::core::Direction) -> Self {
        match dir {
            crate::core::Direction::Up => JsonDirection::Up,
            crate::core::Direction::Down => JsonDirection::Down,
            crate::core::Direction::Left => JsonDirection::Left,
            crate::core::Direction::Right => JsonDirection::Right,
        }
    }
}

impl From<crate::core::GameChangeType> for JsonEdgeType {
    fn from(change_type: crate::core::GameChangeType) -> Self {
        match change_type {
            crate::core::GameChangeType::PlayerMove => JsonEdgeType::PlayerMove,
            crate::core::GameChangeType::PlayerAndBoxMove => JsonEdgeType::PlayerAndBoxMove,
        }
    }
}

pub fn get_json_data(graph: &StateGraph, shared: &SharedGameState) -> String {
    let nodes: Vec<JsonNode> = graph
        .nodes
        .iter()
        .map(|(state, id)| {
            let on_targets = state.count_boxes_on_goals(shared);
            JsonNode {
                id: *id,
                on_targets,
            }
        })
        .collect();

    let edges: Vec<JsonEdge> = graph
        .edges
        .iter()
        .map(|edge| {
            let UserAction::Move(direction) = edge.action;
            JsonEdge {
                source: edge.from,
                target: edge.to,
                dir: direction.into(),
                change_type: edge.game_change_type.into(),
            }
        })
        .collect();

    let json_data = JsonData { nodes, edges };
    serde_json::to_string_pretty(&json_data).unwrap()
}
