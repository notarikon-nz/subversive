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

// To adjust settings, access CombatTextSettings resource:
// combat_text_settings.enabled = false; // Turn off
// combat_text_settings.font_size = 20.0; // Make bigger

impl Default for CombatTextSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            font_size: 16.0,
        }
    }
}

// === EXPLOSION SYSTEMS ===
pub fn explosion_damage_system(
    mut explosion_query: Query<(Entity, &mut Explosion, &Transform)>,
    mut damageable_query: Query<(&Transform, &mut Health), (Without<Explosion>, Without<Dead>)>,
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
        };
        
        explosion.duration -= time.delta_secs();
        
        // Deal damage only once when explosion starts
        if is_new {
            let explosion_pos = explosion_transform.translation.truncate();
            
            for (target_transform, mut health) in damageable_query.iter_mut() {
                let target_pos = target_transform.translation.truncate();
                let distance = explosion_pos.distance(target_pos);
                
                if distance <= explosion.radius {
                    let damage_factor = (1.0 - (distance / explosion.radius)).max(0.1);
                    let damage = explosion.damage * damage_factor;
                    
                    health.0 = (health.0 - damage).max(0.0);
                    
                    // Spawn floating combat text
                    if combat_text_settings.enabled {
                        spawn_damage_text(&mut commands, target_pos, damage, &combat_text_settings);
                    }
                }
            }
            
            // Play explosion sound
            audio_events.write(AudioEvent {
                sound: AudioType::Alert, // Reuse for explosion
                volume: 1.0,
            });
        }
        
        // Remove expired explosions
        if explosion.duration <= 0.0 {
            commands.entity(explosion_entity).despawn();
        }
    }
}

pub fn floating_text_system(
    mut text_query: Query<(Entity, &mut Transform, &mut FloatingText)>,
    mut commands: Commands,
    time: Res<Time>,
    game_mode: Res<GameMode>,
) {
    if game_mode.paused { return; }

    for (entity, mut transform, mut floating_text) in text_query.iter_mut() {
        floating_text.lifetime -= time.delta_secs();
        
        if floating_text.lifetime <= 0.0 {
            commands.entity(entity).despawn();
        } else {
            // Move text upward and fade
            transform.translation += floating_text.velocity.extend(0.0) * time.delta_secs();
            floating_text.velocity.y *= 0.95; // Slow down over time
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

// === HELPER FUNCTIONS ===
fn spawn_explosion(
    commands: &mut Commands,
    position: Vec2,
    radius: f32,
    damage: f32,
    explosion_type: ExplosionType,
) {
    let (color, duration) = match explosion_type {
        ExplosionType::Grenade => (Color::srgba(1.0, 0.8, 0.0, 0.7), 2.0),
        ExplosionType::Vehicle => (Color::srgba(1.0, 0.5, 0.0, 0.8), 3.0),
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

fn spawn_damage_text(
    commands: &mut Commands,
    position: Vec2,
    damage: f32,
    settings: &CombatTextSettings,
) {
    let damage_text = format!("{:.0}", damage);
    let text_color = if damage >= 50.0 {
        Color::srgb(1.0, 0.2, 0.2) // High damage = red
    } else if damage >= 25.0 {
        Color::srgb(1.0, 0.8, 0.2) // Medium damage = orange
    } else {
        Color::srgb(1.0, 1.0, 0.2) // Low damage = yellow
    };
    
    commands.spawn((
        Text::new(damage_text),
        TextFont {
            font_size: settings.font_size,
            ..default()
        },
        TextColor(text_color),
        Node {
            position_type: PositionType::Absolute,
            ..default()
        },
        Transform::from_translation((position + Vec2::new(0.0, 30.0)).extend(100.0)),
        FloatingText {
            lifetime: 1.0,
            velocity: Vec2::new(
                (rand::random::<f32>() - 0.5) * 20.0, // Random horizontal drift
                50.0, // Upward movement
            ),
        },
    ));
}