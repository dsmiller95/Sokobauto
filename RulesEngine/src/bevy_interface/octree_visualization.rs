use bevy::prelude::*;
use bevy::asset::{Assets, Handle};
use bevy::color::Color;
use bevy::mesh::{Mesh, Mesh3d};
use bevy::pbr::{MeshMaterial3d, StandardMaterial};
use bevy::pbr::wireframe::Wireframe;
use bevy::prelude::{default, AlphaMode, Bundle, Commands, Component, Cuboid, Entity, Mut, Query, Res, ResMut, Resource, Sphere, Time, Transform, With, Without};
use crate::bevy_interface::{GraphNode, PhysicsConfig, UserConfig};
use crate::bevy_interface::octree::{OctreeResource, OctreeVisualizationNode};


#[derive(Component)]
pub struct OctreeBounds {
    depth: usize,
}

#[derive(Component)]
pub struct OctreeCenterOfMass {
    depth: usize,
}

#[derive(Resource)]
pub struct OctreeVisualizationConfig {
    pub show_octree_bounds: bool,
    pub show_center_of_mass: bool,
    pub show_leaf_only: bool,
    pub max_depth_to_show: usize,
}

#[derive(Resource)]
pub struct OctreeVisualizationMeshes {
    bounds_mesh: Handle<Mesh>,
    center_of_mass_mesh: Handle<Mesh>,
    bounds_material: Handle<StandardMaterial>,
    center_of_mass_material: Handle<StandardMaterial>,
}

impl Default for OctreeVisualizationConfig {
    fn default() -> Self {
        Self {
            show_octree_bounds: false,
            show_center_of_mass: false,
            show_leaf_only: true,
            max_depth_to_show: 8,
        }
    }
}

pub fn setup_octree_visualization(
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

pub fn update_octree_visualization(
    mut commands: Commands,
    octree_resource: Res<OctreeResource>,
    mut bounds_query: Query<(Entity, &mut Transform, &OctreeBounds), (Without<GraphNode>, Without<OctreeCenterOfMass>)>,
    mut center_query: Query<(Entity, &mut Transform, &OctreeCenterOfMass), (Without<GraphNode>, Without<OctreeBounds>)>,
    physics: Res<PhysicsConfig>,
    user_config: Res<UserConfig>,
    visualization_config: Res<OctreeVisualizationConfig>,
    visualization_meshes: Res<OctreeVisualizationMeshes>,
    time: Res<Time>,
) {
    if user_config.is_octree_update_disabled(&time, &physics) {
        return;
    }

    let octree = &octree_resource.octree;
    let visualization_enabled = visualization_config.show_octree_bounds || visualization_config.show_center_of_mass;

    let filtered_visualization: Vec<OctreeVisualizationNode> = if visualization_enabled {
        let visualization_data = octree.get_visualization_data();
        visualization_data.into_iter()
            .filter(|node| {
                node.depth <= visualization_config.max_depth_to_show
            })
            .filter(|node| {
                !visualization_config.show_leaf_only || node.is_leaf
            })
            .collect()
    } else {
        Vec::new()
    };

    let empty: Vec<OctreeVisualizationNode> = Vec::new();

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
    new_data: &[OctreeVisualizationNode],
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