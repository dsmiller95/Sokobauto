use bevy::prelude::Resource;
use bevy::prelude::*;
use crate::bevy_interface::bounds::Bounds;

#[derive(Resource)]
pub struct OctreeResource {
    pub octree: Octree,
}

pub struct Octree {
    root: OctreeNode,
    max_depth: usize,
    max_points_per_leaf: usize,
}

#[derive(Debug, Clone)]
pub struct OctreeNode {
    pub bounds: Bounds,
    pub center_of_mass: Vec3,
    pub total_mass: f32,
    pub node_count: usize,
    pub children: Option<Box<[OctreeNode; 8]>>,
    pub points: Vec<(usize, Vec3)>,
}


#[derive(Debug, Clone)]
pub struct OctreeVisualizationNode {
    pub bounds: Bounds,
    pub center_of_mass: Vec3,
    pub total_mass: f32,
    pub depth: usize,
    pub is_leaf: bool,
}


pub const NODE_MASS: f32 = 1.0;
pub const MINIMUM_DISTANCE: f32 = 0.01;

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

        let mut min = points[0].1;
        let mut max = points[0].1;

        for &(_, pos) in points {
            min = min.min(pos);
            max = max.max(pos);
        }

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
            panic!("Cannot insert point outside of octree bounds. Consider using insert_resize.");
        }

        self.root.insert(node_id, position, mass, self.max_depth, self.max_points_per_leaf);
    }

    pub fn insert_resize(&mut self, node_id: usize, position: Vec3, mass: f32, resize: impl FnOnce(&Bounds, &Vec3) -> Bounds) {
        if !self.root.bounds.contains(position) {
            let new_bounds = resize(&self.root.bounds, &position);
            if(!new_bounds.contains(position)) {
                panic!("Resize function did not produce bounds that contain the new point");
            }

            self.resize_bounds(new_bounds);
        }

        self.root.insert(node_id, position, mass, self.max_depth, self.max_points_per_leaf);
    }

    pub fn remove(&mut self, node_id: usize, position: Vec3) -> bool {
        self.root.remove(node_id, position, self.max_depth, self.max_points_per_leaf)
    }

    pub fn update(&mut self, node_id: usize, old_pos: Vec3, new_pos: Vec3) -> bool {
        let removed = self.root.remove(node_id, old_pos, self.max_depth, self.max_points_per_leaf);
        if !removed {
            return false;
        }

        self.root.insert(node_id, new_pos, NODE_MASS, self.max_depth, self.max_points_per_leaf);
        true
    }
    pub fn update_resize(&mut self, node_id: usize, old_pos: Vec3, new_pos: Vec3, resize: impl FnOnce(&Bounds, &Vec3) -> Bounds) -> bool {
        let removed = self.root.remove(node_id, old_pos, self.max_depth, self.max_points_per_leaf);
        if !removed {
            return false;
        }

        self.insert_resize(node_id, new_pos, NODE_MASS, resize);
        true
    }

    pub fn resize_bounds(&mut self, new_bounds: Bounds) {
        let all_points = self.get_all_points();
        let mut new_root = OctreeNode::new(new_bounds);
        for (id, pos) in all_points {
            new_root.insert(id, pos, NODE_MASS, self.max_depth, self.max_points_per_leaf);
        }
        self.root = new_root;
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
                    continue;
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
        self.root.collect_all_points(&mut points);
        points
    }

    pub fn get_visualization_data(&self) -> Vec<OctreeVisualizationNode> {
        let mut data = Vec::new();
        self.collect_visualization_recursive(&self.root, 0, &mut data);
        data
    }

    fn collect_visualization_recursive(&self, node: &OctreeNode, depth: usize, data: &mut Vec<OctreeVisualizationNode>) {
        if node.node_count == 0 {
            return;
        }

        data.push(OctreeVisualizationNode {
            bounds: node.bounds,
            center_of_mass: node.center_of_mass,
            total_mass: node.total_mass,
            depth,
            is_leaf: node.is_leaf(),
        });

        if let Some(ref children) = node.children {
            for child in children.iter() {
                self.collect_visualization_recursive(child, depth + 1, data);
            }
        }
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

            if self.points.len() > max_points_per_leaf && max_depth > 0 {
                self.subdivide(max_depth - 1, max_points_per_leaf);
            }
        } else {
            let octant_index = self.bounds.octant_index(position);
            if let Some(ref mut children) = self.children {
                children[octant_index].insert(node_id, position, mass, max_depth - 1, max_points_per_leaf);
            }
        }
    }

    fn subdivide(&mut self, remaining_depth: usize, max_points_per_leaf: usize) {
        let mut children = Vec::with_capacity(8);
        for i in 0..8 {
            children.push(OctreeNode::new(self.bounds.octant(i)));
        }
        self.children = Some(children.try_into().unwrap());

        let points = std::mem::take(&mut self.points);
        for (node_id, position) in points {
            let octant_index = self.bounds.octant_index(position);
            if let Some(ref mut children) = self.children {
                children[octant_index].insert(node_id, position, NODE_MASS, remaining_depth, max_points_per_leaf);
            }
        }
    }

    pub fn remove(&mut self, node_id: usize, position: Vec3, max_depth: usize, max_points_per_leaf: usize) -> bool {
        if !self.bounds.contains(position) {
            return false;
        }
        if self.is_leaf() {
            let orig_len = self.points.len();
            self.points.retain(|(id, _)| *id != node_id);
            if self.points.len() < orig_len {
                // Recompute mass and center of mass
                self.node_count = self.points.len();
                self.total_mass = self.node_count as f32 * NODE_MASS;
                if self.node_count > 0 {
                    let sum: Vec3 = self.points.iter().fold(Vec3::ZERO, |acc, (_, p)| acc + *p);
                    self.center_of_mass = sum / self.node_count as f32;
                } else {
                    self.center_of_mass = Vec3::ZERO;
                }
                return true;
            }
            return false;
        } else if let Some(ref mut children) = self.children {
            // TODO: implement tree pruning, if children are empty or node count is too low.
            let octant_index = self.bounds.octant_index(position);
            let removed = children[octant_index].remove(node_id, position, max_depth - 1, max_points_per_leaf);
            if removed {
                self.node_count = children.iter().map(|c| c.node_count).sum::<usize>();
                self.total_mass = children.iter().map(|c| c.total_mass).sum::<f32>();
                if self.node_count > 0 && self.total_mass > 0.0 {
                    let sum: Vec3 = children.iter().fold(Vec3::ZERO, |acc, c| acc + c.center_of_mass * c.total_mass);
                    self.center_of_mass = sum / self.total_mass;
                } else {
                    self.center_of_mass = Vec3::ZERO;
                }
            }
            return removed;
        }
        false
    }

    pub fn collect_all_points(&self, out: &mut Vec<(usize, Vec3)>) {
        if self.is_leaf() {
            out.extend_from_slice(&self.points);
        } else if let Some(ref children) = self.children {
            for c in children.iter() {
                c.collect_all_points(out);
            }
        }
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_remove_and_update_point() {
        let points = vec![
            (1, Vec3::new(1.0, 2.0, 3.0)),
            (2, Vec3::new(4.0, 5.0, 6.0)),
        ];
        let mut octree = Octree::from_points(&points, 3, 1);

        assert_eq!(octree.root.node_count, 2);
        assert!(octree.remove(1, Vec3::new(1.0, 2.0, 3.0)));
        assert_eq!(octree.root.node_count, 1);

        octree.update(2, Vec3::new(4.0, 5.0, 6.0), Vec3::new(7.0, 8.0, 9.0));
        let pts = octree.get_all_points();
        assert_eq!(pts.len(), 1);
        assert_eq!(pts[0].0, 2);
        assert!((pts[0].1 - Vec3::new(7.0, 8.0, 9.0)).length() < 0.01);
    }

    #[test]
    fn test_resize_bounds_keeps_points() {
        let points = vec![
            (0, Vec3::new(1.0, 1.0, 1.0)),
            (1, Vec3::new(9.0, 9.0, 9.0)),
            (2, Vec3::new(5.0, 5.0, 5.0)),
        ];

        let mut octree = Octree::from_points(&points, 3, 1);
        octree.resize_bounds(octree.root.bounds.doubled());

        let pts = octree.get_all_points();
        assert_eq!(pts.len(), 3);
    }

    #[test]
    fn test_update_resize_resizes_tree() {
        let points = vec![
            (1, Vec3::splat(0.0)),
            (2, Vec3::splat(1.0)),
        ];
        let mut octree = Octree::from_points(&points, 3, 1);

        let updated = octree.update_resize(1, Vec3::splat(0.0), Vec3::splat(10.0), Bounds::resize_expand);
        assert!(updated);

        let expected_bounds = Bounds {
            min: Vec3::splat(-5.0),
            max: Vec3::splat(15.0),
        };

        assert!(octree.root.bounds.contains_other(&expected_bounds));
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

    #[test]
    fn test_visualization_data() {
        let points = vec![
            (0, Vec3::new(1.0, 1.0, 1.0)),
            (1, Vec3::new(9.0, 9.0, 9.0)),
            (2, Vec3::new(5.0, 5.0, 5.0)),
        ];
        
        let octree = Octree::from_points(&points, 3, 1);
        let viz_data = octree.get_visualization_data();
        
        // Should have at least the root node
        assert!(!viz_data.is_empty());
        
        // Check that we have visualization data with proper structure
        let root_node = &viz_data[0];
        assert_eq!(root_node.depth, 0);
        assert_eq!(root_node.total_mass, 3.0);
        assert!(root_node.bounds.contains(Vec3::new(1.0, 1.0, 1.0)));
        assert!(root_node.bounds.contains(Vec3::new(9.0, 9.0, 9.0)));
        assert!(root_node.bounds.contains(Vec3::new(5.0, 5.0, 5.0)));
        
        // Should have child nodes since points are spread out
        let child_nodes: Vec<_> = viz_data.iter().filter(|node| node.depth > 0).collect();
        assert!(!child_nodes.is_empty());
    }
}