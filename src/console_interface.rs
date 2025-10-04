use crate::models::Cell::{
    BoxOnFloor, BoxOnTarget, Floor, PlayerOnFloor, PlayerOnTarget, Target, Wall,
};
use crate::models::{Cell, Vec2};

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

pub fn dir_from_input(bytes: &[u8]) -> Option<Vec2> {
    // Accept first meaningful byte/sequence: arrows start with 0x1B '[' 'A/B/C/D'
    // Also support WASD/wads.
    if bytes.is_empty() {
        return None;
    }
    match bytes[0] {
        b'w' | b'W' => Some(Vec2 { i: -1, j: 0 }),
        b's' | b'S' => Some(Vec2 { i: 1, j: 0 }),
        b'a' | b'A' => Some(Vec2 { i: 0, j: -1 }),
        b'd' | b'D' => Some(Vec2 { i: 0, j: 1 }),
        0x1B => {
            // arrow sequence
            if bytes.len() >= 3 && bytes[1] == b'[' {
                match bytes[2] {
                    b'A' => Some(Vec2 { i: -1, j: 0 }), // up
                    b'B' => Some(Vec2 { i: 1, j: 0 }),  // down
                    b'D' => Some(Vec2 { i: 0, j: -1 }), // left
                    b'C' => Some(Vec2 { i: 0, j: 1 }),  // right
                    _ => None,
                }
            } else {
                None
            }
        }
        b'q' | b'Q' => None,
        _ => None,
    }
}
