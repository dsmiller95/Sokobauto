use bevy::prelude::*;
use crate::bevy_interface::{GraphNode, GraphVisualizationAssets};

#[derive(Component)]
pub struct SelectedNode;

#[derive(Resource)]
struct SelectionVisualizationAssets {
    selected_node_material: Handle<StandardMaterial>,
}

pub struct NodeSelectionPlugin;

impl Plugin for NodeSelectionPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Startup, setup_shared_meshes)
            .add_systems(Update, (set_selected_material_when_selected, set_unselected_material_when_unselected))
        ;
    }
}

fn setup_shared_meshes(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let selected_node_material = materials.add(StandardMaterial {
        base_color: bevy::color::palettes::basic::TEAL.into(),
        unlit: true,
        ..default()
    });

    commands.insert_resource(SelectionVisualizationAssets {
        selected_node_material,
    });
}

fn set_selected_material_when_selected(
    visualization_assets: Res<SelectionVisualizationAssets>,
    mut added_selection_query: Query<&mut MeshMaterial3d<StandardMaterial>, (Added<SelectedNode>, With<GraphNode>)>,
) {
    for mut mesh_material in added_selection_query.iter_mut() {
        mesh_material.0 = visualization_assets.selected_node_material.clone();
    }
}

fn set_unselected_material_when_unselected(
    external_visualization_assets: Res<GraphVisualizationAssets>,
    mut node_materials: Query<(&GraphNode, &mut MeshMaterial3d<StandardMaterial>)>,
    mut removed: RemovedComponents<SelectedNode>,
) {
    removed.read().for_each(|entity| {
        let Ok((node, mut material)) = node_materials.get_mut(entity) else {
            return;
        };

        let new_material = external_visualization_assets.node_materials[node.on_targets].clone();
        material.0 = new_material;
    })
}