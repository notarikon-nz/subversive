// src/systems/vehicles.rs
use bevy::prelude::*;
use crate::core::*;
use crate::systems::explosions::*;

use crate::systems::spawners::*;

pub fn vehicle_explosion_system(
    mut commands: Commands,
    mut vehicle_query: Query<(Entity, &Transform, &Vehicle, &Health), (With<Vehicle>, Added<Dead>, Without<MarkedForDespawn>)>,
    mut audio_events: EventWriter<AudioEvent>,
) {
    for (entity, transform, vehicle, health) in vehicle_query.iter_mut() {
        if health.0 <= 0.0 {
            let explosion_pos = transform.translation.truncate();

            spawn_explosion(
                &mut commands,
                explosion_pos,
                vehicle.explosion_radius(),
                vehicle.explosion_damage(),
                ExplosionType::Vehicle
            );
            audio_events.write(AudioEvent {
                sound: AudioType::Alert, // Reuse alert sound for explosion
                volume: 1.0,
            });

            commands.entity(entity).insert(MarkedForDespawn);
        }
    }
}

pub fn vehicle_cover_system(
    vehicle_query: Query<(&Transform, &Vehicle), With<Vehicle>>,
    agent_query: Query<&Transform, (With<Agent>, Without<Vehicle>)>,
    mut enemy_query: Query<(&mut GoapAgent, &Transform), (With<Enemy>, Without<Vehicle>)>,
) {
    for (mut goap_agent, enemy_transform) in enemy_query.iter_mut() {
        let enemy_pos = enemy_transform.translation.truncate();

        let near_vehicle_cover = vehicle_query.iter().any(|(vehicle_transform, vehicle)| {
            let distance = enemy_pos.distance(vehicle_transform.translation.truncate());
            distance <= vehicle.cover_value
        });

        goap_agent.update_world_state(WorldKey::CoverAvailable, near_vehicle_cover);
        goap_agent.update_world_state(WorldKey::InCover, near_vehicle_cover && enemy_pos.distance_squared(
            vehicle_query.iter().min_by(|(a_transform, _), (b_transform, _)| {
                let a_dist = enemy_pos.distance_squared(a_transform.translation.truncate());
                let b_dist = enemy_pos.distance_squared(b_transform.translation.truncate());
                a_dist.partial_cmp(&b_dist).unwrap_or(std::cmp::Ordering::Equal)
            }).map(|(t, _)| t.translation.truncate()).unwrap_or(Vec2::ZERO)
        ) <= 900.0); // 30 units squared
    }
}

pub fn vehicle_spawn_system(
    mut commands: Commands,
    vehicle_query: Query<Entity, With<Vehicle>>,
    sprites: Res<GameSprites>,
    mut spawn_timer: Local<f32>,
    time: Res<Time>,
    game_mode: Res<GameMode>,
) {
    if game_mode.paused { return; }

    *spawn_timer -= time.delta_secs();

    if *spawn_timer <= 0.0 && vehicle_query.iter().count() < 6 {
        let spawn_positions = [
            Vec2::new(-100.0, -200.0),
            Vec2::new(300.0, -200.0),
            Vec2::new(-200.0, 200.0),
            Vec2::new(400.0, 150.0),
            Vec2::new(100.0, 250.0),
        ];

        let pos = spawn_positions[rand::random::<usize>() % spawn_positions.len()];
        let vehicle_type = match rand::random::<f32>() {
            x if x < 0.6 => VehicleType::CivilianCar,
            x if x < 0.8 => VehicleType::PoliceCar,
            x if x < 0.95 => VehicleType::APC,
            _ => VehicleType::Tank,
        };

        spawn_vehicle(&mut commands, pos, vehicle_type, &sprites);
        *spawn_timer = 15.0 + rand::random::<f32>() * 10.0;
    }
}