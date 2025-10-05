#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum Cell {
    Wall,
    Floor,
    Target,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[derive(PartialOrd, Ord)]
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

pub struct SharedGameState {
    pub grid: Vec<Vec<Cell>>,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct GameState {
    pub boxes: Vec<Vec2>,
    pub player: Vec2,
}

#[derive(Debug)]
pub enum GameUpdate {
    NextState(GameState, GameChangeType),
    Error(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GameChangeType {
    PlayerMove,
    PlayerAndBoxMove,
}
