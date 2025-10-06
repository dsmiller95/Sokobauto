mod spatial_hash;
mod octree;
mod config_ui;

use bevy::prelude::*;
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};
use crate::state_graph::StateGraph;
use crate::core::SharedGameState;
use rand::Rng;
use std::collections::HashMap;
use bevy::mesh::ConeAnchor;
use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::pbr::wireframe::{Wireframe, WireframePlugin};
use crate::bevy_interface::spatial_hash::SpatialHash;
use crate::bevy_interface::octree::{Octree, OctreeVisualizationNode};
use crate::bevy_interface::config_ui::{
    setup_config_panel, handle_toggle_interactions, 
    on_toggle_octree_bounds, on_toggle_center_of_mass, on_toggle_leaf_only
};

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

#[derive(Component)]
struct FpsText;

#[derive(Component)]
struct OctreeBounds {
    depth: usize,
}

#[derive(Component)]
struct OctreeCenterOfMass {
    depth: usize,
}

#[derive(Resource)]
struct OctreeVisualizationConfig {
    show_octree_bounds: bool,
    show_center_of_mass: bool,
    show_leaf_only: bool,
    max_depth_to_show: usize,
}

impl Default for OctreeVisualizationConfig {
    fn default() -> Self {
        Self {
            show_octree_bounds: true,
            show_center_of_mass: true,
            show_leaf_only: true,
            max_depth_to_show: 8,
        }
    }
}

#[derive(Resource)]
struct OctreeVisualizationMeshes {
    bounds_mesh: Handle<Mesh>,
    center_of_mass_mesh: Handle<Mesh>,
    bounds_material: Handle<StandardMaterial>,
    center_of_mass_material: Handle<StandardMaterial>,
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
        .add_systems(Startup, (setup_scene, setup_graph_from_data, setup_compute_cache, setup_fps_counter, setup_octree_visualization, setup_config_panel).chain())
        .add_systems(Update, (apply_forces, update_edges, update_fps_counter, update_octree_visualization, handle_toggle_interactions))
        .add_observer(on_toggle_octree_bounds)
        .add_observer(on_toggle_center_of_mass)
        .add_observer(on_toggle_leaf_only)
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
    // Add camera with pan/orbit controls
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
    // Apply damping
    node.velocity *= physics.damping;
    
    // Apply force
    node.velocity += total_force * dt;
    
    // Limit velocity
    if node.velocity.length() > physics.max_velocity {
        node.velocity = node.velocity.normalize() * physics.max_velocity;
    }
    
    // Update position
    transform.translation += node.velocity * dt;
    
    // Update node positions resource
}

fn apply_forces(
    mut node_query: Query<(&mut Transform, &mut GraphNode)>,
    mut node_positions: ResMut<NodePositions>,
    compute_cache: Res<GraphComputeCache>,
    physics: Res<PhysicsConfig>,
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

    match physics.physics_mode {
        PhysicsMode::BruteForce => {
            for (mut transform, mut node) in node_query.iter_mut() {
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

                integrate_physics(&mut node, &mut transform, force, &physics, dt);

                node_positions.positions.insert(node.id, transform.translation);
            }
        }
        PhysicsMode::SpatialHash => {
            let mut spatial_hash: SpatialHash<(usize, Vec3)> = SpatialHash::new(physics.spatial_hash_size);
            for &(id, pos) in &nodes_data {
                spatial_hash.insert(pos, (id, pos));
            }

            for (mut transform, mut node) in node_query.iter_mut() {
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
                
                integrate_physics(&mut node, &mut transform, force, &physics, dt);
                
                node_positions.positions.insert(node.id, transform.translation);
            }
        }
        PhysicsMode::Octree => {
            let octree = Octree::from_points(
                &nodes_data,
                physics.octree_max_depth,
                physics.octree_max_points_per_leaf,
            );

            for (mut transform, mut node) in node_query.iter_mut() {
                let mut force = Vec3::ZERO;
                let current_pos = transform.translation;

                // Calculate repulsion using octree (Barnes-Hut algorithm)
                force += octree.calculate_force(
                    current_pos,
                    physics.octree_theta,
                    physics.repulsion_strength,
                );

                force += apply_attraction_forces(&node, current_pos, &compute_cache, &node_positions, &physics);
                
                integrate_physics(&mut node, &mut transform, force, &physics, dt);
                
                node_positions.positions.insert(node.id, transform.translation);
            }
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

fn setup_fps_counter(mut commands: Commands) {
    commands.spawn((
        Text::new("FPS: --"),
        TextFont {
            font_size: 24.0,
            ..default()
        },
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            left: Val::Px(10.0),
            ..default()
        },
        FpsText,
    ));
}

fn update_fps_counter(
    diagnostics: Res<DiagnosticsStore>,
    mut fps_text_query: Query<&mut Text, With<FpsText>>,
) {
    let Ok(mut text) = fps_text_query.single_mut() else { return };
    let Some(fps_diagnostic) = diagnostics.get(&FrameTimeDiagnosticsPlugin::FPS) else { return };
    let Some(fps_smoothed) = fps_diagnostic.smoothed() else { return };
    text.0 = format!("FPS: {:.1}", fps_smoothed);
}

fn setup_octree_visualization(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let bounds_mesh = meshes.add(Cuboid::new(0.99, 0.99, 0.99));
    let center_of_mass_mesh = meshes.add(Sphere::new(0.8).mesh().ico(4).unwrap());

    let bounds_material = materials.add(StandardMaterial {
        base_color: Color::hsl(10.0, 0.3, 0.0).with_alpha(0.0), // Semi-transparent wireframe
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        ..default()
    });

    let center_of_mass_material = materials.add(StandardMaterial {
        base_color: Color::hsl(190.0, 1.0, 0.5),
        unlit: true,
        ..default()
    });

    commands.insert_resource(OctreeVisualizationMeshes {
        bounds_mesh,
        center_of_mass_mesh,
        bounds_material,
        center_of_mass_material,
    });
}

fn update_octree_visualization(
    mut commands: Commands,
    node_query: Query<&Transform, (With<GraphNode>, Without<OctreeBounds>, Without<OctreeCenterOfMass>)>,
    mut bounds_query: Query<(Entity, &mut Transform, &OctreeBounds), (Without<GraphNode>, Without<OctreeCenterOfMass>)>,
    mut center_query: Query<(Entity, &mut Transform, &OctreeCenterOfMass), (Without<GraphNode>, Without<OctreeBounds>)>,
    physics: Res<PhysicsConfig>,
    visualization_config: Res<OctreeVisualizationConfig>,
    visualization_meshes: Res<OctreeVisualizationMeshes>,
    time: Res<Time>,
) {
    if physics.physics_mode != PhysicsMode::Octree || time.elapsed().as_secs_f32() > 60.0 {
        return;
    }

    let nodes_data: Vec<(usize, Vec3)> = node_query.iter()
        .enumerate()
        .map(|(i, transform)| (i, transform.translation))
        .collect();

    if nodes_data.is_empty() {
        return;
    }

    let octree = Octree::from_points(
        &nodes_data,
        physics.octree_max_depth,
        physics.octree_max_points_per_leaf,
    );

    let visualization_data = octree.get_visualization_data();

    let filtered_visualization: Vec<&OctreeVisualizationNode> = visualization_data.iter()
        .filter(|node| {
            node.depth <= visualization_config.max_depth_to_show
        })
        .filter(|node| {
            !visualization_config.show_leaf_only || node.is_leaf
        })
        .collect();

    let empty: Vec<&OctreeVisualizationNode> = Vec::new();

    // bounds visualization
    update_visualization_entities(
        &mut commands,
        bounds_query.iter_mut().map(|(e, t, c)| (e, t, c.depth)).collect(),
        if visualization_config.show_octree_bounds { &filtered_visualization } else { &empty },
        |node| (node.bounds.center(), node.bounds.size(), node.depth),
        |depth| (
            Mesh3d(visualization_meshes.bounds_mesh.clone()),
            MeshMaterial3d(visualization_meshes.bounds_material.clone()),
            Wireframe::default(),
            OctreeBounds { depth }
        ),
    );

    // center of mass visualization
    update_visualization_entities(
        &mut commands,
        center_query.iter_mut().map(|(e, t, c)| (e, t, c.depth)).collect(),
        if visualization_config.show_center_of_mass { &filtered_visualization } else { &empty },
        |node| {
            // Scale based on mass, per volume
            let size = (node.total_mass).powf(1.0/3.0).clamp(0.2, 20.0);
            (node.center_of_mass, Vec3::splat(size), node.depth)
        },
        |depth| (
            Mesh3d(visualization_meshes.center_of_mass_mesh.clone()),
            MeshMaterial3d(visualization_meshes.center_of_mass_material.clone()),
            OctreeCenterOfMass { depth }
        ),
    );
}

fn update_visualization_entities<C: Bundle>(
    commands: &mut Commands,
    mut existing_entities: Vec<(Entity, Mut<Transform>, usize)>,
    new_data: &[&OctreeVisualizationNode],
    extract_transform_data: impl Fn(&OctreeVisualizationNode) -> (Vec3, Vec3, usize),
    create_component: impl Fn(usize) -> C,
) {
    for (i, data) in new_data.iter().enumerate() {
        let (position, scale, depth) = extract_transform_data(data);
        
        if let Some((_, transform, _)) = existing_entities.get_mut(i) {
            transform.translation = position;
            transform.scale = scale;
        } else {
            commands.spawn((
                Transform::from_translation(position).with_scale(scale),
                create_component(depth),
            ));
        }
    }

    for (entity, _, _) in existing_entities.iter().skip(new_data.len()) {
        commands.entity(*entity).despawn();
    }
}