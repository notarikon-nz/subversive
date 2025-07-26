// src/systems/projectiles.rs - Compact projectile system
use bevy::prelude::*;
use crate::core::*;
use serde::{Deserialize, Serialize};

#[derive(Component)]
pub struct Projectile {
    pub target: Entity,
    pub damage: f32,
    pub speed: f32,
    pub weapon_type: WeaponType,
    pub attacker: Entity,
}

impl Projectile {
    pub fn new(target: Entity, damage: f32, speed: f32, weapon_type: WeaponType, attacker: Entity) -> Self {
        Self {
            target,
            damage,
            speed,
            weapon_type,
            attacker,
        }
    }
}

#[derive(Component)]
pub struct ProjectileVisual {
    pub lifetime: f32,
    pub max_lifetime: f32,
}


// Additional Types
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ProjectileType {
    Bullet,
    Grenade { fuse_time: f32 },
    Rocket { fuel_time: f32 },
    Energy { beam_width: f32 },
}

// Rocket Launcher
#[derive(Component)]
pub struct GuidedProjectile {
    pub target_tracking: bool,
    pub turn_rate: f32,
}

#[derive(Component)]
pub struct RocketTrail {
    pub spawn_timer: f32,
    pub particle_lifetime: f32,
}

// Energy Weapons
#[derive(Component)]
pub struct EnergyBeam {
    pub start_pos: Vec2,
    pub end_pos: Vec2,
    pub beam_width: f32,
    pub target: Entity,
    pub damage: f32,
    pub attacker: Entity,
}

#[derive(Clone)]
pub enum EnergyType {
    Laser { heat_buildup: f32 },
    Plasma { splash_radius: f32 },
    Ion { emp_effect: bool },
}

pub fn spawn_projectile(
    commands: &mut Commands,
    attacker: Entity,
    target: Entity,
    attacker_pos: Vec2,
    target_pos: Vec2,
    damage: f32,
    weapon_type: WeaponType,
) {
    match weapon_type {
        WeaponType::GrenadeLauncher => {
            spawn_grenade_projectile(commands, attacker, target, attacker_pos, target_pos, damage);
        },
        WeaponType::RocketLauncher => {
            spawn_rocket_projectile(commands, attacker, target, attacker_pos, target_pos, damage);
        },
        WeaponType::LaserRifle => {
            spawn_energy_beam(commands, attacker, target, attacker_pos, target_pos, damage);
        },
        _ => {
            spawn_bullet_projectile(commands, attacker, target, attacker_pos, target_pos, damage, weapon_type);
        }
    }
}

fn spawn_bullet_projectile(
    commands: &mut Commands,
    attacker: Entity,
    target: Entity,
    attacker_pos: Vec2,
    target_pos: Vec2,
    damage: f32,
    weapon_type: WeaponType,
) {
    let (speed, color, size) = match weapon_type {
        WeaponType::Pistol => (800.0, Color::srgb(1.0, 1.0, 0.8), Vec2::new(4.0, 2.0)),
        WeaponType::Rifle => (1200.0, Color::srgb(1.0, 0.9, 0.6), Vec2::new(6.0, 2.0)),
        WeaponType::Minigun => (1000.0, Color::srgb(1.0, 0.7, 0.3), Vec2::new(3.0, 8.0)),
        WeaponType::Flamethrower => (300.0, Color::srgb(1.0, 0.4, 0.1), Vec2::new(8.0, 8.0)),
        _ => (800.0, Color::srgb(1.0, 1.0, 0.8), Vec2::new(4.0, 2.0)),
    };

    let direction = (target_pos - attacker_pos).normalize();
    let rotation = direction.y.atan2(direction.x);

    commands.spawn((
        Sprite {
            color,
            custom_size: Some(size),
            ..default()
        },
        Transform::from_translation(attacker_pos.extend(10.0))
            .with_rotation(Quat::from_rotation_z(rotation)),
        Projectile::new(target, damage, speed, weapon_type, attacker),
        ProjectileVisual {
            lifetime: 0.0,
            max_lifetime: 2.0,
        },
    ));
}

fn spawn_grenade_projectile(
    commands: &mut Commands,
    attacker: Entity,
    target: Entity,
    attacker_pos: Vec2,
    target_pos: Vec2,
    damage: f32,
) {
    let direction = (target_pos - attacker_pos).normalize();
    let distance = attacker_pos.distance(target_pos);
    
    // Calculate arc trajectory
    let speed = 400.0;
    let time_to_target = distance / speed;
    let initial_velocity_y = (time_to_target * 250.0) / 2.0; // Gravity compensation
    
    commands.spawn((
        Sprite {
            color: Color::srgb(0.2, 0.8, 0.2),
            custom_size: Some(Vec2::new(8.0, 8.0)),
            ..default()
        },
        Transform::from_translation(attacker_pos.extend(10.0)),
        Projectile::new(target, damage, 400.0, WeaponType::GrenadeLauncher, attacker),
        ProjectileVisual {
            lifetime: 0.0,
            max_lifetime: 4.0,
        },
        GrenadeProjectile {
            initial_velocity: Vec2::new(direction.x * speed, initial_velocity_y),
            gravity: -250.0,
            fuse_time: 3.0,
        },
    ));
}

fn spawn_rocket_projectile(
    commands: &mut Commands,
    attacker: Entity,
    target: Entity,
    attacker_pos: Vec2,
    target_pos: Vec2,
    damage: f32,
) {
    let direction = (target_pos - attacker_pos).normalize();
    let rotation = direction.y.atan2(direction.x);

    commands.spawn((
        Sprite {
            color: Color::srgb(0.9, 0.1, 0.1),
            custom_size: Some(Vec2::new(12.0, 4.0)),
            ..default()
        },
        Transform::from_translation(attacker_pos.extend(10.0))
            .with_rotation(Quat::from_rotation_z(rotation)),
        Projectile::new(target, damage, 600.0, WeaponType::RocketLauncher, attacker),
        ProjectileVisual {
            lifetime: 0.0,
            max_lifetime: 3.0,
        },
        GuidedProjectile {
            target_tracking: true,
            turn_rate: 2.0,
        },
        RocketTrail {
            spawn_timer: 0.0,
            particle_lifetime: 0.5,
        },
    ));
}

fn spawn_energy_beam(
    commands: &mut Commands,
    attacker: Entity,
    target: Entity,
    attacker_pos: Vec2,
    target_pos: Vec2,
    damage: f32,
) {
    let direction = (target_pos - attacker_pos).normalize();
    let distance = attacker_pos.distance(target_pos);
    let midpoint = attacker_pos + direction * (distance / 2.0);
    let rotation = direction.y.atan2(direction.x);

    // Instant beam - no projectile travel time
    commands.spawn((
        Sprite {
            color: Color::srgb(0.1, 0.9, 1.0),
            custom_size: Some(Vec2::new(distance, 3.0)),
            ..default()
        },
        Transform::from_translation(midpoint.extend(15.0))
            .with_rotation(Quat::from_rotation_z(rotation)),
        EnergyBeam {
            start_pos: attacker_pos,
            end_pos: target_pos,
            beam_width: 3.0,
            target,
            damage,
            attacker,
        },
        ProjectileVisual {
            lifetime: 0.0,
            max_lifetime: 0.2, // Very short beam duration
        },
    ));
}

pub fn projectile_movement_system(
    mut commands: Commands,
    mut projectiles: Query<(Entity, &mut Transform, &mut Projectile, &mut ProjectileVisual), Without<GrenadeProjectile>>,
    targets: Query<&Transform, (Without<Projectile>, Or<(With<Enemy>, With<Vehicle>, With<Agent>)>)>,
    mut combat_events: EventWriter<CombatEvent>,
    mut damage_text_events: EventWriter<DamageTextEvent>,
    mut target_health: Query<&mut Health>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();
    
    for (entity, mut transform, projectile, mut visual) in projectiles.iter_mut() {
        visual.lifetime += dt;
        
        // Remove old projectiles
        if visual.lifetime > visual.max_lifetime {
            commands.entity(entity).insert(MarkedForDespawn);
            continue;
        }
        
        // Get target position
        let target_pos = if let Ok(target_transform) = targets.get(projectile.target) {
            target_transform.translation.truncate()
        } else {
            // Target doesn't exist, remove projectile
            commands.entity(entity).insert(MarkedForDespawn);
            continue;
        };
        
        let current_pos = transform.translation.truncate();
        let direction = (target_pos - current_pos).normalize();
        let move_distance = projectile.speed * dt;
        
        // Check if we've reached the target
        if current_pos.distance(target_pos) <= move_distance + 10.0 {
            // Hit target
            if let Ok(mut health) = target_health.get_mut(projectile.target) {
                health.0 = (health.0 - projectile.damage).max(0.0);
                
                damage_text_events.write(DamageTextEvent {
                    position: target_pos,
                    damage: projectile.damage,
                });
                
                combat_events.write(CombatEvent {
                    attacker: projectile.attacker,
                    target: projectile.target,
                    damage: projectile.damage,
                    hit: true,
                });
            }
            
            // Spawn impact effect
            spawn_impact_effect(&mut commands, target_pos, projectile.weapon_type.clone());
            commands.entity(entity).insert(MarkedForDespawn);
        } else {
            // Move projectile
            let new_pos = current_pos + direction * move_distance;
            transform.translation = new_pos.extend(transform.translation.z);
            
            // Update rotation to face movement direction
            let rotation = direction.y.atan2(direction.x);
            transform.rotation = Quat::from_rotation_z(rotation);
        }
    }
}

// Separate system for grenade physics
pub fn grenade_movement_system(
    mut commands: Commands,
    mut grenades: Query<(Entity, &mut Transform, &Projectile, &mut ProjectileVisual, &mut GrenadeProjectile)>,
    mut combat_events: EventWriter<CombatEvent>,
    mut damage_text_events: EventWriter<DamageTextEvent>,
    targets: Query<&Transform, (Without<Projectile>, Or<(With<Enemy>, With<Vehicle>, With<Agent>)>)>,
    mut target_health: Query<&mut Health>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();
    
    for (entity, mut transform, projectile, mut visual, mut grenade) in grenades.iter_mut() {
        visual.lifetime += dt;
        
        // Check if fuse timer expired
        if visual.lifetime >= grenade.fuse_time {
            // Explode
            let explosion_pos = transform.translation.truncate();
            
            // Area damage to all nearby targets
            for (target_entity, target_transform) in targets.iter().enumerate() {
                let distance = explosion_pos.distance(target_transform.translation.truncate());
                if distance <= 100.0 { // Explosion radius
                    let damage_multiplier = (1.0 - distance / 100.0).max(0.0);
                    let actual_damage = projectile.damage * damage_multiplier;
                    
                    // Apply damage if this is a valid target
                    if let Ok(mut health) = target_health.get_mut(projectile.target) {
                        health.0 = (health.0 - actual_damage).max(0.0);
                        
                        damage_text_events.write(DamageTextEvent {
                            position: target_transform.translation.truncate(),
                            damage: actual_damage,
                        });
                    }
                }
            }
            
            // Spawn explosion effect
            spawn_explosion_effect(&mut commands, explosion_pos);
            commands.entity(entity).insert(MarkedForDespawn);
            continue;
        }
        
        // Physics movement with gravity
        grenade.initial_velocity.y += grenade.gravity * dt;
        let new_pos = transform.translation.truncate() + grenade.initial_velocity * dt;
        transform.translation = new_pos.extend(transform.translation.z);
        
        // Check ground collision (simple y <= 0 check)
        if transform.translation.y <= 0.0 {
            grenade.initial_velocity.y = 0.0;
            grenade.initial_velocity.x *= 0.7; // Friction
            transform.translation.y = 0.0;
        }
    }
}

// Energy beam system (instant damage)
pub fn energy_beam_system(
    mut commands: Commands,
    mut beams: Query<(Entity, &EnergyBeam, &mut Sprite, &mut ProjectileVisual)>,
    mut combat_events: EventWriter<CombatEvent>,
    mut damage_text_events: EventWriter<DamageTextEvent>,
    mut target_health: Query<&mut Health>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();
    
    for (entity, beam, mut sprite, mut visual) in beams.iter_mut() {
        visual.lifetime += dt;
        
        // Apply damage immediately on first frame
        if visual.lifetime <= dt {
            if let Ok(mut health) = target_health.get_mut(beam.target) {
                health.0 = (health.0 - beam.damage).max(0.0);
                
                damage_text_events.write(DamageTextEvent {
                    position: beam.end_pos,
                    damage: beam.damage,
                });
                
                combat_events.write(CombatEvent {
                    attacker: beam.attacker,
                    target: beam.target,
                    damage: beam.damage,
                    hit: true,
                });
            }
        }
        
        // Fade out beam
        let alpha = 1.0 - (visual.lifetime / visual.max_lifetime);
        sprite.color = sprite.color.with_alpha(alpha);
        
        if visual.lifetime >= visual.max_lifetime {
            commands.entity(entity).insert(MarkedForDespawn);
        }
    }
}

fn spawn_impact_effect(commands: &mut Commands, position: Vec2, weapon_type: WeaponType) {
    let (color, size, lifetime) = match weapon_type {
        WeaponType::Pistol => (Color::srgb(1.0, 1.0, 0.5), 8.0, 0.2),
        WeaponType::Rifle => (Color::srgb(1.0, 0.8, 0.3), 12.0, 0.3),
        WeaponType::Minigun => (Color::srgb(1.0, 0.6, 0.2), 10.0, 0.25),
        WeaponType::Flamethrower => (Color::srgb(1.0, 0.3, 0.1), 16.0, 0.4),
        _ => (Color::srgb(1.0, 1.0, 0.5), 8.0, 0.2),
    };
    
    commands.spawn((
        Sprite {
            color,
            custom_size: Some(Vec2::splat(size)),
            ..default()
        },
        Transform::from_translation(position.extend(5.0)),
        ImpactEffect { lifetime },
    ));
}

fn spawn_explosion_effect(commands: &mut Commands, position: Vec2) {
    // Main explosion
    commands.spawn((
        Sprite {
            color: Color::srgb(1.0, 0.5, 0.1),
            custom_size: Some(Vec2::splat(50.0)),
            ..default()
        },
        Transform::from_translation(position.extend(5.0)),
        ImpactEffect { lifetime: 0.5 },
    ));
    
    // Particle effects around explosion
    for _ in 0..8 {
        let offset = Vec2::new(
            (fastrand::f32() - 0.5) * 80.0,
            (fastrand::f32() - 0.5) * 80.0,
        );
        
        commands.spawn((
            Sprite {
                color: Color::srgb(1.0, fastrand::f32() * 0.5 + 0.3, 0.1),
                custom_size: Some(Vec2::splat(8.0 + fastrand::f32() * 8.0)),
                ..default()
            },
            Transform::from_translation((position + offset).extend(4.0)),
            ImpactEffect { lifetime: 0.3 + fastrand::f32() * 0.4 },
        ));
    }
}





#[derive(Component)]
pub struct ImpactEffect {
    lifetime: f32,
}

#[derive(Component)]
pub struct GrenadeProjectile {
    pub initial_velocity: Vec2,
    pub gravity: f32,
    pub fuse_time: f32,
}

pub fn impact_effect_system(
    mut commands: Commands,
    mut impacts: Query<(Entity, &mut Sprite, &mut ImpactEffect)>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();
    
    for (entity, mut sprite, mut impact) in impacts.iter_mut() {
        impact.lifetime -= dt;
        
        if impact.lifetime <= 0.0 {
            commands.entity(entity).insert(MarkedForDespawn);
        } else {
            // Fade out effect
            let alpha = impact.lifetime / 0.4; // Assuming max lifetime is 0.4
            sprite.color = sprite.color.with_alpha(alpha);
        }
    }
}

// Flamethrower special effect system
pub fn flamethrower_stream_system(
    mut commands: Commands,
    flamethrower_projectiles: Query<&Transform, (With<Projectile>, Changed<Transform>)>,
    // Could add particle spawning logic here for continuous streams
) {
    // For flamethrowers, you might want to spawn multiple small fire particles
    // along the projectile path for a more realistic flame effect
    for transform in flamethrower_projectiles.iter() {
        // Spawn fire particles behind the main projectile
        let pos = transform.translation.truncate();
        
        // Random offset for particle spread
        let offset = Vec2::new(
            (fastrand::f32() - 0.5) * 20.0,
            (fastrand::f32() - 0.5) * 20.0,
        );
        
        commands.spawn((
            Sprite {
                color: Color::srgb(1.0, fastrand::f32() * 0.5 + 0.3, 0.1),
                custom_size: Some(Vec2::splat(4.0 + fastrand::f32() * 4.0)),
                ..default()
            },
            Transform::from_translation((pos + offset).extend(8.0)),
            ImpactEffect { lifetime: 0.3 + fastrand::f32() * 0.2 },
        ));
    }
}
