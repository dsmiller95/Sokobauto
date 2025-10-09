use std::collections::HashMap;
use bevy::prelude::*;
use crate::bevy_interface::{GraphNode, NodePositions, PhysicsConfig, PhysicsMode, UserConfig};
use crate::bevy_interface::bounds::Bounds;
use crate::bevy_interface::octree::OctreeResource;
use crate::bevy_interface::spatial_hash::SpatialHash;
use crate::core::SharedGameState;
use crate::state_graph::StateGraph;

#[derive(Resource)]
pub struct NodeIdToIndex(HashMap<usize, usize>);

// Resource to hold graph data
#[derive(Resource)]
pub struct GraphData {
    pub nodes: Vec<GraphNodeData>,
    pub edges: Vec<GraphEdgeData>,
    pub max_on_targets: usize,
}

pub struct GraphNodeData {
    pub id: usize,
    pub on_targets: usize,
}

pub struct GraphEdgeData {
    pub from: usize,
    pub to: usize,
}

impl GraphData {
    pub fn from_state_graph(graph: &StateGraph, shared: &SharedGameState) -> Self {
        let nodes: Vec<GraphNodeData> = graph.nodes.iter()
            .map(|(state, &id)| GraphNodeData {
                id,
                on_targets: shared.count_boxes_on_goals(&state.environment.boxes),
            })
            .collect();

        let edges: Vec<GraphEdgeData> = graph.edges.iter()
            .map(|edge| GraphEdgeData {
                from: edge.from,
                to: edge.to,
            })
            .collect();

        let max_on_targets = nodes.iter()
            .map(|node| node.on_targets)
            .max()
            .unwrap_or(1);

        Self { nodes, edges, max_on_targets }
    }
}

// resource which holds precomputed data. must be updated accordingly whenever the graph changes.
#[derive(Resource)]
pub struct GraphComputeCache {
    // map of node IDs to every one of their neighbors, in both directions.
    neighbor_map: HashMap<usize, Vec<usize>>,
    // map of node IDs to their Entity
    entity_map: HashMap<usize, Entity>,
}

impl GraphComputeCache {
    pub fn from_graph(graph: &GraphData, all_nodes: Vec<(usize, Entity)>) -> Self {
        let mut neighbor_map: HashMap<usize, Vec<usize>> = HashMap::new();

        for edge in &graph.edges {
            neighbor_map.entry(edge.from).or_default().push(edge.to);
            neighbor_map.entry(edge.to).or_default().push(edge.from);
        }

        for neighbors in neighbor_map.values_mut() {
            neighbors.sort_unstable();
            neighbors.dedup();
            neighbors.shrink_to_fit();
        }

        let entity_map: HashMap<usize, Entity> = all_nodes.into_iter().collect();

        Self {
            neighbor_map,
            entity_map
        }
    }

    pub fn iterate_neighbors(&self, node_id: &usize) -> impl Iterator<Item = &usize> + '_ {
        self.neighbor_map.get(node_id)
            .into_iter()
            .flatten()
    }
}

impl NodeIdToIndex {
    pub fn new(id_to_index_map: HashMap<usize, usize>) -> Self {
        Self(id_to_index_map)
    }

    pub fn get_indexed_vertex_positions(&self, node_positions: &NodePositions) -> Vec<Vec3> {
        let mut vertices = vec![Vec3::ZERO; self.0.len()];
        for (&node_id, &index) in &self.0 {
            if let Some(&position) = node_positions.positions.get(&node_id) {
                if index < vertices.len() {
                    vertices[index] = position;
                }
            }
        }
        vertices
    }
}

pub fn setup_compute_cache(
    node_query: Query<(&GraphNode, Entity)>,
    mut commands: Commands,
    graph_data: Res<GraphData>,
) {
    let all_nodes: Vec<(usize, Entity)> = node_query.iter()
        .map(|(node, entity)| (node.id, entity))
        .collect();
    let compute_cache = GraphComputeCache::from_graph(
        &graph_data,
        all_nodes
    );
    commands.insert_resource(compute_cache);
}

fn apply_attraction_forces(
    node: &GraphNode,
    current_pos: Vec3,
    compute_cache: &GraphComputeCache,
    node_positions: &NodePositions,
    physics: &PhysicsConfig,
) -> Vec3 {
    let mut attraction_force = Vec3::ZERO;

    for &neighbor_id in compute_cache.iterate_neighbors(&node.id) {
        if let Some(&neighbor_pos) = node_positions.positions.get(&neighbor_id) {
            let diff = neighbor_pos - current_pos;
            let distance = diff.length().max(0.1);
            let attraction = diff.normalize() * physics.attraction_strength * distance;
            attraction_force += attraction;
        }
    }

    attraction_force
}

fn integrate_physics(
    node: &mut GraphNode,
    transform: &mut Transform,
    total_force: Vec3,
    physics: &PhysicsConfig,
    dt: f32,
) {
    node.velocity *= physics.damping;

    node.velocity += total_force * dt;

    if node.velocity.length() > physics.max_velocity {
        node.velocity = node.velocity.normalize() * physics.max_velocity;
    }

    transform.translation += node.velocity * dt;
}

pub fn apply_forces_and_update_octree(
    mut node_query: Query<(&mut Transform, &mut GraphNode)>,
    mut node_positions: ResMut<NodePositions>,
    compute_cache: Res<GraphComputeCache>,
    physics: Res<PhysicsConfig>,
    user_config: Res<UserConfig>,
    mut octree_resource: ResMut<OctreeResource>,
    time: Res<Time>,
) {
    if user_config.is_simulation_disabled(&time) {
        return;
    }

    let dt = time.delta_secs();
    let nodes_data: Vec<(usize, Vec3)> = node_query.iter()
        .map(|(transform, node)| (node.id, transform.translation))
        .collect();

    let mut forces: HashMap<usize, Vec3> = HashMap::<usize, Vec3>::with_capacity(nodes_data.len());
    match physics.physics_mode {
        PhysicsMode::BruteForce => {
            for (transform, node) in node_query.iter() {
                let mut force = Vec3::ZERO;
                let current_pos = transform.translation;
                for &(other_id, other_pos) in nodes_data.iter() {
                    if node.id == other_id { continue; }
                    let diff = current_pos - other_pos;
                    let distance = diff.length().max(0.1);
                    let repulsion = diff.normalize() * physics.repulsion_strength / (distance * distance);
                    force += repulsion;
                }
                force += apply_attraction_forces(&node, current_pos, &compute_cache, &node_positions, &physics);
                forces.insert(node.id, force);
            }
        }
        PhysicsMode::SpatialHash => {
            let mut spatial_hash: SpatialHash<(usize, Vec3)> = SpatialHash::new(physics.spatial_hash_size);
            for &(id, pos) in &nodes_data {
                spatial_hash.insert(pos, (id, pos));
            }
            for (transform, node) in node_query.iter() {
                let mut force = Vec3::ZERO;
                let current_pos = transform.translation;
                for &(other_id, other_pos) in spatial_hash.iter_all_nearby(current_pos) {
                    if node.id == other_id { continue; }
                    let diff = current_pos - other_pos;
                    let distance = diff.length().max(0.1);
                    let repulsion = diff.normalize() * physics.repulsion_strength / (distance * distance);
                    force += repulsion;
                }
                force += apply_attraction_forces(&node, current_pos, &compute_cache, &node_positions, &physics);
                forces.insert(node.id, force);
            }
        }
        PhysicsMode::Octree => {
            let octree = &octree_resource.octree;
            for (transform, node) in node_query.iter() {
                let mut force = Vec3::ZERO;
                let current_pos = transform.translation;
                force += octree.calculate_force(
                    current_pos,
                    physics.octree_theta,
                    physics.repulsion_strength,
                );
                force += apply_attraction_forces(&node, current_pos, &compute_cache, &node_positions, &physics);
                forces.insert(node.id, force);
            }
        }
    }


    for (mut transform, mut node) in node_query.iter_mut() {
        if let Some(force) = forces.get(&node.id) {
            let old_pos = transform.translation;
            integrate_physics(&mut node, &mut transform, *force, &physics, dt);
            let new_pos = transform.translation;

            octree_resource.octree.update_resize(node.id, old_pos, new_pos, Bounds::resize_expand);
            node_positions.positions.insert(node.id, new_pos);
        }
    }
}