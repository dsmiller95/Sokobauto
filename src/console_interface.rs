use std::io;
use std::io::Read;
use crate::models::Cell::{
    BoxOnFloor, BoxOnTarget, Floor, PlayerOnFloor, PlayerOnTarget, Target, Wall,
};
use crate::models::{Cell, Direction, UserAction, Vec2};

pub fn parse_level(s: &str) -> (Vec<Vec<Cell>>, Vec2) {
    let mut grid: Vec<Vec<Cell>> = Vec::new();
    let mut player = Vec2 { i: 0, j: 0 };
    let max_width = s.lines().map(|line| line.len()).max().unwrap_or(0);
    for (i, line) in s.lines().enumerate() {
        let mut row = Vec::new();
        for (j, ch) in line.chars().enumerate() {
            let c = match ch {
                '#' => Wall,
                ' ' => Floor,
                '.' => Target,
                '$' => BoxOnFloor,
                '*' => BoxOnTarget,
                '@' => {
                    player = Vec2 {
                        i: i as i32,
                        j: j as i32,
                    };
                    PlayerOnFloor
                }
                '+' => {
                    player = Vec2 {
                        i: i as i32,
                        j: j as i32,
                    };
                    PlayerOnTarget
                }
                _ => Floor,
            };
            row.push(c);
        }
        // Pad row to max width with Floor
        while row.len() < max_width {
            row.push(Floor);
        }
        grid.push(row);
    }

    (grid, player)
}

pub fn render(grid: &[Vec<Cell>]) {
    // Clear screen and move cursor to home (ANSI)
    // print!("\x1b[2J\x1b[H");
    for row in grid {
        for c in row {
            let ch = match c {
                Wall => '#',
                Floor => ' ',
                Target => '.',
                BoxOnFloor => '$',
                BoxOnTarget => '*',
                PlayerOnFloor => '@',
                PlayerOnTarget => '+',
            };
            print!("{ch}");
        }
        println!();
    }
    println!("\nMove: WASD / arrows + Enter. Q to quit.");
}


pub fn dir_from_input(byte: u8) -> Option<Direction> {
    match byte {
        b'w' | b'W' => Some(Direction::Up),
        b's' | b'S' => Some(Direction::Down),
        b'a' | b'A' => Some(Direction::Left),
        b'd' | b'D' => Some(Direction::Right),
        _ => None,
    }
}

pub fn input_from_console() -> Option<UserAction> {
    io::stdin()
        .bytes()
        .next()
        .and_then(|result| result.ok())
        .and_then(dir_from_input)
        .map(|dir| UserAction{ dir })
}
