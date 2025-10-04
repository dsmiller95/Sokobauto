use crate::state_graph::StateGraph;
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use ratatui::layout::Alignment;
use ratatui::prelude::{Color, Style};
use ratatui::widgets::*;
use std::io;

pub struct GraphRenderState<'a> {
    pub graph: &'a StateGraph,
    pub processed_since_last_render: usize,
    pub start_time: std::time::Instant,
    pub last_render_time: std::time::Instant,
    pub current_time: std::time::Instant,
}

pub fn render_graph(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    render: GraphRenderState,
) -> Result<(), Box<dyn std::error::Error>> {
    // Placeholder for future implementation of graph rendering
    terminal.draw(|f| {
        let size = f.area();

        let graph_info = GraphInfo::new(render.graph);
        let graph_description = graph_info.to_string();

        let visited_per_second = if render.current_time == render.last_render_time {
            0.0
        } else {
            render.processed_since_last_render as f64 / (render.current_time - render.last_render_time).as_secs_f64()
        };
        let total_visited_per_second = if render.start_time == render.current_time {
            0.0
        } else {
            graph_info.visited as f64
                / (render.current_time - render.start_time).as_secs_f64()
        };
        let time_description = format!(
            "Processed {} nodes since last render, {:?} since. {:.1} nodes/sec. Total {:.1} nodes/sec.",
            render.processed_since_last_render,
            render.current_time - render.last_render_time,
            visited_per_second,
            total_visited_per_second
        );

        let description = format!("{}\n{}", graph_description, time_description);

        let paragraph = Paragraph::new(description)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("State Graph Info"),
            )
            .style(Style::default().fg(Color::White))
            .alignment(Alignment::Center);

        f.render_widget(paragraph, size);
    })?;
    Ok(())
}

pub struct GraphInfo {
    pub nodes: usize,
    pub edges: usize,
    pub visited: usize,
    pub percent_visited: f64,
}

impl GraphInfo {
    pub fn new(graph: &StateGraph) -> Self {
        let nodes = graph.nodes.len();
        let edges = graph.edges.len();
        let visited = graph
            .metadata
            .values()
            .filter(|m| m.state == crate::state_graph::models::NodeState::Visited)
            .count();
        let percent_visited = if nodes > 0 {
            (visited as f64 / nodes as f64) * 100.0
        } else {
            0.0
        };
        Self {
            nodes,
            edges,
            visited,
            percent_visited,
        }
    }

    pub fn to_string(&self) -> String {
        format!(
            "Graph has {} nodes, {} edges, {} visited nodes ({:.1}%).",
            self.nodes, self.edges, self.visited, self.percent_visited
        )
    }
}

pub fn get_graph_info(graph: &StateGraph) -> String {
    GraphInfo::new(graph).to_string()
}
