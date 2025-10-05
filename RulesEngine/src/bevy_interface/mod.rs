use bevy::prelude::*;
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};
use crate::state_graph::StateGraph;
use crate::core::SharedGameState;
use rand::Rng;
use std::collections::HashMap;

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

#[derive(Resource)]
struct PhysicsConfig {
    repulsion_strength: f32,
    attraction_strength: f32,
    damping: f32,
    max_velocity: f32,
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
        .insert_resource(graph_data)
        .insert_resource(PhysicsConfig {
            repulsion_strength: 50.0,
            attraction_strength: 2.0,
            damping: 0.95,
            max_velocity: 10.0,
        })
        .add_systems(Startup, (setup_scene, setup_graph_from_data).chain())
        .add_systems(Update, (apply_forces, update_edges))
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

fn setup_scene(
    mut commands: Commands,
) {
    // Add lighting
    commands.spawn((
        DirectionalLight {
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // Add ambient light
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 0.3,
        affects_lightmapped_meshes: true,
    });

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
    
    // Create nodes
    for node_data in &graph_data.nodes {
        let position = Vec3::new(
            rng.gen_range(-15.0..15.0),
            rng.gen_range(-15.0..15.0),
            rng.gen_range(-15.0..15.0),
        );
        
        let color = interpolate_color(node_data.on_targets, graph_data.max_on_targets);
        
        commands.spawn((
            // TODO: share mesh and materials?
            Mesh3d(meshes.add(Sphere::new(0.8).mesh().ico(4).unwrap())),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: color,
                metallic: 0.3,
                perceptual_roughness: 0.4,
                ..default()
            })),
            Transform::from_translation(position),
            GraphNode {
                id: node_data.id,
                velocity: Vec3::ZERO,
                on_targets: node_data.on_targets,
            },
        ));
        
        node_positions.insert(node_data.id, position);
    }
    
    // Create edges
    for edge in graph_data.edges.iter() {
        let from_id = edge.from;
        let to_id = edge.to;
        if let (Some(&from_pos), Some(&to_pos)) = (
            node_positions.get(&from_id),
            node_positions.get(&to_id),
        ) {
            let direction = (to_pos - from_pos).normalize();
            let distance = from_pos.distance(to_pos);
            let center = (from_pos + to_pos) / 2.0;
            
            commands.spawn((
                Mesh3d(meshes.add(Capsule3d::new(0.1, 1.0))),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: Color::srgb(0.7, 0.7, 0.7),
                    metallic: 0.1,
                    perceptual_roughness: 0.8,
                    ..default()
                })),
                Transform::from_translation(center)
                    .align(Dir3::NEG_X, direction, Dir3::Y, Vec3::Y),
                GraphEdge {
                    from_id,
                    to_id,
                },
            ));
        }
    }
    
    commands.insert_resource(NodePositions { positions: node_positions });
}

fn apply_forces(
    mut node_query: Query<(&mut Transform, &mut GraphNode)>,
    edge_query: Query<&GraphEdge>,
    mut node_positions: ResMut<NodePositions>,
    physics: Res<PhysicsConfig>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();
    
    // Collect all node data for iteration
    let mut nodes_data: Vec<(usize, Vec3, Vec3)> = Vec::new();
    for (transform, node) in node_query.iter() {
        nodes_data.push((node.id, transform.translation, node.velocity));
    }
    
    // Collect all edges for attraction forces
    let edges: Vec<_> = edge_query.iter().collect();
    
    for (mut transform, mut node) in node_query.iter_mut() {
        let mut force = Vec3::ZERO;
        let current_pos = transform.translation;
        
        // Repulsion forces from all other nodes
        for &(other_id, other_pos, _) in &nodes_data {
            if node.id != other_id {
                let diff = current_pos - other_pos;
                let distance = diff.length().max(0.1);
                let repulsion = diff.normalize() * physics.repulsion_strength / (distance * distance);
                force += repulsion;
            }
        }
        
        // Attraction forces along edges
        for edge in &edges {
            let mut connected_node_pos = None;
            
            if edge.from_id == node.id {
                // Find the "to" node position
                for &(other_id, other_pos, _) in &nodes_data {
                    if other_id == edge.to_id {
                        connected_node_pos = Some(other_pos);
                        break;
                    }
                }
            } else if edge.to_id == node.id {
                // Find the "from" node position
                for &(other_id, other_pos, _) in &nodes_data {
                    if other_id == edge.from_id {
                        connected_node_pos = Some(other_pos);
                        break;
                    }
                }
            }
            
            if let Some(connected_pos) = connected_node_pos {
                let diff = connected_pos - current_pos;
                let distance = diff.length().max(0.1);
                let attraction = diff.normalize() * physics.attraction_strength * distance;
                force += attraction;
            }
        }
        
        // Apply damping
        node.velocity *= physics.damping;
        
        // Apply force
        node.velocity += force * dt;
        
        // Limit velocity
        if node.velocity.length() > physics.max_velocity {
            node.velocity = node.velocity.normalize() * physics.max_velocity;
        }
        
        // Update position
        transform.translation += node.velocity * dt;
        
        // Update node positions resource
        node_positions.positions.insert(node.id, transform.translation);
    }
}

fn update_edges(
    mut edge_query: Query<(&mut Transform, &GraphEdge)>,
    node_positions: Res<NodePositions>,
) {
    for (mut transform, edge) in edge_query.iter_mut() {
        if let (Some(&from_pos), Some(&to_pos)) = (
            node_positions.positions.get(&edge.from_id),
            node_positions.positions.get(&edge.to_id),
        ) {
            let direction = (to_pos - from_pos).normalize();
            let distance = from_pos.distance(to_pos);
            let center = (from_pos + to_pos) / 2.0;
            
            transform.translation = center;
            transform
                .align(Dir3::Y, direction, Dir3::Z, Vec3::Z);
            transform.scale.y = distance;
        }
    }
}

fn interpolate_color(on_targets: usize, max_on_targets: usize) -> Color {
    let t = if max_on_targets == 0 {
        0.0
    } else {
        on_targets as f32 / max_on_targets as f32
    };
    
    // Interpolate from red (1,0,0) to blue (0,0,1)
    Color::srgb(1.0 - t, 0.0, t)
}