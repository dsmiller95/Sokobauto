use bevy::math::IVec2;
use crate::core::bounds::BoundsOriginRoot;

pub struct BoundedGrid<T> {
    bounds: BoundsOriginRoot,
    cells: Vec<T>,
}

impl<T> BoundedGrid<T> {
    pub fn new_with_size(size: IVec2, default: T) -> Self
    where
        T: Clone,
    {
        let bounds = BoundsOriginRoot::new(size.x, size.y);
        BoundedGrid::new(bounds, default)
    }

    pub fn new(bounds: BoundsOriginRoot, default: T) -> Self
    where
        T: Clone,
    {
        let cells = vec![default; bounds.area() as usize];
        BoundedGrid {
            bounds,
            cells,
        }
    }

    pub fn size(&self) -> BoundsOriginRoot {
        self.bounds
    }
    
    pub fn contains(&self, pos: &IVec2) -> bool {
        self.bounds.contains(pos)
    }
    
    pub fn get(&self, pos: &IVec2) -> Option<&T> {
        if !self.bounds.contains(pos) {
            return None;
        }
        Some(&self[pos])
    }
}

impl<T> std::ops::Index<&IVec2> for BoundedGrid<T> {
    type Output = T;

    fn index(&self, index: &IVec2) -> &Self::Output {
        &self.cells[(index.y * self.bounds.extent.x + index.x) as usize]
    }
}

impl<T> std::ops::IndexMut<&IVec2> for BoundedGrid<T> {
    fn index_mut(&mut self, index: &IVec2) -> &mut Self::Output {
        &mut self.cells[(index.y * self.bounds.extent.x + index.x) as usize]
    }
}