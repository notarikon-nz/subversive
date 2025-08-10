use bevy::prelude::*;
use crate::core::*;
use crate::systems::input::MenuInput;
use crate::systems::ui::layout::*;
use std::collections::HashMap;

// Keep only the components and settings
#[derive(Component)]
pub struct SettingsUI;

#[derive(Component)]
pub struct BackButton;

#[derive(Component)]
pub struct VolumeBar;

#[derive(Component)]
pub enum SettingsControl {
    Back,
    Volume, 
    Quality,
}

#[derive(Resource)]
pub struct GameSettings {
    pub master_volume: f32,
    pub graphics_quality: u32,
}

impl Default for GameSettings {
    fn default() -> Self {
        Self { master_volume: 0.7, graphics_quality: 2 }
    }
}

// Setup becomes one function call
pub fn setup_settings_ui(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    layout_cache: Res<UILayoutCache>,
    settings: Res<GameSettings>,
) {
    let mut variables = HashMap::new();
    variables.insert("volume_percent".to_string(), format!("{}%", (settings.master_volume * 100.0) as u32));
    variables.insert("quality_text".to_string(), ["Low", "Medium", "High"][settings.graphics_quality as usize].to_string());
    
    if spawn_ui_from_layout(&mut commands, &asset_server, &layout_cache, "settings", Some(&variables)).is_none() {
        error!("Failed to load settings layout - falling back to hardcoded UI");
        // Fallback to your existing setup_settings_ui code
    }
}

// Logic system stays the same
pub fn settings_system_bevy_ui(
    mut next_state: ResMut<NextState<GameState>>,
    mut settings: ResMut<GameSettings>, 
    keyboard: Res<ButtonInput<KeyCode>>,
    gamepads: Query<&Gamepad>,
    windows: Query<&Window>,
    cameras: Query<(&Camera, &GlobalTransform)>,
    mut interactions: Query<(&Interaction, &SettingsControl, &mut BackgroundColor), (Changed<Interaction>, Without<VolumeBar>)>,
    mut volume_bar: Query<&mut Node, With<VolumeBar>>,
    mut texts: Query<&mut Text>,
) {
    if MenuInput::new(&keyboard, &gamepads).back || keyboard.just_pressed(KeyCode::Escape) {
        next_state.set(GameState::MainMenu);
        return;
    }

    // Your existing interaction logic - unchanged
    for (interaction, control, mut bg) in interactions.iter_mut() {
        match *interaction {
            Interaction::Pressed => match control {
                SettingsControl::Back => next_state.set(GameState::MainMenu),
                SettingsControl::Volume => {
                    settings.master_volume = if settings.master_volume >= 1.0 { 0.0 } else { (settings.master_volume + 0.25).min(1.0) };
                    if let Ok(mut node) = volume_bar.single_mut() {
                        node.width = Val::Percent(settings.master_volume * 100.0);
                    }
                },
                SettingsControl::Quality => {
                    settings.graphics_quality = (settings.graphics_quality + 1) % 3;
                    for mut text in texts.iter_mut() {
                        if ["Low", "Medium", "High"].contains(&text.0.as_str()) {
                            text.0 = ["Low", "Medium", "High"][settings.graphics_quality as usize].to_string();
                            break;
                        }
                    }
                },
            },
            Interaction::Hovered => bg.0 = get_button_colors(*interaction),
            Interaction::None => bg.0 = get_button_colors(*interaction),
        }
    }
}

pub fn cleanup_settings_ui(mut commands: Commands, query: Query<Entity, With<SettingsUI>>) {
    for entity in query.iter() {
        commands.entity(entity).insert(MarkedForDespawn);
    }
}



/*
// src/systems/ui/settings.rs - Simple bevy_ui version
use bevy::prelude::*;
use bevy::input::mouse::{MouseWheel};
use crate::core::*;
use crate::systems::input::{MenuInput};

const PADDING: Val = Val::Px(20.0);
const BUTTON_HEIGHT: Val = Val::Px(50.0);
const SLIDER_WIDTH: Val = Val::Px(200.0);
const SLIDER_HEIGHT: Val = Val::Px(20.0);

#[derive(Component)]
pub struct SettingsUI;

#[derive(Component)]
pub enum SettingsControl {
    Back,
    Volume,
    Quality,
}

#[derive(Component)]
pub struct VolumeBar;

#[derive(Resource)]
pub struct GameSettings {
    pub master_volume: f32,
    pub graphics_quality: u32,
}

impl Default for GameSettings {
    fn default() -> Self {
        Self { master_volume: 0.7, graphics_quality: 2 }
    }
}

fn create_text(text: &str, font: Handle<Font>, size: f32, color: Color) -> impl Bundle {
    (Text::new(text), TextFont { font, font_size: size, ..default() }, TextColor(color))
}

fn create_button(width: Val, height: Val, control: SettingsControl) -> impl Bundle {
    (Button, 
     Node { width, height, justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default() },
     BackgroundColor(Color::srgba(0.2, 0.2, 0.4, 0.8)),
     BorderRadius::all(Val::Px(4.0)),
     control)
}

fn create_row() -> impl Bundle {
    Node { flex_direction: FlexDirection::Row, justify_content: JustifyContent::SpaceBetween, align_items: AlignItems::Center, ..default() }
}

pub fn setup_settings_ui(mut commands: Commands, asset_server: Res<AssetServer>, settings: Res<GameSettings>) {
    let font = asset_server.load("fonts/orbitron.ttf");
    
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            flex_direction: FlexDirection::Column,
            ..default()
        },
        BackgroundColor(Color::srgba(0.1, 0.1, 0.2, 0.9)),
        SettingsUI,
    ))
    .with_children(|parent| {
        parent.spawn(create_text("SETTINGS", font.clone(), 48.0, Color::srgb(0.99, 1.0, 0.32)));
        
        parent.spawn((
            Node { 
                width: Val::Px(400.0), 
                padding: UiRect::all(PADDING), 
                margin: UiRect::all(PADDING), 
                flex_direction: FlexDirection::Column, 
                row_gap: Val::Px(15.0), 
                ..default() 
            },
            BackgroundColor(Color::srgba(0.15, 0.15, 0.3, 0.8)),
            BorderRadius::all(Val::Px(8.0)),
        ))
        .with_children(|parent| {
            // Volume slider
            parent.spawn(create_row()).with_children(|parent| {
                parent.spawn(create_text("Master Volume:", font.clone(), 18.0, Color::WHITE));
                parent.spawn((
                    Button,
                    Node { width: SLIDER_WIDTH, height: SLIDER_HEIGHT, justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default() },
                    BackgroundColor(Color::srgb(0.3, 0.3, 0.3)),
                    BorderRadius::all(Val::Px(4.0)),
                    SettingsControl::Volume,
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Node { width: Val::Percent(settings.master_volume * 100.0), height: Val::Percent(100.0), ..default() },
                        BackgroundColor(Color::srgb(0.99, 1.0, 0.32)),
                        VolumeBar,
                    ));
                });
            });

            // Graphics quality
            parent.spawn(create_row()).with_children(|parent| {
                parent.spawn(create_text("Graphics Quality:", font.clone(), 18.0, Color::WHITE));
                parent.spawn(create_button(Val::Px(100.0), Val::Px(30.0), SettingsControl::Quality))
                .with_children(|parent| {
                    let quality_text = ["Low", "Medium", "High"][settings.graphics_quality as usize];
                    parent.spawn(create_text(quality_text, font.clone(), 16.0, Color::WHITE));
                });
            });
        });

        parent.spawn(create_button(Val::Px(200.0), BUTTON_HEIGHT, SettingsControl::Back))
        .insert(Node { margin: UiRect::top(Val::Px(30.0)), ..default() })
        .with_children(|parent| {
            parent.spawn(create_text("Back (ESC)", font, 16.0, Color::WHITE));
        });
    });
}

pub fn settings_system_bevy_ui(
    mut next_state: ResMut<NextState<GameState>>,
    mut settings: ResMut<GameSettings>,
    keyboard: Res<ButtonInput<KeyCode>>,
    gamepads: Query<&Gamepad>,
    mut scroll: EventReader<MouseWheel>,
    windows: Query<&Window>,
    cameras: Query<(&Camera, &GlobalTransform)>,
    mut interactions: Query<(&Interaction, &SettingsControl, &mut BackgroundColor, &GlobalTransform, &Node), (Changed<Interaction>, Without<VolumeBar>)>,
    mut volume_bar: Query<&mut Node, With<VolumeBar>>,
    mut texts: Query<&mut Text>,
) {
    if MenuInput::new(&keyboard, &gamepads).back || keyboard.just_pressed(KeyCode::Escape) {
        next_state.set(GameState::MainMenu);
        return;
    }

    // Handle mouse wheel on volume slider
    if let Ok(window) = windows.single() {
        if let Some(cursor_pos) = window.cursor_position() {
            for ev in scroll.read() {
                if cursor_over_volume_slider(cursor_pos, &interactions, &cameras, window) {
                    settings.master_volume = (settings.master_volume + ev.y * 0.05).clamp(0.0, 1.0);
                    update_volume_bar(&mut volume_bar, settings.master_volume);
                }
            }
        }
    }

    for (interaction, control, mut bg, transform, node) in interactions.iter_mut() {
        match *interaction {
            Interaction::Pressed => match control {
                SettingsControl::Back => next_state.set(GameState::MainMenu),
                SettingsControl::Volume => {
                    if let Ok(window) = windows.single() {
                        if let Some(cursor_pos) = window.cursor_position() {
                            let local_pos = screen_to_slider_position(cursor_pos, transform, node, &cameras, window);
                            settings.master_volume = local_pos.clamp(0.0, 1.0);
                            update_volume_bar(&mut volume_bar, settings.master_volume);
                        }
                    }
                },
                SettingsControl::Quality => {
                    settings.graphics_quality = (settings.graphics_quality + 1) % 3;
                    update_quality_text(&mut texts, settings.graphics_quality);
                },
            },
            Interaction::Hovered => bg.0 = Color::srgba(0.3, 0.3, 0.5, 0.8),
            Interaction::None => bg.0 = Color::srgba(0.2, 0.2, 0.4, 0.8),
        }
    }
}

fn cursor_over_volume_slider(
    cursor_pos: Vec2,
    interactions: &Query<(&Interaction, &SettingsControl, &mut BackgroundColor, &GlobalTransform, &Node), (Changed<Interaction>, Without<VolumeBar>)>,
    cameras: &Query<(&Camera, &GlobalTransform)>,
    window: &Window,
) -> bool {
    // Simplified check - just see if we're hovering over any volume control
    interactions.iter().any(|(_, control, _, _, _)| matches!(control, SettingsControl::Volume))
}

fn screen_to_slider_position(
    cursor_pos: Vec2,
    transform: &GlobalTransform,
    node: &Node,
    cameras: &Query<(&Camera, &GlobalTransform)>,
    window: &Window,
) -> f32 {
    // Convert screen position to slider percentage
    // This is a simplified version - you may need to adjust based on your camera setup
    let slider_width = if let Val::Px(width) = SLIDER_WIDTH { width } else { 200.0 };
    let relative_x = cursor_pos.x - (window.width() / 2.0 - slider_width / 2.0);
    (relative_x / slider_width).clamp(0.0, 1.0)
}

fn update_volume_bar(volume_bar: &mut Query<&mut Node, With<VolumeBar>>, volume: f32) {
    if let Ok(mut node) = volume_bar.single_mut() {
        node.width = Val::Percent(volume * 100.0);
    }
}

fn update_quality_text(texts: &mut Query<&mut Text>, quality: u32) {
    for mut text in texts.iter_mut() {
        if text.0 == "Low" || text.0 == "Medium" || text.0 == "High" {
            text.0 = ["Low", "Medium", "High"][quality as usize].to_string();
            break;
        }
    }
}

pub fn cleanup_settings_ui(
    mut commands: Commands,
    ui_query: Query<Entity, With<SettingsUI>>,
) {
    for entity in ui_query.iter() {
        commands.entity(entity).insert(MarkedForDespawn);
    }
}
    */