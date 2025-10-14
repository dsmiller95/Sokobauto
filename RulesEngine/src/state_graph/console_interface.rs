use std::fs::File;
use crate::state_graph::StateGraph;
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use ratatui::layout::Alignment;
use ratatui::prelude::{Color, Style};
use ratatui::widgets::*;
use std::io;
use std::io::Write;

pub struct GraphRenderState<'a> {
    pub graph: &'a StateGraph,
    pub processed_since_last_render: usize,
    pub start_time: std::time::Instant,
    pub last_render_time: std::time::Instant,
    pub current_time: std::time::Instant,
}

pub fn render_graph(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    file_out: &mut File,
    render: GraphRenderState,
) -> Result<(), Box<dyn std::error::Error>> {

    let graph_info = GraphInfo::new(render.graph);

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

    file_out.write(graph_info.to_log_string().as_bytes())?;

    let description = format!("{}\n{}", graph_info.to_human_string(), time_description);

    terminal.draw(|f| {
        let size = f.area();

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
        let unvisited = graph.unvisited.len();
        let visited = nodes - unvisited;
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

    pub fn to_human_string(&self) -> String {
        format!(
            "Graph has {} nodes, {} edges, {} visited nodes ({:.1}%), {} unvisited.",
            self.nodes, self.edges, self.visited, self.percent_visited, self.nodes - self.visited
        )
    }

    pub fn to_log_string(&self) -> String {
        format!(
            "nodes: {}, edges: {}, visited: {}\n",
            self.nodes, self.edges, self.visited
        )
    }
}

pub fn get_graph_info(graph: &StateGraph) -> String {
    GraphInfo::new(graph).to_human_string()
}
