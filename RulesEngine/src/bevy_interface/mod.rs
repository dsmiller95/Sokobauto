mod spatial_hash;
pub mod octree;
mod config_ui;
pub mod bounds;
mod fps_ui;
mod octree_visualization;
mod edge_renderer;
mod graph_compute;
mod node_selection;
mod tile_render;
mod selected_game_navigation;

use bevy::prelude::*;
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};
use crate::state_graph::StateGraph;
use crate::core::{Cell, SharedGameState};
use rand::Rng;
use std::collections::{HashMap};
use std::hint::black_box;
use bevy::diagnostic::{FrameTimeDiagnosticsPlugin};
use bevy::input::keyboard::{Key};
use bevy::pbr::wireframe::{WireframePlugin};
use crate::bevy_interface::octree::{Octree, OctreeResource};
use crate::bevy_interface::config_ui::{setup_config_panel, handle_toggle_interactions, on_toggle_event, on_slider_event, ConfigChangedEvent, ConfigType, SliderType};
use crate::bevy_interface::fps_ui::{setup_fps_counter, update_fps_counter};
use crate::bevy_interface::octree_visualization::{setup_octree_visualization, update_octree_visualization, OctreeVisualizationConfig};
use crate::bevy_interface::edge_renderer::{EdgeRenderPlugin, EdgeRenderData, spawn_edge_mesh};
use crate::bevy_interface::graph_compute::{apply_forces_and_update_octree, setup_compute_cache, GraphData, NodeIdToIndex};
use crate::bevy_interface::node_selection::{NodeSelectionPlugin, SelectedNode};
use crate::bevy_interface::selected_game_navigation::{PlayingGameState, SelectedGameNavigationPlugin};
use crate::bevy_interface::tile_render::{TileRenderPlugin, TileType, Tiles};

const RENDER_NODES: bool = black_box(true);

#[derive(Component)]
struct GraphNode {
    id: usize,
    velocity: Vec3,
    on_targets: usize,
}

#[derive(Resource)]
struct NodePositions {
    positions: HashMap<usize, Vec3>,
}

#[derive(Resource)]
struct SourceGraphData {
    graph: StateGraph,
    shared: SharedGameState,
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
struct GraphVisualizationAssets {
    node_mesh: Handle<Mesh>,
    node_materials: Vec<Handle<StandardMaterial>>,
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
    let source_data = SourceGraphData {
        graph: graph.clone(),
        shared: shared.clone(),
    };

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
        .insert_resource(source_data)
        .insert_resource(user_config)
        .insert_resource(graph_data)
        .insert_resource(OctreeVisualizationConfig::default());

    app
        .add_plugins((
            EdgeRenderPlugin,
            NodeSelectionPlugin,
            TileRenderPlugin,
            SelectedGameNavigationPlugin));

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
        .add_systems(Startup, (setup_scene, setup_shared_meshes, setup_graph_from_data, setup_octree_resource, setup_compute_cache, setup_fps_counter, setup_octree_visualization, setup_config_panel).chain())
        .add_systems(Update, (
            apply_forces_and_update_octree, update_octree_visualization,
            update_fps_counter,
            handle_toggle_interactions,
            start_playing_random_node_when_space_pressed, // select_random_adjacent_node_when_space_bar_pressed,
            select_nodes_with_playing_games,
            visualize_playing_games,
            focus_newly_selected_node,
        ));

    app.add_systems(Startup, spawn_edge_mesh.after(setup_graph_from_data))
        .add_systems(Update, update_shader_edge_data);

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
        Camera {
            order: 0,
            ..default()
        }
    ));
    commands.spawn((
        Camera2d::default(),
        Transform::default(),
        Camera {
            order: 1,
            ..default()
        }
    ));
}

const DEFAULT_NODE_SPHERE_SIZE: f32 = 0.8;

fn on_config_changed(
    trigger: On<ConfigChangedEvent>,
    mut _commands: Commands,
    shared_meshes: Res<GraphVisualizationAssets>,
    mut meshes: ResMut<Assets<Mesh>>,
    _config: Res<OctreeVisualizationConfig>,
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

fn setup_shared_meshes(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    graph_data: Res<GraphData>,
) {
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

    commands.insert_resource(GraphVisualizationAssets {
        node_mesh,
        node_materials,
    });
}

fn setup_graph_from_data(
    mut commands: Commands,
    graph_assets: Res<GraphVisualizationAssets>,
    graph_data: Res<GraphData>,
) {
    let mut rng = rand::rng();
    let mut node_positions = HashMap::new();

    // let mut arrow_mesh = Cone::new(0.15, 0.2).mesh()
    //     .anchor(ConeAnchor::Tip).resolution(4).build();
    // arrow_mesh.translate_by(Vec3::new(0.0, 0.3, 0.0));
    // let mut edge_mesh = Mesh::from(Capsule3d::new(0.03, 1.0).mesh()
    //     .latitudes(4).longitudes(4).build());
    // edge_mesh.merge(&arrow_mesh).unwrap();

    for node_data in &graph_data.nodes {
        let position = Vec3::new(
            rng.random_range(-15.0..15.0),
            rng.random_range(-15.0..15.0),
            rng.random_range(-15.0..15.0),
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
                Mesh3d(graph_assets.node_mesh.clone()),
                MeshMaterial3d(graph_assets.node_materials[node_data.on_targets].clone()),
            ));
        }

        node_positions.insert(node_data.id, position);
    }

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
            physics.octree_min_points_per_node,
        )
    };
    commands.insert_resource(octree_resource);
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

fn visualize_playing_games(
    any_changed: Query<Entity, Changed<PlayingGameState>>,
    all_playing: Query<(&GraphNode, &PlayingGameState)>,
    source_graph_data: Res<SourceGraphData>,
    mut tiles: ResMut<Tiles>,
){
    if any_changed.is_empty() {
        return;
    }

    // TODO: ideally, we would not iterate -every- node -every- time -any- game playing changed.
    let all_games = all_playing.iter()
        .filter_map(|(graph_node, playing_game_state)| {
            let Some(selected_game_state) = source_graph_data.graph.nodes.get_by_right(&graph_node.id) else {
                eprintln!("Node not found {}", graph_node.id);
                return None;
            };
            let game_state = playing_game_state.apply_to_node(selected_game_state.clone());
            Some(game_state)
        })
        .collect::<Vec<_>>();

    let grid_size = IVec2::new(source_graph_data.shared.width(), source_graph_data.shared.height());

    let new_grids = all_games.into_iter()
        .map(|game_state| {
            let mut new_grid = vec![vec![TileType::Empty; grid_size.x as usize]; grid_size.y as usize];
            for x in 0..grid_size.x {
                for y in 0..grid_size.y {
                    let cell = source_graph_data.shared.grid[y as usize][x as usize];
                    let vec = IVec2 { x, y };
                    let is_player = game_state.player == vec.into();
                    let is_box = game_state.environment.boxes.contains(&vec.into());
                    let tile = match cell {
                        Cell::Wall => TileType::Wall,
                        Cell::Floor =>
                            if is_player { TileType::Player }
                            else if is_box { TileType::Box }
                            else { TileType::Floor } ,
                        Cell::Target =>
                            if is_player { TileType::Player }
                            else if is_box { TileType::Box }
                            else { TileType::Target },
                    };

                    new_grid[(grid_size.y - y - 1) as usize][x as usize] = tile;
                }
            }

            new_grid
        })
        .collect::<Vec<_>>();

    println!("{} total grids", new_grids.len());

    tiles.assign_new_grids(new_grids)
}

fn focus_newly_selected_node(
    selected_nodes: Query<&Transform, Added<SelectedNode>>,
    mut orbit_cameras: Query<&mut PanOrbitCamera>,
){
    let Ok(selected_transform) = selected_nodes.single() else {
        return;
    };

    let new_focus = selected_transform.translation;

    for mut orbit_camera in orbit_cameras.iter_mut() {
        orbit_camera.target_focus = new_focus;
    }
}

fn select_nodes_with_playing_games(
    mut commands: Commands,
    to_select: Query<Entity, (With<PlayingGameState>, Without<SelectedNode>)>,
    to_unselect: Query<Entity, (Without<PlayingGameState>, With<SelectedNode>)>,
) {
    for entity in to_select.iter() {        commands.entity(entity).insert(SelectedNode);
    }
    for entity in to_unselect.iter() {
        commands.entity(entity).remove::<SelectedNode>();
    }
}

fn start_playing_random_node_when_space_pressed(
    mut commands: Commands,
    unplaying_nodes: Query<(Entity, &GraphNode), Without<PlayingGameState>>,
    graph_data: Res<SourceGraphData>,
    button_input: Res<ButtonInput<Key>>
) {
    if !button_input.just_pressed(Key::Space) {
        return;
    }

    use rand::seq::IteratorRandom;
    let mut rng = rand::rng();

    let (to_play, graph_node) = unplaying_nodes.iter().choose(&mut rng).expect("all nodes already being played?");
    let unique_node = graph_data.graph.nodes.get_by_right(&graph_node.id).expect("node id not found");
    commands.entity(to_play).insert(PlayingGameState::new_playing_state(unique_node));
}

fn select_random_adjacent_node_when_space_bar_pressed(
    mut commands: Commands,
    unselected_nodes: Query<(Entity, &GraphNode), Without<SelectedNode>>,
    selected_nodes: Query<(Entity, &GraphNode), With<SelectedNode>>,
    graph_data: Res<GraphData>,
    button_input: Res<ButtonInput<Key>>
) {
    if !button_input.just_pressed(Key::Space) {
        return;
    }

    use rand::seq::IteratorRandom;
    let mut rng = rand::rng();

    let last_selected_nodes = selected_nodes.iter().map(|(_, node)| node.id).collect::<Vec<_>>();

    let mut picked_entity = None;
    if last_selected_nodes.len() > 0 {
        let possible_next_nodes = graph_data.edges.iter()
            .filter(|edge| last_selected_nodes.contains(&edge.from))
            .map(|edge| edge.to)
            .collect::<Vec<_>>();
        if possible_next_nodes.len() > 0 {
            picked_entity = unselected_nodes.iter()
                .filter(|(_, node)| possible_next_nodes.contains(&node.id))
                .map(|(entity, _)| entity)
                .choose(&mut rng)
        }
    }

    let picked_entity = match picked_entity {
        Some(picked_entity) => Some(picked_entity),
        None => {
            // if there are no other ways to move through the graph, then, clear the entire selection and pick a new random node
            for (selected_entity, _) in selected_nodes.iter() {
                commands.entity(selected_entity).remove::<SelectedNode>();
            }

            unselected_nodes.iter().choose(&mut rng).map(|x| x.0)
        }
    };

    if let Some(picked_entity) = picked_entity {
        commands.entity(picked_entity).insert(SelectedNode);
    }
}

fn interpolate_color(on_targets: usize, max_on_targets: usize) -> Color {
    let t = if max_on_targets == 0 {
        0.0
    } else {
        on_targets as f32 / max_on_targets as f32
    };
    
    Color::srgb(1.0 - t, 0.0, t)
}
