use std::collections::HashMap;
use bevy::prelude::*;
use rand::Rng;

#[derive(Resource)]
pub struct Tiles {
    grids: Vec<Vec<Vec<TileType>>>,
    root: Vec3,
    cell_size: Vec2,
    grid_size: IVec3,
    rendered_grid_size: IVec3,
    tile_contents_dirty: bool,
}

#[derive(Resource)]
pub struct TileAssets {
    images: HashMap<TileType, Handle<Image>>,
    tile_size: Vec2,
}

#[derive(Clone, Copy, Hash, PartialEq, Eq, Debug)]
pub enum TileType {
    Empty,
    Floor,
    Wall,
    Box,
    Target,
    Player,
}

#[derive(Component)]
pub struct TileGrid;


#[derive(Component)]
pub struct TileSlot {
    pub tile_location: TileLocation,
    pub tile_type: TileType,
}

pub struct TileLocation {
    pub location: IVec2,
    pub depth: usize,
}

const ALL_TILE_TYPES: &[TileType] = &[
    TileType::Empty,
    TileType::Floor,
    TileType::Wall,
    TileType::Box,
    TileType::Target,
    TileType::Player,
];

impl TileType {
    pub fn all() -> &'static [TileType] {
        ALL_TILE_TYPES
    }

    pub fn file_name(&self) -> &'static str {
        match self {
            TileType::Empty => "sprites/tiles/empty.png",
            TileType::Floor => "sprites/tiles/floor.png",
            TileType::Wall => "sprites/tiles/wall.png",
            TileType::Box => "sprites/tiles/box.png",
            TileType::Target => "sprites/tiles/target.png",
            TileType::Player => "sprites/tiles/player.png"
        }
    }
}

impl Tiles {
    pub fn new_empty() -> Tiles {
        Tiles {
            grids: vec![],
            root: Vec3::splat(0.0),
            cell_size: Vec2::ZERO,
            grid_size: IVec3::splat(0),
            rendered_grid_size: IVec3::splat(0),
            tile_contents_dirty: false,
        }
    }

    pub fn new_random(assets: &TileAssets) -> Tiles {
        let grid_size = IVec3::new(10, 10, 1);
        let mut rng = rand::rng();
        let mut grid = vec![vec![TileType::Empty; grid_size.x as usize]; grid_size.y as usize];
        for x in 0..grid_size.x as usize {
            for y in 0..grid_size.y as usize {
                grid[y][x] = rng.random();
            }
        }
        Tiles {
            grids: vec![grid],
            // TODO: configure root from the top level module
            root: Vec3::new(200.0, 200.0, 0.0),
            cell_size: assets.tile_size,
            grid_size,
            rendered_grid_size: grid_size,
            tile_contents_dirty: false,
        }
    }

    pub fn assign_new_grids(&mut self, new_grids: Vec<Vec<Vec<TileType>>>) {
        if new_grids.is_empty() {
            self.grids = new_grids;
            self.tile_contents_dirty = true;
            self.grid_size = self.grid_size.with_z(0);
            return;
        }

        let y = new_grids[0].len();
        let x = if y > 0 { new_grids[0][0].len() } else { 0 };
        let depth = new_grids.len();

        new_grids.iter().for_each(|grid| {
            assert_eq!(grid.len(), y, "Grids must be uniform size");
            grid.iter().for_each(|row| {
                assert_eq!(row.len(), x, "Grids must be uniform size");
            });
        });

        self.grids = new_grids;
        self.tile_contents_dirty = true;
        self.grid_size = IVec3::new(x as i32, y as i32, depth as i32);
    }

    pub fn get_grid_size(&self) -> IVec3 {
        self.grid_size
    }

    /// If the grid size does not match the rendered size, this returns the new size. Otherwise None
    pub fn get_new_rendered_size(&self) -> Option<IVec3> {
        if self.grid_size == self.rendered_grid_size {
            None
        }else {
            Some(self.grid_size)
        }
    }

    /// Mark that the grid rendered to the given new size.
    pub fn mark_grid_rendered_to_size(&mut self, new_size: IVec3) {
        if self.grid_size != new_size {
            eprintln!("Warning: grid size rendered to {:?}, does not match current grid size {:?}", new_size, self.grid_size);
        }
        self.rendered_grid_size = new_size;
    }

    pub fn get_tile_count(&self) -> usize {
        self.grids.len()
    }

    pub fn get_tiles_at(&self, location: &IVec2) -> impl Iterator<Item=TileType> {
        self.grids.iter().map(move |grid|
            grid
                .get(location.y as usize)
                .and_then(|v| v.get(location.x as usize))
                .copied()
                .unwrap_or(TileType::Empty)
        )
    }

    pub fn get_tile_at(&self, slot: &TileLocation) -> TileType {
        self.grids.get(slot.depth)
            .and_then(|grid| grid.get(slot.location.y as usize))
            .and_then(|v| v.get(slot.location.x as usize))
            .copied()
            .unwrap_or(TileType::Empty)
    }

    pub fn get_tile_world_position(&self, slot: &TileLocation) -> Vec3 {
        (slot.location.as_vec2() * self.cell_size).extend(slot.depth as f32 * 0.1) + self.root
    }

    pub fn tiles_dirty(&self) -> bool {
        self.tile_contents_dirty
    }

    pub fn mark_tiles_not_dirty(&mut self) {
        self.tile_contents_dirty = false
    }
}

impl TileAssets {
    pub fn new_load(asset_server: Res<AssetServer>) -> TileAssets {
        let mut images: HashMap<TileType, Handle<Image>> = HashMap::new();
        for &tile in TileType::all() {
            let image_asset = asset_server.load(tile.file_name());

            images.insert(tile, image_asset);
        }

        TileAssets::new(images)
    }

    pub fn new(images: HashMap<TileType, Handle<Image>>) -> TileAssets {
        TileType::all().into_iter().for_each(|t| {
            images.get(&t).expect("No image loaded");
        });

        TileAssets {
            images,
            tile_size: Vec2::splat(32.),
        }
    }

    pub fn get_sprite_for_tile(&self, tile_type: TileType, alpha: f32) -> Sprite {
        match self.images.get(&tile_type) {
            Some(image) => {
                Sprite {
                    image: image.clone(),
                    color: Color::srgba(1.0, 1.0, 1.0, alpha),
                    ..default()
                }
            },
            None => {
                let mut tmp_color = bevy::color::palettes::basic::MAROON;
                tmp_color.alpha = alpha;
                Sprite::from_color(tmp_color, Vec2::splat(1.0))
            },
        }
    }

    pub fn get_ui_bundle_for_tile(&self, tile_type: TileType) -> impl Bundle {
        const SIZE: f32 = 16.0;
        let Some(image) = self.images.get(&tile_type) else {
            panic!("No image loaded for tile type {:?}", tile_type);
        };

        (
            Node {
                height: Val::Px(SIZE),
                width: Val::Px(SIZE),
                padding: UiRect::all(Val::Px(5.0)),
                ..default()
            },
            ImageNode::new(image.clone()),
        )
    }

    pub fn get_image_for_tile(&self, tile_type: TileType) -> Handle<Image> {
        let Some(image) = self.images.get(&tile_type) else {
            panic!("No image loaded for tile type {:?}", tile_type);
        };
        image.clone()
    }
}