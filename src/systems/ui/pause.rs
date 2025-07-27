// src/systems/ui/screens.rs - All the screen UIs updated for Bevy 0.16
use bevy::prelude::*;
use crate::systems::ui::*;

#[derive(Component)]
pub struct PauseScreen;

// Pause system with mission abort
pub fn pause_system(
    mut commands: Commands,
    mut next_state: ResMut<NextState<GameState>>,
    mut post_mission: ResMut<PostMissionResults>,
    processed: ResMut<PostMissionProcessed>,
    game_mode: Res<GameMode>,
    input: Res<ButtonInput<KeyCode>>,
    screen_query: Query<Entity, With<PauseScreen>>,
    mission_data: Res<MissionData>,
) {
    let should_show = game_mode.paused;
    let ui_exists = !screen_query.is_empty();
    
    // Handle abort input when paused
    if should_show && input.just_pressed(KeyCode::KeyQ) {
        // Set mission as failed/aborted
        *post_mission = PostMissionResults {
            success: false,
            time_taken: mission_data.timer,
            enemies_killed: mission_data.enemies_killed,
            terminals_accessed: mission_data.terminals_accessed,
            credits_earned: 0, // No credits for abort
            alert_level: mission_data.alert_level,
        };
        
        // Clear pause UI
        for entity in screen_query.iter() {
            commands.entity(entity).insert(MarkedForDespawn);
        }
        
        // Go to post-mission to handle agent recovery properly
        next_state.set(GameState::PostMission);
        return;
    }
    
    if should_show && !ui_exists {
        commands.spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.5)),
            ZIndex(100),
            PauseScreen,
        )).with_children(|parent| {
            parent.spawn((
                Text::new("PAUSED"),
                TextFont { font_size: 32.0, ..default() },
                TextColor(Color::WHITE),
            ));
            
            parent.spawn((
                Text::new("\nSPACE: Resume Mission\nQ: Abort Mission"),
                TextFont { font_size: 20.0, ..default() },
                TextColor(Color::srgb(0.8, 0.8, 0.8)),
            ));
            
            parent.spawn((
                Text::new("\n(Aborting will count as mission failure)"),
                TextFont { font_size: 14.0, ..default() },
                TextColor(Color::srgb(0.8, 0.3, 0.3)),
            ));
        });
    } else if !should_show && ui_exists {
        for entity in screen_query.iter() {
            commands.entity(entity).insert(MarkedForDespawn);
        }
    }
}