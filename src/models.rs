
pub(crate) use crate::core::{Cell, Vec2};
use crate::core::GameState;

pub struct GameRenderState { 
    pub game: GameState,
    pub won: bool,
    pub error: Option<String>,
}