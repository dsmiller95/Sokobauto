use bevy::mesh::PrimitiveTopology;
use bevy::prelude::*;

use super::resources::EdgeRenderData;

pub struct EdgeRenderPlugin;

impl Plugin for EdgeRenderPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<EdgeRenderData>()
            .add_systems(Update, update_edge_mesh);
    }
}

#[derive(Component)]
pub struct EdgeMeshEntity;

pub fn spawn_edge_mesh(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    edge_data: Res<EdgeRenderData>,
) {
    if edge_data.edges.is_empty() {
        return;
    }

    let mesh = create_edge_mesh(&edge_data);
    let mesh_handle = meshes.add(mesh);
    let material_handle = materials.add(StandardMaterial {
        base_color: Color::srgb(0.7, 0.7, 0.7),
        unlit: true,
        ..default()
    });

    commands.spawn((
        Mesh3d(mesh_handle),
        MeshMaterial3d(material_handle),
        EdgeMeshEntity,
    ));
}

fn update_edge_mesh(
    mut edge_data: ResMut<EdgeRenderData>,
    mut meshes: ResMut<Assets<Mesh>>,
    query: Query<&Mesh3d, With<EdgeMeshEntity>>,
) {
    if !edge_data.dirty || edge_data.edges.is_empty() {
        return;
    }

    for mesh_handle in query.iter() {
        if let Some(mesh) = meshes.get_mut(&mesh_handle.0) {
            *mesh = create_edge_mesh(&edge_data);
        }
    }
    
    edge_data.clear_dirty();
}

fn create_edge_mesh(edge_data: &EdgeRenderData) -> Mesh {
    use bevy::asset::RenderAssetUsages;

    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    // Add all vertices
    for vertex in &edge_data.vertices {
        vertices.push([vertex.x, vertex.y, vertex.z]);
    }

    // Add indices for edges
    for edge in &edge_data.edges {
        indices.push(edge[0]);
        indices.push(edge[1]);
    }

    let mut mesh = Mesh::new(
        PrimitiveTopology::LineList,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    );
    
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
    mesh.insert_indices(bevy::mesh::Indices::U32(indices));
    
    mesh
}