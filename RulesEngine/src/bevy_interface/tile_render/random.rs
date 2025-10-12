use rand::distr::StandardUniform;
use rand::prelude::*;
use crate::bevy_interface::tile_render::models::TileType;

impl Distribution<TileType> for StandardUniform {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> TileType {
        TileType::all().choose(rng).unwrap().clone()
    }
}