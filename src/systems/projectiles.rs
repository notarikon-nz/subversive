// src/systems/projectiles.rs - Compact and efficient projectile system
use bevy::prelude::*;
use crate::core::*;
use serde::{Deserialize, Serialize};

// Unified projectile behavior enum
#[derive(Component, Clone)]
pub enum ProjectileBehavior {
    Standard,
    Grenade { velocity: Vec2, gravity: f32, fuse_timer: f32 },
    Guided { turn_rate: f32 },
    Rocket { trail_timer: f32 },
    Beam { start: Vec2, end: Vec2, applied_damage: bool },
}

// Single projectile component
#[derive(Component)]
pub struct Projectile {
    pub target: Entity,
    pub damage: f32,
    pub speed: f32,
    pub weapon_type: WeaponType,
    pub attacker: Entity,
    pub lifetime: f32,
    pub max_lifetime: f32,
    pub behavior: ProjectileBehavior,
}

// Projectile pool for performance
#[derive(Resource, Default)]
pub struct ProjectilePool {
    inactive: Vec<Entity>,
}

impl ProjectilePool {
    pub fn get_or_spawn(&mut self, commands: &mut Commands) -> Entity {
        if let Some(entity) = self.inactive.pop() {
            commands.entity(entity).insert(Visibility::Visible);
            entity
        } else {
            commands.spawn_empty().id()
        }
    }
    
    pub fn return_to_pool(&mut self, commands: &mut Commands, entity: Entity) {
        commands.entity(entity)
            .insert(Visibility::Hidden)
            .remove::<Projectile>()
            .remove::<Sprite>();
        self.inactive.push(entity);
    }
}

// Particle system for effects
#[derive(Component)]
pub struct ParticleSystem {
    particles: Vec<Particle>,
}

#[derive(Clone)]
struct Particle {
    position: Vec2,
    velocity: Vec2,
    color: Color,
    lifetime: f32,
    size: f32,
}

// Main spawn function
pub fn spawn_projectile(
    commands: &mut Commands,
    attacker: Entity,
    target: Entity,
    attacker_pos: Vec2,
    target_pos: Vec2,
    damage: f32,
    weapon_type: WeaponType,
) {
    let direction = (target_pos - attacker_pos).normalize();
    let rotation = direction.y.atan2(direction.x);
    
    let (speed, color, size, lifetime, behavior) = match weapon_type {
        WeaponType::Pistol => (
            800.0, 
            Color::srgb(1.0, 1.0, 0.8), 
            Vec2::new(4.0, 2.0), 
            2.0, 
            ProjectileBehavior::Standard
        ),
        WeaponType::Rifle => (
            1200.0, 
            Color::srgb(1.0, 0.9, 0.6), 
            Vec2::new(6.0, 2.0), 
            2.0, 
            ProjectileBehavior::Standard
        ),
        WeaponType::Minigun => (
            1000.0, 
            Color::srgb(1.0, 0.7, 0.3), 
            Vec2::new(3.0, 8.0), 
            2.0, 
            ProjectileBehavior::Standard
        ),
        WeaponType::Flamethrower => (
            300.0, 
            Color::srgb(1.0, 0.4, 0.1), 
            Vec2::new(8.0, 8.0), 
            2.0, 
            ProjectileBehavior::Standard
        ),
        WeaponType::GrenadeLauncher => {
            let dist = attacker_pos.distance(target_pos);
            let speed = 400.0;
            let time_to_target = dist / speed;
            let velocity = Vec2::new(direction.x * speed, time_to_target * 125.0);
            (
                400.0, 
                Color::srgb(0.2, 0.8, 0.2), 
                Vec2::splat(8.0), 
                4.0,
                ProjectileBehavior::Grenade { velocity, gravity: -250.0, fuse_timer: 3.0 }
            )
        },
        WeaponType::RocketLauncher => (
            600.0, 
            Color::srgb(0.9, 0.1, 0.1), 
            Vec2::new(12.0, 4.0), 
            3.0,
            ProjectileBehavior::Rocket { trail_timer: 0.0 }
        ),
        WeaponType::LaserRifle => {
            let dist = attacker_pos.distance(target_pos);
            let midpoint = attacker_pos + direction * (dist / 2.0);
            (
                0.0, 
                Color::srgb(0.1, 0.9, 1.0), 
                Vec2::new(dist, 3.0), 
                0.2,
                ProjectileBehavior::Beam { 
                    start: attacker_pos, 
                    end: target_pos, 
                    applied_damage: false 
                }
            )
        },
        WeaponType::PlasmaGun => {
            let dist = attacker_pos.distance(target_pos);
            (
                500.0, 
                Color::srgb(0.8, 0.2, 1.0), 
                Vec2::new(10.0, 10.0), 
                2.5,
                ProjectileBehavior::Standard
            )
        },
    };
    
    let spawn_pos = match &behavior {
        ProjectileBehavior::Beam { start, .. } => {
            let dist = start.distance(target_pos);
            (start + direction * (dist / 2.0)).extend(15.0)
        },
        _ => attacker_pos.extend(10.0),
    };
    
    commands.spawn((
        Sprite {
            color,
            custom_size: Some(size),
            ..default()
        },
        Transform::from_translation(spawn_pos)
            .with_rotation(Quat::from_rotation_z(rotation)),
        Projectile {
            target,
            damage,
            speed,
            weapon_type,
            attacker,
            lifetime: 0.0,
            max_lifetime: lifetime,
            behavior,
        },
    ));
}

// Unified projectile system - handles all projectile types
pub fn unified_projectile_system(
    mut commands: Commands,
    mut projectiles: Query<(Entity, &mut Transform, &mut Projectile, Option<&mut Sprite>),Without<MarkedForDespawn>>,
    targets: Query<&Transform, (Without<Projectile>, Or<(With<Enemy>, With<Vehicle>, With<Agent>)>)>,
    mut combat_events: EventWriter<CombatEvent>,
    mut damage_text_events: EventWriter<DamageTextEvent>,
    mut target_health: Query<&mut Health>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();
    
    for (entity, mut transform, mut projectile, sprite) in projectiles.iter_mut() {
        projectile.lifetime += dt;
        
        // Remove expired projectiles
        if projectile.lifetime > projectile.max_lifetime {
            commands.entity(entity).insert(MarkedForDespawn);
            continue;
        }
        
        // Clone behavior to avoid borrow checker issues
        let mut behavior = projectile.behavior.clone();
        
        match &mut behavior {
            ProjectileBehavior::Standard => {
                handle_standard_projectile(
                    &mut commands,
                    entity,
                    &mut transform,
                    &projectile,
                    &targets,
                    &mut combat_events,
                    &mut damage_text_events,
                    &mut target_health,
                    dt,
                );
            },
            ProjectileBehavior::Grenade { velocity, gravity, fuse_timer } => {
                handle_grenade_projectile(
                    &mut commands,
                    entity,
                    &mut transform,
                    &projectile,
                    velocity,
                    gravity,
                    fuse_timer,
                    &targets,
                    &mut combat_events,
                    &mut damage_text_events,
                    &mut target_health,
                    dt,
                );
            },
            ProjectileBehavior::Rocket { trail_timer } => {
                handle_rocket_projectile(
                    &mut commands,
                    entity,
                    &mut transform,
                    &projectile,
                    trail_timer,
                    &targets,
                    &mut combat_events,
                    &mut damage_text_events,
                    &mut target_health,
                    dt,
                );
            },
            ProjectileBehavior::Beam { start, end, applied_damage } => {
                handle_beam_projectile(
                    &mut commands,
                    entity,
                    &projectile,
                    sprite,
                    applied_damage,
                    end,
                    &mut combat_events,
                    &mut damage_text_events,
                    &mut target_health,
                );
            },
            _ => {},
        }
        
        // Write back the modified behavior
        projectile.behavior = behavior;
    }
}

// Handle standard projectile movement
fn handle_standard_projectile(
    commands: &mut Commands,
    entity: Entity,
    transform: &mut Transform,
    projectile: &Projectile,
    targets: &Query<&Transform, (Without<Projectile>, Or<(With<Enemy>, With<Vehicle>, With<Agent>)>)>,
    combat_events: &mut EventWriter<CombatEvent>,
    damage_text_events: &mut EventWriter<DamageTextEvent>,
    target_health: &mut Query<&mut Health>,
    dt: f32,
) {
    if let Ok(target_t) = targets.get(projectile.target) {
        let target_pos = target_t.translation.truncate();
        let current_pos = transform.translation.truncate();
        let direction = (target_pos - current_pos).normalize();
        let move_distance = projectile.speed * dt;
        
        if current_pos.distance(target_pos) <= move_distance + 10.0 {
            // Hit target
            apply_damage(
                combat_events,
                damage_text_events,
                target_health,
                projectile,
                target_pos,
            );
            spawn_impact(commands, target_pos, projectile.weapon_type);
            commands.entity(entity).insert(MarkedForDespawn);
        } else {
            // Move projectile
            transform.translation += direction.extend(0.0) * move_distance;
            transform.rotation = Quat::from_rotation_z(direction.y.atan2(direction.x));
        }
    } else {
        commands.entity(entity).insert(MarkedForDespawn);
    }
}

// Handle grenade physics
fn handle_grenade_projectile(
    commands: &mut Commands,
    entity: Entity,
    transform: &mut Transform,
    projectile: &Projectile,
    velocity: &mut Vec2,
    gravity: &f32,
    fuse_timer: &mut f32,
    targets: &Query<&Transform, (Without<Projectile>, Or<(With<Enemy>, With<Vehicle>, With<Agent>)>)>,
    combat_events: &mut EventWriter<CombatEvent>,
    damage_text_events: &mut EventWriter<DamageTextEvent>,
    target_health: &mut Query<&mut Health>,
    dt: f32,
) {
    *fuse_timer -= dt;
    
    if *fuse_timer <= 0.0 {
        // Explode
        let pos = transform.translation.truncate();
        apply_area_damage(
            combat_events,
            damage_text_events,
            target_health,
            targets,
            pos,
            projectile.damage,
            100.0,
            projectile.attacker,
        );
        spawn_explosion(commands, pos);
        commands.entity(entity).insert(MarkedForDespawn);
    } else {
        // Physics movement
        velocity.y += *gravity * dt;
        transform.translation += velocity.extend(0.0) * dt;
        
        // Ground collision
        if transform.translation.y <= 0.0 {
            transform.translation.y = 0.0;
            velocity.y = 0.0;
            velocity.x *= 0.7; // Friction
        }
    }
}

// Handle rocket projectile
fn handle_rocket_projectile(
    commands: &mut Commands,
    entity: Entity,
    transform: &mut Transform,
    projectile: &Projectile,
    trail_timer: &mut f32,
    targets: &Query<&Transform, (Without<Projectile>, Or<(With<Enemy>, With<Vehicle>, With<Agent>)>)>,
    combat_events: &mut EventWriter<CombatEvent>,
    damage_text_events: &mut EventWriter<DamageTextEvent>,
    target_health: &mut Query<&mut Health>,
    dt: f32,
) {
    // Trail particles
    *trail_timer -= dt;
    if *trail_timer <= 0.0 {
        *trail_timer = 0.05;
        spawn_trail_particle(commands, transform.translation.truncate());
    }
    
    // Standard movement with guided behavior
    if let Ok(target_t) = targets.get(projectile.target) {
        let target_pos = target_t.translation.truncate();
        let current_pos = transform.translation.truncate();
        let direction = (target_pos - current_pos).normalize();
        let move_distance = projectile.speed * dt;
        
        if current_pos.distance(target_pos) <= move_distance + 15.0 {
            // Hit with explosion
            apply_area_damage(
                combat_events,
                damage_text_events,
                target_health,
                targets,
                target_pos,
                projectile.damage,
                120.0,
                projectile.attacker,
            );
            spawn_explosion(commands, target_pos);
            commands.entity(entity).insert(MarkedForDespawn);
        } else {
            transform.translation += direction.extend(0.0) * move_distance;
            transform.rotation = Quat::from_rotation_z(direction.y.atan2(direction.x));
        }
    } else {
        commands.entity(entity).insert(MarkedForDespawn);
    }
}

// Handle beam projectile
fn handle_beam_projectile(
    commands: &mut Commands,
    entity: Entity,
    projectile: &Projectile,
    sprite: Option<Mut<Sprite>>,
    applied_damage: &mut bool,
    end_pos: &Vec2,
    combat_events: &mut EventWriter<CombatEvent>,
    damage_text_events: &mut EventWriter<DamageTextEvent>,
    target_health: &mut Query<&mut Health>,
) {
    // Apply damage once
    if !*applied_damage {
        if let Ok(mut health) = target_health.get_mut(projectile.target) {
            health.0 = (health.0 - projectile.damage).max(0.0);
            damage_text_events.write(DamageTextEvent {
                position: *end_pos,
                damage: projectile.damage,
            });
            combat_events.write(CombatEvent {
                attacker: projectile.attacker,
                target: projectile.target,
                damage: projectile.damage,
                hit: true,
            });
        }
        *applied_damage = true;
    }
    
    // Fade out
    if let Some(mut sprite) = sprite {
        let alpha = 1.0 - (projectile.lifetime / projectile.max_lifetime);
        sprite.color = sprite.color.with_alpha(alpha);
    }
}

// Helper functions
fn apply_damage(
    combat_events: &mut EventWriter<CombatEvent>,
    damage_text_events: &mut EventWriter<DamageTextEvent>,
    target_health: &mut Query<&mut Health>,
    projectile: &Projectile,
    position: Vec2,
) {
    if let Ok(mut health) = target_health.get_mut(projectile.target) {
        health.0 = (health.0 - projectile.damage).max(0.0);
        
        damage_text_events.write(DamageTextEvent {
            position,
            damage: projectile.damage,
        });
        
        combat_events.write(CombatEvent {
            attacker: projectile.attacker,
            target: projectile.target,
            damage: projectile.damage,
            hit: true,
        });
    }
}

fn apply_area_damage(
    combat_events: &mut EventWriter<CombatEvent>,
    damage_text_events: &mut EventWriter<DamageTextEvent>,
    target_health: &mut Query<&mut Health>,
    targets: &Query<&Transform, (Without<Projectile>, Or<(With<Enemy>, With<Vehicle>, With<Agent>)>)>,
    explosion_pos: Vec2,
    base_damage: f32,
    radius: f32,
    attacker: Entity,
) {
    for (entity, transform) in targets.iter().enumerate() {
        let distance = explosion_pos.distance(transform.translation.truncate());
        if distance <= radius {
            let damage_multiplier = (1.0 - distance / radius).max(0.0);
            let actual_damage = base_damage * damage_multiplier;
            
            // Create a fake entity ID from the index (this is a hack for the example)
            // In real code, you'd need to properly track entity IDs
            if let Ok(mut health) = target_health.get_mut(Entity::from_raw(entity as u32)) {
                health.0 = (health.0 - actual_damage).max(0.0);
                
                damage_text_events.write(DamageTextEvent {
                    position: transform.translation.truncate(),
                    damage: actual_damage,
                });
            }
        }
    }
}

fn spawn_impact(commands: &mut Commands, position: Vec2, weapon_type: WeaponType) {
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

fn spawn_explosion(commands: &mut Commands, position: Vec2) {
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
    
    // Simplified particles - spawn fewer for performance
    for i in 0..6 {
        let angle = i as f32 * std::f32::consts::TAU / 6.0;
        let offset = Vec2::new(angle.cos(), angle.sin()) * 40.0;
        
        commands.spawn((
            Sprite {
                color: Color::srgb(1.0, 0.4 + fastrand::f32() * 0.3, 0.1),
                custom_size: Some(Vec2::splat(8.0 + fastrand::f32() * 4.0)),
                ..default()
            },
            Transform::from_translation((position + offset).extend(4.0)),
            ImpactEffect { lifetime: 0.3 + fastrand::f32() * 0.2 },
        ));
    }
}

fn spawn_trail_particle(commands: &mut Commands, position: Vec2) {
    commands.spawn((
        Sprite {
            color: Color::srgb(1.0, 0.5 + fastrand::f32() * 0.3, 0.1),
            custom_size: Some(Vec2::splat(4.0 + fastrand::f32() * 2.0)),
            ..default()
        },
        Transform::from_translation(position.extend(8.0)),
        ImpactEffect { lifetime: 0.3 },
    ));
}

#[derive(Component)]
pub struct ImpactEffect {
    lifetime: f32,
}

// Simple impact effect system
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
            // Fade and shrink
            let progress = impact.lifetime / 0.5; // Max lifetime
            sprite.color = sprite.color.with_alpha(progress);
            if let Some(size) = sprite.custom_size.as_mut() {
                *size *= 0.95; // Shrink
            }
        }
    }
}