// Simple CLI Sokoban with ratatui
// Controls: W/A/S/D or arrow keys (immediate response). Q to quit.
// Tiles: '#' wall, '@' player, '$' box, '.' target, '*' box on target, '+' player on target, ' ' floor.

mod console_interface;
mod core;
mod models;

use crate::console_interface::{cleanup_terminal, handle_input, parse_level, render_game, setup_terminal};
use crate::core::{step, GameUpdate, UserAction};
use crate::models::GameRenderState;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // A tiny built-in level (Sokoban-like)
    // You can add more and switch by index.
    let level = r#"
  #######
  #  .  #
  #  $  #
### $# ###
#   @   #
###   ###
  #  .  #
  #######
"#;

    let mut game_state = parse_level(level);
    let mut terminal = setup_terminal()?;

    // Initial render
    let first_render = GameRenderState {
        game: game_state.clone(),
        won: false,
        error: None,
    };
    render_game(&mut terminal, &first_render)?;

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
                render_game(&mut terminal, &to_render)?;

                if to_render.won {
                    // Keep showing the win screen until user quits
                    loop {
                        if let Err(_) = handle_input() {
                            break;
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
