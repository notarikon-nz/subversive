// src/systems/explosion_decal_integration.rs - Connect explosions with interactive decals
use bevy::prelude::*;
use crate::core::*;
use crate::systems::decals::*;
use crate::systems::explosions::*;
use crate::systems::interactive_decals::*;

// === ENHANCED EXPLOSION DAMAGE SYSTEM ===

/// Enhanced version of explosion_damage_system that creates interactive decals
pub fn enhanced_explosion_damage_system(
    mut explosion_query: Query<(Entity, &mut Explosion, &Transform), Without<MarkedForDespawn>>,
    mut damageable_query: Query<(Entity, &Transform, &mut Health), (Without<Explosion>, Without<Dead>)>,
    explodable_query: Query<(Entity, &Transform, &Explodable), Without<PendingExplosion>>,
    vehicle_query: Query<(Entity, &Transform, &Vehicle), With<Vehicle>>,
    mut commands: Commands,
    mut audio_events: EventWriter<AudioEvent>,
    time: Res<Time>,
    game_mode: Res<GameMode>,
    combat_text_settings: Res<CombatTextSettings>,
    decal_settings: Res<DecalSettings>,
) {
    if game_mode.paused { return; }

    for (explosion_entity, mut explosion, explosion_transform) in explosion_query.iter_mut() {
        let is_new = explosion.duration == match explosion.explosion_type {
            ExplosionType::Grenade => 2.0,
            ExplosionType::Vehicle => 3.0,
            ExplosionType::TimeBomb => 2.5,
            ExplosionType::Cascading => 1.5,
        };
        
        explosion.duration -= time.delta_secs();
        
        if is_new {
            let explosion_pos = explosion_transform.translation.truncate();
            
            // === ORIGINAL DAMAGE LOGIC ===
            for (entity, target_transform, mut health) in damageable_query.iter_mut() {
                let target_pos = target_transform.translation.truncate();
                let distance = explosion_pos.distance(target_pos);
                
                if distance <= explosion.radius {
                    let damage_factor = (1.0 - (distance / explosion.radius)).max(0.1);
                    let damage = explosion.damage * damage_factor;
                    
                    health.0 = (health.0 - damage).max(0.0);
                    
                    // Apply fire effect for some explosions
                    if matches!(explosion.explosion_type, ExplosionType::Vehicle | ExplosionType::TimeBomb) 
                       && damage > 20.0 && rand::random::<f32>() < 0.3 {
                        commands.entity(entity).insert(StatusEffect {
                            effect_type: StatusType::Fire,
                            duration: 5.0,
                            intensity: damage * 0.1,
                            tick_timer: 0.0,
                            tick_rate: 1.0,
                        });
                    }
                    
                    if combat_text_settings.enabled {
                        spawn_damage_text(&mut commands, target_pos, damage, &combat_text_settings);
                    }
                }
            }
            
            // === CHAIN REACTIONS ===
            for (explodable_entity, explodable_transform, explodable) in explodable_query.iter() {
                let explodable_pos = explodable_transform.translation.truncate();
                let distance = explosion_pos.distance(explodable_pos);
                
                if distance <= explodable.chain_radius {
                    commands.entity(explodable_entity).insert(PendingExplosion {
                        timer: explodable.delay,
                        damage: explodable.damage,
                        radius: explodable.radius,
                        explosion_type: ExplosionType::Cascading,
                    });
                }
            }
            
            // === NEW: CREATE INTERACTIVE DECALS ===
            match explosion.explosion_type {
                ExplosionType::Vehicle => {
                    // Create scorch mark decal (visual only)
                    spawn_decal(
                        &mut commands,
                        explosion_pos,
                        DecalType::Scorch,
                        explosion.radius * 1.2,
                        &decal_settings,
                    );
                    
                    // Check if this was a vehicle explosion and create appropriate spills
                    for (vehicle_entity, vehicle_transform, vehicle) in vehicle_query.iter() {
                        let vehicle_pos = vehicle_transform.translation.truncate();
                        let distance = explosion_pos.distance(vehicle_pos);
                        
                        if distance <= 50.0 { // Explosion close to vehicle
                            create_vehicle_spill_from_explosion(
                                &mut commands,
                                vehicle_pos,
                                vehicle,
                                explosion.damage,
                            );
                        }
                    }
                },
                ExplosionType::Grenade => {
                    // Grenades create smaller scorch marks
                    spawn_decal(
                        &mut commands,
                        explosion_pos,
                        DecalType::Scorch,
                        explosion.radius * 0.8,
                        &decal_settings,
                    );
                },
                ExplosionType::TimeBomb => {
                    // Time bombs create large scorch marks and possible oil spills
                    spawn_decal(
                        &mut commands,
                        explosion_pos,
                        DecalType::Scorch,
                        explosion.radius * 1.4,
                        &decal_settings,
                    );
                    
                    // 30% chance to create an oil spill from ruptured pipes/containers
                    if rand::random::<f32>() < 0.3 {
                        spawn_oil_spill(&mut commands, explosion_pos, explosion.radius * 0.6);
                    }
                },
                ExplosionType::Cascading => {
                    // Cascading explosions from chain reactions
                    spawn_decal(
                        &mut commands,
                        explosion_pos,
                        DecalType::Explosion,
                        explosion.radius,
                        &decal_settings,
                    );
                },
            }
            
            audio_events.write(AudioEvent {
                sound: AudioType::Alert,
                volume: 1.0,
            });
        }
        
        if explosion.duration <= 0.0 {
            commands.entity(explosion_entity).insert(MarkedForDespawn);
        }
    }
}

// === VEHICLE SPILL CREATION ===

fn create_vehicle_spill_from_explosion(
    commands: &mut Commands,
    vehicle_pos: Vec2,
    vehicle: &Vehicle,
    explosion_damage: f32,
) {
    // Determine vehicle type based on vehicle properties (you'll need to adapt this)
    let vehicle_type = determine_vehicle_type(vehicle);
    
    // Larger explosions create bigger spills
    let spill_size_modifier = (explosion_damage / 50.0).clamp(0.5, 2.0);
    
    match vehicle_type {
        VehicleType::CivilianCar => {
            let size = 40.0 * spill_size_modifier;
            spawn_oil_spill(commands, vehicle_pos, size);
        },
        VehicleType::Truck => {
            let size = 60.0 * spill_size_modifier;
            spawn_oil_spill(commands, vehicle_pos, size);
            
            // Trucks might also spill some debris
            for i in 0..3 {
                let offset = Vec2::new(
                    (rand::random::<f32>() - 0.5) * 80.0,
                    (rand::random::<f32>() - 0.5) * 80.0,
                );
                spawn_oil_spill(commands, vehicle_pos + offset, 20.0);
            }
        },
        VehicleType::FuelTruck => {
            let size = 100.0 * spill_size_modifier;
            spawn_gasoline_spill(commands, vehicle_pos, size);
            
            // Fuel trucks create multiple gasoline spills in a pattern
            for i in 0..5 {
                let angle = (i as f32 / 5.0) * std::f32::consts::TAU;
                let offset = Vec2::new(angle.cos(), angle.sin()) * 60.0;
                spawn_gasoline_spill(commands, vehicle_pos + offset, 40.0);
            }
        },
        VehicleType::ElectricCar => {
            let size = 50.0 * spill_size_modifier;
            spawn_electric_puddle(commands, vehicle_pos, size);
        },
        _ => {
            let size = 60.0 * spill_size_modifier;
            spawn_oil_spill(commands, vehicle_pos, size);
        }
    }
}

// You'll need to implement this based on your Vehicle component structure
fn determine_vehicle_type(vehicle: &Vehicle) -> VehicleType {
    // This is a placeholder - adapt based on your actual Vehicle component
    // You might have fields like vehicle.vehicle_type or check other properties
    match vehicle.explosion_damage() {
        x if x > 150.0 => VehicleType::FuelTruck,
        x if x > 100.0 => VehicleType::Truck,
        x if x < 80.0 => VehicleType::ElectricCar,
        _ => VehicleType::CivilianCar,
    }
}

// === ENHANCED GRENADE EVENT HANDLER ===

/// Enhanced grenade handler that creates interactive decals
pub fn enhanced_handle_grenade_events(
    mut grenade_events: EventReader<GrenadeEvent>,
    mut commands: Commands,
    decal_settings: Res<DecalSettings>,
) {
    for event in grenade_events.read() {
        // Create the explosion
        spawn_explosion(
            &mut commands,
            event.target_pos,
            event.explosion_radius,
            event.damage,
            ExplosionType::Grenade,
        );
        
        // Create scorch decal
        spawn_decal(
            &mut commands,
            event.target_pos,
            DecalType::Scorch,
            event.explosion_radius * 0.8,
            &decal_settings,
        );
        
        // Small chance for grenade to rupture nearby containers
        if rand::random::<f32>() < 0.15 {
            let offset = Vec2::new(
                (rand::random::<f32>() - 0.5) * 40.0,
                (rand::random::<f32>() - 0.5) * 40.0,
            );
            spawn_oil_spill(&mut commands, event.target_pos + offset, 25.0);
        }
    }
}

// === ENHANCED VEHICLE EXPLOSION HANDLER ===

/// Enhanced vehicle explosion handler with spill creation
pub fn enhanced_handle_vehicle_explosions(
    mut commands: Commands,
    mut vehicle_query: Query<(Entity, &Transform, &Vehicle), (With<Vehicle>, Added<Dead>, Without<MarkedForDespawn>)>,
    decal_settings: Res<DecalSettings>,
) {
    for (entity, transform, vehicle) in vehicle_query.iter_mut() {
        let vehicle_pos = transform.translation.truncate();
        
        // Create the explosion
        spawn_explosion(
            &mut commands,
            vehicle_pos,
            vehicle.explosion_radius(),
            vehicle.explosion_damage(),
            ExplosionType::Vehicle,
        );
        
        // Create scorch decal
        spawn_decal(
            &mut commands,
            vehicle_pos,
            DecalType::Scorch,
            vehicle.explosion_radius() * 1.2,
            &decal_settings,
        );
        
        // Create vehicle-specific spills
        create_vehicle_spill_from_explosion(
            &mut commands,
            vehicle_pos,
            vehicle,
            vehicle.explosion_damage(),
        );
        
        commands.entity(entity).insert(MarkedForDespawn);
    }
}

// === PROJECTILE IMPACT INTEGRATION ===

/// Enhanced projectile impact system that can ignite flammable decals
pub fn enhanced_projectile_impact_decals(
    mut commands: Commands,
    impact_query: Query<&Transform, (With<ProjectileImpact>, Added<ProjectileImpact>)>,
    mut flammable_decals: Query<(Entity, &Transform, &Flammable, &mut InteractiveDecal), Without<OnFire>>,
    settings: Res<DecalSettings>,
) {
    for impact_transform in impact_query.iter() {
        let impact_pos = impact_transform.translation.truncate();
        
        // Create bullet hole decal
        spawn_decal(
            &mut commands,
            impact_pos,
            DecalType::BulletHole,
            8.0,
            &settings,
        );
        
        // Check if projectile hit near flammable decals (tracer rounds, incendiary, etc.)
        // Small chance for special ammo to ignite spills
        if rand::random::<f32>() < 0.05 { // 5% chance for regular bullets
            for (entity, decal_transform, flammable, mut decal) in flammable_decals.iter_mut() {
                let decal_pos = decal_transform.translation.truncate();
                let distance = impact_pos.distance(decal_pos);
                
                if distance <= 15.0 { // Very close hit
                    ignite_decal_from_impact(&mut commands, entity, &mut decal, flammable);
                }
            }
        }
    }
}

fn ignite_decal_from_impact(
    commands: &mut Commands,
    entity: Entity,
    decal: &mut InteractiveDecal,
    flammable: &Flammable,
) {
    // Only ignite gasoline easily, oil needs more heat
    let can_ignite = match decal.decal_type {
        InteractiveDecalType::GasolineSpill => true,
        InteractiveDecalType::OilSpill => rand::random::<f32>() < 0.3, // Oil harder to ignite
        _ => false,
    };
    
    if can_ignite {
        commands.entity(entity).insert(OnFire {
            intensity: 0.5, // Lower intensity from bullet impact
            spread_timer: 1.0 / flammable.burn_rate,
            burn_timer: decal.fuel_remaining,
        });
        
        info!("Bullet ignited {:?} decal!", decal.decal_type);
    }
}
