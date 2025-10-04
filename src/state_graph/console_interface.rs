use std::io;
use ratatui::backend::CrosstermBackend;
use ratatui::layout::Alignment;
use ratatui::prelude::{Color, Style};
use ratatui::Terminal;
use ratatui::widgets::*;
use crate::state_graph::StateGraph;

pub fn render_graph(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    graph: &StateGraph
) -> Result<(), Box<dyn std::error::Error>> {
    // Placeholder for future implementation of graph rendering
    terminal.draw(|f| {
        let size = f.area();

        let description = get_graph_info(graph);

        let paragraph = Paragraph::new(description)
            .block(Block::default().borders(Borders::ALL).title("State Graph Info"))
            .style(Style::default().fg(Color::White))
            .alignment(Alignment::Center);

        f.render_widget(paragraph, size);
    })?;
    Ok(())
}

pub fn get_graph_info(graph: &StateGraph) -> String {
    let nodes = graph.nodes.len();
    let edges = graph.edges.len();
    let visited = graph.metadata.values().filter(|m| m.state == crate::state_graph::models::NodeState::Visited).count();
    format!(
        "Graph has {} nodes, {} edges, {} visited nodes.",
        nodes, edges, visited
    )
}