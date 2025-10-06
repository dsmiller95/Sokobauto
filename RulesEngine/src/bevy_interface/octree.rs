use bevy::prelude::*;

#[derive(Debug, Clone)]
pub struct OctreeNode {
    pub bounds: Bounds,
    pub center_of_mass: Vec3,
    pub total_mass: f32,
    pub node_count: usize,
    pub children: Option<Box<[OctreeNode; 8]>>,
    pub points: Vec<(usize, Vec3)>, // (node_id, position) for leaf nodes
}

#[derive(Debug, Clone, Copy)]
pub struct Bounds {
    pub min: Vec3,
    pub max: Vec3,
}

pub const NODE_MASS: f32 = 1.0;
pub const MINIMUM_DISTANCE: f32 = 0.01;

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
}

impl OctreeNode {
    pub fn new(bounds: Bounds) -> Self {
        Self {
            bounds,
            center_of_mass: Vec3::ZERO,
            total_mass: 0.0,
            node_count: 0,
            children: None,
            points: Vec::new(),
        }
    }

    pub fn is_leaf(&self) -> bool {
        self.children.is_none()
    }

    pub fn insert(&mut self, node_id: usize, position: Vec3, mass: f32, max_depth: usize, max_points_per_leaf: usize) {
        // Update center of mass and total mass
        let total_mass = self.total_mass + mass;
        if total_mass > 0.0 {
            self.center_of_mass = (self.center_of_mass * self.total_mass + position * mass) / total_mass;
        } else {
            self.center_of_mass = position;
        }
        self.total_mass = total_mass;
        self.node_count += 1;

        if self.is_leaf() {
            self.points.push((node_id, position));
            
            // Check if we need to subdivide
            if self.points.len() > max_points_per_leaf && max_depth > 0 {
                self.subdivide(max_depth - 1, max_points_per_leaf);
            }
        } else {
            // Insert into appropriate child
            let octant_index = self.bounds.octant_index(position);
            if let Some(ref mut children) = self.children {
                // TODO: should max_depth be decremented? do we have tests to ensure max depth is respected?
                children[octant_index].insert(node_id, position, mass, max_depth - 1, max_points_per_leaf);
            }
        }
    }

    fn subdivide(&mut self, remaining_depth: usize, max_points_per_leaf: usize) {
        // Create 8 children
        let mut children = Vec::with_capacity(8);
        for i in 0..8 {
            children.push(OctreeNode::new(self.bounds.octant(i)));
        }
        self.children = Some(children.try_into().unwrap());

        // Move points to children
        let points = std::mem::take(&mut self.points);
        for (node_id, position) in points {
            let octant_index = self.bounds.octant_index(position);
            if let Some(ref mut children) = self.children {
                children[octant_index].insert(node_id, position, NODE_MASS, remaining_depth, max_points_per_leaf);
            }
        }
    }
}

pub struct Octree {
    root: OctreeNode,
    max_depth: usize,
    max_points_per_leaf: usize,
}

impl Octree {
    pub fn new(bounds: Bounds, max_depth: usize, max_points_per_leaf: usize) -> Self {
        Self {
            root: OctreeNode::new(bounds),
            max_depth,
            max_points_per_leaf,
        }
    }

    pub fn from_points(points: &[(usize, Vec3)], max_depth: usize, max_points_per_leaf: usize) -> Self {
        if points.is_empty() {
            return Self::new(
                Bounds::new(Vec3::splat(-1.0), Vec3::splat(1.0)),
                max_depth,
                max_points_per_leaf,
            );
        }

        // Calculate bounds from all points
        let mut min = points[0].1;
        let mut max = points[0].1;
        
        for &(_, pos) in points {
            min = min.min(pos);
            max = max.max(pos);
        }

        // Add some padding to bounds
        let padding = (max - min) * 0.1;
        min -= padding;
        max += padding;

        let mut octree = Self::new(Bounds::new(min, max), max_depth, max_points_per_leaf);
        
        for &(node_id, position) in points {
            octree.insert(node_id, position, NODE_MASS);
        }

        octree
    }

    pub fn insert(&mut self, node_id: usize, position: Vec3, mass: f32) {
        if !self.root.bounds.contains(position) {
            // For now, just skip points outside bounds
            // In a more robust implementation, we could resize the octree
            return;
        }
        
        self.root.insert(node_id, position, mass, self.max_depth, self.max_points_per_leaf);
    }

    pub fn calculate_force(&self, position: Vec3, theta: f32, repulsion_strength: f32) -> Vec3 {
        self.calculate_force_recursive(&self.root, position, theta, repulsion_strength)
    }

    fn calculate_force_recursive(&self, node: &OctreeNode, position: Vec3, theta: f32, repulsion_strength: f32) -> Vec3 {
        if node.node_count == 0 {
            return Vec3::ZERO;
        }

        let diff = position - node.center_of_mass;
        let distance = diff.length();
        
        if distance < 0.1 {
            return Vec3::ZERO; // Too close, avoid division by zero
        }

        // Barnes-Hut criterion: if the node is far enough, treat it as a single mass
        if (node.bounds.width() / distance) < theta && distance > MINIMUM_DISTANCE {
            let force_magnitude = repulsion_strength * node.total_mass / (distance * distance);
            return diff.normalize() * force_magnitude;
        }

        // Otherwise, recurse into children
        let mut total_force = Vec3::ZERO;
        if node.is_leaf() {
            for &(_, point_pos) in &node.points {
                let point_diff = position - point_pos;
                let point_distance = point_diff.length();
                if point_distance < MINIMUM_DISTANCE {
                    // TODO: handle self-force or very close points differently. take in a node ID to compare?
                    continue; // Skip too close points
                }
                let force_magnitude = NODE_MASS * repulsion_strength / (point_distance * point_distance);
                total_force += point_diff.normalize() * force_magnitude;
            }
        } else if let Some(ref children) = node.children {
            for child in children.iter() {
                total_force += self.calculate_force_recursive(child, position, theta, repulsion_strength);
            }
        }

        total_force
    }

    pub fn get_all_points(&self) -> Vec<(usize, Vec3)> {
        let mut points = Vec::new();
        self.collect_points_recursive(&self.root, &mut points);
        points
    }

    fn collect_points_recursive(&self, node: &OctreeNode, points: &mut Vec<(usize, Vec3)>) {
        if node.is_leaf() {
            points.extend_from_slice(&node.points);
        } else if let Some(ref children) = node.children {
            for child in children.iter() {
                self.collect_points_recursive(child, points);
            }
        }
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

    #[test]
    fn test_octree_single_point() {
        let bounds = Bounds::new(Vec3::ZERO, Vec3::splat(10.0));
        let mut octree = Octree::new(bounds, 3, 1);
        
        octree.insert(0, Vec3::new(5.0, 5.0, 5.0), 1.0);
        
        assert_eq!(octree.root.node_count, 1);
        assert_eq!(octree.root.center_of_mass, Vec3::new(5.0, 5.0, 5.0));
        assert_eq!(octree.root.total_mass, 1.0);
        assert!(octree.root.is_leaf());
        assert_eq!(octree.root.points.len(), 1);
    }

    #[test]
    fn test_octree_subdivision() {
        let bounds = Bounds::new(Vec3::ZERO, Vec3::splat(10.0));
        let mut octree = Octree::new(bounds, 3, 1); // max 1 point per leaf
        
        // Insert two points that should cause subdivision
        octree.insert(0, Vec3::new(2.0, 2.0, 2.0), 1.0);
        octree.insert(1, Vec3::new(8.0, 8.0, 8.0), 1.0);
        
        assert_eq!(octree.root.node_count, 2);
        assert!(!octree.root.is_leaf()); // Should have subdivided
        assert!(octree.root.children.is_some());
        
        // Center of mass should be at the midpoint
        assert!((octree.root.center_of_mass - Vec3::new(5.0, 5.0, 5.0)).length() < 0.01);
        assert_eq!(octree.root.total_mass, 2.0);
    }

    #[test]
    fn test_octree_max_depth() {
        let bounds = Bounds::new(Vec3::ZERO, Vec3::splat(10.0));
        let mut octree = Octree::new(bounds, 2, 1); // max 1 point per leaf

        // Insert points that will force subdivision beyond max depth
        octree.insert(0, Vec3::new(2.0, 2.0, 2.0), 1.0);
        octree.insert(1, Vec3::new(2.1, 2.0, 2.0), 1.0);
        octree.insert(2, Vec3::new(2.0, 2.1, 2.0), 1.0);
        octree.insert(3, Vec3::new(2.1, 2.1, 2.0), 1.0);
        octree.insert(4, Vec3::new(2.0, 2.0, 2.1), 1.0);
        octree.insert(5, Vec3::new(2.1, 2.0, 2.1), 1.0);
        octree.insert(6, Vec3::new(2.0, 2.1, 2.1), 1.0);
        octree.insert(7, Vec3::new(2.1, 2.1, 2.1), 1.0);

        assert_eq!(octree.root.node_count, 8);
        assert!(!octree.root.is_leaf()); // Should have subdivided
        assert!(octree.root.children.is_some());

        let child = &octree.root.children.as_ref().unwrap()[0];
        assert_eq!(child.node_count, 8);
        assert!(!child.is_leaf());
        assert!(child.children.is_some());

        let grandchild = &child.children.as_ref().unwrap()[0];
        assert_eq!(grandchild.points.iter().count(), 8);
        assert_eq!(grandchild.node_count, 8);
        assert!(grandchild.is_leaf()); // Should not subdivide further due to max depth

        // Center of mass should be at the midpoint for all nodes
        let expected_com = Vec3::new(2.05, 2.05, 2.05);
        assert!((octree.root.center_of_mass - expected_com).length() < 0.0001);
        assert_eq!(octree.root.total_mass, NODE_MASS * 8.0);
        assert!((child.center_of_mass - expected_com).length() < 0.0001);
        assert_eq!(child.total_mass, NODE_MASS * 8.0);
        assert!((grandchild.center_of_mass - expected_com).length() < 0.0001);
        assert_eq!(grandchild.total_mass, NODE_MASS * 8.0);
    }

    #[test]
    fn test_octree_from_points() {
        let points = vec![
            (0, Vec3::new(1.0, 1.0, 1.0)),
            (1, Vec3::new(9.0, 9.0, 9.0)),
            (2, Vec3::new(5.0, 5.0, 5.0)),
        ];
        
        let octree = Octree::from_points(&points, 3, 1);
        
        assert_eq!(octree.root.node_count, 3);
        assert_eq!(octree.root.total_mass, 3.0);
        
        // Verify all points are retrievable
        let retrieved_points = octree.get_all_points();
        assert_eq!(retrieved_points.len(), 3);
        
        // Sort for comparison
        let mut original_sorted = points.clone();
        original_sorted.sort_by_key(|&(id, _)| id);
        let mut retrieved_sorted = retrieved_points;
        retrieved_sorted.sort_by_key(|&(id, _)| id);
        
        for (i, &(orig_id, orig_pos)) in original_sorted.iter().enumerate() {
            let (retr_id, retr_pos) = retrieved_sorted[i];
            assert_eq!(orig_id, retr_id);
            assert!((orig_pos - retr_pos).length() < 0.01);
        }
    }

    #[test]
    fn test_force_calculation_single_point() {
        let points = vec![(0, Vec3::new(0.0, 0.0, 0.0))];
        let octree = Octree::from_points(&points, 3, 1);
        
        let force = octree.calculate_force(Vec3::new(1.0, 0.0, 0.0), 0.5, 1.0);
        
        // Force should be in positive X direction (repelling from origin)
        assert!(force.x > 0.0);
        assert!(force.y.abs() < 0.01);
        assert!(force.z.abs() < 0.01);
    }

    #[test]
    fn test_force_calculation_multiple_points() {
        let points = vec![
            (0, Vec3::new(-1.0, 0.0, 0.0)),
            (1, Vec3::new(1.0, 0.0, 0.0)),
        ];
        let octree = Octree::from_points(&points, 3, 1);
        
        // Test point at origin should be repelled equally in both directions
        let force = octree.calculate_force(Vec3::ZERO, 0.5, 1.0);
        
        // Forces should cancel out in X direction since points are symmetric
        assert!(force.x.abs() < 0.01);
        assert!(force.y.abs() < 0.01);
        assert!(force.z.abs() < 0.01);
    }

    #[test]
    fn test_center_of_mass_calculation() {
        let bounds = Bounds::new(Vec3::ZERO, Vec3::splat(10.0));
        let mut octree = Octree::new(bounds, 3, 10); // High threshold to keep as leaf
        
        // Insert points with different masses
        octree.insert(0, Vec3::new(0.0, 0.0, 0.0), 1.0);
        octree.insert(1, Vec3::new(10.0, 0.0, 0.0), 2.0);
        
        // Center of mass should be closer to the heavier point
        // CoM = (0*1 + 10*2) / (1+2) = 20/3 ≈ 6.67
        let expected_x = 20.0 / 3.0;
        assert!((octree.root.center_of_mass.x - expected_x).abs() < 0.01);
        assert!(octree.root.center_of_mass.y.abs() < 0.01);
        assert!(octree.root.center_of_mass.z.abs() < 0.01);
        assert_eq!(octree.root.total_mass, 3.0);
    }

    #[test]
    fn test_empty_octree() {
        let points = vec![];
        let octree = Octree::from_points(&points, 3, 1);
        
        assert_eq!(octree.root.node_count, 0);
        assert_eq!(octree.root.total_mass, 0.0);
        
        let force = octree.calculate_force(Vec3::ZERO, 0.5, 1.0);
        assert_eq!(force, Vec3::ZERO);
    }

    #[test]
    fn test_theta_criterion() {
        // Create a large cluster of points far away
        let mut points = Vec::new();
        for i in 0..10 {
            points.push((i, Vec3::new(100.0 + i as f32 * 0.1, 100.0, 100.0)));
        }
        
        let octree = Octree::from_points(&points, 5, 2);
        
        // Calculate force with different theta values
        let test_pos = Vec3::ZERO;
        
        let force_low_theta = octree.calculate_force(test_pos, 0.1, 1.0);
        let force_high_theta = octree.calculate_force(test_pos, 2.0, 1.0);
        
        // Both should produce similar results since the cluster is far away
        // The high theta should use approximation more aggressively
        assert!(force_low_theta.length() > 0.0);
        assert!(force_high_theta.length() > 0.0);
        
        // The direction should be roughly the same
        let dot_product = force_low_theta.normalize().dot(force_high_theta.normalize());
        assert!(dot_product > 0.9); // Vectors should be pointing in similar directions
    }
}