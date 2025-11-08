// Simple CLI Sokoban with ratatui
// Controls: W/A/S/D or arrow keys (immediate response). Q to quit.
// Tiles: '#' wall, '@' player, '$' box, '.' target, '*' box on target, '+' player on target, ' ' floor.

mod console_interface;
mod core;
mod models;
mod state_graph;
mod test;
mod bevy_interface;

use crate::console_interface::ConsoleInput::*;
use crate::console_interface::{
    cleanup_terminal, handle_input, parse_level, render_game, setup_terminal,
};
use crate::core::{step, GameState, GameUpdate, SharedGameState, TRIM_UNWINNABLE};
use crate::models::GameRenderState;
use crate::state_graph::{get_graph_info, get_json_data, populate_step, render_graph, trim_unwinnable, GraphRenderState, PopulateResult, StateGraph, UniqueNode};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use std::io;
use std::io::Write;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let switch = std::env::args().nth(1).unwrap_or("interactive".to_string());

    let level = r#"
       ####
########  ##
#          ###
# @$$ ##   ..#
# $$   ##  ..#
#         ####
###########
"#;
    //         let level = r#"
    // ########
    // # @$  .#
    // # $  $ #
    // # .# $ #
    // #..#   #
    // ########
    //     "#;
//     let level = r#"
//  ########
//  #  ##  ####
//  # @$   #  ####
//  # ##$ $   #  #
//  # ##  #$ $   ###
//  # #####  #$ $  #
//  # #   ####  #$ #
//  # ##     ####  #
// ## .##       # ##
// #.  .##      # #
// #..  .##     # ##
// ###   .#######  #
//   #  # .        #
//   ###############
// "#;
//     let level = r#"
//    ######
// ####..$@#
// #   #..*#
// #    #* #
// # $#$ ..#
// # $ $ $ #
// #      ##
// ########
// "#;
//     let level = r#"
//     #####
// #####@. #
// #   #.**#
// #  $ #..#
// #  #   .#
// # $$$$ *#
// #    #  #
// #########
// "#;
    let level = r#"
 ### ### 
#   #  .#
#   # . #
##$     #
 # $.* #
  # $##
   #@#
    #
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
    let first_node = UniqueNode::from_game_state(game_state, shared);
    let first_state_id = state_graph.upsert_state(first_node);

    let start_time = std::time::Instant::now();
    let mut last_render_time = start_time;
    let mut processed_since_last_render = 0;

    let mut log_out = std::fs::OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open("exports/solve_log.log")?;

    render_graph(terminal, &mut log_out, GraphRenderState {
        graph: &state_graph,
        processed_since_last_render,
        start_time,
        last_render_time,
        current_time: last_render_time,
    })?;

    'outer: loop {
        let stop_time = std::time::Instant::now() + std::time::Duration::from_millis(1000);
        while std::time::Instant::now() < stop_time {
            let PopulateResult::Populated = populate_step(&mut state_graph, shared) else {
                break 'outer;
            };
            processed_since_last_render += 1;
        }

        let current_time = std::time::Instant::now();
        render_graph(terminal, &mut log_out, GraphRenderState {
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
    if TRIM_UNWINNABLE {
        let trimmed_stats = trim_unwinnable(&mut state_graph, shared);
        println!("Trimmed to only winnable states: {:?}", trimmed_stats  );
        println!("Trimmed {} ({:.1}%) nodes  and {} ({:.1}%) edges",
                 trimmed_stats.nodes_removed(), trimmed_stats.nodes_removed_percentage(),
                 trimmed_stats.edges_removed(), trimmed_stats.edges_removed_percentage());
        println!("{}", get_graph_info(&state_graph));
    }


    // let json_data = get_json_data(&state_graph, shared);
    //
    // std::fs::create_dir_all("exports")?;
    // let mut f = std::fs::OpenOptions::new()
    //     .write(true)
    //     .truncate(true)
    //     .open("exports/state_graph.json")?;
    // f.write_all(json_data.as_bytes())?;
    // println!("State graph exported to exports/state_graph.json");

    // render_interactive_graph(&state_graph);
    
    // Launch 3D graph visualization
    println!("Launching 3D graph visualization...");
    crate::bevy_interface::visualize_graph(first_state_id, &state_graph, shared);
    
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
                    won: shared.is_won(&game_state),
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
