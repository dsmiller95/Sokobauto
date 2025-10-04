use std::ops::IndexMut;
use crate::core::Cell::{Floor, Target, Wall};
use crate::core::{Direction, GameChangeType, GameState, GameUpdate, UserAction, Vec2};

pub fn step(game: &GameState, action: UserAction) -> GameUpdate {
    let h = game.height();
    let w = game.width();

    let dir = match action {
        UserAction::Move(d) => vec_from_dir(d),
    };


    let ni = game.player.i + dir.i;
    let nj = game.player.j + dir.j;
    let dest_pos = Vec2{
        i: ni,
        j: nj,
    };
    if ni < 0 || nj < 0 || ni >= h || nj >= w {
        return GameUpdate::Error("Cannot move out of bounds".to_string());
    }

    let dest = game.grid[ni as usize][nj as usize];
    
    let pushing = game.boxes.iter().position(|&x| x == dest_pos);
    let mut new_boxes = game.boxes.clone();
    if let Some(pushed_box_index) = pushing {
        let bi = ni + dir.i;
        let bj = nj + dir.j;
        let new_box_pos = Vec2{
            i: bi,
            j: bj,
        };
        if bi < 0 || bj < 0 || bi >= h || bj >= w {
            return GameUpdate::Error("Cannot push block out of bounds".to_string());
        }
        let beyond = game.grid[bi as usize][bj as usize];
        if beyond == Wall {
            return GameUpdate::Error("Cannot push block into wall".to_string());
        }
        if game.boxes.contains(&new_box_pos) {
            return GameUpdate::Error("Cannot push block into another block".to_string());
        }

        new_boxes[pushed_box_index] = new_box_pos;
    } else {
        if !(dest == Floor || dest == Target) {
            return GameUpdate::Error("Cannot walk into a wall".to_string());
        }
    }

    GameUpdate::NextState(
        GameState {
            // TODO: don't clone
            grid: game.grid.clone(),
            player: dest_pos,
            boxes: new_boxes,
        },
        if pushing.is_some() {
            GameChangeType::PlayerAndBoxMove
        } else {
            GameChangeType::PlayerMove
        },
    )
}

fn vec_from_dir(dir: Direction) -> Vec2 {
    match dir {
        Direction::Up => Vec2 { i: -1, j: 0 },
        Direction::Down => Vec2 { i: 1, j: 0 },
        Direction::Left => Vec2 { i: 0, j: -1 },
        Direction::Right => Vec2 { i: 0, j: 1 },
    }
}
