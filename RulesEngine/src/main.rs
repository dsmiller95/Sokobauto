// Simple CLI Sokoban with ratatui
// Controls: W/A/S/D or arrow keys (immediate response). Q to quit.
// Tiles: '#' wall, '@' player, '$' box, '.' target, '*' box on target, '+' player on target, ' ' floor.

mod console_interface;
mod core;
mod models;
mod state_graph;
mod tests;

use crate::console_interface::ConsoleInput::*;
use crate::console_interface::{
    cleanup_terminal, handle_input, parse_level, render_game, setup_terminal,
};
use crate::core::{GameState, GameUpdate, step, SharedGameState};
use crate::models::GameRenderState;
use crate::state_graph::{PopulateResult, StateGraph, get_graph_info, get_json_data, populate_step, render_graph, render_interactive_graph, GraphRenderState, get_box_only_graph};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use std::io;
use std::io::Write;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let switch = std::env::args().nth(1).unwrap_or("interactive".to_string());

    // A tiny built-in level (Sokoban-like)
    // You can add more and switch by index.
//     let level = r#"
//         ####
// #########  ##
// #           ###
// #  @$$ ##   ..#
// #  $$   ##  ..#
// #          ####
// ############
// "#;
    let level = r#"
########
#  $  .#
#  @ $ #
# .#   #
########
"#;

    let (game_state, shared) = parse_level(level);
    let mut terminal = setup_terminal()?;

    match switch.as_str() {
        "graph" => {
            run_state_graph(&shared, game_state, &mut terminal)?;
        }
        "interactive" => {
            run_interactive(&shared, game_state, &mut terminal)?;
        }
        _ => {
            println!(
                "Unknown mode: {}. Use 'interactive' or 'graph'. defaulting to interactive",
                switch
            );
            run_interactive(&shared, game_state, &mut terminal)?;
        }
    }

    Ok(())
}

fn run_state_graph(
    shared: &SharedGameState,
    game_state: GameState,
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut state_graph = StateGraph::new();
    state_graph.upsert_state(game_state);

    let start_time = std::time::Instant::now();
    let mut last_render_time = start_time;
    let mut processed_since_last_render = 0;

    render_graph(terminal, GraphRenderState {
        graph: &state_graph,
        processed_since_last_render,
        start_time,
        last_render_time,
        current_time: last_render_time,
    })?;

    'outer: loop {
        let stop_time = std::time::Instant::now() + std::time::Duration::from_millis(100);
        while std::time::Instant::now() < stop_time {
            let PopulateResult::Populated = populate_step(&mut state_graph, shared) else {
                break 'outer;
            };
            processed_since_last_render += 1;
        }

        let current_time = std::time::Instant::now();
        render_graph(terminal, GraphRenderState {
            graph: &state_graph,
            processed_since_last_render,
            start_time,
            last_render_time,
            current_time,
        })?;

        last_render_time = current_time;
        processed_since_last_render = 0;
    }

    cleanup_terminal()?;

    println!("{}", get_graph_info(&state_graph));

    let box_only_graph = get_box_only_graph(&state_graph);
    let json_data = get_json_data(&box_only_graph, shared);

    std::fs::create_dir_all("exports")?;
    let mut f = std::fs::OpenOptions::new()
        .write(true)
        .truncate(true)
        .open("exports/state_graph.json")?;
    f.write_all(json_data.as_bytes())?;
    println!("State graph exported to exports/state_graph.json");

    render_interactive_graph(&box_only_graph);
    Ok(())
}

fn run_interactive(
    shared: &SharedGameState,
    game_state: GameState,
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut game_state = game_state;
    // Initial render
    let first_render = GameRenderState {
        game: game_state.clone(),
        won: false,
        error: None,
        last_change: None,
    };
    render_game(terminal, shared, &first_render)?;

    loop {
        match handle_input() {
            Ok(Quit) => break,
            Ok(UserAction(user_action)) => {
                let game_update = step(shared, &game_state, user_action);
                let mut change = None;
                if let GameUpdate::NextState(new_state, change_type) = &game_update {
                    game_state = new_state.clone();
                    change = Some(change_type.clone());
                }
                let to_render = GameRenderState {
                    game: game_state.clone(),
                    won: game_state.is_won(shared),
                    error: match game_update {
                        GameUpdate::Error(err) => Some(err),
                        _ => None,
                    },
                    last_change: change,
                };
                render_game(terminal, shared, &to_render)?;

                if to_render.won {
                    // Keep showing the win screen until user inputs
                    loop {
                        match handle_input() {
                            Ok(Timeout) => {}
                            Ok(_) => break,
                            Err(_) => {
                                println!("error reading input");
                                break;
                            }
                        }
                    }
                    break;
                }
            }
            Ok(_) => {
                // No input, continue polling
            }
            Err(_) => {
                println!("error reading input");
                break;
            }
        }
    }

    cleanup_terminal()?;
    Ok(())
}
