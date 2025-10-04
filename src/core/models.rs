#[derive(Clone, Copy, PartialEq)]
pub enum Cell {
    Wall,
    Floor,
    Target,
    BoxOnFloor,
    BoxOnTarget,
    PlayerOnFloor,
    PlayerOnTarget,
}

#[derive(Clone, Copy)]
pub struct Vec2 {
    pub i: i32,
    pub j: i32,
}

#[derive(Clone, Copy)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Clone, Copy)]
pub enum UserAction {
    Move(Direction),
    Quit,
}

#[derive(Clone)]
pub struct GameState {
    pub grid: Vec<Vec<Cell>>,
    pub player: Vec2,
}

pub enum GameUpdate{
    NextState(GameState),
    NoChange,
    Error(String),
}
