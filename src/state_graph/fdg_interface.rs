use grapher::renderer::Renderer;
use grapher::simulator::SimulatorBuilder;
use petgraph::Directed;
use crate::state_graph::StateGraph;

pub fn render_interactive_graph(graph: &StateGraph) {
    // Build a PetGraph
    let mut rng = rand::thread_rng();
    let graph: petgraph::Graph<(), (), Directed> = convert_to_petgraph(graph);

    // Configure the simulator
    let simulator = SimulatorBuilder::new()
        .delta_time(0.01)
        .freeze_threshold(-1.0)
        .build(graph.into());

    // Start the renderer
    let renderer = Renderer::new(simulator);
    renderer.create_window();
}

pub fn convert_to_petgraph(graph: &StateGraph) -> petgraph::Graph<(), (), Directed> {
    let mut petgraph = petgraph::Graph::new();

    let node_map: std::collections::HashMap<usize, petgraph::graph::NodeIndex> = graph
        .nodes
        .iter()
        .map(|node| {
            let index = petgraph.add_node(());
            (node.1.clone(), index)
        })
        .collect();

    for edge in &graph.edges {
        if let (Some(&from_index), Some(&to_index)) = (
            node_map.get(&edge.from),
            node_map.get(&edge.to),
        ) {
            petgraph.add_edge(from_index, to_index, ());
        }
    }

    petgraph
}