use bevy::math::IVec2;

/// A bounding box with one corner fixed at 0,0 and assumed to have positive extent
#[derive(Clone, Copy, Debug)]
pub struct BoundsOriginRoot {
    pub extent: IVec2,
}

impl BoundsOriginRoot {
    pub fn new(x: i32, y: i32) -> BoundsOriginRoot {
        BoundsOriginRoot{
            extent: IVec2 { x, y }
        }
    }

    pub fn contains(&self, pos: &IVec2) -> bool {
        pos.x >= 0 && pos.x < self.extent.x && pos.y >= 0 && pos.y < self.extent.y
    }

    pub fn area(&self) -> i32 {
        self.extent.x * self.extent.y
    }
}