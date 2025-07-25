// src/systems/explosions.rs - Unified explosion system
use bevy::prelude::*;
use crate::core::*;

// === EXPLOSION COMPONENTS ===
#[derive(Component)]
pub struct Explosion {
    pub radius: f32,
    pub damage: f32,
    pub duration: f32,
    pub explosion_type: ExplosionType,
}

#[derive(Clone)]
pub enum ExplosionType {
    Grenade,
    Vehicle,
    TimeBomb,
    Cascading
}

// === NEW COMPONENTS ===
#[derive(Component)]
pub struct TimeBomb {
    pub timer: f32,
    pub damage: f32,
    pub radius: f32,
    pub armed: bool,
}

#[derive(Component)]
pub struct Explodable {
    pub chain_radius: f32,    // How close explosion needs to be to trigger
    pub damage: f32,
    pub radius: f32,
    pub delay: f32,           // Delay before exploding when triggered
}

#[derive(Component)]
pub struct StatusEffect {
    pub effect_type: StatusType,
    pub duration: f32,
    pub intensity: f32,
    pub tick_timer: f32,
    pub tick_rate: f32,       // How often effect applies (seconds)
}

#[derive(Clone, PartialEq)]
pub enum StatusType {
    Fire,
    EMP,
    //Poison,
    // Add more later
}

#[derive(Component)]
pub struct PendingExplosion {
    pub timer: f32,
    pub damage: f32,
    pub radius: f32,
    pub explosion_type: ExplosionType,
}

#[derive(Component)]
pub struct StasisField {
    pub radius: f32,
    pub slow_factor: f32,  // Multiplier for movement speed
}

// Destructible buildings/walls incoming
#[derive(Component)]
pub struct Destructible {
    pub health: f32,
    pub debris_count: u32,
}

// === FLOATING COMBAT TEXT ===
#[derive(Component)]
pub struct FloatingText {
    pub lifetime: f32,
    pub velocity: Vec2,
}

#[derive(Resource)]
pub struct CombatTextSettings {
    pub enabled: bool,
    pub font_size: f32,
}

impl Default for CombatTextSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            font_size: 16.0,
        }
    }
}

// === ENHANCED EXPLOSION DAMAGE SYSTEM ===
pub fn explosion_damage_system(
    mut explosion_query: Query<(Entity, &mut Explosion, &Transform)>,
    mut damageable_query: Query<(Entity, &Transform, &mut Health), (Without<Explosion>, Without<Dead>)>,
    explodable_query: Query<(Entity, &Transform, &Explodable), Without<PendingExplosion>>,
    mut commands: Commands,
    mut audio_events: EventWriter<AudioEvent>,
    time: Res<Time>,
    game_mode: Res<GameMode>,
    combat_text_settings: Res<CombatTextSettings>,
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
            
            // Apply damage to damageable entities
            for (entity, target_transform, mut health) in damageable_query.iter_mut() {
                let target_pos = target_transform.translation.truncate();
                let distance = explosion_pos.distance(target_pos);
                
                if distance <= explosion.radius {
                    let damage_factor = (1.0 - (distance / explosion.radius)).max(0.1);
                    let damage = explosion.damage * damage_factor;
                    
                    health.0 = (health.0 - damage).max(0.0);
                    
                    // Apply fire status effect for certain explosion types
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
            
            // CASCADING EXPLOSIONS: Check for nearby explodables
            // Over time may need to chunk checks
            for (explodable_entity, explodable_transform, explodable) in explodable_query.iter() {
                let explodable_pos = explodable_transform.translation.truncate();
                let distance = explosion_pos.distance(explodable_pos);
                
                if distance <= explodable.chain_radius {
                    // Schedule delayed explosion
                    commands.entity(explodable_entity).insert(PendingExplosion {
                        timer: explodable.delay,
                        damage: explodable.damage,
                        radius: explodable.radius,
                        explosion_type: ExplosionType::Cascading,
                    });
                }
            }
            
            audio_events.write(AudioEvent {
                sound: AudioType::Alert,
                volume: 1.0,
            });
        }
        
        if explosion.duration <= 0.0 {
            commands.entity(explosion_entity).despawn();
        }
    }
}

// === TIME BOMB SYSTEM ===
pub fn time_bomb_system(
    mut bomb_query: Query<(Entity, &mut TimeBomb, &Transform)>,
    mut commands: Commands,
    time: Res<Time>,
    game_mode: Res<GameMode>,
) {
    if game_mode.paused { return; }

    for (entity, mut bomb, transform) in bomb_query.iter_mut() {
        if !bomb.armed { continue; }
        
        bomb.timer -= time.delta_secs();
        
        if bomb.timer <= 0.0 {
            spawn_explosion(
                &mut commands,
                transform.translation.truncate(),
                bomb.radius,
                bomb.damage,
                ExplosionType::TimeBomb,
            );
            commands.entity(entity).despawn();
        }
    }
}

// === PENDING EXPLOSION SYSTEM ===
pub fn pending_explosion_system(
    mut pending_query: Query<(Entity, &mut PendingExplosion, &Transform)>,
    mut commands: Commands,
    time: Res<Time>,
    game_mode: Res<GameMode>,
) {
    if game_mode.paused { return; }

    for (entity, mut pending, transform) in pending_query.iter_mut() {
        pending.timer -= time.delta_secs();
        
        if pending.timer <= 0.0 {
            spawn_explosion(
                &mut commands,
                transform.translation.truncate(),
                pending.radius,
                pending.damage,
                pending.explosion_type.clone(),
            );
            // Remove the explodable component and pending explosion
            commands.entity(entity)
                .remove::<Explodable>()
                .remove::<PendingExplosion>();
        }
    }
}

// === STATUS EFFECT SYSTEM ===
pub fn status_effect_system(
    mut affected_query: Query<(Entity, &mut StatusEffect, &mut Health, &Transform)>,
    mut commands: Commands,
    time: Res<Time>,
    game_mode: Res<GameMode>,
    combat_text_settings: Res<CombatTextSettings>,
) {
    if game_mode.paused { return; }

    for (entity, mut status, mut health, transform) in affected_query.iter_mut() {
        status.duration -= time.delta_secs();
        status.tick_timer -= time.delta_secs();
        
        // Apply effect
        if status.tick_timer <= 0.0 {
            match status.effect_type {
                StatusType::Fire => {
                    health.0 = (health.0 - status.intensity).max(0.0);
                    
                    if combat_text_settings.enabled {
                        spawn_fire_text(&mut commands, transform.translation.truncate(), status.intensity);
                    }
                },
                StatusType::EMP => {
                    // SLOW UNIT, TO BE IMPLEMENTED
                }

            }
            status.tick_timer = status.tick_rate;
        }
        
        // Remove expired effects
        if status.duration <= 0.0 {
            commands.entity(entity).remove::<StatusEffect>();
        }
    }
}

// === SPAWNING HELPERS ===
pub fn spawn_time_bomb(
    commands: &mut Commands,
    position: Vec2,
    timer: f32,
    damage: f32,
    radius: f32,
) -> Entity {
    commands.spawn((
        Sprite {
            color: Color::srgb(0.8, 0.2, 0.2),
            custom_size: Some(Vec2::new(12.0, 8.0)),
            ..default()
        },
        Transform::from_translation(position.extend(1.0)),
        TimeBomb {
            timer,
            damage,
            radius,
            armed: true,
        },
    )).id()
}

pub fn spawn_explodable_object(
    commands: &mut Commands,
    position: Vec2,
    object_type: ExplodableType,
) -> Entity {
    let (size, color, explodable) = match object_type {
        ExplodableType::FuelBarrel => (
            Vec2::new(16.0, 20.0),
            Color::srgb(0.8, 0.6, 0.2),
            Explodable {
                chain_radius: 40.0,
                damage: 60.0,
                radius: 50.0,
                delay: 0.5,
            }
        ),
        ExplodableType::GasCanister => (
            Vec2::new(8.0, 16.0),
            Color::srgb(0.6, 0.8, 0.6),
            Explodable {
                chain_radius: 30.0,
                damage: 40.0,
                radius: 35.0,
                delay: 0.2,
            }
        ),
        ExplodableType::PowerCell => (
            Vec2::new(12.0, 12.0),
            Color::srgb(0.2, 0.6, 0.8),
            Explodable {
                chain_radius: 25.0,
                damage: 35.0,
                radius: 30.0,
                delay: 0.3,
            }
        ),
    };
    
    commands.spawn((
        Sprite {
            color,
            custom_size: Some(size),
            ..default()
        },
        Transform::from_translation(position.extend(1.0)),
        explodable,
        Health(50.0), // Can be destroyed by gunfire too
    )).id()
}

pub enum ExplodableType {
    FuelBarrel,
    GasCanister,
    PowerCell,
}

// === ENHANCED HELPERS ===
fn spawn_explosion(
    commands: &mut Commands,
    position: Vec2,
    radius: f32,
    damage: f32,
    explosion_type: ExplosionType,
) {
    let (color, duration) = match explosion_type {
        ExplosionType::Grenade => (Color::srgba(1.0, 0.8, 0.0, 0.25), 2.0),
        ExplosionType::Vehicle => (Color::srgba(1.0, 0.5, 0.0, 0.25), 3.0),
        ExplosionType::TimeBomb => (Color::srgba(1.0, 0.3, 0.1, 0.25), 2.5),
        ExplosionType::Cascading => (Color::srgba(0.9, 0.7, 0.1, 0.25), 1.5),
    };
    
    commands.spawn((
        Explosion {
            radius,
            damage,
            duration,
            explosion_type,
        },
        Transform::from_translation(position.extend(1.0)),
        Sprite {
            color,
            custom_size: Some(Vec2::splat(radius * 2.0)),
            ..default()
        },
    ));
}

fn spawn_fire_text(commands: &mut Commands, position: Vec2, damage: f32) {
    commands.spawn((
        Text2d::new("🔥"),
        TextFont {
            font_size: 12.0,
            ..default()
        },
        TextColor(Color::srgb(1.0, 0.4, 0.0)),
        Transform::from_translation((position + Vec2::new(10.0, 20.0)).extend(100.0)),
        FloatingText {
            lifetime: 0.8,
            velocity: Vec2::new(0.0, 30.0),
        },
    ));
}



pub fn floating_text_system(
    mut text_query: Query<(Entity, &mut Transform, &mut FloatingText, &mut TextColor)>,
    mut commands: Commands,
    time: Res<Time>,
    game_mode: Res<GameMode>,
) {
    if game_mode.paused { return; }

    for (entity, mut transform, mut floating_text, mut text_color) in text_query.iter_mut() {
        floating_text.lifetime -= time.delta_secs();
        
        if floating_text.lifetime <= 0.0 {
            commands.entity(entity).despawn();
        } else {
            // Move text upward and fade
            transform.translation += floating_text.velocity.extend(0.0) * time.delta_secs();
            floating_text.velocity.y *= 0.95; // Slow down over time
            
            // Fade out
            let alpha = floating_text.lifetime;
            text_color.0.set_alpha(alpha);
        }
    }
}

// === GRENADE EXPLOSION HANDLER ===
pub fn handle_grenade_events(
    mut grenade_events: EventReader<GrenadeEvent>,
    mut commands: Commands,
) {
    for event in grenade_events.read() {
        spawn_explosion(
            &mut commands,
            event.target_pos,
            event.explosion_radius,
            event.damage,
            ExplosionType::Grenade,
        );
    }
}

// === VEHICLE EXPLOSION HANDLER ===
pub fn handle_vehicle_explosions(
    mut commands: Commands,
    mut vehicle_query: Query<(Entity, &Transform, &Vehicle, &Health), (With<Vehicle>, Added<Dead>)>,
) {
    for (entity, transform, vehicle, _) in vehicle_query.iter_mut() {
        spawn_explosion(
            &mut commands,
            transform.translation.truncate(),
            vehicle.explosion_radius(),
            vehicle.explosion_damage(),
            ExplosionType::Vehicle,
        );
        
        commands.entity(entity).despawn();
    }
}
