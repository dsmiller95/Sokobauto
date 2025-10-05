use std::collections::{HashMap, HashSet};
use crate::state_graph::models::{BoxOnlyEdge, BoxOnlyGameState, BoxOnlyStateGraph};
use crate::state_graph::StateGraph;

pub fn get_box_only_graph(graph: &StateGraph) -> BoxOnlyStateGraph {
    let mut new_nodes = HashMap::<BoxOnlyGameState, usize>::new();
    let mut id_rewrites = HashMap::<usize, usize>::new();
    for (state, &id) in &graph.nodes {
        let boxes = BoxOnlyGameState {
            boxes: state.boxes.clone(),
        };
        let entry = new_nodes.entry(boxes).or_insert(id);
        id_rewrites.insert(id, *entry);
    }

    let edges: HashSet<BoxOnlyEdge> = graph.edges.iter().filter_map(|edge| {
        let new_edge = BoxOnlyEdge {
            from: *id_rewrites.get(&edge.from).unwrap(),
            to: *id_rewrites.get(&edge.to).unwrap(),
        };
        if(new_edge.from == edge.to) {
            None
        } else {
            Some(new_edge)
        }
    })
        .collect();

    BoxOnlyStateGraph {
        nodes: new_nodes.into_iter().collect(),
        edges,
    }
}