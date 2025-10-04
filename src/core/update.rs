use crate::models::Cell::{BoxOnFloor, BoxOnTarget, Floor, PlayerOnFloor, PlayerOnTarget, Target};
use crate::models::{Cell, Vec2};

pub fn won(grid: &[Vec<Cell>]) -> bool {
    for row in grid {
        for c in row {
            if *c == Target || *c == PlayerOnTarget {
                // if any bare target remains, not yet solved
                return false;
            }
        }
    }
    true
}

pub fn step(grid: &mut [Vec<Cell>], player: &mut Vec2, dir: Vec2) {
    let h = grid.len() as i32;
    let w = grid[0].len() as i32;

    let ni = player.i + dir.i;
    let nj = player.j + dir.j;
    if ni < 0 || nj < 0 || ni >= h || nj >= w {
        println!("Out of range 0? width: {}, height: {}", w, h);
        return;
    }

    let dest = grid[ni as usize][nj as usize];
    let pushing = dest == BoxOnFloor || dest == BoxOnTarget;

    if pushing {
        let bi = ni + dir.i;
        let bj = nj + dir.j;
        if bi < 0 || bj < 0 || bi >= h || bj >= w {
            println!("Out of range 1?");
            return;
        }
        let beyond = grid[bi as usize][bj as usize];
        if !(beyond == Floor || beyond == Target) {
            println!("Out of range 2?");
            return;
        }

        // Move box
        grid[bi as usize][bj as usize] = if beyond == Target {
            BoxOnTarget
        } else {
            BoxOnFloor
        };

        // Clear old box spot (player will step into it)
        grid[ni as usize][nj as usize] = if dest == BoxOnTarget { Target } else { Floor };
    } else {
        if !(dest == Floor || dest == Target) {
            println!("Out of range 3?");
            return;
        }
    }

    // Move player
    let (pi, pj) = (player.i, player.j);
    let cur = grid[pi as usize][pj as usize];
    grid[pi as usize][pj as usize] = if cur == PlayerOnTarget { Target } else { Floor };

    let dest_now = grid[ni as usize][nj as usize];
    grid[ni as usize][nj as usize] = if dest_now == Target {
        PlayerOnTarget
    } else {
        PlayerOnFloor
    };

    player.i = ni;
    player.j = nj;
}
