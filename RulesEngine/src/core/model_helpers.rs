use crate::core::{Cell, Direction, GameChangeType, GameState, GameStateEnvironment, SharedGameState, UserAction};
use crate::core::bounded_grid::BoundedGrid;
use crate::models::Vec2;

pub struct WonCheckHelper {
    target_positions_sorted: Vec<Vec2>,
}

impl WonCheckHelper {
    pub fn is_won(&self, game_state: &GameStateEnvironment) -> bool {
        self.target_positions_sorted.iter().all(|p| game_state.boxes.contains(p))
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

    pub fn get_won_check_helper(&self) -> WonCheckHelper {
        let mut target_positions = Vec::new();
        for (i, row) in self.grid.iter().enumerate() {
            for (j, &c) in row.iter().enumerate() {
                if c == Cell::Target {
                    target_positions.push(Vec2 { i: i as i32, j: j as i32 });
                }
            }
        }
        target_positions.sort();
        WonCheckHelper {
            target_positions_sorted: target_positions,
        }
    }

    pub fn is_won(&self, game_state: &GameState) -> bool {
        for (i, row) in self.grid.iter().enumerate() {
            for (j, &c) in row.iter().enumerate() {
                if c == Cell::Target {
                    let pos = Vec2 { i: i as i32, j: j as i32 };
                    if !game_state.environment.boxes.contains(&pos) {
                        return false;
                    }
                }
            }
        }
        true
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
        self.visit_all_reachable_position(game_state, |pos| {
            reachable.push(*pos);
        });
        reachable
    }

    pub fn min_reachable_position(&self, game_state: &GameState) -> Vec2 {
        let mut min_reachable = Vec2 { i: i32::MAX, j: i32::MAX };
        self.visit_all_reachable_position(game_state, |pos| {
            if pos < &min_reachable {
                min_reachable = *pos;
            }
        });
        min_reachable
    }

    fn visit_all_reachable_position(&self, game_state: &GameState, mut next_reachable: impl FnMut(&Vec2)) {
        let mut visited = BoundedGrid::<VisitationState>::new(self.area(), VisitationState::Walkable);
        let mut stack = vec![game_state.player];

        for box_pos in &game_state.environment.boxes {
            if box_pos.inside(&self.area()) {
                visited[*box_pos] = VisitationState::Blocked;
            }
        }

        while let Some(pos) = stack.pop() {
            if visited[pos] != VisitationState::Walkable {
                continue;
            }
            visited[pos] = VisitationState::Visited;

            next_reachable(&pos);

            for new_pos in pos.neighbors() {
                if visited.contains(new_pos) &&
                    visited[new_pos] == VisitationState::Walkable &&
                    self[new_pos].is_walkable() {
                    stack.push(new_pos);
                }
            }
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum VisitationState {
    Walkable,
    Blocked,
    Visited,
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
            UserAction::Move(Direction::Up),
            UserAction::Move(Direction::Down),
            UserAction::Move(Direction::Left),
            UserAction::Move(Direction::Right),
        ]
    }
    
    pub fn all_push_actions_around(pos: &Vec2) -> Vec<(Vec2, UserAction)> {
        vec![
            (*pos + Vec2 { i: -1, j: 0 }, UserAction::Move(Direction::Down)),
            (*pos + Vec2 { i: 1, j: 0 }, UserAction::Move(Direction::Up)),
            (*pos + Vec2 { i: 0, j: -1 }, UserAction::Move(Direction::Right)),
            (*pos + Vec2 { i: 0, j: 1 }, UserAction::Move(Direction::Left)),
        ]
    }
}

impl Default for Vec2 {
    fn default() -> Self {
        Vec2 { i: 0, j: 0 }
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