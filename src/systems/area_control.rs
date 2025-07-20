// src/systems/area_control.rs
use bevy::prelude::*;
use crate::core::*;

#[derive(Component)]
pub struct AreaDenial {
    pub weapon_type: WeaponType,
    pub control_radius: f32,
    pub duration: f32,
    pub damage_per_second: f32,
}

#[derive(Component)]
pub struct SuppressionZone {
    pub center: Vec2,
    pub radius: f32,
    pub intensity: f32,
    pub duration: f32,
}

pub fn weapon_area_control_system(
    mut commands: Commands,
    mut area_weapons_query: Query<(Entity, &Transform, &Inventory, &mut GoapAgent), (With<Enemy>, Without<Dead>)>,
    agent_query: Query<&Transform, With<Agent>>,
    mut action_events: EventWriter<ActionEvent>,
    time: Res<Time>,
    game_mode: Res<GameMode>,
) {
    if game_mode.paused { return; }

    for (enemy_entity, enemy_transform, inventory, mut goap_agent) in area_weapons_query.iter_mut() {
        if let Some(weapon_config) = &inventory.equipped_weapon {
            let enemy_pos = enemy_transform.translation.truncate();

            match weapon_config.base_weapon {
                WeaponType::Flamethrower => {
                    handle_flamethrower_control(
                        enemy_entity,
                        enemy_pos,
                        &agent_query,
                        &mut goap_agent,
                        &mut action_events,
                        &mut commands,
                    );
                },
                WeaponType::Minigun => {
                    handle_minigun_suppression(
                        enemy_entity,
                        enemy_pos,
                        &agent_query,
                        &mut goap_agent,
                        &mut action_events,
                        &mut commands,
                    );
                },
                _ => {}
            }
        }
    }
}

fn handle_flamethrower_control(
    enemy_entity: Entity,
    enemy_pos: Vec2,
    agent_query: &Query<&Transform, With<Agent>>,
    goap_agent: &mut GoapAgent,
    action_events: &mut EventWriter<ActionEvent>,
    commands: &mut Commands,
) {
    let agents_in_range: Vec<Vec2> = agent_query.iter()
        .map(|t| t.translation.truncate())
        .filter(|&pos| enemy_pos.distance(pos) <= 80.0)
        .collect();

    if agents_in_range.len() >= 1 {
        let target_center = agents_in_range.iter().fold(Vec2::ZERO, |acc, &pos| acc + pos) / agents_in_range.len() as f32;
        
        commands.spawn((
            AreaDenial {
                weapon_type: WeaponType::Flamethrower,
                control_radius: 60.0,
                duration: 8.0,
                damage_per_second: 15.0,
            },
            Transform::from_translation(target_center.extend(0.5)),
            Sprite {
                color: Color::srgba(1.0, 0.3, 0.0, 0.4),
                custom_size: Some(Vec2::new(120.0, 120.0)),
                ..default()
            },
        ));

        action_events.write(ActionEvent {
            entity: enemy_entity,
            action: Action::Attack(Entity::PLACEHOLDER),
        });

        goap_agent.update_world_state(WorldKey::ControllingArea, true);
    }
}

fn handle_minigun_suppression(
    enemy_entity: Entity,
    enemy_pos: Vec2,
    agent_query: &Query<&Transform, With<Agent>>,
    goap_agent: &mut GoapAgent,
    action_events: &mut EventWriter<ActionEvent>,
    commands: &mut Commands,
) {
    if let Some(target_transform) = agent_query.iter().next() {
        let target_pos = target_transform.translation.truncate();
        let distance = enemy_pos.distance(target_pos);

        if distance <= 200.0 && distance >= 100.0 {
            commands.spawn((
                SuppressionZone {
                    center: target_pos,
                    radius: 40.0,
                    intensity: 0.8,
                    duration: 6.0,
                },
                Transform::from_translation(target_pos.extend(0.5)),
                Sprite {
                    color: Color::srgba(1.0, 1.0, 0.0, 0.3),
                    custom_size: Some(Vec2::new(80.0, 80.0)),
                    ..default()
                },
            ));

            action_events.write(ActionEvent {
                entity: enemy_entity,
                action: Action::Attack(Entity::PLACEHOLDER),
            });

            goap_agent.update_world_state(WorldKey::SuppressingTarget, true);
        }
    }
}

pub fn area_effect_system(
    mut area_query: Query<(Entity, &mut AreaDenial, &Transform)>,
    mut suppression_query: Query<(Entity, &mut SuppressionZone, &Transform), Without<AreaDenial>>,
    mut agent_query: Query<(&Transform, &mut Health), With<Agent>>,
    mut commands: Commands,
    time: Res<Time>,
    game_mode: Res<GameMode>,
) {
    if game_mode.paused { return; }

    for (entity, mut area_denial, area_transform) in area_query.iter_mut() {
        area_denial.duration -= time.delta_secs();
        
        if area_denial.duration <= 0.0 {
            commands.entity(entity).despawn();
            continue;
        }

        let area_pos = area_transform.translation.truncate();
        
        for (agent_transform, mut health) in agent_query.iter_mut() {
            let distance = area_pos.distance(agent_transform.translation.truncate());
            
            if distance <= area_denial.control_radius {
                health.0 -= area_denial.damage_per_second * time.delta_secs();
            }
        }
    }

    for (entity, mut suppression, _) in suppression_query.iter_mut() {
        suppression.duration -= time.delta_secs();
        
        if suppression.duration <= 0.0 {
            commands.entity(entity).despawn();
        }
    }
}

pub fn suppression_movement_system(
    suppression_query: Query<(&SuppressionZone, &Transform), Without<Agent>>,
    mut agent_query: Query<(&Transform, &mut MovementSpeed), With<Agent>>,
    game_mode: Res<GameMode>,
) {
    if game_mode.paused { return; }

    for (agent_transform, mut movement_speed) in agent_query.iter_mut() {
        let agent_pos = agent_transform.translation.truncate();
        let mut suppressed = false;

        for (suppression, _) in suppression_query.iter() {
            let distance = agent_pos.distance(suppression.center);
            
            if distance <= suppression.radius {
                movement_speed.0 *= 1.0 - suppression.intensity;
                suppressed = true;
                break;
            }
        }

        if !suppressed {
            movement_speed.0 = 150.0;
        }
    }
}