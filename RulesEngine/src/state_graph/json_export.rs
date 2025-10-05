use crate::core::{SharedGameState};
use serde::{Deserialize, Serialize};
use crate::state_graph::StateGraph;

#[derive(Serialize, Deserialize, Debug)]
struct JsonData {
    nodes: Vec<JsonNode>,
    links: Vec<JsonEdge>,
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
}

pub fn get_json_data(graph: &StateGraph, shared: &SharedGameState) -> String {
    let nodes: Vec<JsonNode> = graph.nodes.iter().map(|(state, id)| {
        let on_targets = shared.count_boxes_on_goals(&state.environment.boxes);
        JsonNode {
            id: *id,
            on_targets,
        }
    }).collect();

    let links: Vec<JsonEdge> = graph.edges.iter().map(|edge| {
        JsonEdge {
            source: edge.from,
            target: edge.to,
        }
    })
    .collect();

    let json_data = JsonData { nodes, links };
    serde_json::to_string_pretty(&json_data).unwrap()
}
