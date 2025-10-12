use bevy::math::{IVec2};
use bevy::prelude::{Component};

/// Placed on any Node which is currently being played, to represent the unique
/// state not already captured by that node ?? ? ? ??
#[derive(Component)]
pub struct PlayingGameState {
    player_pos: IVec2,
}
