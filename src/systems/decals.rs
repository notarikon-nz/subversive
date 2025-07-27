// src/systems/decals.rs - Enhanced death handling and decal system
// Future extensibility ideas:
// 1. **Weather Effects**: Rain washing away blood, snow covering decals
// 2. **Interactive Decals**: Oil spills that affect movement, explosive barrels
// 3. **Forensic System**: Decals as evidence for police investigation
// 4. **Environmental Storytelling**: Pre-placed decals that tell a story
// 5. **Dynamic Decals**: Footprints, tire tracks that fade over time
// 6. **Decal Layers**: Multiple decals can overlap (blood on scorch marks)
// 7. **Material-Specific**: Different decals for concrete, grass, metal surfaces
// 8. **Decal Scaling**: Size varies based on damage amount or weapon type
// 9. **Mission Cleanup**: Janitor NPCs that clean up decals over time
// 10. **Photo Mode**: Players can inspect detailed battle aftermath

use bevy::prelude::*;
use crate::core::*;
use crate::systems::explosions::*;

// === DECAL SYSTEM ===
#[derive(Component)]
pub struct Decal {
    pub decal_type: DecalType,
    pub fade_timer: Option<f32>, // None = permanent
    pub alpha: f32,
}

#[derive(Clone, Debug)]
pub enum DecalType {
    Blood,
    Scorch,
    BulletHole,
    Explosion,
    Tire,
    Oil,
}

#[derive(Resource)]
pub struct DecalSettings {
    pub max_decals: usize,
    pub cleanup_distance: f32, // Clean up decals beyond this distance from camera
    pub fade_enabled: bool,
    pub blood_fade_time: f32,
    pub scorch_fade_time: f32,
}

impl Default for DecalSettings {
    fn default() -> Self {
        Self {
            max_decals: 500,
            cleanup_distance: 2000.0,
            fade_enabled: false, // Persistent by default
            blood_fade_time: 300.0,  // 5 minutes if enabled
            scorch_fade_time: 600.0, // 10 minutes if enabled
        }
    }
}

// === UTILITY FUNCTIONS ===

/// Helper to spawn various decal types with common parameters
pub mod decal_helpers {
    use super::*;
    
    pub fn blood_splatter(commands: &mut Commands, position: Vec2, settings: &DecalSettings) {
        spawn_decal(commands, position, DecalType::Blood, 25.0, settings);
    }
    
    pub fn explosion_mark(commands: &mut Commands, position: Vec2, size: f32, settings: &DecalSettings) {
        spawn_decal(commands, position, DecalType::Explosion, size, settings);
    }
    
    pub fn bullet_impact(commands: &mut Commands, position: Vec2, settings: &DecalSettings) {
        spawn_decal(commands, position, DecalType::BulletHole, 6.0, settings);
    }
    
    pub fn tire_marks(commands: &mut Commands, position: Vec2, settings: &DecalSettings) {
        spawn_decal(commands, position, DecalType::Tire, 15.0, settings);
    }
    
    pub fn oil_spill(commands: &mut Commands, position: Vec2, size: f32, settings: &DecalSettings) {
        spawn_decal(commands, position, DecalType::Oil, size, settings);
    }
}

/// Call this from your projectile impact system to add bullet holes
pub fn add_bullet_impact_decal(
    commands: &mut Commands,
    impact_position: Vec2,
    decal_settings: &DecalSettings,
) {
    spawn_decal(
        commands,
        impact_position,
        DecalType::BulletHole,
        6.0,
        decal_settings,
    );
}

/// Call this from explosion systems
pub fn add_explosion_decal(
    commands: &mut Commands,
    explosion_position: Vec2,
    explosion_radius: f32,
    decal_settings: &DecalSettings,
) {
    let decal_size = explosion_radius * 1.2; // Slightly larger than explosion
    spawn_decal(
        commands,
        explosion_position,
        DecalType::Explosion,
        decal_size,
        decal_settings,
    );
}


// === DECAL SPAWNING ===

pub fn spawn_decal(
    commands: &mut Commands,
    position: Vec2,
    decal_type: DecalType,
    size: f32,
    settings: &DecalSettings,
) {
    let (color, z_order, fade_time) = match decal_type {
        DecalType::Blood => (
            Color::srgba(0.4, 0.1, 0.1, 0.8),
            -10.0, // Under everything
            if settings.fade_enabled { Some(settings.blood_fade_time) } else { None }
        ),
        DecalType::Scorch => (
            Color::srgba(0.1, 0.1, 0.1, 0.9),
            -9.0,
            if settings.fade_enabled { Some(settings.scorch_fade_time) } else { None }
        ),
        DecalType::BulletHole => (
            Color::srgba(0.2, 0.2, 0.2, 0.7),
            -8.0,
            if settings.fade_enabled { Some(120.0) } else { None }
        ),
        DecalType::Explosion => (
            Color::srgba(0.15, 0.1, 0.05, 0.8),
            -9.0,
            if settings.fade_enabled { Some(settings.scorch_fade_time) } else { None }
        ),
        DecalType::Tire => (
            Color::srgba(0.1, 0.1, 0.1, 0.6),
            -7.0,
            if settings.fade_enabled { Some(180.0) } else { None }
        ),
        DecalType::Oil => (
            Color::srgba(0.05, 0.05, 0.1, 0.7),
            -8.0,
            if settings.fade_enabled { Some(300.0) } else { None }
        ),
    };

    commands.spawn((
        Sprite {
            color,
            custom_size: Some(Vec2::splat(size)),
            ..default()
        },
        Transform::from_translation(position.extend(z_order)),
        Decal {
            decal_type,
            fade_timer: fade_time,
            alpha: color.alpha(),
        },
    ));
}

// === DECAL MANAGEMENT ===

pub fn decal_fade_system(
    mut decal_query: Query<(Entity, &mut Decal, &mut Sprite)>,
    mut commands: Commands,
    time: Res<Time>,
    game_mode: Res<GameMode>,
) {
    if game_mode.paused { return; }

    for (entity, mut decal, mut sprite) in decal_query.iter_mut() {
        if let Some(ref mut timer) = decal.fade_timer {
            *timer -= time.delta_secs();
            
            if *timer <= 0.0 {
                commands.entity(entity).insert(MarkedForDespawn);
            } else if *timer < 60.0 { // Start fading in last minute
                let fade_factor = (*timer / 60.0).clamp(0.0, 1.0);
                sprite.color.set_alpha(decal.alpha * fade_factor);
            }
        }
    }
}

pub fn decal_cleanup_system(
    decal_query: Query<(Entity, &Transform), With<Decal>>,
    camera_query: Query<&Transform, (With<Camera>, Without<Decal>)>,
    mut commands: Commands,
    settings: Res<DecalSettings>,
) {
    let Ok(camera_transform) = camera_query.single() else { return; };
    let camera_pos = camera_transform.translation.truncate();
    
    let decal_count = decal_query.iter().count();
    
    // If we're over the limit, clean up distant decals
    if decal_count > settings.max_decals {
        let mut distant_decals: Vec<_> = decal_query
            .iter()
            .map(|(entity, transform)| {
                let distance = camera_pos.distance(transform.translation.truncate());
                (entity, distance)
            })
            .collect();
        
        // Sort by distance (farthest first)
        distant_decals.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        
        // Remove excess decals
        let to_remove = decal_count - settings.max_decals + 50; // Remove extra for buffer
        for (entity, _) in distant_decals.iter().take(to_remove) {
            commands.entity(*entity).insert(MarkedForDespawn);
        }
    }
    
    // Clean up very distant decals regardless of count
    for (entity, transform) in decal_query.iter() {
        let distance = camera_pos.distance(transform.translation.truncate());
        if distance > settings.cleanup_distance {
            commands.entity(entity).insert(MarkedForDespawn);
        }
    }
}


// === ADDITIONAL DECAL TRIGGERS ===

/// System to add bullet hole decals when projectiles hit walls/objects
pub fn projectile_impact_decals(
    mut commands: Commands,
    impact_query: Query<&Transform, (With<ProjectileImpact>, Added<ProjectileImpact>)>,
    settings: Res<DecalSettings>,
) {
    for transform in impact_query.iter() {
        spawn_decal(
            &mut commands,
            transform.translation.truncate(),
            DecalType::BulletHole,
            8.0,
            &settings,
        );
    }
}

/// System to add scorch decals for explosions
pub fn explosion_scorch_decals(
    mut commands: Commands,
    explosion_query: Query<(&Transform, &Explosion), Added<Explosion>>,
    settings: Res<DecalSettings>,
) {
    for (transform, explosion) in explosion_query.iter() {
        let scorch_size = explosion.radius * 1.2; // Slightly larger than explosion
        spawn_decal(
            &mut commands,
            transform.translation.truncate(),
            DecalType::Scorch,
            scorch_size,
            &settings,
        );
    }
}
