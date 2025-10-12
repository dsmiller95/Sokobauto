use bevy::prelude::*;
use crate::bevy_interface::tile_render::models::{TileAssets, TileSlot, TileType, Tiles};

pub fn setup_tile_render(mut commands: Commands, asset_server: Res<AssetServer>){
    let tile_assets = TileAssets::new_load(asset_server);
    let tiles = Tiles::new_random(&tile_assets);

    commands.insert_resource(tile_assets);
    commands.insert_resource(tiles);
}

pub fn update_grid_size(
    mut commands: Commands,
    existing_tiles: Query<Entity, With<TileSlot>>,
    mut tiles: ResMut<Tiles>,
    tile_assets: Res<TileAssets>,
) {
    if !tiles.is_changed() {
        return;
    }

    let new_size = match tiles.get_new_rendered_size() {
        Some(size) => size,
        None => if tiles.is_added() { tiles.get_grid_size() } else { return; },
    };

    // despawn
    for entity in existing_tiles.iter() {
        commands.entity(entity).despawn();
    }

    // respawn
    for x in 0..new_size.x {
        for y in 0..new_size.y {
            let location = IVec2 { x, y };
            let tile_type = tiles.get_tile_at(location);
            commands.spawn((
                TileSlot {
                    tile_type,
                    location,
                },
                Transform::from_translation(tiles.get_tile_world_position(location)),
                tile_assets.get_sprite_for_tile(tile_type)
            ));
        }
    }

    tiles.mark_grid_rendered_to_size(new_size);
    tiles.mark_tiles_not_dirty();
}

pub fn update_grid(
    mut existing_tiles: Query<(&TileSlot, &mut Sprite)>,
    mut tiles: ResMut<Tiles>,
    tile_assets: Res<TileAssets>) {

    if !tiles.is_changed() || !tiles.tiles_dirty() {
        return;
    }

    for (tile_slot, mut sprite) in existing_tiles.iter_mut() {
        let tile_type = tiles.get_tile_at(tile_slot.location);
        if tile_type == tile_slot.tile_type { continue; }

        *sprite = tile_assets.get_sprite_for_tile(tile_type);
    }

    tiles.mark_tiles_not_dirty();
}