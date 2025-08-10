// src/systems/ui/credits.rs - Pure bevy_ui version
use bevy::prelude::*;
use crate::core::*;
use crate::systems::input::MenuInput;

const PADDING: Val = Val::Px(20.0);
const SPACING_LARGE: Val = Val::Px(50.0);
const SPACING_MEDIUM: Val = Val::Px(30.0);
const SPACING_SMALL: Val = Val::Px(10.0);

#[derive(Component)]
pub struct CreditsUI;

#[derive(Component)]
pub struct BackButton;

fn create_text(text: &str, font: Handle<Font>, size: f32, color: Color) -> impl Bundle {
    (
        Text::new(text),
        TextFont { font, font_size: size, ..default() },
        TextColor(color),
    )
}

fn create_title_text(text: &str, font: Handle<Font>) -> impl Bundle {
    create_text(text, font, 32.0, Color::srgb(0.99, 1.0, 0.32)) // Cyberpunk yellow
}

fn create_subtitle_text(text: &str, font: Handle<Font>) -> impl Bundle {
    create_text(text, font, 24.0, Color::srgb(0.0, 1.0, 1.0)) // Cyan
}

fn create_header_text(text: &str, font: Handle<Font>) -> impl Bundle {
    create_text(text, font, 16.0, Color::srgb(1.0, 0.0, 0.59)) // Hot pink
}

fn create_body_text(text: &str, font: Handle<Font>) -> impl Bundle {
    create_text(text, font, 14.0, Color::WHITE)
}

pub fn setup_credits_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
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
        CreditsUI,
    ))
    .with_children(|parent| {
        parent.spawn((create_title_text("CREDITS", font.clone()), Node { margin: UiRect::bottom(SPACING_LARGE), ..default() }));
        
        // Credits content panel
        parent.spawn((
            Node {
                width: Val::Px(500.0),
                padding: UiRect::all(PADDING),
                margin: UiRect::all(PADDING),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: SPACING_SMALL,
                ..default()
            },
            BackgroundColor(Color::srgba(0.15, 0.15, 0.3, 0.8)),
            BorderRadius::all(Val::Px(8.0)),
        ))
        .with_children(|parent| {
            // Game title
            parent.spawn((create_subtitle_text("SUBVERSIVE", font.clone()), Node { margin: UiRect::bottom(SPACING_MEDIUM), ..default() }));
            
            // Copyright
            parent.spawn(create_body_text("Copyright (c) 2005 Matt Orsborn. All rights reserved.", font.clone()));
            parent.spawn((create_body_text("Developed with Bevy Engine, Rapier2D and eGUI", font.clone()), Node { margin: UiRect::bottom(SPACING_MEDIUM), ..default() }));
            
            // Special thanks header
            parent.spawn(create_header_text("Special Thanks:", font.clone()));
            
            // Thanks list
            parent.spawn(create_body_text("• Bevy Community", font.clone()));
            parent.spawn(create_body_text("• egui Contributors", font.clone()));
        });

        // Back button
        parent.spawn((
            Button,
            Node {
                width: Val::Px(200.0),
                height: Val::Px(50.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                margin: UiRect::top(SPACING_LARGE),
                ..default()
            },
            BackgroundColor(Color::srgba(0.2, 0.2, 0.4, 0.8)),
            BorderRadius::all(Val::Px(4.0)),
            BackButton,
        ))
        .with_children(|parent| {
            parent.spawn(create_text("Back to Menu (ESC)", font, 16.0, Color::WHITE));
        });
    });
}

pub fn credits_system_bevy_ui(
    mut next_state: ResMut<NextState<GameState>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    gamepads: Query<&Gamepad>,
    mut button_query: Query<(&Interaction, &mut BackgroundColor), (Changed<Interaction>, With<BackButton>)>,
) {
    let input = MenuInput::new(&keyboard, &gamepads);

    // Handle back navigation
    if input.back || keyboard.just_pressed(KeyCode::Escape) {
        next_state.set(GameState::MainMenu);
        return;
    }

    // Handle button interaction
    for (interaction, mut background) in button_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                next_state.set(GameState::MainMenu);
            },
            Interaction::Hovered => {
                background.0 = Color::srgba(0.3, 0.3, 0.5, 0.8);
            },
            Interaction::None => {
                background.0 = Color::srgba(0.2, 0.2, 0.4, 0.8);
            },
        }
    }
}

pub fn cleanup_credits_ui(mut commands: Commands, query: Query<Entity, With<CreditsUI>>) {
    for entity in query.iter() {
        commands.entity(entity).insert(MarkedForDespawn);
    }
}