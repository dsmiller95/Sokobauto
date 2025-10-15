use std::cmp::Ordering;
use bevy::math::IVec2;
use crate::core::{Cell, Direction, GameChangeType, GameState, SharedGameState, UserAction};
use crate::core::bounded_grid::BoundedGrid;
use crate::core::bounds::BoundsOriginRoot;
use crate::core::game_state_environment::GameStateEnvironment;
use crate::core::models::Vec2;

pub struct WonCheckHelper {
    target_positions_sorted: Vec<Vec2>,
}

impl WonCheckHelper {
    pub fn is_won(&self, game_state: &GameStateEnvironment) -> bool {
        self.target_positions_sorted.iter().all(|p| game_state.has_box_at(p))
    }
}

impl SharedGameState {
    pub fn height(&self) -> i8 {
        self.grid.len() as i8
    }

    pub fn width(&self) -> i8 {
        if self.grid.is_empty() {
            0
        } else {
            self.grid[0].len() as i8
        }
    }

    pub fn size(&self) -> Vec2 {
        Vec2 {
            i: self.height(),
            j: self.width(),
        }
    }

    pub fn bounds(&self) -> BoundsOriginRoot {
        BoundsOriginRoot{
            extent: self.size().into()
        }
    }

    pub fn get_won_check_helper(&self) -> WonCheckHelper {
        let mut target_positions = Vec::new();
        for (i, row) in self.grid.iter().enumerate() {
            for (j, &c) in row.iter().enumerate() {
                if c == Cell::Target {
                    target_positions.push(Vec2 { i: i as i8, j: j as i8 });
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
                    let pos = Vec2 { i: i as i8, j: j as i8 };
                    if !game_state.environment.has_box_at(&pos) {
                        return false;
                    }
                }
            }
        }
        true
    }

    pub fn count_boxes_on_goals(&self, environment: &GameStateEnvironment) -> usize {
        let mut count = 0;
        for (i, row) in self.grid.iter().enumerate() {
            for (j, &c) in row.iter().enumerate() {
                if c == Cell::Target {
                    let pos = Vec2 { i: i as i8, j: j as i8 };
                    if environment.has_box_at(&pos) {
                        count += 1;
                    }
                }
            }
        }
        count
    }
    
    pub fn total_targets(&self) -> usize {
        self.grid.iter().flat_map(|r| r.iter())
            .filter(|c| **c == Cell::Target)
            .count()
    }

    pub fn reachable_positions(&self, game_state: &GameState) -> Vec<Vec2> {
        let mut reachable = Vec::<Vec2>::new();
        self.visit_all_reachable_position(game_state, |pos| {
            reachable.push((*pos).into());
        });
        reachable
    }

    pub fn min_reachable_position(&self, game_state: &GameState) -> Vec2 {
        let mut min_reachable = Vec2 { i: i8::MAX, j: i8::MAX };
        self.visit_all_reachable_position(game_state, |&pos| {
            let pos: Vec2 = pos.into();
            if pos < min_reachable {
                min_reachable = pos;
            }
        });
        min_reachable
    }

    pub fn reachable_positions_visitation(&self, game_state: &GameState) -> BoundedGrid<VisitationState> {
        self.visit_all_reachable_position(game_state, |_| {})
    }

    fn visit_all_reachable_position(&self, game_state: &GameState, mut next_reachable: impl FnMut(&IVec2)) -> BoundedGrid<VisitationState> {
        let mut visited = BoundedGrid::<VisitationState>::new(self.bounds(), VisitationState::Walkable);
        let mut stack: Vec<IVec2> = vec![game_state.player.into()];

        for &box_pos in game_state.environment.iter_boxes() {
            let pos = box_pos.into();
            if self.bounds().contains(&pos) {
                visited[&pos] = VisitationState::Blocked;
            }
        }

        while let Some(pos) = stack.pop() {
            if visited[&pos] != VisitationState::Walkable {
                continue;
            }
            visited[&pos] = VisitationState::Visited;

            next_reachable(&pos);

            for new_pos in pos.neighbors() {
                if visited.contains(&new_pos) &&
                    visited[&new_pos] == VisitationState::Walkable &&
                    self[new_pos].is_walkable() {
                    stack.push(new_pos);
                }
            }
        }

        visited
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum VisitationState {
    Walkable,
    Blocked,
    Visited,
}

impl VisitationState {
    pub fn is_reachable(&self) -> bool {
        *self == VisitationState::Visited
    }
}

impl std::ops::Index<Vec2> for SharedGameState {
    type Output = Cell;

    fn index(&self, index: Vec2) -> &Self::Output {
        let index: IVec2 = index.into();
        &self[index]
    }
}

impl std::ops::Index<IVec2> for SharedGameState {
    type Output = Cell;

    fn index(&self, index: IVec2) -> &Self::Output {
        &self.grid[index.y as usize][index.x as usize]
    }
}

const ALL_PUSH_ACTIONS_AROUND: &[(Vec2, UserAction)] = &[
    (Vec2 { i: -1, j: 0 }, UserAction::Move(Direction::Down)),
    (Vec2 { i: 1, j: 0 }, UserAction::Move(Direction::Up)),
    (Vec2 { i: 0, j: -1 }, UserAction::Move(Direction::Right)),
    (Vec2 { i: 0, j: 1 }, UserAction::Move(Direction::Left)),
];

impl UserAction {
    pub fn all_push_actions_around(&pos: &Vec2) -> impl Iterator<Item=(Vec2, UserAction)> {
        ALL_PUSH_ACTIONS_AROUND.iter().map(move |&(neighbor, action)| {
            (neighbor + pos, action)
        })
    }
}

impl Default for Vec2 {
    fn default() -> Self {
        Vec2 { i: 0, j: 0 }
    }
}

impl From<IVec2> for Vec2 {
    fn from(value: IVec2) -> Self {
        Vec2 {
            i: value.y.try_into().expect("must fit in i8"),
            j: value.x.try_into().expect("must fit in i8"),
        }
    }
}

impl From<Vec2> for IVec2 {
    fn from(value: Vec2) -> Self {
        IVec2 { x: value.j as i32, y: value.i as i32 }
    }
}

pub trait Vec2GameLogicAdapter {
    fn neighbors(&self) -> [Self; 4] where Self: Sized;

    fn cmp(&self, other: &Self) -> std::cmp::Ordering;
}

impl Vec2GameLogicAdapter for Vec2 {
    fn neighbors(&self) -> [Vec2; 4] {
        [
            Vec2 { i: self.i - 1, j: self.j },
            Vec2 { i: self.i + 1, j: self.j },
            Vec2 { i: self.i, j: self.j - 1 },
            Vec2 { i: self.i, j: self.j + 1 },
        ]
    }

    fn cmp(&self, other: &Self) -> Ordering {
        Ord::cmp(self, other)
    }
}

impl Vec2GameLogicAdapter for IVec2 {
    fn neighbors(&self) -> [IVec2; 4] {
        [
            IVec2 { y: self.y - 1, x: self.x },
            IVec2 { y: self.y + 1, x: self.x },
            IVec2 { y: self.y, x: self.x - 1 },
            IVec2 { y: self.y, x: self.x + 1 },
        ]
    }

    fn cmp(&self, other: &Self) -> Ordering {
        let x = self.x.cmp(&other.x);
        let y = self.y.cmp(&other.y);
        match (x, y) {
            (Ordering::Equal, second) => second,
            (first, _) => first,
        }
    }
}

impl Vec2 {
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
