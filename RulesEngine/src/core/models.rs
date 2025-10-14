#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum Cell {
    Wall,
    Floor,
    Target,
}

// TODO: use glam::i32::ivec2 instead
// Or, alternatively, use a custom u8 x u8 type. Unlikely any game will be bigger than 255x255
// when the graph gets large we can see 10s of GB of memory - the algo is memory bound
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[derive(PartialOrd, Ord)]
pub struct Vec2 {
    pub i: i8,
    pub j: i8,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum UserAction {
    Move(Direction),
}

#[derive(Clone)]
pub struct SharedGameState {
    pub grid: Vec<Vec<Cell>>,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct GameState {
    pub environment: GameStateEnvironment,
    pub player: Vec2,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct GameStateEnvironment {
    pub boxes: Vec<Vec2>,
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
