use crate::core::{Cell, GameState, UserAction};

impl GameState {
    pub fn is_won(&self) -> bool {
        for row in &self.grid {
            for c in row {
                if *c == Cell::Target || *c == Cell::PlayerOnTarget {
                    return false;
                }
            }
        }
        true
    }
    
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