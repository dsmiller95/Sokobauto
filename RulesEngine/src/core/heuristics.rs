use petgraph::visit::IntoNeighbors;
use crate::core::{GameState, SharedGameState};
use crate::core::Cell::Target;
use crate::core::models::Vec2;

#[derive(Eq, PartialEq, Debug)]
pub enum WinnableState {
    WinMaybePossible,
    WinImpossible,
}

const DIRECTIONS_AROUND: [Vec2; 4] =
[
    Vec2 { i: 1, j: 0 },
    Vec2 { i: 0, j: 1 },
    Vec2 { i: -1, j: 0 },
    Vec2 { i: 0, j: -1 },
];

pub fn is_winnable(
    shared: &SharedGameState,
    game: &GameState) -> WinnableState {

    let mut total_trapped_boxes = 0;
    for &game_box in game.environment.iter_boxes() {
        if is_box_trapped(shared, game_box) {
            total_trapped_boxes += 1;
        }
    }

    let total_free_boxes = game.environment.iter_boxes().count() - total_trapped_boxes;
    let total_targets = shared.total_targets();

    if total_free_boxes >= total_targets {
        WinnableState::WinMaybePossible
    } else {
        WinnableState::WinImpossible
    }
}

/// a box is trapped if the player can never move it, and it is not on a target
fn is_box_trapped(shared: &SharedGameState, game_box: Vec2) -> bool {
    let is_target = shared[game_box] == Target;
    if is_target {
        return false;
    }

    // if any 2 consecutive directions are blocked, then we are in a corner, and we are trapped
    let blocked_directions = DIRECTIONS_AROUND.iter().map(|&dir| {
        let cell = shared[game_box + dir];
        !cell.is_walkable()
    }).collect::<Vec<_>>();

    for i in 0..4 {
        if blocked_directions[i] && blocked_directions[(i + 1) % 4] {
            return true;
        }
    }

    false
}