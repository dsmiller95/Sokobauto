// Simple CLI Sokoban with ratatui
// Controls: W/A/S/D or arrow keys (immediate response). Q to quit.
// Tiles: '#' wall, '@' player, '$' box, '.' target, '*' box on target, '+' player on target, ' ' floor.

mod console_interface;
mod core;
mod models;

use crate::console_interface::{cleanup_terminal, handle_input, parse_level, render_game, setup_terminal};
use crate::core::{step, won};
use crate::models::UserAction;

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

    let (mut grid, mut player) = parse_level(level);
    let mut terminal = setup_terminal()?;

    // Initial render
    render_game(&mut terminal, &grid, false)?;

    loop {
        match handle_input() {
            Ok(Some(UserAction::Quit)) => break,
            Ok(Some(user_action)) => {
                step(&mut grid, &mut player, user_action);
                let game_won = won(&grid);
                render_game(&mut terminal, &grid, game_won)?;

                if game_won {
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
