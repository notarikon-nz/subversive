// src/systems/health_bars.rs - Efficient health bar rendering

use bevy::prelude::*;
use crate::core::*;

#[derive(Component)]
pub struct HealthBar {
    pub max_health: f32,
}

#[derive(Component)]
pub struct HealthBarFill;

#[derive(Component)]
pub struct HealthBarBackground;

// Only show health bars for damaged entities
pub fn spawn_health_bar_system(
    mut commands: Commands,
    query: Query<(Entity, &Health), (Without<HealthBar>, Without<Dead>)>,
) {
    for (entity, health) in query.iter() {
        // Only show health bar if damaged (assuming 100.0 is max health)
        if health.0 < 100.0 && health.0 > 0.0 {
            let max_health = 100.0;
            
            // Spawn health bar as child of the entity
            let health_bar = commands.spawn((
                Sprite {
                        color: Color::srgb(0.2, 0.2, 0.2),
                        custom_size: Some(Vec2::new(32.0, 4.0)),
                        ..default()
                    },
                Transform::from_translation(Vec3::new(0.0, 25.0, 0.1)),
                HealthBarBackground,
            )).with_children(|parent| {
                parent.spawn((
                    Sprite {
                            color: Color::srgb(0.2, 0.8, 0.2),
                            custom_size: Some(Vec2::new(32.0, 4.0)),
                            ..default()
                        },
                    Transform::from_translation(Vec3::new(0.0, 0.0, 0.1)),
                    HealthBarFill,
                ));
            }).id();
            
            // Add health bar component to the original entity
            commands.entity(entity)
                .insert(HealthBar { max_health })
                .add_child(health_bar);
        }
    }
}

pub fn update_health_bars_system(
    mut health_bar_query: Query<(&mut Transform, &mut Sprite), (With<HealthBarFill>, Without<Health>)>,
    health_query: Query<(&Health, &HealthBar, &Children), Changed<Health>>,
    background_query: Query<&Children, With<HealthBarBackground>>,
) {
    for (health, health_bar, children) in health_query.iter() {
        // Find the health bar background
        for child in children.iter() {
            if let Ok(bg_children) = background_query.get(child) {
                // Find the fill bar
                for fill_child in bg_children.iter() {
                    if let Ok((mut transform, mut sprite)) = health_bar_query.get_mut(fill_child) {
                        let ratio = (health.0 / health_bar.max_health).clamp(0.0, 1.0);
                        
                        // Update fill width
                        if let Some(ref mut size) = sprite.custom_size {
                            size.x = 32.0 * ratio;
                        }
                        
                        // Update fill position (anchor to left)
                        transform.translation.x = -16.0 + (16.0 * ratio);
                        
                        // Update color based on health ratio
                        sprite.color = match ratio {
                            r if r > 0.6 => Color::srgb(0.2, 0.8, 0.2),
                            r if r > 0.3 => Color::srgb(0.8, 0.8, 0.2),
                            _ => Color::srgb(0.8, 0.2, 0.2),
                        };
                        
                        break;
                    }
                }
                break;
            }
        }
    }
}

// Clean up health bars when entities die
pub fn cleanup_health_bars_system(
    mut commands: Commands,
    dead_query: Query<(Entity, &Children), (With<Dead>, With<HealthBar>)>,
    health_bar_query: Query<Entity, With<HealthBarBackground>>,
) {
    for (entity, children) in dead_query.iter() {
        // Remove health bar components
        for child in children.iter() {
            if health_bar_query.contains(child) {
                commands.entity(child).despawn();
            }
        }
        commands.entity(entity).remove::<HealthBar>();
    }
}