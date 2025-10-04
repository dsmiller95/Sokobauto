use crate::core::Cell::{BoxOnFloor, BoxOnTarget, Floor, PlayerOnFloor, PlayerOnTarget, Target};
use crate::core::{Direction, GameChangeType, GameState, GameUpdate, UserAction, Vec2};

pub fn step(game: &GameState, action: UserAction) -> GameUpdate {
    let h = game.height();
    let w = game.width();

    let dir = match action {
        UserAction::Move(d) => vec_from_dir(d),
    };

    let ni = game.player.i + dir.i;
    let nj = game.player.j + dir.j;
    if ni < 0 || nj < 0 || ni >= h || nj >= w {
        return GameUpdate::Error("Cannot move out of bounds".to_string());
    }

    let dest = game.grid[ni as usize][nj as usize];
    let pushing = dest == BoxOnFloor || dest == BoxOnTarget;

    let mut new_grid = game.grid.clone();

    if pushing {
        let bi = ni + dir.i;
        let bj = nj + dir.j;
        if bi < 0 || bj < 0 || bi >= h || bj >= w {
            return GameUpdate::Error("Cannot push block out of bounds".to_string());
        }
        let beyond = new_grid[bi as usize][bj as usize];
        if !(beyond == Floor || beyond == Target) {
            return GameUpdate::Error("Cannot push block".to_string());
        }

        // Move box
        new_grid[bi as usize][bj as usize] = if beyond == Target {
            BoxOnTarget
        } else {
            BoxOnFloor
        };

        // Clear old box spot (player will step into it)
        new_grid[ni as usize][nj as usize] = if dest == BoxOnTarget { Target } else { Floor };
    } else {
        if !(dest == Floor || dest == Target) {
            return GameUpdate::Error("Cannot walk into a wall".to_string());
        }
    }

    // Move player
    let (pi, pj) = (game.player.i, game.player.j);
    let cur = new_grid[pi as usize][pj as usize];
    new_grid[pi as usize][pj as usize] = if cur == PlayerOnTarget { Target } else { Floor };

    let dest_now = new_grid[ni as usize][nj as usize];
    new_grid[ni as usize][nj as usize] = if dest_now == Target {
        PlayerOnTarget
    } else {
        PlayerOnFloor
    };

    GameUpdate::NextState(
        GameState {
            grid: new_grid,
            player: Vec2 { i: ni, j: nj },
        },
        if pushing {
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
