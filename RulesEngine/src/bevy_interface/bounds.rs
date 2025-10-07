use bevy::math::Vec3;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Bounds {
    pub min: Vec3,
    pub max: Vec3,
}
impl Bounds {
    pub fn new(min: Vec3, max: Vec3) -> Self {
        Self { min, max }
    }

    pub fn center(&self) -> Vec3 {
        (self.min + self.max) * 0.5
    }

    pub fn size(&self) -> Vec3 {
        self.max - self.min
    }

    pub fn width(&self) -> f32 {
        self.size().max_element()
    }

    pub fn contains(&self, point: Vec3) -> bool {
        point.x >= self.min.x && point.x <= self.max.x &&
            point.y >= self.min.y && point.y <= self.max.y &&
            point.z >= self.min.z && point.z <= self.max.z
    }

    pub fn contains_other(&self, other: &Bounds) -> bool {
        self.contains(other.min) && self.contains(other.max)
    }

    pub fn include(&mut self, point: Vec3) {
        self.min = self.min.min(point);
        self.max = self.max.max(point);
    }

    pub fn doubled(&self) -> Bounds {
        let center = self.center();
        let size = self.size();
        let new_half = size * 1.0;
        Bounds::new(center - new_half, center + new_half)
    }

    pub fn octant(&self, index: usize) -> Bounds {
        let center = self.center();
        let half_size = self.size() * 0.5;

        let offset = half_size * 0.5;

        let offset = offset * Vec3::new(
            if index & 1 != 0 { 1.0 } else { -1.0 },
            if index & 2 != 0 { 1.0 } else { -1.0 },
            if index & 4 != 0 { 1.0 } else { -1.0 },
        );

        let octant_center = center + offset;
        Bounds::new(
            octant_center - half_size * 0.5,
            octant_center + half_size * 0.5,
        )
    }

    pub fn octant_index(&self, point: Vec3) -> usize {
        let center = self.center();
        let mut index = 0;
        if point.x > center.x { index |= 1; }
        if point.y > center.y { index |= 2; }
        if point.z > center.z { index |= 4; }
        index
    }

    pub fn resize_expand(&self, point: &Vec3) -> Bounds {
        let mut new_bounds = *self;
        new_bounds.include(*point);
        new_bounds.doubled()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bounds_basic() {
        let bounds = Bounds::new(Vec3::ZERO, Vec3::splat(10.0));
        assert_eq!(bounds.center(), Vec3::splat(5.0));
        assert_eq!(bounds.size(), Vec3::splat(10.0));
        assert_eq!(bounds.width(), 10.0);

        assert!(bounds.contains(Vec3::splat(5.0)));
        assert!(bounds.contains(Vec3::ZERO));
        assert!(bounds.contains(Vec3::splat(10.0)));
        assert!(!bounds.contains(Vec3::splat(-1.0)));
        assert!(!bounds.contains(Vec3::splat(11.0)));
    }

    #[test]
    fn test_bounds_octant() {
        let bounds = Bounds::new(Vec3::ZERO, Vec3::splat(10.0));

        // Test octant 0 (negative x, y, z)
        let octant0 = bounds.octant(0);
        assert_eq!(octant0.center(), Vec3::new(2.5, 2.5, 2.5));

        // Test octant 7 (positive x, y, z)
        let octant7 = bounds.octant(7);
        assert_eq!(octant7.center(), Vec3::new(7.5, 7.5, 7.5));
    }

    #[test]
    fn test_bounds_octant_index() {
        let bounds = Bounds::new(Vec3::ZERO, Vec3::splat(10.0));
        let _center = bounds.center(); // (5, 5, 5)

        assert_eq!(bounds.octant_index(Vec3::new(2.0, 2.0, 2.0)), 0); // All negative relative to center
        assert_eq!(bounds.octant_index(Vec3::new(8.0, 2.0, 2.0)), 1); // +x
        assert_eq!(bounds.octant_index(Vec3::new(2.0, 8.0, 2.0)), 2); // +y
        assert_eq!(bounds.octant_index(Vec3::new(8.0, 8.0, 2.0)), 3); // +x, +y
        assert_eq!(bounds.octant_index(Vec3::new(2.0, 2.0, 8.0)), 4); // +z
        assert_eq!(bounds.octant_index(Vec3::new(8.0, 8.0, 8.0)), 7); // All positive
    }
}