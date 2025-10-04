#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum Cell {
    Wall,
    Floor,
    Target,
    BoxOnFloor,
    BoxOnTarget,
    PlayerOnFloor,
    PlayerOnTarget,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Vec2 {
    pub i: i32,
    pub j: i32,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum UserAction {
    Move(Direction),
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct GameState {
    pub grid: Vec<Vec<Cell>>,
    pub player: Vec2,
}

pub enum GameUpdate{
    NextState(GameState, GameChangeType),
    Error(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GameChangeType {
    PlayerMove,
    PlayerAndBoxMove,
}