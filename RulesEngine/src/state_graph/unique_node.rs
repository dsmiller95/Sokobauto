use bevy::math::IVec2;
use crate::core::{GameState, GameStateEnvironment, SharedGameState};

#[derive(Hash, Eq, PartialEq, Clone, Debug)]
pub struct UniqueNode {
    pub environment: GameStateEnvironment,
    pub minimum_reachable_player_position: IVec2,
}

impl UniqueNode {
    pub fn from_game_state(game: GameState, shared: &SharedGameState) -> Self {
        let min_reachable_position = shared.min_reachable_position(&game);
        UniqueNode {
            environment: game.environment,
            minimum_reachable_player_position: min_reachable_position.into(),
        }
    }
}