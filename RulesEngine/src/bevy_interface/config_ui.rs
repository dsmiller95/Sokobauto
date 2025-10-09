use bevy::ecs::query::QueryEntityError;
use bevy::ecs::relationship::RelatedSpawnerCommands;
use bevy::math::FloatPow;
use bevy::prelude::*;
use super::{OctreeVisualizationConfig, UserConfig};

#[derive(Component)]
pub struct ConfigPanel;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToggleType {
    OctreeBounds,
    CenterOfMass,
    LeafOnly,
    ForceSimulate,
    DisableRendering,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SliderType {
    NodeSizeMultiplier,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigType {
    Toggle(ToggleType),
    Slider(SliderType),
}

#[derive(Component)]
pub struct ConfigToggle {
    pub toggle_type: ToggleType,
}

#[derive(Event)]
pub struct ToggleEvent {
    pub toggle_type: ToggleType,
}

#[derive(Component)]
pub struct ConfigSlider {
    pub slider_type: SliderType,
}

#[derive(Component)]
pub struct ConfigSliderHandle;

#[derive(Event)]
pub struct SliderEvent {
    pub slider_type: SliderType,
    pub new_value: f32,
}

#[derive(Event)]
pub struct ConfigChangedEvent {
    pub config_type: ConfigType,
}

impl ToggleType {
    pub fn all_types() -> &'static [ToggleType] {
        &[
            ToggleType::OctreeBounds,
            ToggleType::CenterOfMass,
            ToggleType::LeafOnly,
            ToggleType::ForceSimulate,
            ToggleType::DisableRendering,
        ]
    }
    
    pub fn label(&self) -> &'static str {
        match self {
            ToggleType::OctreeBounds => "Show Octree Bounds",
            ToggleType::CenterOfMass => "Show Center of Mass",
            ToggleType::LeafOnly => "Show Leaf Only",
            ToggleType::ForceSimulate => "Force Simulation On",
            ToggleType::DisableRendering => "Disable Rendering",
        }
    }

    pub fn get_value(&self, config: &OctreeVisualizationConfig, user_config: &UserConfig) -> bool {
        match self {
            ToggleType::OctreeBounds => config.show_octree_bounds,
            ToggleType::CenterOfMass => config.show_center_of_mass,
            ToggleType::LeafOnly => config.show_leaf_only,
            ToggleType::ForceSimulate => user_config.force_simulation_enabled,
            ToggleType::DisableRendering => user_config.disable_rendering,
        }
    }

    pub fn set_value(&self, config: &mut OctreeVisualizationConfig, user_config: &mut UserConfig, value: bool) {
        match self {
            ToggleType::OctreeBounds => config.show_octree_bounds = value,
            ToggleType::CenterOfMass => config.show_center_of_mass = value,
            ToggleType::LeafOnly => config.show_leaf_only = value,
            ToggleType::ForceSimulate => user_config.force_simulation_enabled = value,
            ToggleType::DisableRendering => user_config.disable_rendering = value,
        }
    }

    pub fn toggle_value(&self, config: &mut OctreeVisualizationConfig, user_config: &mut UserConfig) {
        let current = self.get_value(config, user_config);
        self.set_value(config, user_config, !current);
    }
}

impl SliderType {
    pub fn all_types() -> &'static [SliderType] {
        &[
            SliderType::NodeSizeMultiplier,
        ]
    }

    pub fn label(&self) -> &'static str {
        match self {
            SliderType::NodeSizeMultiplier => "Node Size",
        }
    }

    pub fn get_value(&self, config: &OctreeVisualizationConfig, user_config: &UserConfig) -> f32 {
        match self {
            SliderType::NodeSizeMultiplier => user_config.node_size_multiplier.sqrt(),
        }
    }

    pub fn set_value(&self, config: &mut OctreeVisualizationConfig, user_config: &mut UserConfig, value: f32) {
        match self {
            SliderType::NodeSizeMultiplier => user_config.node_size_multiplier = value.squared(),
        }
    }

    pub fn range(&self) -> (f32, f32) {
        match self {
            SliderType::NodeSizeMultiplier => (0.1, 5.0),
        }
    }
}

trait NumRange {
    fn normalized(&self, value_in_range: f32) -> f32;
    fn denormalized(&self, normalized_value: f32) -> f32;
    fn clamp(&self, value: f32) -> f32;
    fn size(&self) -> f32;
}

impl NumRange for (f32, f32) {
    fn normalized(&self, value_in_range: f32) -> f32 {
        let (min, max) = *self;
        (value_in_range - min) / (max - min)
    }

    fn denormalized(&self, normalized_value: f32) -> f32 {
        let (min, max) = *self;
        (normalized_value * (max - min)) + min
    }

    fn clamp(&self, value: f32) -> f32 {
        let (min, max) = *self;
        value.clamp(min, max)
    }

    fn size(&self) -> f32 {
        let (min, max) = *self;
        max - min
    }
}

const NORMAL_BUTTON: Color = Color::srgb(0.15, 0.15, 0.15);
const PRESSED_BUTTON: Color = Color::srgb(0.35, 0.75, 0.35);
const ACTIVE_BUTTON: Color = Color::srgb(0.2, 0.6, 0.2);

pub fn setup_config_panel(mut commands: Commands, visualization_config: Res<OctreeVisualizationConfig>, user_config: Res<UserConfig>) {
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

            for &toggle_type in ToggleType::all_types() {
                let initial_state = toggle_type.get_value(&visualization_config, &user_config);
                create_toggle_row(parent, initial_state, toggle_type);
            }

            for &slider_type in SliderType::all_types() {
                let initial_value = slider_type.get_value(&visualization_config, &user_config);
                create_slider_row(parent, initial_value, slider_type);
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

fn create_slider_row(
    parent: &mut RelatedSpawnerCommands<ChildOf>,
    initial_value: f32,
    slider_type: SliderType,
) {
    let (min, max) = slider_type.range();
    parent
        .spawn(Node {
            width: Val::Percent(100.0),
            height: Val::Px(40.0),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::SpaceAround,
            row_gap: Val::Px(4.0),
            ..default()
        })
        .with_children(|parent| {
            parent.spawn(Node {
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::SpaceBetween,
                column_gap: Val::Px(2.0),
                ..default()
            }).with_children(|parent| {
                parent.spawn((
                    Text::new(format!("{:.2}", min)),
                    TextFont {
                        font_size: 14.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                    BackgroundColor(Color::srgb(0.3, 0.5, 0.3)),
                    BorderRadius::all(Val::Px(5.0)),
                ));
                parent.spawn((
                    Text::new(format!("{}", slider_type.label())),
                    TextFont {
                        font_size: 14.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                ));
                parent.spawn((
                    Text::new(format!("{:.2}", max)),
                    TextFont {
                        font_size: 14.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                    BackgroundColor(Color::srgb(0.3, 0.5, 0.3)),
                    BorderRadius::all(Val::Px(5.0)),
                ));
            });

            parent.spawn((
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Px(10.0),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.3, 0.3, 0.3)),
                BorderColor::all(Color::srgb(0.5, 0.5, 0.5)),
                BorderRadius::all(Val::Px(5.0)),
                ConfigSlider { slider_type },
            ))
            .with_children(|parent| {
                let position = ((initial_value - min) / (max - min)).clamp(0.0, 1.0);
                parent.spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        left: Val::Percent(position * 100.0),
                        top: Val::Px(-5.0),
                        width: Val::Px(20.0),
                        height: Val::Px(20.0),
                        ..default()
                    },
                    BackgroundColor(Color::WHITE),
                    BorderColor::all(Color::srgb(0.5, 0.5, 0.5)),
                    BorderRadius::all(Val::Px(10.0)),
                    ConfigSliderHandle,
                ));
            }).observe(|
                drag: On<Pointer<Drag>>,
                mut slider_query: Query<(&ComputedNode, &Children, &ConfigSlider)>,
                mut node_query: Query<&mut Node, With<ConfigSliderHandle>>,
                mut commands: Commands,
                config: ResMut<OctreeVisualizationConfig>,
                user_config: ResMut<UserConfig>,
            | {
                try_entity_errors(|| {
                    let (computed_node, children, config_slider) = slider_query.get_mut(drag.entity)?;
                    let range = config_slider.slider_type.range();

                    let normalized_value_change = (drag.delta.x / (computed_node.size.x));

                    let current_value = config_slider.slider_type.get_value(&config, &user_config);
                    let change_in_value = range.size() * normalized_value_change;
                    let new_value = range.clamp(current_value + change_in_value);

                    let new_position = Val::Percent(range.normalized(new_value).clamp(0.0, 1.0) * 100.0);
                    for child in children.iter() {
                        if let Ok(mut node) = node_query.get_mut(child) {
                            node.left = new_position;
                        }
                    }

                    commands.trigger(SliderEvent {
                        slider_type: config_slider.slider_type,
                        new_value,
                    });

                    Ok(())
                });
            });
        });
}

pub fn try_entity_errors<T>(mut fun: impl FnMut() -> Result<T, QueryEntityError>) -> Option<T> {
    match fun() {
        Ok(val) => Some(val),
        Err(e) => {
            eprintln!("Entity query error: {:?}", e);
            None
        },
    }
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
    user_config: Res<UserConfig>,
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

                let is_active = toggle.toggle_type.get_value(&config, &user_config);
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
    mut commands: Commands,
    mut config: ResMut<OctreeVisualizationConfig>,
    mut user_config: ResMut<UserConfig>,
) {
    let toggle_type = trigger.event().toggle_type;
    toggle_type.toggle_value(&mut config, &mut user_config);

    commands.trigger(ConfigChangedEvent {
        config_type: ConfigType::Toggle(toggle_type),
    });

    println!("Toggled {:?}: {}", toggle_type, toggle_type.get_value(&config, &user_config));
}

pub fn on_slider_event(
    trigger: On<SliderEvent>,
    mut commands: Commands,
    mut config: ResMut<OctreeVisualizationConfig>,
    mut user_config: ResMut<UserConfig>,
) {
    let slider_type = trigger.event().slider_type;
    let value = trigger.event().new_value;
    slider_type.set_value(&mut config, &mut user_config, value);

    commands.trigger(ConfigChangedEvent {
        config_type: ConfigType::Slider(slider_type),
    });

    println!("Set {:?} to {}", slider_type, slider_type.get_value(&config, &user_config));
}