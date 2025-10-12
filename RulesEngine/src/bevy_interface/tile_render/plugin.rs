use bevy::app::{App, Plugin};
use bevy::prelude::*;
use crate::bevy_interface::tile_render::systems::*;

pub struct TileRenderPlugin;

impl Plugin for TileRenderPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Startup, setup_tile_render)
            .add_systems(Update, (update_grid_size, update_grid).chain());
    }
}
