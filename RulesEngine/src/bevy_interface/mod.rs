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
use std::collections::{HashMap, HashSet};
use std::hint::black_box;
use bevy::diagnostic::{FrameTimeDiagnosticsPlugin};
use bevy::input::keyboard::{Key};
use bevy::pbr::wireframe::{WireframePlugin};
use crate::bevy_interface::octree::{Octree, OctreeResource};
use crate::bevy_interface::config_ui::{setup_config_panel, handle_toggle_interactions, on_toggle_event, on_slider_event, ConfigChangedEvent, ConfigType, SliderType};
use crate::bevy_interface::fps_ui::{setup_fps_counter, update_fps_counter};
use crate::bevy_interface::octree_visualization::{setup_octree_visualization, update_octree_visualization, OctreeVisualizationConfig};
use crate::bevy_interface::edge_renderer::{EdgeRenderPlugin, EdgeRenderData, spawn_edge_mesh, EdgeRenderSystemSet};
use crate::bevy_interface::graph_compute::{apply_forces_and_update_octree, setup_compute_cache, AllEdgeIndexes, GraphComputeCache, GraphData, NodeIdToIndex};
use crate::bevy_interface::node_selection::{NodeSelectionPlugin, RecentlySelectedNode, SelectedNode};
use crate::bevy_interface::selected_game_navigation::{PlayingGameState, SelectedGameNavigationPlugin};
use crate::bevy_interface::tile_render::{TileRenderPlugin, TileType, Tiles};

const RENDER_NODES: bool = black_box(true);


#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
enum GraphNodeSpawnSystemSet {
    EntitiesSpawned,
    ComputeCacheSetup
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum StartupSystemSet {
    General,
    AfterGraphNodesSpawned,
    AfterGraphComputeCache,
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum UpdateSystemSet {
    General,
    MoveNodes,
    Display,
}

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
    initial_node_id: usize,
}

#[derive(PartialEq, Eq, Clone, Copy)]
enum PhysicsMode {
    Octree,
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
    fixed_timestep: Option<f32>,
    hide_never_selected: bool,
    max_viewed_games: f32,
    random_selects_per_second: f32,
    focus_selected: bool,
}

#[derive(Resource)]
struct GraphVisualizationAssets {
    node_mesh: Handle<Mesh>,
    node_materials: Vec<Handle<StandardMaterial>>,
}

impl UserConfig {
    fn get_timestep_secs(&self, time: &Time) -> f32 {
        match self.fixed_timestep {
            Some(fixed_timestep) => fixed_timestep,
            None => time.delta_secs(),
        }
    }
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
    fn get_total_to_select(&self, time: &Time) -> usize {
        (self.random_selects_per_second * time.delta_secs()).ceil() as usize
    }
}

pub fn visualize_graph(
    initial_node_id: usize,
    graph: &StateGraph,
    shared: &SharedGameState) {
    let source_data = SourceGraphData {
        graph: graph.clone(),
        shared: shared.clone(),
        initial_node_id
    };

    let graph_data = GraphData::from_state_graph(graph, shared);
    let user_config = UserConfig {
        force_simulation_enabled: false,
        disable_rendering: false,
        node_size_multiplier: 1.0,
        fixed_timestep: None,
        hide_never_selected: false,
        max_viewed_games: 4.,
        random_selects_per_second: 1000.0,
        focus_selected: true,
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
        .add_plugins(MeshPickingPlugin::default())
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
        .configure_sets(Startup, (
            StartupSystemSet::AfterGraphNodesSpawned
                .after(GraphNodeSpawnSystemSet::EntitiesSpawned),
            StartupSystemSet::AfterGraphComputeCache
                .after(GraphNodeSpawnSystemSet::ComputeCacheSetup),
        ))
        .configure_sets(Update, (
            UpdateSystemSet::Display
                .after(UpdateSystemSet::MoveNodes),
        ))
        .add_systems(Startup, (
            (
                setup_scene,
                setup_fps_counter,
                setup_config_panel,
                setup_octree_visualization,
                (
                    setup_shared_meshes,
                    setup_graph_from_data,
                ).chain()
                    .in_set(GraphNodeSpawnSystemSet::EntitiesSpawned),
            ).in_set(StartupSystemSet::General),
            (
                setup_octree_resource,
                setup_compute_cache
                    .in_set(GraphNodeSpawnSystemSet::ComputeCacheSetup),
            ).in_set(StartupSystemSet::AfterGraphNodesSpawned),
            select_initial_node
                .in_set(StartupSystemSet::AfterGraphComputeCache),
        ))
        .add_systems(Update, (
            (
                update_fps_counter,
                handle_toggle_interactions
            ).in_set(UpdateSystemSet::General),
            (
                on_b_pressed_select_random_adjacent_node,
                apply_forces_and_update_octree,
            ).in_set(UpdateSystemSet::MoveNodes),
            (
                update_octree_visualization,
                on_space_pressed_clear_recently_selected_nodes,
                on_c_pressed_clear_all_selected_nodes,
                select_nodes_with_playing_games,
                visualize_playing_games,
                focus_all_selected_nodes, // focus_newly_selected_nodes,
                display_only_recently_selected_nodes,
            ).in_set(UpdateSystemSet::Display)
        ))
        .add_observer(on_node_clicked_toggle_playing_game);

    app.add_systems(Startup, spawn_edge_mesh.after(setup_graph_from_data))
        .add_systems(Update, update_shader_edge_data.before(EdgeRenderSystemSet::UpdateEdgeMesh).after(UpdateSystemSet::Display));

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

fn display_only_recently_selected_nodes(
    user_config: Res<UserConfig>,
    mut node_selection_visibility_query: Query<(&mut Visibility, Option<&SelectedNode>, Option<&RecentlySelectedNode>), With<GraphNode>>,
) {
    for (mut visibility, selected, recently_selected) in node_selection_visibility_query.iter_mut() {
        let visible = if !user_config.hide_never_selected {
            Visibility::Visible
        } else if selected.is_some() || recently_selected.is_some() {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
        *visibility = visible;
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
                Visibility::Visible,
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
    edge_data.update_edges(edges.clone());
    commands.insert_resource(edge_data);
    commands.insert_resource(NodeIdToIndex::new(id_to_index));
    commands.insert_resource(AllEdgeIndexes::new(edges));

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
    node_id_to_index: Res<NodeIdToIndex>,
    all_edges: Res<AllEdgeIndexes>,
    mut edge_data: ResMut<EdgeRenderData>,
    user_config: Res<UserConfig>,
    visibility_query: Query<(&GraphNode, &Visibility, &Transform)>,
) {
    if user_config.is_rendering_disabled() {
        return;
    }

    let visible_indexes: HashSet<u32> = visibility_query.iter().filter_map(|(node, visibility, transform)| {
        if visibility == Visibility::Visible {
            node_id_to_index.get_index(&node.id).map(|&x| x as u32)
        } else {
            None
        }
    }).collect();

    let vertices = node_id_to_index.get_indexed_vertex_positions_from_iter_query(visibility_query.iter().filter_map(|(node, visibility, transform)| {
        if visibility == Visibility::Visible {
            Some((node, transform))
        } else {
            None
        }
    }));

    edge_data.update_vertices(vertices);

    let edges = all_edges.iter_edges()
        .filter(|edge| {
            visible_indexes.contains(&edge[0]) && visible_indexes.contains(&edge[1])
        })
        .copied()
        .collect::<Vec<_>>();

    edge_data.update_edges(edges);
}

fn visualize_playing_games(
    any_changed: Query<Entity, Changed<PlayingGameState>>,
    all_playing: Query<(&GraphNode, &PlayingGameState)>,
    source_graph_data: Res<SourceGraphData>,
    user_config: Res<UserConfig>,
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
        .take(user_config.max_viewed_games as usize)
        .collect::<Vec<_>>();

    let grid_size: IVec2 = source_graph_data.shared.size().into();

    let new_grids = all_games.into_iter()
        .map(|game_state| {
            let mut new_grid = vec![vec![TileType::Empty; grid_size.x as usize]; grid_size.y as usize];
            for x in 0..grid_size.x {
                for y in 0..grid_size.y {
                    let cell = source_graph_data.shared.grid[y as usize][x as usize];
                    let vec = IVec2 { x, y };
                    let is_player = game_state.player == vec.into();
                    let is_box = game_state.environment.has_box_at(&vec.into());
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

fn focus_newly_selected_nodes(
    selected_nodes: Query<&Transform, Added<SelectedNode>>,
    mut orbit_cameras: Query<&mut PanOrbitCamera>,
){
    if selected_nodes.is_empty() {
        return;
    }
    let location_sums = selected_nodes.iter()
        .map(|transform| {
            transform.translation
        })
        .fold((0, Vec3::splat(0.0)), |acc, elem| {
            (acc.0 + 1, acc.1 + elem)
        });
    let new_focus = location_sums.1 / (location_sums.0 as f32);
    for mut orbit_camera in orbit_cameras.iter_mut() {
        orbit_camera.target_focus = new_focus;
    }
}

fn focus_all_selected_nodes(
    selected_nodes: Query<&Transform, With<SelectedNode>>,
    user_config: Res<UserConfig>,
    mut orbit_cameras: Query<&mut PanOrbitCamera>,
){
    if !user_config.focus_selected || selected_nodes.is_empty() {
        return;
    }
    let location_sums = selected_nodes.iter()
        .map(|transform| {
            transform.translation
        })
        .fold((0, Vec3::splat(0.0)), |acc, elem| {
            (acc.0 + 1, acc.1 + elem)
        });
    let new_focus = location_sums.1 / (location_sums.0 as f32);
    for mut orbit_camera in orbit_cameras.iter_mut() {
        orbit_camera.target_focus = new_focus;
    }
}

fn select_nodes_with_playing_games(
    mut commands: Commands,
    to_select: Query<Entity, (With<PlayingGameState>, Without<SelectedNode>)>,
    to_unselect: Query<Entity, (Without<PlayingGameState>, With<SelectedNode>)>,
) {
    for entity in to_select.iter() {
        commands.entity(entity).insert(SelectedNode);
    }
    for entity in to_unselect.iter() {
        commands.entity(entity).remove::<SelectedNode>();
    }
}

fn on_c_pressed_clear_all_selected_nodes(
    mut commands: Commands,
    recently_selected_nodes: Query<(Entity), With<RecentlySelectedNode>>,
    selected_nodes: Query<(Entity), With<PlayingGameState>>,
    button_input: Res<ButtonInput<Key>>
) {
    if !button_input.just_pressed(Key::Character("c".into())) {
        return;
    }

    for entity in recently_selected_nodes.iter() {
        commands.entity(entity).remove::<RecentlySelectedNode>();
    }

    for entity in selected_nodes.iter() {
    commands.entity(entity).remove::<PlayingGameState>();
    }
}

fn on_space_pressed_clear_recently_selected_nodes(
    mut commands: Commands,
    recently_selected_nodes: Query<(Entity), With<RecentlySelectedNode>>,
    button_input: Res<ButtonInput<Key>>
) {
    if !button_input.just_pressed(Key::Space) {
        return;
    }

    for entity in recently_selected_nodes.iter() {
        commands.entity(entity).remove::<RecentlySelectedNode>();
    }
}

fn select_initial_node(
    mut commands: Commands,
    graph_data: Res<SourceGraphData>,
    entity_lookup: Res<GraphComputeCache>,
) {
    let initial_id = graph_data.initial_node_id;
    let Some(&node_entity) = entity_lookup.get_entity(&initial_id) else {
        eprintln!("Failed to find initial node entity");
        return;
    };

    select_node(&mut commands, &graph_data, &initial_id, node_entity)
}

fn on_space_pressed_start_playing_random_node(
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

fn on_node_clicked_toggle_playing_game(
    clicked: On<Pointer<Click>>,
    mut commands: Commands,
    graph_data: Res<SourceGraphData>,
    graph_nodes: Query<(&GraphNode, Option<&PlayingGameState>)>,
) {
    let clicked_entity = clicked.entity;
    let Ok((clicked_node, playing_game_state)) = graph_nodes.get(clicked_entity) else {
        return;
    };

    match playing_game_state {
        Some(_) => {
            println!("Deselecting {}", clicked_node.id);
            commands.entity(clicked_entity).remove::<PlayingGameState>();
        }
        None => {
            select_node(&mut commands, &graph_data, &clicked_node.id, clicked_entity);
        }
    }
}


fn on_b_pressed_select_random_adjacent_node(
    mut commands: Commands,
    past_selected_nodes: Query<(), Or<(With<RecentlySelectedNode>, With<SelectedNode>)>>,
    selected_nodes: Query<(Entity, &GraphNode, &Transform), With<SelectedNode>>,
    source_graph_data: Res<SourceGraphData>,
    graph_compute_cache: Res<GraphComputeCache>,
    mut node_positions: ResMut<NodePositions>,
    user_config: Res<UserConfig>,
    time: Res<Time>,
    button_input: Res<ButtonInput<Key>>
) {
    if !button_input.pressed(Key::Character("b".into())) {
        return;
    }

    if selected_nodes.is_empty() {
        return;
    }

    let total_to_select = user_config.get_total_to_select(&time);

    if total_to_select <= 0 {
        return;
    }

    use rand::seq::IteratorRandom;
    let mut rng = rand::rng();

    for (entity, node, transform) in selected_nodes.iter().choose_multiple(&mut rng, total_to_select) {
        let random_unselected_neighbor = graph_compute_cache.iterate_neighbors(&node.id)
            .filter_map(|&neighbor_id| {
                let &neighbor_entity = graph_compute_cache.get_entity(&neighbor_id).expect("every node must be in cache - neighbor");
                if past_selected_nodes.contains(neighbor_entity) {
                    None
                } else {
                    Some((neighbor_id, neighbor_entity))
                }
            })
            .choose(&mut rng);

        match random_unselected_neighbor {
            Some((to_select_id, to_select_entity)) => {
                select_node(&mut commands, &source_graph_data, &to_select_id, to_select_entity);

                // place the new node right next to where its neighbor is
                let jittered = *transform * Transform::from_translation(rng.random::<Vec3>() * 0.1);
                let new_pos = jittered.translation;
                node_positions.positions.insert(to_select_id, new_pos);
                commands.entity(to_select_entity).insert(jittered);
            }
            None => {
                // if no neighbors left to visit, stop "selecting"
                println!("Deselecting {}", node.id);
                commands.entity(entity).remove::<PlayingGameState>();
            }
        }
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


fn select_node(commands: &mut Commands, graph_data: &SourceGraphData, node_id: &usize, node_entity: Entity) {
    println!("Selecting {}", node_id);
    let unique_node = graph_data.graph.nodes.get_by_right(&node_id).expect("node id not found");
    commands.entity(node_entity).insert(PlayingGameState::new_playing_state(unique_node));
}