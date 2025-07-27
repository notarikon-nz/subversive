// src/systems/death.rs - Enhanced death handling and decal system
use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::core::*;
use crate::systems::*;
use crate::systems::explosions::*;

// === DEATH COMPONENTS ===

#[derive(Component)]
pub struct Corpse {
    pub entity_type: CorpseType,
    pub decay_timer: Option<f32>, // None = permanent
}

#[derive(Clone, Debug)]
pub enum CorpseType {
    Agent,
    Enemy, 
    Civilian,
    Police,
    Vehicle,
}

// Replace your existing death_system with this enhanced version
pub fn death_system(
    mut commands: Commands,
    mut target_query: Query<(
        Entity, 
        &mut Health, 
        &mut Sprite,
        &Transform,
        Option<&Agent>,
        Option<&Enemy>,
        Option<&Civilian>,
        Option<&Police>,
        Option<&Vehicle>,
    ), (Or<(With<Enemy>, With<Vehicle>, With<Civilian>, With<Police>)>, Without<Dead>)>,
    mut mission_data: ResMut<MissionData>,
    decal_settings: Res<DecalSettings>,
) {
    for (entity, mut health, mut sprite, transform, agent, enemy, civilian, police, vehicle) in target_query.iter_mut() {
        if health.0 <= 0.0 {
            // === IMMEDIATE AI SHUTDOWN ===
            // Stop all movement and physics immediately
            commands.entity(entity)
                .remove::<Velocity>()
                .remove::<ExternalForce>()
                .remove::<RigidBody>()
                .remove::<Collider>()
                // Remove all AI components to prevent continued behavior
                .remove::<GoapAgent>()
                .remove::<AIState>()
                .remove::<Vision>()
                .remove::<Patrol>()
                .remove::<Morale>()
                .remove::<WeaponState>()
                .remove::<Inventory>()
                .insert(Dead);

            // === DETERMINE ENTITY TYPE ===
            let corpse_type = if agent.is_some() {
                CorpseType::Agent
            } else if enemy.is_some() {
                mission_data.enemies_killed += 1;
                CorpseType::Enemy
            } else if civilian.is_some() {
                CorpseType::Civilian
            } else if police.is_some() {
                CorpseType::Police
            } else if vehicle.is_some() {
                CorpseType::Vehicle
            } else {
                CorpseType::Enemy // Default fallback
            };

            // === HANDLE DEATH VISUALS ===
            match corpse_type {
                CorpseType::Vehicle => {
                    // Vehicle becomes a grey wreck
                    sprite.color = Color::srgb(0.2, 0.2, 0.2);
                    
                    // Add large scorch mark underneath
                    spawn_decal(
                        &mut commands,
                        transform.translation.truncate(),
                        DecalType::Scorch,
                        80.0,
                        &decal_settings,
                    );
                }
                _ => {
                    // Living entities become dark red corpses
                    sprite.color = Color::srgb(0.3, 0.1, 0.1);
                    
                    // Add blood pool underneath the body
                    spawn_decal(
                        &mut commands,
                        transform.translation.truncate(),
                        DecalType::Blood,
                        25.0,
                        &decal_settings,
                    );
                }
            }

            // === ADD CORPSE COMPONENT ===
            commands.entity(entity).insert(Corpse {
                entity_type: corpse_type,
                decay_timer: None, // Persistent corpses
            });
        }
    }
}

pub fn explodable_death_system(
    mut commands: Commands,
    mut explodables: Query<(Entity, &Health, &Explodable, &Transform), (Without<Dead>, Without<PendingExplosion>)>,
) {
    for (entity, health, explodable, transform) in explodables.iter_mut() {
        if health.0 <= 0.0 {
            let pos = transform.translation.truncate();
            
            // Mark as dead
            commands.entity(entity).insert(Dead);
            
            // Create pending explosion
            commands.entity(entity).insert(PendingExplosion {
                timer: explodable.delay,
                damage: explodable.damage,
                radius: explodable.radius,
                explosion_type: ExplosionType::Cascading,
            });
            
            info!("Explodable at {:?} destroyed, pending explosion in {}s", pos, explodable.delay);
        }
    }
}

// === ENHANCED DEATH SYSTEM ===

pub fn enhanced_death_system(
    mut commands: Commands,
    mut dying_query: Query<(
        Entity, 
        &mut Health, 
        &Transform, 
        &mut Sprite,
        Option<&Agent>,
        Option<&Enemy>,
        Option<&Civilian>,
        Option<&Police>,
        Option<&Vehicle>,
    ), (Without<Dead>, Without<Corpse>)>,
    mut mission_data: ResMut<MissionData>,
    decal_settings: Res<DecalSettings>,
) {
    for (entity, mut health, transform, mut sprite, agent, enemy, civilian, police, vehicle) in dying_query.iter_mut() {
        if health.0 <= 0.0 {
            // Stop all AI behaviors immediately
            commands.entity(entity)
                .remove::<Velocity>()
                .remove::<ExternalForce>()
                .remove::<Collider>()
                .remove::<RigidBody>()
                .insert(Dead);

            // Determine corpse type and handle accordingly
            let corpse_type = if agent.is_some() {
                CorpseType::Agent
            } else if enemy.is_some() {
                mission_data.enemies_killed += 1;
                CorpseType::Enemy
            } else if civilian.is_some() {
                CorpseType::Civilian
            } else if police.is_some() {
                CorpseType::Police
            } else if vehicle.is_some() {
                CorpseType::Vehicle
            } else {
                CorpseType::Enemy // Default
            };

            // Create corpse appearance
            match corpse_type {
                CorpseType::Vehicle => {
                    // Vehicles become wrecks
                    sprite.color = Color::srgb(0.2, 0.2, 0.2);
                    commands.entity(entity).insert(Corpse {
                        entity_type: corpse_type,
                        decay_timer: None, // Vehicles don't decay
                    });
                    
                    // Add scorch decal for vehicle explosion
                    spawn_decal(
                        &mut commands,
                        transform.translation.truncate(),
                        DecalType::Scorch,
                        80.0, // Large scorch mark
                        &decal_settings,
                    );
                }
                _ => {
                    // Living entities become grey corpses
                    sprite.color = Color::srgb(0.3, 0.1, 0.1);
                    commands.entity(entity).insert(Corpse {
                        entity_type: corpse_type,
                        decay_timer: None, // Bodies don't decay by default
                    });
                    
                    // Add blood decal underneath
                    spawn_decal(
                        &mut commands,
                        transform.translation.truncate(),
                        DecalType::Blood,
                        25.0, // Blood pool size
                        &decal_settings,
                    );
                }
            }

            // Remove AI components to prevent continued behavior
            commands.entity(entity)
                .remove::<GoapAgent>()
                .remove::<AIState>()
                .remove::<Vision>()
                .remove::<Patrol>()
                .remove::<Morale>()
                .remove::<WeaponState>()
                .remove::<Inventory>();
        }
    }
}

// === CORPSE MANAGEMENT ===

pub fn corpse_cleanup_system(
    corpse_query: Query<(Entity, &Transform, &Corpse)>,
    camera_query: Query<&Transform, (With<Camera>, Without<Corpse>)>,
    mut commands: Commands,
) {
    let Ok(camera_transform) = camera_query.single() else { return; };
    let camera_pos = camera_transform.translation.truncate();
    
    // Only clean up corpses that are very far from camera and old
    for (entity, transform, corpse) in corpse_query.iter() {
        let distance = camera_pos.distance(transform.translation.truncate());
        
        // Only clean up very distant corpses (much larger distance than decals)
        if distance > 5000.0 {
            if let Some(timer) = corpse.decay_timer {
                if timer <= 0.0 {
                    commands.entity(entity).insert(MarkedForDespawn);
                }
            }
        }
    }
}

