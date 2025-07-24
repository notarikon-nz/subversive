// src/systems/health_bars.rs - Optimized health bar rendering

use bevy::prelude::*;
use crate::core::*;

#[derive(Component)]
pub struct HealthBar {
    pub max_health: f32,
    pub fill: Entity,
}

const BAR_SIZE: Vec2 = Vec2::new(32.0, 4.0);
const BAR_OFFSET: Vec3 = Vec3::new(0.0, 25.0, 0.1);

// Spawn health bars when damaged
pub fn spawn_health_bars(
    mut commands: Commands,
    query: Query<(Entity, &Health), (Without<HealthBar>, Changed<Health>)>,
) {
    for (entity, health) in query.iter() {
        if health.0 < 100.0 && health.0 > 0.0 {
            let ratio = health.0 / 100.0;
            
            // Create fill entity first
            let fill = commands.spawn((
                Sprite {
                    color: health_color(ratio),
                    custom_size: Some(Vec2::new(BAR_SIZE.x * ratio, BAR_SIZE.y)),
                    anchor: bevy::sprite::Anchor::CenterLeft,
                    ..default()
                },
                Transform::from_translation(
                    BAR_OFFSET + Vec3::new(-BAR_SIZE.x * 0.5, 0.0, 0.1)
                ),
            )).id();
            
            // Background
            let bg = commands.spawn((
                Sprite {
                    color: Color::srgb(0.2, 0.2, 0.2),
                    custom_size: Some(BAR_SIZE),
                    ..default()
                },
                Transform::from_translation(BAR_OFFSET),
            )).id();
            
            // Add component and children to entity
            commands.entity(entity)
                .insert(HealthBar { max_health: 100.0, fill })
                .add_child(bg)
                .add_child(fill);
        }
    }
}

// Update and cleanup in one system
pub fn update_health_bars(
    mut commands: Commands,
    mut sprites: Query<&mut Sprite>,
    query: Query<(Entity, &Health, &HealthBar, &Children), Changed<Health>>,
) {
    for (entity, health, bar, children) in query.iter() {
        // Remove if dead or healed
        if health.0 <= 0.0 || health.0 >= 100.0 {
            for child in children.iter() {
                commands.entity(child).despawn();
            }
            commands.entity(entity).remove::<HealthBar>();
            continue;
        }
        
        // Update fill bar
        if let Ok(mut sprite) = sprites.get_mut(bar.fill) {
            let ratio = (health.0 / bar.max_health).clamp(0.0, 1.0);
            sprite.custom_size = Some(Vec2::new(BAR_SIZE.x * ratio, BAR_SIZE.y));
            sprite.color = health_color(ratio);
        }
    }
}

// Cleanup dead entities
pub fn cleanup_dead_health_bars(
    mut commands: Commands,
    query: Query<(Entity, &Children), (With<HealthBar>, With<Dead>)>,
) {
    for (entity, children) in query.iter() {
        for child in children.iter() {
            commands.entity(child).despawn();
        }
        commands.entity(entity).remove::<HealthBar>();
    }
}

#[inline]
fn health_color(ratio: f32) -> Color {
    if ratio > 0.6 { Color::srgb(0.2, 0.8, 0.2) }
    else if ratio > 0.3 { Color::srgb(0.8, 0.8, 0.2) }
    else { Color::srgb(0.8, 0.2, 0.2) }
}