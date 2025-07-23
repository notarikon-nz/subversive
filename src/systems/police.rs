// src/systems/police.rs
use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use crate::core::*;
use crate::systems::*;

pub fn police_tracking_system(
    mut police_response: ResMut<PoliceResponse>,
    mut combat_events: EventReader<CombatEvent>,
    mut audio_events: EventReader<AudioEvent>,
    civilian_query: Query<&Transform, With<Civilian>>,
    enemy_query: Query<&Transform, With<Enemy>>,
    time: Res<Time>,
    game_mode: Res<GameMode>,
) {
    if game_mode.paused { return; }

    police_response.next_spawn_timer -= time.delta_secs();
    police_response.heat_level = (police_response.heat_level - time.delta_secs() * 2.0).max(0.0);

    for combat_event in combat_events.read() {
        if combat_event.hit {
            if let Ok(civilian_transform) = civilian_query.get(combat_event.target) {
                police_response.civilian_casualties += 1;
                police_response.add_incident(
                    civilian_transform.translation.truncate(), 
                    50.0
                );
            }
        }
    }

    for audio_event in audio_events.read() {
        if matches!(audio_event.sound, AudioType::Gunshot) {
            if let Some(pos) = police_response.last_incident_pos {
                police_response.add_incident(pos, 5.0);
            }
        }
    }
}

pub fn police_spawn_system(
    mut commands: Commands,
    mut police_response: ResMut<PoliceResponse>,
    sprites: Res<GameSprites>,
    agent_query: Query<&Transform, With<Agent>>,
    time: Res<Time>,
    game_mode: Res<GameMode>,
) {
    if game_mode.paused { return; }

    if police_response.next_spawn_timer <= 0.0 && police_response.should_spawn_police() {
        let spawn_count = police_response.get_spawn_count();
        let spawn_pos = police_response.last_incident_pos.unwrap_or(Vec2::ZERO);
        let spawn_offset = Vec2::new(400.0, 0.0);

        for i in 0..spawn_count {
            let offset = Vec2::new(i as f32 * 30.0, (i % 2) as f32 * 30.0);
            spawn_police_unit(&mut commands, spawn_pos + spawn_offset + offset, &sprites);
        }

        police_response.next_spawn_timer = 30.0;
        police_response.heat_level = (police_response.heat_level * 0.7).max(25.0);
    }
}

fn spawn_police_unit(commands: &mut Commands, position: Vec2, sprites: &GameSprites) {
    let (mut sprite, mut transform) = crate::core::sprites::create_enemy_sprite(sprites);
    sprite.color = Color::srgb(0.2, 0.2, 0.8);
    transform.translation = position.extend(1.0);

    let patrol_points = vec![position, position + Vec2::new(100.0, 0.0)];
    let mut inventory = Inventory::default();
    inventory.equipped_weapon = Some(WeaponConfig::new(WeaponType::Rifle));

    let police_unit = commands.spawn_empty()
    .insert((
        sprite,
        transform,
        Enemy,
        Police { response_level: 1 },
        Health(120.0),
        Morale::new(150.0, 20.0),
        MovementSpeed(140.0),
        Vision::new(140.0, 50.0),
    ))
    .insert((
        Patrol::new(patrol_points),
        AIState::default(),
        GoapAgent::default(),
        WeaponState::new_from_type(&WeaponType::Rifle),
        inventory,
        RigidBody::Dynamic,
        Collider::ball(9.0),
        Velocity::default(),
        Damping { linear_damping: 10.0, angular_damping: 10.0 },
    ));
}