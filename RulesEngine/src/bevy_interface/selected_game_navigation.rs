use bevy::app::{App, Plugin, Update};
use bevy::input::keyboard::Key;
use bevy::prelude::*;
use crate::bevy_interface::{GraphNode, SourceGraphData};
use crate::bevy_interface::graph_compute::GraphComputeCache;
use crate::core::{step, Direction, GameChangeType, GameState, GameUpdate, SharedGameState, UserAction};
use crate::state_graph::UniqueNode;

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

impl PlayingGameState {
    pub fn new_playing_state(node: &UniqueNode) -> Self {
        PlayingGameState {
            player_pos: node.minimum_reachable_player_position,
        }
    }

    pub fn apply_to_node(&self, node: UniqueNode) -> GameState {
        GameState {
            environment: node.environment,
            player: self.player_pos.into(),
        }
    }

    pub fn extract_from_state(state: GameState, shared: &SharedGameState) -> (Self, UniqueNode) {
        (
            PlayingGameState {
                player_pos: state.player.into(),
            },
            UniqueNode::from_game_state(state, shared),
        )
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

/// Whenever a user inputs an action
/// Apply that action to the game state associated with each PlayingGameState.
/// Then modify the PlayingGameState to hold the new player position.
/// Or, if that particular game transitioned to a new game state (node),
/// move the PlayingGameState to the new node.
fn process_game_input(
    mut commands: Commands,
    mut play_states: Query<(Entity, &mut PlayingGameState, &GraphNode)>,
    game_graph_data: Res<SourceGraphData>,
    graph_entity_lookup: Res<GraphComputeCache>,
    input: Res<ButtonInput<Key>>
) {
    let Some(action) = user_action_from_input(&input) else {
        return;
    };

    println!("taking action {:?}", action);

    let shared = &game_graph_data.shared;

    for (entity, mut playing_game_state, node) in play_states.iter_mut() {
        let game_node = game_graph_data.graph.nodes.get_by_right(&node.id).expect("game node not found!");
        let game_state = playing_game_state.apply_to_node(game_node.clone());
        let update = step(shared, &game_state, action);

        match update {
            GameUpdate::Error(_) => {
                // noop, game did not change
            }
            GameUpdate::NextState(game_state, GameChangeType::PlayerMove) => {
                playing_game_state.player_pos = game_state.player.into();
            }
            GameUpdate::NextState(game_state, GameChangeType::PlayerAndBoxMove) => {
                let (new_playing, new_node) = PlayingGameState::extract_from_state(game_state, shared);
                let new_game_id = game_graph_data.graph.nodes.get_by_left(&new_node);
                let Some(new_game_id) = new_game_id else {
                    // if the game does not exist in the graph, we abort the move. the game will remain.
                    println!("Action would end game. Aborting for game {:}", node.id);
                    continue;
                };
                
                commands.entity(entity).remove::<PlayingGameState>();

                let Some(&entity) = graph_entity_lookup.get_entity(new_game_id) else {
                    eprintln!("Could not find game entity for game ID {:?}", new_game_id);
                    continue;
                };

                commands.entity(entity).insert(new_playing);
            }
        }
    }
}
