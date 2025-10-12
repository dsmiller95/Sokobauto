use bevy::prelude::*;
use crate::bevy_interface::tile_render::models::{TileAssets, TileSlot, Tiles};

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

    // TODO: bug! we need to re-render if depth changed, too.
    let new_size = match tiles.get_new_rendered_size() {
        Some(size) => size,
        None => if tiles.is_added() { tiles.get_grid_size() } else { return; },
    };

    println!("New grid size: {:?}", new_size);

    // despawn
    for entity in existing_tiles.iter() {
        commands.entity(entity).despawn();
    }

    let total_tiles = tiles.get_tile_count();
    let alpha = 1.0 / (total_tiles as f32);

    // respawn
    for x in 0..new_size.x {
        for y in 0..new_size.y {
            let location = IVec2 { x, y };

            for (depth, tile_type) in tiles.get_tiles_at(&location).enumerate() {
                let slot = TileSlot {
                    tile_type,
                    depth,
                    location,
                };
                commands.spawn((
                    Transform::from_translation(tiles.get_tile_world_position(&slot)),
                    slot,
                    tile_assets.get_sprite_for_tile(tile_type, alpha)
                ));
            }
        }
    }

    tiles.mark_grid_rendered_to_size(new_size);
    tiles.mark_tiles_not_dirty();
}

pub fn update_grid(
    mut existing_tiles: Query<(&mut TileSlot, &mut Sprite)>,
    mut tiles: ResMut<Tiles>,
    tile_assets: Res<TileAssets>) {

    if !tiles.tiles_dirty() {
        return;
    }

    let total_tiles = tiles.get_tile_count();
    let alpha = 1.0 / (total_tiles as f32);

    for (mut tile_slot, mut sprite) in existing_tiles.iter_mut() {
        let tile_type = tiles.get_tile_at(&tile_slot);

        if tile_type == tile_slot.tile_type { continue; }
        tile_slot.tile_type = tile_type;

        *sprite = tile_assets.get_sprite_for_tile(tile_type, alpha);
    }

    tiles.mark_tiles_not_dirty();
}