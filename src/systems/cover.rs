// src/systems/cover.rs - Cover management systems
use bevy::prelude::*;
use crate::core::*;

// PLACEHOLDER - SHOULD BE IN MAP SYSTEM
pub fn spawn_cover_points(commands: &mut Commands) {
    let cover_positions = [
        Vec2::new(50.0, -50.0),   // Near terminals
        Vec2::new(250.0, -150.0), // Corner positions
        Vec2::new(-50.0, 100.0),  // Scattered around map
        Vec2::new(300.0, 50.0),
        Vec2::new(150.0, 150.0),
    ];
    
    for &pos in &cover_positions {
        commands.spawn((
            Sprite {
                    color: Color::srgba(0.4, 0.2, 0.1, 0.7), // Brown, semi-transparent
                    custom_size: Some(Vec2::new(20.0, 40.0)), // Rectangular cover
                    ..default()
                },
            Transform::from_translation(pos.extend(0.5)), // Slightly behind other objects
            CoverPoint {
                capacity: 2,
                current_users: 0,
                cover_direction: Vec2::X, // Covers from the right (will be calculated dynamically)
            },
        ));
    }
}

pub fn cover_management_system(
    mut cover_query: Query<(Entity, &mut CoverPoint)>,
    in_cover_query: Query<&InCover>,
) {
    // Reset cover usage counts
    for (_, mut cover_point) in cover_query.iter_mut() {
        cover_point.current_users = 0;
    }
    
    // Count current users
    for in_cover in in_cover_query.iter() {
        if let Ok((_, mut cover_point)) = cover_query.get_mut(in_cover.cover_entity) {
            cover_point.current_users += 1;
        }
    }
}

pub fn cover_exit_system(
    mut commands: Commands,
    in_cover_query: Query<(Entity, &InCover, &Transform), With<Enemy>>,
    cover_query: Query<&Transform, (With<CoverPoint>, Without<Enemy>)>,
) {
    for (enemy_entity, in_cover, enemy_transform) in in_cover_query.iter() {
        if let Ok(cover_transform) = cover_query.get(in_cover.cover_entity) {
            let distance = enemy_transform.translation.truncate()
                .distance(cover_transform.translation.truncate());
            
            // If enemy moved far from cover, remove InCover component
            if distance > 30.0 {
                commands.entity(enemy_entity).remove::<InCover>();
            }
        }
    }
}