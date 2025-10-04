pub(crate) use crate::core::{Cell, Vec2};
use crate::core::{GameChangeType, GameState};

pub struct GameRenderState {
    pub game: GameState,
    pub won: bool,
    pub error: Option<String>,
    pub last_change: Option<GameChangeType>,
}
