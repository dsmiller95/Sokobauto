use bevy::ecs::relationship::RelatedSpawnerCommands;
use bevy::prelude::*;
use super::OctreeVisualizationConfig;

#[derive(Component)]
pub struct ConfigPanel;

#[derive(Component)]
pub struct OctreeBoundsToggle;

#[derive(Component)]
pub struct CenterOfMassToggle;

#[derive(Component)]
pub struct LeafOnlyToggle;

#[derive(Event)]
pub struct ToggleOctreeBounds;

#[derive(Event)]
pub struct ToggleCenterOfMass;

#[derive(Event)]
pub struct ToggleLeafOnly;

const NORMAL_BUTTON: Color = Color::srgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::srgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::srgb(0.35, 0.75, 0.35);
const ACTIVE_BUTTON: Color = Color::srgb(0.2, 0.6, 0.2);

pub fn setup_config_panel(mut commands: Commands) {
    // Root UI container
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(10.0),
                right: Val::Px(10.0),
                width: Val::Px(250.0),
                height: Val::Auto,
                padding: UiRect::all(Val::Px(15.0)),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(8.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.8)),
            BorderRadius::all(Val::Px(8.0)),
            ConfigPanel,
        ))
        .with_children(|parent| {
            // Panel title
            parent.spawn((
                Text::new("Octree Visualization"),
                TextFont {
                    font_size: 18.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                Node {
                    margin: UiRect::bottom(Val::Px(10.0)),
                    ..default()
                },
            ));

            create_toggle_row(
                parent,
                "Show Octree Bounds",
                true,
                OctreeBoundsToggle,
            );

            create_toggle_row(
                parent,
                "Show Center of Mass",
                true,
                CenterOfMassToggle,
            );

            create_toggle_row(
                parent,
                "Show Leaf Only",
                true,
                LeafOnlyToggle,
            );
        });
}

fn create_toggle_row<T: Component>(
    parent: &mut RelatedSpawnerCommands<ChildOf>,
    label: &str,
    initial_state: bool,
    toggle_component: T,
) {
    parent
        .spawn(Node {
            width: Val::Percent(100.0),
            height: Val::Px(30.0),
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            column_gap: Val::Px(10.0),
            ..default()
        })
        .with_children(|parent| {
            // Toggle button (checkbox-style)
            parent
                .spawn((
                    Button,
                    Node {
                        width: Val::Px(20.0),
                        height: Val::Px(20.0),
                        border: UiRect::all(Val::Px(2.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(if initial_state { ACTIVE_BUTTON } else { NORMAL_BUTTON }),
                    BorderColor::all(Color::srgb(0.5, 0.5, 0.5)),
                    BorderRadius::all(Val::Px(3.0)),
                    toggle_component,
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Text::new(if initial_state {"✓"} else {""}),
                        TextFont {
                            font_size: 14.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));
                });

            parent.spawn((
                Text::new(label),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                Node {
                    flex_grow: 1.0,
                    ..default()
                },
            ));
        });
}

pub fn handle_toggle_interactions(
    mut interaction_query: Query<
        (
            &Interaction,
            &mut BackgroundColor,
            &mut BorderColor,
            Option<&OctreeBoundsToggle>,
            Option<&CenterOfMassToggle>,
            Option<&LeafOnlyToggle>,
            &Children,
        ),
        (Changed<Interaction>, With<Button>),
    >,
    mut text_query: Query<&mut Text>,
    mut commands: Commands,
    config: Res<OctreeVisualizationConfig>,
) {
    for (interaction, mut color, mut border_color, bounds_toggle, center_toggle, leaf_toggle, children) in
        interaction_query.iter_mut()
    {
        match *interaction {
            Interaction::Pressed => {
                *color = PRESSED_BUTTON.into();
                *border_color = BorderColor::all(Color::WHITE);

                // Trigger appropriate event
                if bounds_toggle.is_some() {
                    commands.trigger(ToggleOctreeBounds);
                } else if center_toggle.is_some() {
                    commands.trigger(ToggleCenterOfMass);
                } else if leaf_toggle.is_some() {
                    commands.trigger(ToggleLeafOnly);
                }
            }
            Interaction::Hovered => {
                *border_color = BorderColor::all(Color::WHITE);
            }
            Interaction::None => {
                *border_color = BorderColor::all(Color::srgb(0.5, 0.5, 0.5));
                
                let is_active = if bounds_toggle.is_some() {
                    config.show_octree_bounds
                } else if center_toggle.is_some() {
                    config.show_center_of_mass
                } else if leaf_toggle.is_some() {
                    config.show_leaf_only
                } else {
                    false
                };

                *color = if is_active { ACTIVE_BUTTON } else { NORMAL_BUTTON }.into();

                // Update checkmark visibility
                for child in children.iter() {
                    if let Ok(mut text) = text_query.get_mut(child) {
                        text.0 = if is_active { "✓".to_string() } else { "".to_string() };
                    }
                }
            }
        }
    }
}

pub fn on_toggle_octree_bounds(
    _trigger: On<ToggleOctreeBounds>,
    mut config: ResMut<OctreeVisualizationConfig>,
) {
    config.show_octree_bounds = !config.show_octree_bounds;
    println!("Toggled octree bounds: {}", config.show_octree_bounds);
}

pub fn on_toggle_center_of_mass(
    _trigger: On<ToggleCenterOfMass>,
    mut config: ResMut<OctreeVisualizationConfig>,
) {
    config.show_center_of_mass = !config.show_center_of_mass;
    println!("Toggled center of mass: {}", config.show_center_of_mass);
}

pub fn on_toggle_leaf_only(
    _trigger: On<ToggleLeafOnly>,
    mut config: ResMut<OctreeVisualizationConfig>,
) {
    config.show_leaf_only = !config.show_leaf_only;
    println!("Toggled leaf only: {}", config.show_leaf_only);
}