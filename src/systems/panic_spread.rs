// src/systems/panic_spread.rs
use bevy::prelude::*;
use crate::core::*;

#[derive(Component)]
pub struct PanicSpreader {
    pub spread_radius: f32,
    pub spread_cooldown: f32,
}

impl Default for PanicSpreader {
    fn default() -> Self {
        Self {
            spread_radius: 60.0,
            spread_cooldown: 0.0,
        }
    }
}

pub fn panic_spread_system(
    mut civilian_query: Query<(Entity, &Transform, &mut Morale, &mut PanicSpreader), With<Civilian>>,
    nearby_query: Query<(Entity, &Transform), (With<Civilian>, Without<PanicSpreader>)>,
    mut commands: Commands,
    time: Res<Time>,
    game_mode: Res<GameMode>,
) {
    if game_mode.paused { return; }

    for (spreader_entity, spreader_transform, spreader_morale, mut spreader) in civilian_query.iter_mut() {
        if spreader.spread_cooldown > 0.0 {
            spreader.spread_cooldown -= time.delta_secs();
            continue;
        }

        if spreader_morale.is_panicked() {
            let spreader_pos = spreader_transform.translation.truncate();
            
            for (nearby_entity, nearby_transform) in nearby_query.iter() {
                let distance = spreader_pos.distance(nearby_transform.translation.truncate());
                
                if distance <= spreader.spread_radius {
                    commands.entity(nearby_entity).insert(PanicSpreader::default());
                    spreader.spread_cooldown = 2.0;
                    break;
                }
            }
        }
    }
}

pub fn panic_morale_reduction_system(
    mut civilian_query: Query<(Entity, &Transform, &mut Morale), (With<Civilian>, With<PanicSpreader>)>,
    time: Res<Time>,
    game_mode: Res<GameMode>,
) {
    if game_mode.paused { return; }

    for (_, _, mut morale) in civilian_query.iter_mut() {
        morale.reduce(30.0 * time.delta_secs());
    }
}