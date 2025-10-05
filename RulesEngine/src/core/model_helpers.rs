use crate::core::{Cell, GameState, SharedGameState, UserAction};
use crate::models::Vec2;

impl GameState {
    pub fn is_won(&self, shared: &SharedGameState) -> bool {
        for (i, row) in shared.grid.iter().enumerate() {
            for (j, &c) in row.iter().enumerate() {
                if c == Cell::Target {
                    let pos = Vec2 { i: i as i32, j: j as i32 };
                    if !self.boxes.contains(&pos) {
                        return false;
                    }
                }
            }
        }
        true
    }
}

impl SharedGameState {
    pub fn height(&self) -> i32 {
        self.grid.len() as i32
    }

    pub fn width(&self) -> i32 {
        if self.grid.is_empty() {
            0
        } else {
            self.grid[0].len() as i32
        }
    }

    pub fn count_boxes_on_goals(&self, boxes: &Vec<Vec2>) -> usize {
        let mut count = 0;
        for (i, row) in self.grid.iter().enumerate() {
            for (j, &c) in row.iter().enumerate() {
                if c == Cell::Target {
                    let pos = Vec2 { i: i as i32, j: j as i32 };
                    if boxes.contains(&pos) {
                        count += 1;
                    }
                }
            }
        }
        count
    }
}

impl UserAction {
    pub fn all_actions() -> Vec<UserAction> {
        vec![
            UserAction::Move(crate::core::Direction::Up),
            UserAction::Move(crate::core::Direction::Down),
            UserAction::Move(crate::core::Direction::Left),
            UserAction::Move(crate::core::Direction::Right),
        ]
    }
}
