// src/systems/vehicles.rs
use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use crate::core::*;

pub fn vehicle_explosion_system(
    mut commands: Commands,
    mut vehicle_query: Query<(Entity, &Transform, &Vehicle, &Health), (With<Vehicle>, Added<Dead>)>,
    mut audio_events: EventWriter<AudioEvent>,
) {
    for (entity, transform, vehicle, health) in vehicle_query.iter_mut() {
        if health.0 <= 0.0 {
            let explosion_pos = transform.translation.truncate();
            
            commands.spawn((
                VehicleExplosion {
                    radius: vehicle.explosion_radius(),
                    damage: vehicle.explosion_damage(),
                    duration: 3.0,
                },
                Transform::from_translation(explosion_pos.extend(1.0)),
                Sprite {
                    color: Color::srgba(1.0, 0.5, 0.0, 0.8),
                    custom_size: Some(Vec2::splat(vehicle.explosion_radius() * 2.0)),
                    ..default()
                },
            ));
            
            audio_events.write(AudioEvent {
                sound: AudioType::Alert, // Reuse alert sound for explosion
                volume: 1.0,
            });
            
            commands.entity(entity).insert(MarkedForDespawn); // ← Safe mark
        }
    }
}

pub fn explosion_damage_system(
    mut explosion_query: Query<(Entity, &mut VehicleExplosion, &Transform)>,
    mut damageable_query: Query<(&Transform, &mut Health), (Without<VehicleExplosion>, Without<Dead>)>,
    mut commands: Commands,
    time: Res<Time>,
    game_mode: Res<GameMode>,
) {
    if game_mode.paused { return; }

    for (explosion_entity, mut explosion, explosion_transform) in explosion_query.iter_mut() {
        explosion.duration -= time.delta_secs();
        
        if explosion.duration <= 0.0 {

            commands.entity(explosion_entity).insert(MarkedForDespawn); // ← Safe mark
            continue;
        }
        
        let explosion_pos = explosion_transform.translation.truncate();
        
        for (target_transform, mut health) in damageable_query.iter_mut() {
            let distance = explosion_pos.distance(target_transform.translation.truncate());
            
            if distance <= explosion.radius {
                let damage_factor = 1.0 - (distance / explosion.radius);
                let damage = explosion.damage * damage_factor * time.delta_secs();
                health.0 -= damage;
            }
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

pub fn spawn_vehicle(
    commands: &mut Commands,
    position: Vec2,
    vehicle_type: VehicleType,
    sprites: &GameSprites,
) {
    let vehicle = Vehicle::new(vehicle_type.clone());
    let max_health = vehicle.max_health();
    
    let (color, size) = match vehicle_type {
        VehicleType::CivilianCar => (Color::srgb(0.6, 0.6, 0.8), Vec2::new(40.0, 20.0)),
        VehicleType::PoliceCar => (Color::srgb(0.2, 0.2, 0.8), Vec2::new(40.0, 20.0)),
        VehicleType::APC => (Color::srgb(0.4, 0.6, 0.4), Vec2::new(50.0, 30.0)),
        VehicleType::VTOL => (Color::srgb(0.3, 0.3, 0.3), Vec2::new(60.0, 40.0)),
        VehicleType::Tank => (Color::srgb(0.5, 0.5, 0.2), Vec2::new(60.0, 35.0)),
    };
    
    commands.spawn((
        Sprite {
            color,
            custom_size: Some(size),
            ..default()
        },
        Transform::from_translation(position.extend(0.8)),
        vehicle,
        Health(max_health),
        RigidBody::Fixed,
        Collider::cuboid(size.x / 2.0, size.y / 2.0),
    ));
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