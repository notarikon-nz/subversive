use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use crate::core::*;
use crate::systems::*;

pub fn system(
    mut commands: Commands,
    mut action_events: EventReader<ActionEvent>,
    mut controllable_query: Query<(Entity, &mut Velocity, &MovementSpeed, &Transform), (With<Controllable>, Without<Enemy>)>,
    mut target_query: Query<&mut MoveTarget>,
    mut enemy_query: Query<(Entity, &mut Patrol, &Transform, &mut Velocity, &MovementSpeed, &AIState, &mut Vision), (With<Enemy>, Without<Dead>)>,
    game_mode: Res<GameMode>,
    time: Res<Time>,
) {
    if game_mode.paused { return; }

    // Handle movement actions for controllable entities
    for event in action_events.read() {
        if let Action::MoveTo(target_pos) = event.action {
            if let Ok((_, mut velocity, speed, transform)) = controllable_query.get_mut(event.entity) {
                if let Ok(mut move_target) = target_query.get_mut(event.entity) {
                    move_target.position = target_pos;
                } else {
                    commands.entity(event.entity).insert(MoveTarget { position: target_pos });
                }
            }
        }
    }

    // Execute movement for controllable entities (agents and controlled civilians)
    let mut entities_to_remove_target = Vec::new();
    
    for (entity, mut velocity, speed, transform) in controllable_query.iter_mut() {
        if let Ok(move_target) = target_query.get(entity) {
            let current_pos = transform.translation.truncate();
            let direction = (move_target.position - current_pos).normalize_or_zero();
            let distance = current_pos.distance(move_target.position);

            if distance > 5.0 {
                velocity.linvel = direction * speed.0;
            } else {
                velocity.linvel = Vec2::ZERO;
                entities_to_remove_target.push(entity);
            }
        }
    }
    
    // Remove completed move targets
    for entity in entities_to_remove_target {
        commands.entity(entity).remove::<MoveTarget>();
    }

    // Handle enemy movement based on AI state
    for (entity, mut patrol, transform, mut velocity, speed, ai_state, mut vision) in enemy_query.iter_mut() {
        let current_pos = transform.translation.truncate();
        
        match &ai_state.mode {
            AIMode::Patrol => {
                // Normal patrol behavior
                if let Some(target) = patrol.current_target() {
                    let direction = (target - current_pos).normalize_or_zero();
                    let distance = current_pos.distance(target);

                    if distance > 10.0 {
                        velocity.linvel = direction * speed.0;
                        // Update vision direction to face movement
                        if direction != Vec2::ZERO {
                            vision.direction = direction;
                        }
                    } else {
                        patrol.advance();
                        velocity.linvel = Vec2::ZERO;
                    }
                }
            },
            
            AIMode::Combat { target: _ } => {
                // Check if we have a move target from AI
                if let Ok(move_target) = target_query.get(entity) {
                    let direction = (move_target.position - current_pos).normalize_or_zero();
                    let distance = current_pos.distance(move_target.position);

                    if distance > 15.0 {
                        velocity.linvel = direction * (speed.0 * 1.5); // Move faster in combat
                        // Update vision direction to face movement
                        if direction != Vec2::ZERO {
                            vision.direction = direction;
                        }
                    } else {
                        velocity.linvel = Vec2::ZERO;
                        commands.entity(entity).remove::<MoveTarget>();
                    }
                } else {
                    // No move target in combat - stand still
                    velocity.linvel = Vec2::ZERO;
                }
            },
            
            AIMode::Investigate { location } => {
                let direction = (*location - current_pos).normalize_or_zero();
                let distance = current_pos.distance(*location);

                if distance > 20.0 {
                    velocity.linvel = direction * (speed.0 * 1.2);
                    // Update vision direction to face movement
                    if direction != Vec2::ZERO {
                        vision.direction = direction;
                    }
                } else {
                    velocity.linvel = Vec2::ZERO;
                }
            },
            
            AIMode::Search { area: _ } => {
                velocity.linvel = Vec2::ZERO;
            },
        }
    }
}