mod spatial_hash;
mod octree;
mod config_ui;
mod bounds;
mod fps_ui;
mod octree_visualization;

use bevy::prelude::*;
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};
use crate::state_graph::StateGraph;
use crate::core::SharedGameState;
use rand::Rng;
use std::collections::{HashMap};
use bevy::mesh::ConeAnchor;
use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::pbr::wireframe::{Wireframe, WireframePlugin};
use crate::bevy_interface::bounds::Bounds;
use crate::bevy_interface::spatial_hash::SpatialHash;
use crate::bevy_interface::octree::{Octree, OctreeVisualizationNode, OctreeResource};
use crate::bevy_interface::config_ui::{
    setup_config_panel, handle_toggle_interactions, on_toggle_event
};
use crate::bevy_interface::fps_ui::{setup_fps_counter, update_fps_counter};
use crate::bevy_interface::octree_visualization::{setup_octree_visualization, update_octree_visualization, OctreeVisualizationConfig};

#[derive(Component)]
struct GraphNode {
    id: usize,
    velocity: Vec3,
    on_targets: usize,
}

#[derive(Component)]
struct GraphEdge {
    from_id: usize,
    to_id: usize,
}

#[derive(Resource)]
struct NodePositions {
    positions: HashMap<usize, Vec3>,
}

#[derive(PartialEq, Eq, Clone, Copy)]
enum PhysicsMode {
    Octree,
    SpatialHash,
    BruteForce,
}

#[derive(Resource)]
struct PhysicsConfig {
    repulsion_strength: f32,
    attraction_strength: f32,
    damping: f32,
    max_velocity: f32,
    physics_mode: PhysicsMode,
    // Spatial hash settings
    spatial_hash_size: f32,
    // Octree settings
    // what is the maximum allowed ratio between the size of a node cluster and its distance from the target node
    // smaller values yield more accurate results but are slower
    octree_theta: f32,
    octree_max_depth: usize,
    octree_max_points_per_leaf: usize,
}

pub fn visualize_graph(graph: &StateGraph, shared: &SharedGameState) {
    let graph_data = GraphData::from_state_graph(graph, shared);

    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "3D Force-Directed Graph".to_string(),
                resolution: (1200, 800).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(PanOrbitCameraPlugin)
        .add_plugins(WireframePlugin::default())
        .add_plugins(FrameTimeDiagnosticsPlugin::default())
        .insert_resource(graph_data)
        .insert_resource(OctreeVisualizationConfig::default())
        .insert_resource(PhysicsConfig {
            repulsion_strength: 50.0,
            attraction_strength: 2.0,
            damping: 0.95,
            max_velocity: 10.0,
            physics_mode: PhysicsMode::Octree,
            spatial_hash_size: 5.0,
            // Octree settings - default to octree with good parameters for 10k-50k nodes
            octree_theta: 0.8, // Good balance between accuracy and performance
            octree_max_depth: 8, // Should handle 50k nodes well
            octree_max_points_per_leaf: 16, // Reasonable leaf size
        })
        .add_systems(Startup, (setup_scene, setup_graph_from_data, setup_octree_resource, setup_compute_cache, setup_fps_counter, setup_octree_visualization, setup_config_panel).chain())
        .add_systems(Update, (apply_forces_and_update_octree, update_edges, update_fps_counter, update_octree_visualization, handle_toggle_interactions))
        .add_observer(on_toggle_event)
        .run();
}

// Resource to hold graph data
#[derive(Resource)]
struct GraphData {
    nodes: Vec<GraphNodeData>,
    edges: Vec<GraphEdgeData>,
    max_on_targets: usize,
}

struct GraphNodeData {
    id: usize,
    on_targets: usize,
}

struct GraphEdgeData {
    from: usize,
    to: usize,
}

impl GraphData {
    fn from_state_graph(graph: &StateGraph, shared: &SharedGameState) -> Self {
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
struct GraphComputeCache {
    // map of node IDs to every one of their neighbors, in both directions.
    neighbor_map: HashMap<usize, Vec<usize>>,
    // map of node IDs to their Entity
    entity_map: HashMap<usize, Entity>,
}

impl GraphComputeCache {
    fn from_graph(graph: &GraphData, all_nodes: Vec<(usize, Entity)>) -> Self {
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
}

fn setup_scene(
    mut commands: Commands,
) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(20.0, 20.0, 20.0).looking_at(Vec3::ZERO, Vec3::Y),
        PanOrbitCamera::default(),
    ));
}

fn setup_graph_from_data(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    graph_data: Res<GraphData>,
) {
    let mut rng = rand::thread_rng();
    let mut node_positions = HashMap::new();

    let node_mesh = meshes.add(Sphere::new(0.8).mesh().ico(4).unwrap());

    let node_materials = (0..=graph_data.max_on_targets).map(|on_targets| {
        let color = interpolate_color(on_targets, graph_data.max_on_targets);
        materials.add(StandardMaterial {
            base_color: color,
            unlit: true,
            ..default()
        })
    })
        .collect::<Vec<_>>();

    for node_data in &graph_data.nodes {
        let position = Vec3::new(
            rng.gen_range(-15.0..15.0),
            rng.gen_range(-15.0..15.0),
            rng.gen_range(-15.0..15.0),
        );
        
        commands.spawn((
            Mesh3d(node_mesh.clone()),
            MeshMaterial3d(node_materials[node_data.on_targets].clone()),
            Transform::from_translation(position),
            GraphNode {
                id: node_data.id,
                velocity: Vec3::ZERO,
                on_targets: node_data.on_targets,
            },
        ));
        
        node_positions.insert(node_data.id, position);
    }

    let mut arrow_mesh = Cone::new(0.15, 0.2).mesh()
        .anchor(ConeAnchor::Tip).resolution(8).build();
    arrow_mesh.translate_by(Vec3::new(0.0, 0.3, 0.0));
    let mut edge_mesh = Mesh::from(Capsule3d::new(0.03, 1.0));
    edge_mesh.merge(&arrow_mesh).unwrap();

    let edge_mesh_handle = meshes.add(edge_mesh);
    let edge_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.7, 0.7, 0.7),
        unlit: true,
        ..default()
    });

    for edge in graph_data.edges.iter() {
        let from_id = edge.from;
        let to_id = edge.to;
        if let (Some(&from_pos), Some(&to_pos)) = (
            node_positions.get(&from_id),
            node_positions.get(&to_id),
        ) {
            let mut transform = Transform::default();
            set_edge_transform(&mut transform, from_pos, to_pos);

            commands.spawn((
                Mesh3d(edge_mesh_handle.clone()),
                MeshMaterial3d(edge_material.clone()),
                transform,
                GraphEdge {
                    from_id,
                    to_id,
                },
            ));
        }
    }
    
    commands.insert_resource(NodePositions { positions: node_positions });
}

fn setup_octree_resource(
    mut commands: Commands,
    node_positions: Res<NodePositions>,
    physics: Res<PhysicsConfig>,
) {
    let points: Vec<(usize, Vec3)> = node_positions.positions.iter().map(|(&id, &pos)| (id, pos)).collect();
    let octree_resource = OctreeResource {
        octree: Octree::from_points(
            &points,
            physics.octree_max_depth,
            physics.octree_max_points_per_leaf,
        )
    };
    commands.insert_resource(octree_resource);
}

fn setup_compute_cache(
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
    
    if let Some(neighbors) = compute_cache.neighbor_map.get(&node.id) {
        for &neighbor_id in neighbors {
            if let Some(&neighbor_pos) = node_positions.positions.get(&neighbor_id) {
                let diff = neighbor_pos - current_pos;
                let distance = diff.length().max(0.1);
                let attraction = diff.normalize() * physics.attraction_strength * distance;
                attraction_force += attraction;
            }
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

fn apply_forces_and_update_octree(
    mut node_query: Query<(&mut Transform, &mut GraphNode)>,
    mut node_positions: ResMut<NodePositions>,
    compute_cache: Res<GraphComputeCache>,
    physics: Res<PhysicsConfig>,
    mut octree_resource: ResMut<OctreeResource>,
    time: Res<Time>,
) {
    if time.elapsed().as_secs_f32() > 60.0 {
        return;
    }
    println!("Simulating step at t={:.2}", time.elapsed().as_secs_f32());

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

fn update_edges(
    mut edge_query: Query<(&mut Transform, &GraphEdge)>,
    node_positions: Res<NodePositions>,
    time: Res<Time>,
) {
    if time.elapsed().as_secs_f32() > 60.0 {
        return;
    }
    for (mut transform, edge) in edge_query.iter_mut() {
        if let (Some(&from_pos), Some(&to_pos)) = (
            node_positions.positions.get(&edge.from_id),
            node_positions.positions.get(&edge.to_id),
        ) {
            set_edge_transform(&mut *transform, from_pos, to_pos);
        }
    }
}

fn set_edge_transform(transform: &mut Transform, from: Vec3, to: Vec3) {
    let direction = (to - from).normalize();
    let distance = from.distance(to);
    let center = (from + to) / 2.0;

    transform.translation = center;
    transform
        .align(Dir3::Y, direction, Dir3::Z, Vec3::Z);
    transform.scale.y = distance;
}

fn interpolate_color(on_targets: usize, max_on_targets: usize) -> Color {
    let t = if max_on_targets == 0 {
        0.0
    } else {
        on_targets as f32 / max_on_targets as f32
    };
    
    Color::srgb(1.0 - t, 0.0, t)
}
