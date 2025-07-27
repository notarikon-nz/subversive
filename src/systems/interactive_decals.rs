// src/systems/interactive_decals.rs - Interactive decal system with oil spills and ignition
use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use crate::core::*;
use crate::systems::explosions::*;

// === INTERACTIVE DECAL COMPONENTS ===

#[derive(Component)]
pub struct InteractiveDecal {
    pub decal_type: InteractiveDecalType,
    pub radius: f32,
    pub intensity: f32,
    pub fuel_remaining: f32, // For flammable decals
}

#[derive(Clone, Debug, PartialEq)]
pub enum InteractiveDecalType {
    OilSpill,
    GasolineSpill,
    ElectricPuddle,
    IceSlick,
    TarPit,
}

#[derive(Component)]
pub struct MovementHindrance {
    pub slow_factor: f32,    // Multiplier for movement speed (0.0 = no movement, 1.0 = normal)
    pub stuck_chance: f32,   // Chance per second to get temporarily stuck
}

#[derive(Component)]
pub struct Flammable {
    pub ignition_temperature: f32,
    pub burn_rate: f32,
    pub spread_radius: f32,
    pub explosion_chance: f32, // Chance to explode when fully consumed
}

#[derive(Component)]
pub struct OnFire {
    pub intensity: f32,
    pub spread_timer: f32,
    pub burn_timer: f32,
}

#[derive(Component)]
pub struct ElectricalHazard {
    pub damage_per_second: f32,
    pub stun_chance: f32,
}

#[derive(Component)]
pub struct TemporarilyStuck {
    pub duration: f32,
}

// === RESOURCE FOR TRACKING EFFECTS ===

#[derive(Resource)]
pub struct InteractiveDecalSettings {
    pub enable_movement_effects: bool,
    pub enable_fire_spread: bool,
    pub oil_slow_factor: f32,
    pub gas_explosion_damage: f32,
    pub fire_spread_rate: f32,
}

impl Default for InteractiveDecalSettings {
    fn default() -> Self {
        Self {
            enable_movement_effects: true,
            enable_fire_spread: true,
            oil_slow_factor: 0.3,
            gas_explosion_damage: 80.0,
            fire_spread_rate: 2.0,
        }
    }
}

// === SPAWNING INTERACTIVE DECALS ===

/// Spawn an oil spill that slows movement and can be ignited
pub fn spawn_oil_spill(
    commands: &mut Commands,
    position: Vec2,
    size: f32,
) -> Entity {
    let oil_entity = commands.spawn((
        Sprite {
            color: Color::srgba(0.1, 0.08, 0.05, 0.8),
            custom_size: Some(Vec2::splat(size)),
            ..default()
        },
        Transform::from_translation(position.extend(-8.0)), // Below most things
        InteractiveDecal {
            decal_type: InteractiveDecalType::OilSpill,
            radius: size * 0.5,
            intensity: 1.0,
            fuel_remaining: 30.0, // Burns for 30 seconds
        },
        MovementHindrance {
            slow_factor: 0.3,
            stuck_chance: 0.1,
        },
        Flammable {
            ignition_temperature: 200.0, // Needs fire or explosion to ignite
            burn_rate: 1.0,
            spread_radius: size * 0.7,
            explosion_chance: 0.2,
        },
        // Physics sensor to detect entities entering the area
        Collider::ball(size * 0.5),
        Sensor,
    )).id();

    info!("Spawned oil spill at {:?} with radius {}", position, size * 0.5);
    oil_entity
}

/// Spawn a gasoline spill - more dangerous than oil
pub fn spawn_gasoline_spill(
    commands: &mut Commands,
    position: Vec2,
    size: f32,
) -> Entity {
    commands.spawn((
        Sprite {
            color: Color::srgba(0.15, 0.12, 0.18, 0.7),
            custom_size: Some(Vec2::splat(size)),
            ..default()
        },
        Transform::from_translation(position.extend(-8.0)),
        InteractiveDecal {
            decal_type: InteractiveDecalType::GasolineSpill,
            radius: size * 0.5,
            intensity: 1.0,
            fuel_remaining: 20.0,
        },
        MovementHindrance {
            slow_factor: 0.4,
            stuck_chance: 0.05,
        },
        Flammable {
            ignition_temperature: 100.0, // Easier to ignite than oil
            burn_rate: 2.0,
            spread_radius: size * 0.9,
            explosion_chance: 0.6, // Much more likely to explode
        },
        Collider::ball(size * 0.5),
        Sensor,
    )).id()
}

/// Spawn an electrical puddle that damages and stuns
pub fn spawn_electric_puddle(
    commands: &mut Commands,
    position: Vec2,
    size: f32,
) -> Entity {
    commands.spawn((
        Sprite {
            color: Color::srgba(0.1, 0.3, 0.8, 0.6),
            custom_size: Some(Vec2::splat(size)),
            ..default()
        },
        Transform::from_translation(position.extend(-8.0)),
        InteractiveDecal {
            decal_type: InteractiveDecalType::ElectricPuddle,
            radius: size * 0.5,
            intensity: 1.0,
            fuel_remaining: 60.0, // Lasts longer
        },
        ElectricalHazard {
            damage_per_second: 15.0,
            stun_chance: 0.3,
        },
        Collider::ball(size * 0.5),
        Sensor,
    )).id()
}

// === MOVEMENT EFFECTS SYSTEM ===

pub fn interactive_decal_movement_system(
    mut collision_events: EventReader<CollisionEvent>,
    mut commands: Commands,
    interactive_decals: Query<(&InteractiveDecal, &MovementHindrance, Option<&OnFire>)>,
    mut affected_entities: Query<(Entity, &mut Velocity), (Or<(With<Agent>, With<Enemy>, With<Civilian>)>, Without<TemporarilyStuck>)>,
    settings: Res<InteractiveDecalSettings>,
    time: Res<Time>,
) {
    if !settings.enable_movement_effects { return; }

    for collision_event in collision_events.read() {
        if let CollisionEvent::Started(entity1, entity2, _) = collision_event {
            // Check which entity is the decal and which is the unit
            let (decal_entity, unit_entity) = if interactive_decals.contains(*entity1) && affected_entities.contains(*entity2) {
                (*entity1, *entity2)
            } else if interactive_decals.contains(*entity2) && affected_entities.contains(*entity1) {
                (*entity2, *entity1)
            } else {
                continue;
            };

            if let (Ok((decal, hindrance, on_fire)), Ok((_, mut velocity))) = 
                (interactive_decals.get(decal_entity), affected_entities.get_mut(unit_entity)) {
                
                match decal.decal_type {
                    InteractiveDecalType::OilSpill | InteractiveDecalType::GasolineSpill => {
                        // Slow down the entity
                        velocity.linvel *= hindrance.slow_factor;
                        
                        // Chance to get stuck
                        if rand::random::<f32>() < hindrance.stuck_chance * time.delta_secs() {
                            commands.entity(unit_entity).insert(TemporarilyStuck {
                                duration: 1.0,
                            });
                            velocity.linvel = Vec2::ZERO;
                        }

                        // If the spill is on fire, damage the entity
                        if on_fire.is_some() {
                            // Apply fire damage - this will be handled by fire damage system
                            commands.entity(unit_entity).insert(StatusEffect {
                                effect_type: StatusType::Fire,
                                duration: 3.0,
                                intensity: 10.0,
                                tick_timer: 0.0,
                                tick_rate: 0.5,
                            });
                        }
                    },
                    InteractiveDecalType::IceSlick => {
                        // Ice makes you slide
                        let slide_force = velocity.linvel.normalize_or_zero() * 200.0;
                        velocity.linvel += slide_force * time.delta_secs();
                    },
                    InteractiveDecalType::TarPit => {
                        // Tar really slows you down
                        velocity.linvel *= 0.1;
                        if rand::random::<f32>() < 0.3 {
                            commands.entity(unit_entity).insert(TemporarilyStuck {
                                duration: 2.0,
                            });
                        }
                    },
                    _ => {}
                }
            }
        }
    }
}

// === ELECTRICAL HAZARD SYSTEM ===

pub fn electrical_hazard_system(
    mut collision_events: EventReader<CollisionEvent>,
    mut commands: Commands,
    electrical_decals: Query<(&InteractiveDecal, &ElectricalHazard)>,
    mut affected_entities: Query<(Entity, &mut Health), Or<(With<Agent>, With<Enemy>, With<Civilian>)>>,
    time: Res<Time>,
) {
    for collision_event in collision_events.read() {
        if let CollisionEvent::Started(entity1, entity2, _) = collision_event {
            let (decal_entity, unit_entity) = if electrical_decals.contains(*entity1) && affected_entities.contains(*entity2) {
                (*entity1, *entity2)
            } else if electrical_decals.contains(*entity2) && affected_entities.contains(*entity1) {
                (*entity2, *entity1)
            } else {
                continue;
            };

            if let (Ok((_, hazard)), Ok((_, mut health))) = 
                (electrical_decals.get(decal_entity), affected_entities.get_mut(unit_entity)) {
                
                // Apply electrical damage
                health.0 -= hazard.damage_per_second * time.delta_secs();
                
                // Chance to stun
                if rand::random::<f32>() < hazard.stun_chance * time.delta_secs() {
                    commands.entity(unit_entity).insert(TemporarilyStuck {
                        duration: 0.5,
                    });
                }
            }
        }
    }
}

// === FIRE IGNITION SYSTEM ===

pub fn fire_ignition_system(
    mut commands: Commands,
    explosion_query: Query<&Transform, (With<Explosion>, Added<Explosion>)>,
    fire_entities: Query<&Transform, With<OnFire>>,
    mut flammable_decals: Query<(Entity, &Transform, &Flammable, &mut InteractiveDecal), Without<OnFire>>,
    settings: Res<InteractiveDecalSettings>,
) {
    if !settings.enable_fire_spread { return; }

    // Check for explosions that could ignite flammable decals
    for explosion_transform in explosion_query.iter() {
        let explosion_pos = explosion_transform.translation.truncate();
        
        for (entity, decal_transform, flammable, mut decal) in flammable_decals.iter_mut() {
            let decal_pos = decal_transform.translation.truncate();
            let distance = explosion_pos.distance(decal_pos);
            
            if distance <= decal.radius + 50.0 { // Explosion can ignite nearby flammables
                ignite_decal(&mut commands, entity, &mut decal, flammable);
            }
        }
    }

    // Check for fire spreading to nearby flammable decals
    for fire_transform in fire_entities.iter() {
        let fire_pos = fire_transform.translation.truncate();
        
        for (entity, decal_transform, flammable, mut decal) in flammable_decals.iter_mut() {
            let decal_pos = decal_transform.translation.truncate();
            let distance = fire_pos.distance(decal_pos);
            
            if distance <= flammable.spread_radius {
                ignite_decal(&mut commands, entity, &mut decal, flammable);
            }
        }
    }
}

fn ignite_decal(
    commands: &mut Commands,
    entity: Entity,
    decal: &mut InteractiveDecal,
    flammable: &Flammable,
) {
    commands.entity(entity).insert(OnFire {
        intensity: 1.0,
        spread_timer: 1.0 / flammable.burn_rate,
        burn_timer: decal.fuel_remaining,
    });
    
    info!("Ignited {:?} decal!", decal.decal_type);
}

// === FIRE BURN SYSTEM ===

pub fn fire_burn_system(
    mut commands: Commands,
    mut burning_decals: Query<(Entity, &mut OnFire, &mut InteractiveDecal, &Transform, &Flammable, &mut Sprite)>,
    explodable_query: Query<(Entity, &Transform), With<Explodable>>,
    time: Res<Time>,
    settings: Res<InteractiveDecalSettings>,
) {
    for (entity, mut fire, mut decal, transform, flammable, mut sprite) in burning_decals.iter_mut() {
        fire.burn_timer -= time.delta_secs();
        fire.spread_timer -= time.delta_secs();
        
        // Update visual appearance - fire makes it glow
        let fire_intensity = (fire.burn_timer / decal.fuel_remaining).clamp(0.0, 1.0);
        sprite.color = Color::srgba(
            0.8 + fire_intensity * 0.2,
            0.3 + fire_intensity * 0.4,
            0.1,
            0.9
        );
        
        // Consume fuel
        decal.fuel_remaining -= flammable.burn_rate * time.delta_secs();
        
        // Check if fuel is exhausted
        if decal.fuel_remaining <= 0.0 {
            let pos = transform.translation.truncate();
            
            // Chance to explode when fuel is consumed
            if rand::random::<f32>() < flammable.explosion_chance {
                match decal.decal_type {
                    InteractiveDecalType::GasolineSpill => {
                        spawn_explosion(
                            &mut commands,
                            pos,
                            decal.radius * 1.5,
                            settings.gas_explosion_damage,
                            ExplosionType::Cascading,
                        );
                        
                        // Check for nearby explodables to trigger chain reactions
                        for (explodable_entity, explodable_transform) in explodable_query.iter() {
                            let distance = pos.distance(explodable_transform.translation.truncate());
                            if distance <= decal.radius * 2.0 {
                                commands.entity(explodable_entity).insert(PendingExplosion {
                                    timer: 0.5,
                                    damage: 60.0,
                                    radius: 50.0,
                                    explosion_type: ExplosionType::Cascading,
                                });
                            }
                        }
                    },
                    InteractiveDecalType::OilSpill => {
                        // Oil just burns out, smaller explosion
                        spawn_explosion(
                            &mut commands,
                            pos,
                            decal.radius * 0.8,
                            40.0,
                            ExplosionType::Cascading,
                        );
                    },
                    _ => {}
                }
            }
            
            // Remove the decal
            commands.entity(entity).insert(MarkedForDespawn);
        }
    }
}

// === STUCK ENTITIES SYSTEM ===

pub fn stuck_entities_system(
    mut stuck_query: Query<(Entity, &mut TemporarilyStuck, &mut Velocity)>,
    mut commands: Commands,
    time: Res<Time>,
) {
    for (entity, mut stuck, mut velocity) in stuck_query.iter_mut() {
        stuck.duration -= time.delta_secs();
        velocity.linvel *= 0.1; // Heavily reduce movement while stuck
        
        if stuck.duration <= 0.0 {
            commands.entity(entity).remove::<TemporarilyStuck>();
        }
    }
}

// === UTILITY FUNCTIONS ===

/// Helper to spawn oil spills from destroyed vehicles
pub fn spawn_vehicle_oil_leak(
    commands: &mut Commands,
    position: Vec2,
    vehicle_size: f32,
) {
    let spill_size = (vehicle_size * 0.8).max(30.0);
    spawn_oil_spill(commands, position, spill_size);
}

/// Helper to create gasoline trail (for fuel trucks, etc.)
pub fn create_gasoline_trail(
    commands: &mut Commands,
    start_pos: Vec2,
    end_pos: Vec2,
    spill_spacing: f32,
) {
    let direction = (end_pos - start_pos).normalize();
    let distance = start_pos.distance(end_pos);
    let num_spills = (distance / spill_spacing) as usize;
    
    for i in 0..num_spills {
        let t = i as f32 / num_spills as f32;
        let pos = start_pos + direction * distance * t;
        spawn_gasoline_spill(commands, pos, 20.0);
    }
}

// === INTEGRATION HELPERS ===

/// Call this when a vehicle is destroyed to create realistic spills
pub fn handle_vehicle_destruction_spills(
    commands: &mut Commands,
    vehicle_pos: Vec2,
    vehicle_type: VehicleType,
) {
    match vehicle_type {
        VehicleType::CivilianCar => {
            spawn_oil_spill(commands, vehicle_pos, 40.0);
        },
        VehicleType::PoliceCar => {
            spawn_oil_spill(commands, vehicle_pos, 40.0);
        },
        VehicleType::VTOL => {
            spawn_oil_spill(commands, vehicle_pos, 70.0);
        },
        VehicleType::APC => {
            spawn_oil_spill(commands, vehicle_pos, 50.0);
        },
        VehicleType::Tank => {
            spawn_oil_spill(commands, vehicle_pos, 70.0);
        },
        VehicleType::Truck => {
            spawn_oil_spill(commands, vehicle_pos, 60.0);
        },
        VehicleType::FuelTruck => {
            // Fuel truck creates gasoline spill instead of oil
            spawn_gasoline_spill(commands, vehicle_pos, 80.0);
        },
        VehicleType::ElectricCar => {
            spawn_electric_puddle(commands, vehicle_pos, 35.0);
        },
    }
}
