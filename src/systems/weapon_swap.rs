// src/systems/weapon_swap.rs
use bevy::prelude::*;
use crate::core::*;

#[derive(Component)]
pub struct DroppedWeapon {
    pub weapon_config: WeaponConfig,
    pub ammo_remaining: u32,
}

pub fn weapon_drop_system(
    mut commands: Commands,
    mut enemy_death_query: Query<(Entity, &Transform, &WeaponState, &Inventory), (With<Enemy>, Added<Dead>)>,
) {
    for (entity, transform, weapon_state, inventory) in enemy_death_query.iter_mut() {
        if let Some(weapon_config) = &inventory.equipped_weapon {
            commands.spawn((
                Sprite {
                    color: Color::srgb(0.6, 0.6, 0.6),
                    custom_size: Some(Vec2::new(16.0, 8.0)),
                    ..default()
                },
                Transform::from_translation(transform.translation),
                DroppedWeapon {
                    weapon_config: weapon_config.clone(),
                    ammo_remaining: weapon_state.current_ammo,
                },
            ));
        }
    }
}

pub fn weapon_pickup_system(
    mut commands: Commands,
    mut enemy_query: Query<(Entity, &Transform, &mut Inventory, &mut WeaponState, &mut GoapAgent), (With<Enemy>, Without<Dead>)>,
    dropped_weapon_query: Query<(Entity, &Transform, &DroppedWeapon)>,
    mut action_events: EventWriter<ActionEvent>,
) {
    for (enemy_entity, enemy_transform, mut inventory, mut weapon_state, mut goap_agent) in enemy_query.iter_mut() {
        let enemy_pos = enemy_transform.translation.truncate();
        
        for (dropped_entity, dropped_transform, dropped_weapon) in dropped_weapon_query.iter() {
            let distance = enemy_pos.distance(dropped_transform.translation.truncate());
            
            if distance <= 30.0 {
                let current_weapon_value = weapon_value(&inventory.equipped_weapon);
                let dropped_weapon_value = weapon_value(&Some(dropped_weapon.weapon_config.clone()));
                
                if dropped_weapon_value > current_weapon_value {
                    inventory.equipped_weapon = Some(dropped_weapon.weapon_config.clone());
                    *weapon_state = WeaponState::new(&dropped_weapon.weapon_config.base_weapon);
                    weapon_state.current_ammo = dropped_weapon.ammo_remaining;
                    weapon_state.apply_attachment_modifiers(&dropped_weapon.weapon_config);
                    
                    goap_agent.update_world_state(WorldKey::HasBetterWeapon, false);
                    goap_agent.abort_plan();
                    
                    commands.entity(dropped_entity).despawn();
                    
                    action_events.write(ActionEvent {
                        entity: enemy_entity,
                        action: Action::PickupWeapon,
                    });
                    
                    break;
                }
            }
        }
        
        let nearby_better_weapons = dropped_weapon_query.iter()
            .any(|(_, dropped_transform, dropped_weapon)| {
                let distance = enemy_pos.distance(dropped_transform.translation.truncate());
                distance <= 100.0 && weapon_value(&Some(dropped_weapon.weapon_config.clone())) > weapon_value(&inventory.equipped_weapon)
            });
        
        goap_agent.update_world_state(WorldKey::HasBetterWeapon, nearby_better_weapons);
    }
}

pub fn weapon_behavior_system(
    mut enemy_query: Query<(Entity, &Transform, &Inventory, &mut GoapAgent), (With<Enemy>, Without<Dead>)>,
    agent_query: Query<&Transform, With<Agent>>,
    cover_query: Query<&Transform, With<CoverPoint>>,
    mut action_events: EventWriter<ActionEvent>,
) {
    for (enemy_entity, enemy_transform, inventory, mut goap_agent) in enemy_query.iter_mut() {
        if let Some(weapon_config) = &inventory.equipped_weapon {
            let behavior = &weapon_config.behavior;
            let enemy_pos = enemy_transform.translation.truncate();
            
            if let Some(agent_transform) = agent_query.iter().next() {
                let agent_pos = agent_transform.translation.truncate();
                let distance = enemy_pos.distance(agent_pos);
                let effective_range = weapon_config.get_effective_range();
                
                goap_agent.update_world_state(WorldKey::InWeaponRange, distance <= effective_range);
                goap_agent.update_world_state(WorldKey::TooClose, distance < effective_range * 0.5);
                goap_agent.update_world_state(WorldKey::TooFar, distance > effective_range * 1.2);
                
                if behavior.requires_cover && distance <= effective_range {
                    let has_cover = cover_query.iter().any(|cover_transform| {
                        enemy_pos.distance(cover_transform.translation.truncate()) <= 40.0
                    });
                    
                    if !has_cover {
                        action_events.write(ActionEvent {
                            entity: enemy_entity,
                            action: Action::MoveTo(find_cover_position(enemy_pos, agent_pos)),
                        });
                    }
                }
                
                if behavior.area_effect && agent_query.iter().count() >= 2 {
                    let grouped = agent_query.iter()
                        .filter(|transform| {
                            agent_pos.distance(transform.translation.truncate()) <= 80.0
                        })
                        .count() >= 2;
                    
                    goap_agent.update_world_state(WorldKey::TargetGrouped, grouped);
                }
            }
        }
    }
}

fn weapon_value(weapon_config: &Option<WeaponConfig>) -> u32 {
    if let Some(config) = weapon_config {
        let base_value = match config.base_weapon {
            WeaponType::Pistol => 10,
            WeaponType::Rifle => 30,
            WeaponType::Minigun => 50,
            WeaponType::Flamethrower => 40,
        };
        
        let attachment_bonus = config.attachments.values()
            .filter(|att| att.is_some())
            .count() as u32 * 5;
        
        base_value + attachment_bonus
    } else {
        0
    }
}

fn find_cover_position(enemy_pos: Vec2, agent_pos: Vec2) -> Vec2 {
    let away_from_agent = (enemy_pos - agent_pos).normalize_or_zero();
    enemy_pos + away_from_agent * 60.0
}