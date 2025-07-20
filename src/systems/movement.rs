// src/systems/movement.rs - Fixed core movement system (no physics)
use bevy::prelude::*;
use crate::core::*;

pub fn system(
    mut commands: Commands,
    mut action_events: EventReader<ActionEvent>,
    mut moveable_query: Query<(
        Entity, 
        &mut Transform, 
        &MovementSpeed, 
        Option<&Agent>,
        Option<&Enemy>
    )>,
    mut target_query: Query<&mut MoveTarget>,
    game_mode: Res<GameMode>,
    time: Res<Time>,
) {
    if game_mode.paused { return; }

    // Process movement action events
    for event in action_events.read() {
        if let Action::MoveTo(target_pos) = event.action {
            if let Ok((entity, transform, _speed, agent, enemy)) = moveable_query.get(event.entity) {
                let current_pos = transform.translation.truncate();
                let distance = current_pos.distance(target_pos);
                
                if distance > 5.0 {
                    // Add or update move target
                    if let Ok(mut move_target) = target_query.get_mut(event.entity) {
                        move_target.position = target_pos;
                    } else {
                        commands.entity(event.entity).insert(MoveTarget { position: target_pos });
                    }
                }
            }
        }
    }

    // Execute continuous movement for entities with move targets
    let mut entities_to_remove_target = Vec::new();
    
    for (entity, mut transform, speed, agent, enemy) in moveable_query.iter_mut() {
        if let Ok(move_target) = target_query.get(entity) {
            let current_pos = transform.translation.truncate();
            let direction = (move_target.position - current_pos).normalize_or_zero();
            let distance = current_pos.distance(move_target.position);

            if distance > 5.0 {
                // Move via direct transform update
                let movement = direction * speed.0 * time.delta_secs();
                transform.translation += movement.extend(0.0);
            } else {
                // Reached target
                entities_to_remove_target.push(entity);
            }
        }
    }
    
    // Remove completed move targets
    for entity in entities_to_remove_target {
        commands.entity(entity).remove::<MoveTarget>();
    }
}