use crate::core::{Cell, GameChangeType, GameState, SharedGameState, UserAction};
use crate::models::Vec2;

impl GameState {
    pub fn is_won(&self, shared: &SharedGameState) -> bool {
        for (i, row) in shared.grid.iter().enumerate() {
            for (j, &c) in row.iter().enumerate() {
                if c == Cell::Target {
                    let pos = Vec2 { i: i as i32, j: j as i32 };
                    if !self.environment.boxes.contains(&pos) {
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

    pub fn area(&self) -> Vec2 {
        Vec2 {
            i: self.height(),
            j: self.width(),
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

    pub fn reachable_positions(&self, game_state: &GameState) -> Vec<Vec2> {
        let mut reachable = Vec::new();
        let mut visited = std::collections::HashSet::new();
        let mut stack = vec![game_state.player];
        let area = self.area();

        while let Some(pos) = stack.pop() {
            if visited.contains(&pos) {
                continue;
            }
            visited.insert(pos);
            reachable.push(pos);

            for new_pos in pos.neighbors() {
                if new_pos.inside(&area) {
                    if self[new_pos].is_walkable() && !game_state.environment.boxes.contains(&new_pos) {
                        stack.push(new_pos);
                    }
                }
            }
        }

        reachable
    }
}

impl std::ops::Index<Vec2> for SharedGameState {
    type Output = Cell;

    fn index(&self, index: Vec2) -> &Self::Output {
        &self.grid[index.i as usize][index.j as usize]
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

impl Vec2 {
    pub fn neighbors(&self) -> [Vec2; 4] {
        [
            Vec2 { i: self.i - 1, j: self.j },
            Vec2 { i: self.i + 1, j: self.j },
            Vec2 { i: self.i, j: self.j - 1 },
            Vec2 { i: self.i, j: self.j + 1 },
        ]
    }

    pub fn inside(&self, area: &Vec2) -> bool {
        self.i >= 0 && self.j >= 0 && self.i < area.i && self.j < area.j
    }
}

impl std::ops::Add for Vec2 {
    type Output = Vec2;

    fn add(self, rhs: Self) -> Self::Output {
        Vec2 {
            i: self.i + rhs.i,
            j: self.j + rhs.j,
        }
    }
}

impl std::ops::Sub for Vec2 {
    type Output = Vec2;

    fn sub(self, rhs: Self) -> Self::Output {
        Vec2 {
            i: self.i - rhs.i,
            j: self.j - rhs.j,
        }
    }
}

impl GameChangeType {
    pub fn did_box_move(&self) -> bool {
        matches!(self, GameChangeType::PlayerAndBoxMove)
    }
}

impl Cell {
    pub fn is_walkable(&self) -> bool {
        *self != Cell::Wall
    }
}