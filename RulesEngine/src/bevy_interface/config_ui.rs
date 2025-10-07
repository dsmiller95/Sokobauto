use bevy::ecs::relationship::RelatedSpawnerCommands;
use bevy::prelude::*;
use super::OctreeVisualizationConfig;

#[derive(Component)]
pub struct ConfigPanel;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToggleType {
    OctreeBounds,
    CenterOfMass,
    LeafOnly,
}

impl ToggleType {
    pub fn label(&self) -> &'static str {
        match self {
            ToggleType::OctreeBounds => "Show Octree Bounds",
            ToggleType::CenterOfMass => "Show Center of Mass",
            ToggleType::LeafOnly => "Show Leaf Only",
        }
    }

    pub fn get_value(&self, config: &OctreeVisualizationConfig) -> bool {
        match self {
            ToggleType::OctreeBounds => config.show_octree_bounds,
            ToggleType::CenterOfMass => config.show_center_of_mass,
            ToggleType::LeafOnly => config.show_leaf_only,
        }
    }

    pub fn set_value(&self, config: &mut OctreeVisualizationConfig, value: bool) {
        match self {
            ToggleType::OctreeBounds => config.show_octree_bounds = value,
            ToggleType::CenterOfMass => config.show_center_of_mass = value,
            ToggleType::LeafOnly => config.show_leaf_only = value,
        }
    }

    pub fn toggle_value(&self, config: &mut OctreeVisualizationConfig) {
        let current = self.get_value(config);
        self.set_value(config, !current);
    }
}

#[derive(Component)]
pub struct ConfigToggle {
    pub toggle_type: ToggleType,
}

#[derive(Event)]
pub struct ToggleEvent {
    pub toggle_type: ToggleType,
}

const NORMAL_BUTTON: Color = Color::srgb(0.15, 0.15, 0.15);
const PRESSED_BUTTON: Color = Color::srgb(0.35, 0.75, 0.35);
const ACTIVE_BUTTON: Color = Color::srgb(0.2, 0.6, 0.2);

pub fn setup_config_panel(mut commands: Commands, visualization_config: Res<OctreeVisualizationConfig>) {
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

            for toggle_type in [ToggleType::OctreeBounds, ToggleType::CenterOfMass, ToggleType::LeafOnly] {
                let initial_state = toggle_type.get_value(&visualization_config);
                create_toggle_row(parent, initial_state, toggle_type);
            }
        });
}

fn create_toggle_row(
    parent: &mut RelatedSpawnerCommands<ChildOf>,
    initial_state: bool,
    toggle_type: ToggleType,
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
                    ConfigToggle { toggle_type },
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
                Text::new(toggle_type.label()),
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
            &ConfigToggle,
            &Children,
        ),
        (Changed<Interaction>, With<Button>),
    >,
    mut text_query: Query<&mut Text>,
    mut commands: Commands,
    config: Res<OctreeVisualizationConfig>,
) {
    for (interaction, mut color, mut border_color, toggle, children) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                *color = PRESSED_BUTTON.into();
                *border_color = BorderColor::all(Color::WHITE);

                commands.trigger(ToggleEvent {
                    toggle_type: toggle.toggle_type,
                });
            }
            Interaction::Hovered => {
                *border_color = BorderColor::all(Color::WHITE);
            }
            Interaction::None => {
                *border_color = BorderColor::all(Color::srgb(0.5, 0.5, 0.5));

                let is_active = toggle.toggle_type.get_value(&config);
                *color = if is_active { ACTIVE_BUTTON } else { NORMAL_BUTTON }.into();

                for child in children.iter() {
                    if let Ok(mut text) = text_query.get_mut(child) {
                        text.0 = if is_active { "✓".to_string() } else { "".to_string() };
                    }
                }
            }
        }
    }
}

pub fn on_toggle_event(
    trigger: On<ToggleEvent>,
    mut config: ResMut<OctreeVisualizationConfig>,
) {
    let toggle_type = trigger.event().toggle_type;
    toggle_type.toggle_value(&mut config);
    println!("Toggled {:?}: {}", toggle_type, toggle_type.get_value(&config));
}