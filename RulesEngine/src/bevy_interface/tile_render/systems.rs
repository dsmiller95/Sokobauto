use bevy::prelude::*;
use crate::bevy_interface::tile_render::models::{TileAssets, TileGrid, TileLocation, TileSlot, Tiles};

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
            display: Display::Flex,
            flex_direction: FlexDirection::Row,
            flex_wrap: FlexWrap::WrapReverse,
            justify_content: JustifyContent::End,
            padding: UiRect::all(Val::Px(10.0)),
            ..default()
        },
        BackgroundColor(bevy::color::palettes::basic::BLACK.with_alpha(0.4).into()),
        BorderRadius::all(Val::Px(4.0)),
        TileGrid
    ));
}


#[derive(Component)]
pub struct EphemeralTileUiNode;

pub fn update_grid_size(
    mut commands: Commands,
    existing_tiles: Query<Entity, With<TileSlot>>,
    other_ephemerals: Query<Entity, With<EphemeralTileUiNode>>,
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
    for entity in other_ephemerals.iter() {
        commands.entity(entity).despawn();
    }

    let total_tiles = tiles.get_tile_count();
    commands.entity(parent_grid).with_children(|parent| {
        // respawn
        for depth in 0..total_tiles {
            let mut spawned = parent.spawn((
                Node {
                    display: Display::Grid,
                    grid_auto_columns: GridTrack::auto(),
                    grid_auto_rows: GridTrack::auto(),
                    padding: UiRect::all(Val::Px(5.0)),
                    ..default()
                },
                EphemeralTileUiNode
            ));

            for x in 0..new_size.x {
                for y in 0..new_size.y {
                    let tile_location = TileLocation{
                        location: IVec2 { x, y },
                        depth,
                    };
                    let tile_type = tiles.get_tile_at(&tile_location);
                    let slot = TileSlot {
                        tile_location,
                        tile_type,
                    };
                    let image = tile_assets.get_image_for_tile(tile_type);
                    spawned.with_child((
                        Node {
                            grid_row: GridPlacement::start((new_size.y - y) as i16),
                            grid_column: GridPlacement::start(x as i16 + 1),
                            width: Val::Px(32.0),
                            height: Val::Px(32.0),
                            ..default()
                        },
                        ImageNode::new(image.clone()),
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
        let tile_type = tiles.get_tile_at(&tile_slot.tile_location);

        if tile_type == tile_slot.tile_type { continue; }
        tile_slot.tile_type = tile_type;

        image.image = tile_assets.get_image_for_tile(tile_type);
    }

    tiles.mark_tiles_not_dirty();
}