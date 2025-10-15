use bevy::prelude::*;
use crate::bevy_interface::tile_render::models::{TileAssets, TileGrid, TileSlot, Tiles};

pub fn setup_tile_render(
    mut commands: Commands,
    asset_server: Res<AssetServer>){
    let tile_assets = TileAssets::new_load(asset_server);
    let tiles = Tiles::new_random(&tile_assets);

    commands.insert_resource(tile_assets);
    commands.insert_resource(tiles);

    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(10.0),
            right: Val::Px(10.0),
            width: Val::Auto,
            height: Val::Auto,
            padding: UiRect::all(Val::Px(15.0)),
            display: Display::Grid,
            grid_auto_columns: GridTrack::auto(),
            grid_auto_rows: GridTrack::auto(),
            ..default()
        },
        BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.8)),
        BorderRadius::all(Val::Px(4.0)),
        TileGrid
    ));
}

pub fn update_grid_size(
    mut commands: Commands,
    existing_tiles: Query<Entity, With<TileSlot>>,
    parent_grid: Query<Entity, With<TileGrid>>,
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

    println!("New grid size: {:?}", new_size);

    let Ok(parent_grid) = parent_grid.single() else {
        eprintln!("WARNING: The parent grid does not exist");
        return;
    };

    // despawn
    for entity in existing_tiles.iter() {
        commands.entity(entity).despawn();
    }

    let total_tiles = tiles.get_tile_count();
    let alpha = 1.0 / (total_tiles as f32);

    commands.entity(parent_grid).with_children(|parent| {
        // respawn
        for x in 0..new_size.x {
            for y in 0..new_size.y {
                let location = IVec2 { x, y };
                let mut spawned = parent.spawn((
                    Node {
                        grid_row: GridPlacement::start((new_size.y - y) as i16),
                        grid_column: GridPlacement::start(x as i16 + 1),
                        // width: percent(100),
                        // height: percent(100),
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(bevy::color::palettes::basic::MAROON.into()),
                ));

                for (depth, tile_type) in tiles.get_tiles_at(&location).enumerate() {
                    let slot = TileSlot {
                        tile_type,
                        depth,
                        location,
                    };
                    let bundle = tile_assets.get_ui_bundle_for_tile(tile_type, alpha);
                    spawned.with_child((
                        bundle,
                        slot,
                    ));
                }
            }
        }
    });

    tiles.mark_grid_rendered_to_size(new_size);
    tiles.mark_tiles_not_dirty();
}

pub fn update_grid(
    mut existing_tiles: Query<(&mut TileSlot, &mut ImageNode)>,
    mut tiles: ResMut<Tiles>,
    tile_assets: Res<TileAssets>) {

    if !tiles.tiles_dirty() {
        return;
    }

    // let total_tiles = tiles.get_tile_count();
    // let alpha = 1.0 / (total_tiles as f32);

    for (mut tile_slot, mut image) in existing_tiles.iter_mut() {
        let tile_type = tiles.get_tile_at(&tile_slot);

        if tile_type == tile_slot.tile_type { continue; }
        tile_slot.tile_type = tile_type;

        image.image = tile_assets.get_image_for_tile(tile_type);
    }

    tiles.mark_tiles_not_dirty();
}