mod spatial_hash;
pub mod octree;
mod config_ui;
pub mod bounds;
mod fps_ui;
mod octree_visualization;
mod edge_renderer;
mod graph_compute;

use bevy::prelude::*;
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};
use crate::state_graph::StateGraph;
use crate::core::SharedGameState;
use rand::Rng;
use std::collections::{HashMap};
use bevy::diagnostic::{FrameTimeDiagnosticsPlugin};
use bevy::pbr::wireframe::{WireframePlugin};
use crate::bevy_interface::bounds::Bounds;
use crate::bevy_interface::spatial_hash::SpatialHash;
use crate::bevy_interface::octree::{Octree, OctreeResource};
use crate::bevy_interface::config_ui::{setup_config_panel, handle_toggle_interactions, on_toggle_event, on_slider_event, SliderEvent, ConfigChangedEvent, ConfigType, SliderType};
use crate::bevy_interface::fps_ui::{setup_fps_counter, update_fps_counter};
use crate::bevy_interface::octree_visualization::{setup_octree_visualization, update_octree_visualization, OctreeVisualizationConfig};
use crate::bevy_interface::edge_renderer::{EdgeRenderPlugin, EdgeRenderData, spawn_edge_mesh};
use crate::bevy_interface::graph_compute::{apply_forces_and_update_octree, setup_compute_cache, GraphComputeCache, GraphData, NodeIdToIndex};

const RENDER_NODES: bool = true;
const RENDER_EDGES: bool = true;
const USE_SHADER_EDGES: bool = true;

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
    octree_min_points_per_node: usize,
}

#[derive(Resource)]
struct UserConfig {
    force_simulation_enabled: bool,
    disable_rendering: bool,
    node_size_multiplier: f32,
}

#[derive(Resource)]
struct ReusedMeshes {
    node_mesh: Handle<Mesh>,
}

impl UserConfig {
    fn is_simulation_disabled(&self, time: &Time) -> bool {
        !self.force_simulation_enabled && time.elapsed().as_secs_f32() > 10.0
    }
    fn is_octree_update_disabled(&self, time: &Time, physics_config: &PhysicsConfig) -> bool {
        physics_config.physics_mode != PhysicsMode::Octree ||
            self.is_simulation_disabled(time)
    }
    fn is_rendering_disabled(&self) -> bool {
        self.disable_rendering
    }
}

pub fn visualize_graph(graph: &StateGraph, shared: &SharedGameState) {
    let graph_data = GraphData::from_state_graph(graph, shared);
    let user_config = UserConfig {
        force_simulation_enabled: false,
        disable_rendering: false,
        node_size_multiplier: 1.0,
    };

    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
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
        .insert_resource(user_config)
        .insert_resource(graph_data)
        .insert_resource(OctreeVisualizationConfig::default());

    if USE_SHADER_EDGES {
        app.add_plugins(EdgeRenderPlugin);
    }

    app.insert_resource(PhysicsConfig {
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
            octree_min_points_per_node: 8, // Prevent excessive subdivision
        })
        .add_systems(Startup, (setup_scene, setup_graph_from_data, setup_octree_resource, setup_compute_cache, setup_fps_counter, setup_octree_visualization, setup_config_panel).chain())
        .add_systems(Update, (apply_forces_and_update_octree, update_edges, update_fps_counter, update_octree_visualization, handle_toggle_interactions));

    if USE_SHADER_EDGES {
        app.add_systems(Startup, spawn_edge_mesh.after(setup_graph_from_data))
            .add_systems(Update, update_shader_edge_data);
    }

    app
        .add_observer(on_toggle_event)
        .add_observer(on_slider_event)
        .add_observer(on_config_changed)
        .run();
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

const DEFAULT_NODE_SPHERE_SIZE: f32 = 0.8;

fn on_config_changed(
    trigger: On<ConfigChangedEvent>,
    mut commands: Commands,
    shared_meshes: Res<ReusedMeshes>,
    mut meshes: ResMut<Assets<Mesh>>,
    config: Res<OctreeVisualizationConfig>,
    user_config: Res<UserConfig>,
) {
    match &trigger.event().config_type {
        ConfigType::Slider(SliderType::NodeSizeMultiplier) => {
            let new_multiplier = user_config.node_size_multiplier;

            let node_mesh_handle = shared_meshes.node_mesh.clone();
            let node_mesh = meshes.get_mut(&node_mesh_handle).unwrap();

            let mesh_size = DEFAULT_NODE_SPHERE_SIZE * new_multiplier;
            *node_mesh = Sphere::new(mesh_size).mesh().ico(0).unwrap();
        }
        _ => {}
    }
}

fn setup_graph_from_data(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    graph_data: Res<GraphData>,
) {
    let mut rng = rand::thread_rng();
    let mut node_positions = HashMap::new();

    let node_mesh = meshes.add(Sphere::new(DEFAULT_NODE_SPHERE_SIZE).mesh().ico(0).unwrap());

    let node_materials = (0..=graph_data.max_on_targets).map(|on_targets| {
        let color = interpolate_color(on_targets, graph_data.max_on_targets);
        materials.add(StandardMaterial {
            base_color: color,
            unlit: true,
            ..default()
        })
    })
        .collect::<Vec<_>>();

    // let mut arrow_mesh = Cone::new(0.15, 0.2).mesh()
    //     .anchor(ConeAnchor::Tip).resolution(4).build();
    // arrow_mesh.translate_by(Vec3::new(0.0, 0.3, 0.0));
    // let mut edge_mesh = Mesh::from(Capsule3d::new(0.03, 1.0).mesh()
    //     .latitudes(4).longitudes(4).build());
    // edge_mesh.merge(&arrow_mesh).unwrap();

    let edge_mesh_handle = if !USE_SHADER_EDGES {
        let edge_mesh = Mesh::from(Segment3d::new(Vec3::new(0.0, -0.5, 0.0), Vec3::new(0.0, 0.5, 0.0)));
        Some(meshes.add(edge_mesh))
    } else {
        None
    };

    let edge_material_handle = if !USE_SHADER_EDGES {
        Some(materials.add(StandardMaterial {
            base_color: Color::srgb(0.7, 0.7, 0.7),
            unlit: true,
            ..default()
        }))
    } else {
        None
    };

    for node_data in &graph_data.nodes {
        let position = Vec3::new(
            rng.gen_range(-15.0..15.0),
            rng.gen_range(-15.0..15.0),
            rng.gen_range(-15.0..15.0),
        );
        
        let mut entity = commands.spawn((
            Transform::from_translation(position),
            GraphNode {
                id: node_data.id,
                velocity: Vec3::ZERO,
                on_targets: node_data.on_targets,
            },
        ));

        if RENDER_NODES {
            entity.insert((
                Mesh3d(node_mesh.clone()),
                MeshMaterial3d(node_materials[node_data.on_targets].clone()),
            ));
        }
        
        node_positions.insert(node_data.id, position);
    }

    for edge in graph_data.edges.iter() {
        let from_id = edge.from;
        let to_id = edge.to;
        if let (Some(&from_pos), Some(&to_pos)) = (
            node_positions.get(&from_id),
            node_positions.get(&to_id),
        ) {
            let mut transform = Transform::default();
            set_edge_transform(&mut transform, from_pos, to_pos);

            let mut entity = commands.spawn((
                transform,
                GraphEdge {
                    from_id,
                    to_id,
                },
            ));

            if RENDER_EDGES && !USE_SHADER_EDGES {
                if let (Some(mesh_handle), Some(material_handle)) = (&edge_mesh_handle, &edge_material_handle) {
                    entity.insert((
                        Mesh3d(mesh_handle.clone()),
                        MeshMaterial3d(material_handle.clone()),
                    ));
                }
            }
        }
    }

    if USE_SHADER_EDGES {
        let node_ids: Vec<usize> = graph_data.nodes.iter().map(|n| n.id).collect();
        let id_to_index: HashMap<usize, usize> = node_ids.iter().enumerate()
            .map(|(idx, &id)| (id, idx))
            .collect();
        
        let vertices: Vec<Vec3> = node_ids.iter()
            .map(|&id| *node_positions.get(&id).unwrap_or(&Vec3::ZERO))
            .collect();
            
        let edges: Vec<[u32; 2]> = graph_data.edges.iter()
            .filter_map(|edge| {
                let from_idx = id_to_index.get(&edge.from)?;
                let to_idx = id_to_index.get(&edge.to)?;
                Some([*from_idx as u32, *to_idx as u32])
            })
            .collect();
        
        let mut edge_data = EdgeRenderData::new();
        edge_data.update_vertices(vertices);
        edge_data.update_edges(edges);
        commands.insert_resource(edge_data);
        commands.insert_resource(NodeIdToIndex::new(id_to_index));
    }
    
    commands.insert_resource(NodePositions { positions: node_positions });
    commands.insert_resource(ReusedMeshes { node_mesh });
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
            physics.octree_min_points_per_node,
        )
    };
    commands.insert_resource(octree_resource);
}



fn update_edges(
    mut edge_query: Query<(&mut Transform, &GraphEdge)>,
    node_positions: Res<NodePositions>,
    time: Res<Time>,
    user_config: Res<UserConfig>,
) {
    if user_config.is_rendering_disabled() || user_config.is_simulation_disabled(&time) || USE_SHADER_EDGES {
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

fn update_shader_edge_data(
    node_positions: Res<NodePositions>,
    node_id_to_index: Res<NodeIdToIndex>,
    mut edge_data: ResMut<EdgeRenderData>,
    time: Res<Time>,
    user_config: Res<UserConfig>,
) {
    if user_config.is_rendering_disabled() || user_config.is_simulation_disabled(&time) {
        return;
    }

    let vertices = node_id_to_index.get_indexed_vertex_positions(node_positions.as_ref());
    
    edge_data.update_vertices(vertices);
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
