// Simple CLI Sokoban with ratatui
// Controls: W/A/S/D or arrow keys (immediate response). Q to quit.
// Tiles: '#' wall, '@' player, '$' box, '.' target, '*' box on target, '+' player on target, ' ' floor.

mod console_interface;
mod core;
mod models;
mod state_graph;

use std::io;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use crate::console_interface::{cleanup_terminal, handle_input, parse_level, render_game, setup_terminal};
use crate::core::{step, GameState, GameUpdate, UserAction};
use crate::models::GameRenderState;
use crate::state_graph::{get_graph_info, populate_step, render_graph, render_interactive_graph, PopulateResult, StateGraph};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // A tiny built-in level (Sokoban-like)
    // You can add more and switch by index.
    let level = r#"
#######
#. $ @#
#######
"#;

    let game_state = parse_level(level);
    let mut terminal = setup_terminal()?;

    run_state_graph(game_state, &mut terminal)?;
    // run_interactive(game_state, &mut terminal)?;
    Ok(())
}

fn run_state_graph(game_state: GameState, terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<(), Box<dyn std::error::Error>> {
    let mut state_graph = StateGraph::new();
    state_graph.upsert_state(game_state);
    render_graph(terminal, &state_graph)?;

    'outer: loop {
        let stop_time = std::time::Instant::now() + std::time::Duration::from_millis(100);
        while std::time::Instant::now() < stop_time {
            let PopulateResult::Populated(populated_node) = populate_step(&mut state_graph) else {
                break 'outer;
            };
        }
        render_graph(terminal, &state_graph)?;
    }

    cleanup_terminal()?;

    println!("{}", get_graph_info(&state_graph));
    
    render_interactive_graph(&state_graph);
    Ok(())
}

fn run_interactive(game_state: GameState, terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<(), Box<dyn std::error::Error>> {
    let mut game_state = game_state;
    // Initial render
    let first_render = GameRenderState {
        game: game_state.clone(),
        won: false,
        error: None,
    };
    render_game(terminal, &first_render)?;

    loop {
        match handle_input() {
            Ok(Some(UserAction::Quit)) => break,
            Ok(Some(user_action)) => {
                let game_update = step(&game_state, user_action);
                if let GameUpdate::NextState(new_state) = &game_update {
                    game_state = new_state.clone();
                }
                let to_render = GameRenderState{
                    game: game_state.clone(),
                    won: game_state.is_won(),
                    error: match game_update {
                        GameUpdate::Error(err) => Some(err),
                        _ => None,
                    },
                };
                render_game(terminal, &to_render)?;

                if to_render.won {
                    // Keep showing the win screen until user quits
                    loop {
                        match handle_input() {
                            Ok(None) => {}
                            Ok(Some(_)) => break,
                            Err(_) => {
                                println!("error reading input");
                                break;
                            }
                        }
                    }
                    break;
                }
            }
            Ok(None) => {
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