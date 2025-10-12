use bevy::app::{App, Plugin, Update};
use bevy::input::keyboard::Key;
use bevy::prelude::*;
use crate::bevy_interface::{GraphNode, SourceGraphData};
use crate::core::{Direction, UserAction};

/// Placed on any Node which is currently being played, to represent the unique
/// state not already captured by that node ?? ? ? ??
#[derive(Component)]
pub struct PlayingGameState {
    player_pos: IVec2,
}

pub struct SelectedGameNavigationPlugin;

impl Plugin for SelectedGameNavigationPlugin {
    fn build(&self, app: &mut App) {
        app
            // .add_systems(Startup, crate::bevy_interface::node_selection::setup_shared_meshes)
            .add_systems(Update, (process_game_input))
        ;
    }
}

fn user_action_from_input(input: &ButtonInput<Key>) -> Option<UserAction> {
    if input.just_pressed(Key::ArrowDown) {
        Some(UserAction::Move(Direction::Down))
    } else if input.just_pressed(Key::ArrowUp) {
        Some(UserAction::Move(Direction::Up))
    } else if input.just_pressed(Key::ArrowLeft) {
        Some(UserAction::Move(Direction::Left))
    } else if input.just_pressed(Key::ArrowRight) {
        Some(UserAction::Move(Direction::Right))
    } else {
        None
    }
}

fn process_game_input(
    mut commands: Commands,
    mut play_states: Query<(Entity, &mut PlayingGameState, &GraphNode)>,
    mut game_graph_data: Res<SourceGraphData>,
    input: Res<ButtonInput<Key>>
) {
    let Some(action) = user_action_from_input(&input) else {
        return;
    };

    // TODO: apply the action to every game state which has a PlayingGameState
    //  then either update the playing game state or remove it + re-add to a new node

    todo!()
}
