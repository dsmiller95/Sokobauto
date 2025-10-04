// Simple CLI Sokoban (no dependencies)
// Controls: W/A/S/D or arrow keys, then Enter. Q to quit.
// Tiles: '#' wall, '@' player, '$' box, '.' target, '*' box on target, '+' player on target, ' ' floor.

mod console_interface;
mod core;
mod models;

use std::io::{self, Read};

use crate::console_interface::{dir_from_input, parse_level, render};
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
        let mut buf = [0u8; 8];
        let mut input = io::stdin().lock();
        let n = match input.read(&mut buf) {
            Ok(n) => n,
            Err(_) => 0,
        };
        if n == 0 {
            break;
        }
        // Quit on 'q' or EOF after showing win/lose status
        if buf[0] == b'q' || buf[0] == b'Q' {
            println!("Bye!");
            break;
        }

        if let Some(d) = dir_from_input(&buf[..n]) {
            step(&mut grid, &mut player, d);
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
