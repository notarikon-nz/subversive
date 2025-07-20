// src/systems/day_night.rs
use bevy::prelude::*;
use bevy_light_2d::prelude::*;
use crate::core::*;

pub fn day_night_system(
    mut day_night: ResMut<DayNightCycle>,
    time: Res<Time>,
    game_mode: Res<GameMode>,
    mut gizmos: Gizmos,
) {
    if game_mode.paused { return; }
    
    day_night.advance_time(time.delta_secs());
}

pub fn lighting_system(
    day_night: Res<DayNightCycle>,
    mut ambient_light: ResMut<AmbientLight2d>,
) {
    if day_night.is_changed() {
        ambient_light.color = day_night.get_ambient_light();
        ambient_light.brightness = match day_night.current_period {
            TimeOfDay::Day => 1.0,
            TimeOfDay::Dusk => 0.7,
            TimeOfDay::Night => 0.4,
            TimeOfDay::Dawn => 0.6,
        };
    }
}

pub fn time_ui_system(
    mut gizmos: Gizmos,
    day_night: Res<DayNightCycle>,
) {
    let time_str = day_night.get_time_string();
    let period_str = format!("{:?}", day_night.current_period);
    
    // Draw time indicator in top-right corner (gizmos text would require custom implementation)
    // For now, just draw a colored circle indicating time of day
    let indicator_color = match day_night.current_period {
        TimeOfDay::Day => Color::srgb(1.0, 1.0, 0.0),      // Yellow sun
        TimeOfDay::Dusk => Color::srgb(1.0, 0.5, 0.0),     // Orange sunset
        TimeOfDay::Night => Color::srgb(0.2, 0.2, 0.6),    // Dark blue moon
        TimeOfDay::Dawn => Color::srgb(0.8, 0.6, 0.8),     // Purple dawn
    };
    
    // Draw time indicator at fixed screen position
    gizmos.circle_2d(Vec2::new(600.0, 300.0), 15.0, indicator_color);
    
    // Draw smaller indicator for cycle progress
    let cycle_progress = day_night.time_of_day / 24.0;
    let progress_angle = cycle_progress * std::f32::consts::TAU;
    let progress_pos = Vec2::new(600.0, 300.0) + Vec2::new(progress_angle.cos(), progress_angle.sin()) * 20.0;
    gizmos.circle_2d(progress_pos, 3.0, Color::WHITE);
}