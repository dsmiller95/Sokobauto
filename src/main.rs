// Simple CLI Sokoban (no dependencies)
// Controls: W/A/S/D or arrow keys, then Enter. Q to quit.
// Tiles: '#' wall, '@' player, '$' box, '.' target, '*' box on target, '+' player on target, ' ' floor.

mod console_interface;
mod core;
mod models;

use std::io::{self, Read};

use crate::console_interface::{input_from_console, parse_level, render};
use crate::core::{step, won};

fn main() {
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
    render(&grid);

    loop {
        // Read a line (allows both single chars like 'w' and arrow escape sequences).
        if let Some(userAction) = input_from_console() {
            step(&mut grid, &mut player, userAction);
            render(&grid);
            if won(&grid) {
                println!("You win! 🎉");
                break;
            }
        } else {
            // Ignore unknown input; re-render to keep the screen tidy
            render(&grid);
        }
    }
}
