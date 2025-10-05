use std::collections::HashMap;
use bevy::prelude::Vec3;

pub struct SpatialHash<T> {
    cell_size: f32,
    buckets: HashMap<(i32, i32, i32), Vec<T>>,
}

impl<T> SpatialHash<T> {
    pub fn new(cell_size: f32) -> Self {
        Self {
            cell_size,
            buckets: HashMap::new(),
        }
    }

    fn hash_position(&self, position: Vec3) -> (i32, i32, i32) {
        (
            (position.x / self.cell_size).floor() as i32,
            (position.y / self.cell_size).floor() as i32,
            (position.z / self.cell_size).floor() as i32,
        )
    }

    pub fn insert(&mut self, position: Vec3, value: T) {
        let key = self.hash_position(position);
        self.buckets.entry(key).or_default().push(value);
    }

    pub fn get(&self, position: Vec3) -> Option<&Vec<T>> {
        let key = self.hash_position(position);
        self.buckets.get(&key)
    }

    pub fn iter_all_nearby(&self, position: Vec3) -> impl Iterator<Item = &T> {
        let (x, y, z) = self.hash_position(position);
        let mut buckets = Vec::new();
        for dx in -1..=1 {
            for dy in -1..=1 {
                for dz in -1..=1 {
                    buckets.push((x + dx, y + dy, z + dz));
                }
            }
        }
        buckets.into_iter()
            .filter_map(move |key| self.buckets.get(&key))
            .flatten()
    }
}