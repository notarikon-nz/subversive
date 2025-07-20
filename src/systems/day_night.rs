// src/systems/day_night.rs
use bevy::prelude::*;
use crate::core::*;

#[derive(Component)]
pub struct DayNightOverlay;

pub fn day_night_system(
    mut day_night: ResMut<DayNightCycle>,
    time: Res<Time>,
    game_mode: Res<GameMode>,
) {
    if game_mode.paused { return; }
    
    day_night.advance_time(time.delta_secs());
}

pub fn lighting_system(
    mut commands: Commands,
    day_night: Res<DayNightCycle>,
    overlay_query: Query<Entity, With<DayNightOverlay>>,
    mut overlay_sprite_query: Query<&mut Sprite, With<DayNightOverlay>>,
) {
    // DISABLED FOR NOW - DO NOT REMOVE
    /*
    if day_night.is_changed() {
        // Create overlay if it doesn't exist
        if overlay_query.is_empty() {
            commands.spawn((
                Sprite {
                    color: day_night.get_overlay_color(),
                    custom_size: Some(Vec2::new(2560.0, 1440.0)), // Large enough to cover screen
                    ..default()
                },
                Transform::from_translation(Vec3::new(0.0, 0.0, 100.0)), // High Z to be on top
                DayNightOverlay,
            ));
        } else {
            // Update existing overlay
            for mut sprite in overlay_sprite_query.iter_mut() {
                sprite.color = day_night.get_overlay_color();
            }
        }
    }
    */
}

pub fn time_ui_system(
    mut gizmos: Gizmos,
    day_night: Res<DayNightCycle>,
) {
    let time_str = day_night.get_time_string();
    
    // Draw time indicator in top-right corner as colored circle
    let indicator_color = match day_night.current_period {
        TimeOfDay::Day => Color::srgb(1.0, 1.0, 0.0),      // Yellow sun
        TimeOfDay::Dusk => Color::srgb(1.0, 0.5, 0.0),     // Orange sunset
        TimeOfDay::Night => Color::srgb(0.2, 0.2, 0.6),    // Dark blue moon
        TimeOfDay::Dawn => Color::srgb(0.8, 0.6, 0.8),     // Purple dawn
    };
    
    // Draw time indicator at fixed screen position (adjust based on your camera)
    gizmos.circle_2d(Vec2::new(600.0, 300.0), 15.0, indicator_color);
    
    // Draw cycle progress indicator
    let cycle_progress = day_night.time_of_day / 24.0;
    let progress_angle = cycle_progress * std::f32::consts::TAU;
    let progress_pos = Vec2::new(600.0, 300.0) + Vec2::new(progress_angle.cos(), progress_angle.sin()) * 20.0;
    gizmos.circle_2d(progress_pos, 3.0, Color::WHITE);
}

