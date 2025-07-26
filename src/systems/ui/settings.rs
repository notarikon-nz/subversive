use bevy::prelude::*;
use crate::core::GameState;
use crate::core::*;

#[derive(Component)]
pub struct SettingsUI;

pub fn setup_settings(mut commands: Commands) {
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            flex_direction: FlexDirection::Column,
            ..default()
        },
        BackgroundColor(Color::srgb(0.05, 0.05, 0.1)),
        SettingsUI,
    )).with_children(|parent| {
        parent.spawn((
            Text::new("Settings"),
            TextFont { font_size: 32.0, ..default() },
            TextColor(Color::WHITE),
        ));
        
        parent.spawn((
            Text::new("Press ESC to return"),
            TextFont { font_size: 16.0, ..default() },
            TextColor(Color::srgb(0.7, 0.7, 0.7)),
        ));
    });
}

pub fn settings_input(
    input: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if input.just_pressed(KeyCode::Escape) {
        next_state.set(GameState::MainMenu);
    }
}

pub fn cleanup_settings(
    mut commands: Commands,
    settings_ui: Query<Entity, (With<SettingsUI>, Without<MarkedForDespawn>)>,
) {
    for entity in settings_ui.iter() {
        commands.entity(entity).insert(MarkedForDespawn);
    }
}