// src/systems/ui/world.rs - Just the gizmos, simplified
use bevy::prelude::*;
use crate::core::*;

pub fn system(
    mut gizmos: Gizmos,
    game_mode: Res<GameMode>,
    selected_query: Query<(&Transform, &NeurovectorCapability), (With<Agent>, With<Selected>)>,
    all_selected_query: Query<&Transform, (With<Agent>, With<Selected>)>,
    target_query: Query<&Transform, With<NeurovectorTarget>>,
    controlled_query: Query<&Transform, With<NeurovectorControlled>>,
    enemy_query: Query<(&Transform, &Vision), With<Enemy>>,
    neurovector_query: Query<(&Transform, &NeurovectorCapability), With<Agent>>,
    selection: Res<SelectionState>,
) {
    // Selection indicators
    for transform in all_selected_query.iter() {
        let pos = transform.translation.truncate();
        gizmos.circle_2d(pos, 18.0, Color::srgb(0.2, 0.8, 0.2));
    }

    // Neurovector ranges for selected agents
    for (transform, neurovector) in selected_query.iter() {
        let color = if neurovector.current_cooldown > 0.0 {
            Color::srgba(0.8, 0.3, 0.3, 0.3)
        } else {
            Color::srgba(0.3, 0.3, 0.8, 0.3)
        };
        gizmos.circle_2d(transform.translation.truncate(), neurovector.range, color);
    }

    // Formation lines when multiple selected
    if selection.selected.len() > 1 {
        let positions: Vec<Vec2> = all_selected_query.iter()
            .map(|t| t.translation.truncate())
            .collect();
        
        for i in 0..positions.len() {
            for j in (i + 1)..positions.len() {
                gizmos.line_2d(positions[i], positions[j], Color::srgba(0.8, 0.8, 0.2, 0.3));
            }
        }
    }

    // Neurovector targeting highlights
    if let Some(TargetingMode::Neurovector { agent }) = &game_mode.targeting {
        if let Ok((agent_transform, neurovector)) = neurovector_query.get(*agent) {
            for target_transform in target_query.iter() {
                let distance = agent_transform.translation.truncate()
                    .distance(target_transform.translation.truncate());
                
                if distance <= neurovector.range {
                    gizmos.circle_2d(target_transform.translation.truncate(), 20.0, Color::srgb(0.8, 0.8, 0.3));
                }
            }
        }
    }

    // Control connections
    for (agent_transform, neurovector) in neurovector_query.iter() {
        for &controlled_entity in &neurovector.controlled {
            if let Ok(controlled_transform) = controlled_query.get(controlled_entity) {
                gizmos.line_2d(
                    agent_transform.translation.truncate(),
                    controlled_transform.translation.truncate(),
                    Color::srgb(0.8, 0.3, 0.8),
                );
            }
        }
    }

    // Enemy vision cones (simplified - just 8 segments, max 5 enemies)
    for (i, (transform, vision)) in enemy_query.iter().enumerate() {
        if i >= 5 { break; } // Simple LOD
        draw_vision_cone(&mut gizmos, transform.translation.truncate(), vision);
    }
}

fn draw_vision_cone(gizmos: &mut Gizmos, position: Vec2, vision: &Vision) {
    let half_angle = vision.angle / 2.0;
    let color = Color::srgba(1.0, 1.0, 0.3, 0.2);
    
    // Just 8 segments instead of 16
    for i in 0..8 {
        let t1 = i as f32 / 8.0;
        let t2 = (i + 1) as f32 / 8.0;
        
        let angle1 = -half_angle + (vision.angle * t1);
        let angle2 = -half_angle + (vision.angle * t2);
        
        let dir1 = Vec2::new(
            vision.direction.x * angle1.cos() - vision.direction.y * angle1.sin(),
            vision.direction.x * angle1.sin() + vision.direction.y * angle1.cos(),
        );
        
        let dir2 = Vec2::new(
            vision.direction.x * angle2.cos() - vision.direction.y * angle2.sin(),
            vision.direction.x * angle2.sin() + vision.direction.y * angle2.cos(),
        );
        
        let point1 = position + dir1 * vision.range;
        let point2 = position + dir2 * vision.range;
        
        gizmos.line_2d(point1, point2, color);
    }
    
    // Draw cone edges
    let left_dir = Vec2::new(
        vision.direction.x * half_angle.cos() - vision.direction.y * half_angle.sin(),
        vision.direction.x * half_angle.sin() + vision.direction.y * half_angle.cos(),
    );
    
    let right_dir = Vec2::new(
        vision.direction.x * half_angle.cos() + vision.direction.y * half_angle.sin(),
        -vision.direction.x * half_angle.sin() + vision.direction.y * half_angle.cos(),
    );
    
    gizmos.line_2d(position, position + left_dir * vision.range, color);
    gizmos.line_2d(position, position + right_dir * vision.range, color);
}