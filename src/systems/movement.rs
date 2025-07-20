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
        Option<&Enemy>,
        Option<&mut Patrol>,
    )>,
    mut target_query: Query<&mut MoveTarget>,
    game_mode: Res<GameMode>,
    time: Res<Time>,
) {
    if game_mode.paused { return; }

    // Process movement action events
    for event in action_events.read() {
        if let Action::MoveTo(target_pos) = event.action {
            if let Ok((entity, transform, _speed, agent, enemy, _)) = moveable_query.get(event.entity) {
                let current_pos = transform.translation.truncate();
                let distance = current_pos.distance(target_pos);
                
                if distance > 5.0 {
                    if let Ok(mut move_target) = target_query.get_mut(event.entity) {
                        move_target.position = target_pos;
                    } else {
                        commands.entity(event.entity).insert(MoveTarget { position: target_pos });
                    }
                }
            }
        }
    }

    // Execute continuous movement and handle patrol advancement
    let mut entities_to_remove_target = Vec::new();
    
    for (entity, mut transform, speed, agent, enemy, patrol_opt) in moveable_query.iter_mut() {
        if let Ok(move_target) = target_query.get(entity) {
            let current_pos = transform.translation.truncate();
            let direction = (move_target.position - current_pos).normalize_or_zero();
            let distance = current_pos.distance(move_target.position);

            if distance > 5.0 {
                let movement = direction * speed.0 * time.delta_secs();
                transform.translation += movement.extend(0.0);
            } else {
                // Reached target - handle patrol advancement for enemies
                if enemy.is_some() && patrol_opt.is_some() {
                    if let Some(mut patrol) = patrol_opt {
                        patrol.advance();
                        if let Some(next_target) = patrol.current_target() {
                            // Update move target to next patrol point
                            if let Ok(mut move_target) = target_query.get_mut(entity) {
                                move_target.position = next_target;
                                continue; // Don't remove target, continue patrolling
                            }
                        }
                    }
                }
                entities_to_remove_target.push(entity);
            }
        } else if enemy.is_some() && patrol_opt.is_some() {
            // Enemy without move target - start patrolling
            if let Some(patrol) = patrol_opt {
                if let Some(patrol_target) = patrol.current_target() {
                    commands.entity(entity).insert(MoveTarget { position: patrol_target });
                }
            }
        }
    }
    
    // Remove completed move targets
    for entity in entities_to_remove_target {
        commands.entity(entity).remove::<MoveTarget>();
    }
}