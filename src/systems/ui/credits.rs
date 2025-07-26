use bevy::prelude::*;
use crate::core::*;

#[derive(Component)]
pub struct CreditsUI;

pub fn setup_credits(mut commands: Commands) {
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(20.0),
            ..default()
        },
        BackgroundColor(Color::srgb(0.05, 0.05, 0.1)),
        CreditsUI,
    )).with_children(|parent| {
        parent.spawn((
            Text::new("Credits"),
            TextFont { font_size: 32.0, ..default() },
            TextColor(Color::WHITE),
        ));
        
        parent.spawn((
            Text::new("A game by Your Studio"),
            TextFont { font_size: 20.0, ..default() },
            TextColor(Color::srgb(0.7, 0.7, 0.7)),
        ));
        
        parent.spawn((
            Text::new("Press ESC to return"),
            TextFont { font_size: 16.0, ..default() },
            TextColor(Color::srgb(0.5, 0.5, 0.5)),
        ));
    });
}

pub fn credits_input(
    input: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if input.just_pressed(KeyCode::Escape) {
        next_state.set(GameState::MainMenu);
    }
}

pub fn cleanup_credits(
    mut commands: Commands,
    credits_ui: Query<Entity, (With<CreditsUI>, Without<MarkedForDespawn>)>,
) {
    for entity in credits_ui.iter() {
        commands.entity(entity).insert(MarkedForDespawn);
    }
}