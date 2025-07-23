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
                        // check entity exists
                        if let Ok(mut entity_commands) = commands.get_entity(event.entity) {
                            entity_commands.insert(MoveTarget { position: target_pos });
                        }
                    }
                }
            }
        }
    }

    // Execute continuous movement and handle patrol advancement
    let mut entities_to_remove_target = Vec::new();
    let mut patrol_updates = Vec::new(); // Store patrol updates separately

    // Phase 1: Move entities toward their targets
    for (entity, mut transform, speed, agent, enemy, patrol_opt) in moveable_query.iter_mut() {

        // Skip if no move target
        let Ok(move_target) = target_query.get(entity) else { continue; };

        let current_pos = transform.translation.truncate();
        let direction = (move_target.position - current_pos).normalize_or_zero();
        let distance = current_pos.distance(move_target.position);

        if distance > 5.0 {
            // Still moving - update position
            let movement = direction * speed.0 * time.delta_secs();
            transform.translation += movement.extend(0.0);
        } else {
            // Reached target - handle completion
            let is_enemy = enemy.is_some();
            let has_patrol = patrol_opt.is_some();
            
            if is_enemy && has_patrol {
                // Enemy reached patrol point - schedule patrol update
                patrol_updates.push(entity);
            } else {
                // Non-patrolling entity reached target - remove move target
                entities_to_remove_target.push(entity);
            }
        }
    }

    // Phase 2: Handle patrol updates separately
    for entity in patrol_updates {
        // Safely get patrol data
        let Ok((_, _, _, _, _, Some(mut patrol))) = moveable_query.get_mut(entity) else { continue; };
        
        patrol.advance();
        
        if let Some(next_target) = patrol.current_target() {
            // Try to update existing move target
            if let Ok(mut move_target) = target_query.get_mut(entity) {
                move_target.position = next_target;
            } else {
                // No existing move target - this shouldn't happen but handle it
                warn!("Patrol entity {} has no move target", entity.index());
                if let Ok(mut entity_commands) = commands.get_entity(entity) {
                    entity_commands.insert(MoveTarget { position: next_target });
                }
            }
        } else {
            // No patrol target available - remove move target
            entities_to_remove_target.push(entity);
        }
    }

    // Phase 3: Collect entities needing patrol (no insertions yet)
    let mut entities_needing_patrol = Vec::new();

    for (entity, _, _, _, enemy, patrol_opt) in moveable_query.iter() {
        // Only check enemies without existing move targets
        if enemy.is_none() { continue; }
        
        if target_query.get(entity).is_ok() { continue; } // Already has move target
        
        if let Some(patrol) = patrol_opt {
            if let Some(patrol_target) = patrol.current_target() {
                // info!("Entity {} needs patrol target: {:?}", entity.index(), patrol_target);
                entities_needing_patrol.push((entity, patrol_target));
            }
        }
    }

    // Phase 4: Remove completed targets FIRST
    for entity in entities_to_remove_target {
        commands.entity(entity).remove::<MoveTarget>();
    }

    entities_needing_patrol.retain(|(entity, _)| target_query.get(*entity).is_ok());

    // Phase 5: NOW insert new patrol targets (after target_query operations are done)
    for (entity, patrol_target) in entities_needing_patrol {
        if target_query.get(entity).is_ok() {
            // Check entity still exists before inserting
            if let Ok(mut entity_commands) = commands.get_entity(entity) {
                entity_commands.insert(MoveTarget { position: patrol_target });
            }
        }
    }

}