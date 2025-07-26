// src/systems/morale.rs
use bevy::prelude::*;
use crate::core::*;
use crate::systems::*;

pub fn morale_system(
    mut enemy_query: Query<(Entity, &Transform, &mut Morale, &mut AIState, &mut GoapAgent), (With<Enemy>, Without<Dead>)>,
    mut commands: Commands,
    mut combat_events: EventReader<CombatEvent>,
    mut audio_events: EventReader<AudioEvent>,
    agent_query: Query<&Transform, With<Agent>>,
    ally_query: Query<&Transform, (With<Enemy>, Without<Dead>)>,
    time: Res<Time>,
    game_mode: Res<GameMode>,
) {
    if game_mode.paused { return; }

    // Process combat morale effects
    for combat_event in combat_events.read() {
        if combat_event.hit {
            // Get target position first
            let target_pos = if let Ok((_, transform, _, _, _)) = enemy_query.get(combat_event.target) {
                Some(transform.translation.truncate())
            } else {
                None
            };
            
            if let Some(target_pos) = target_pos {
                let target_entity = combat_event.target;
                
                for (entity, enemy_transform, mut morale, mut ai_state, mut goap_agent) in enemy_query.iter_mut() {
                    let distance = enemy_transform.translation.truncate().distance(target_pos);
                    
                    if distance <= 100.0 {
                        let damage_factor = if entity == target_entity {
                            20.0 // Direct hit
                        } else {
                            5.0 // Witness
                        };
                        
                        morale.reduce(damage_factor);
                        
                        if morale.is_panicked() && !matches!(ai_state.mode, AIMode::Panic) {
                            ai_state.mode = AIMode::Panic;
                            goap_agent.update_world_state(WorldKey::IsPanicked, true);
                            goap_agent.abort_plan();
                        }
                    }
                }
            }
        }
    }

    // Process gunshot morale effects for civilians
    for audio_event in audio_events.read() {
        if matches!(audio_event.sound, AudioType::Gunshot) {
            // Handle civilian morale in separate system to avoid query conflicts
        }
    }

    // Update enemy morale and panic states
    for (entity, enemy_transform, mut morale, mut ai_state, mut goap_agent) in enemy_query.iter_mut() {
        let enemy_pos = enemy_transform.translation.truncate();
        
        // Check if surrounded by agents
        let nearby_agents = agent_query.iter()
            .filter(|agent_transform| {
                enemy_pos.distance(agent_transform.translation.truncate()) <= 150.0
            })
            .count();
        
        let nearby_allies = ally_query.iter()
            .filter(|ally_transform| {
                enemy_pos.distance(ally_transform.translation.truncate()) <= 150.0
            })
            .count();

        if nearby_agents > nearby_allies + 1 {
            morale.reduce(10.0 * time.delta_secs());
        } else {
            morale.recover(time.delta_secs());
        }

        // Handle panic state transitions
        if morale.is_panicked() && !matches!(ai_state.mode, AIMode::Panic) {
            ai_state.mode = AIMode::Panic;
            goap_agent.update_world_state(WorldKey::IsPanicked, true);
            goap_agent.abort_plan();
            
            let flee_direction = if let Some(agent_transform) = agent_query.iter().next() {
                (enemy_pos - agent_transform.translation.truncate()).normalize_or_zero()
            } else {
                Vec2::new(1.0, 0.0)
            };
            
            commands.entity(entity).insert(FleeTarget {
                destination: enemy_pos + flee_direction * 200.0,
                flee_speed_multiplier: 1.5,
            });
        } else if !morale.is_panicked() && matches!(ai_state.mode, AIMode::Panic) {
            ai_state.mode = AIMode::Patrol;
            goap_agent.update_world_state(WorldKey::IsPanicked, false);
            commands.entity(entity).remove::<FleeTarget>();
        }
    }
}

pub fn civilian_morale_system(
    mut civilian_query: Query<(Entity, &Transform, &mut Morale), (With<Civilian>, Without<FleeTarget>, Without<MarkedForDespawn>)>,
    mut commands: Commands,
    mut audio_events: EventReader<AudioEvent>,
    agent_query: Query<&Transform, With<Agent>>,
    time: Res<Time>,
    game_mode: Res<GameMode>,
) {
    if game_mode.paused { return; }

    // Process gunshot morale effects for civilians
    for audio_event in audio_events.read() {
        if matches!(audio_event.sound, AudioType::Gunshot) {
            for (_, _, mut morale) in civilian_query.iter_mut() {
                morale.reduce(15.0);
            }
        }
    }

    // Trigger civilian flee behavior
    for (entity, civilian_transform, morale) in civilian_query.iter() {
        if morale.is_panicked() {
            let civilian_pos = civilian_transform.translation.truncate();
            
            let flee_direction = if let Some(agent_transform) = agent_query.iter().next() {
                (civilian_pos - agent_transform.translation.truncate()).normalize_or_zero()
            } else {
                Vec2::new(rand::random::<f32>() - 0.5, rand::random::<f32>() - 0.5).normalize_or_zero()
            };
            
            commands.entity(entity).insert(FleeTarget {
                destination: civilian_pos + flee_direction * 300.0,
                flee_speed_multiplier: 2.0,
            });
        }
    }
}

pub fn flee_system(
    mut flee_query: Query<(Entity, &mut Transform, &MovementSpeed, &FleeTarget)>,
    mut commands: Commands,
    time: Res<Time>,
    game_mode: Res<GameMode>,
) {
    if game_mode.paused { return; }

    for (entity, mut transform, speed, flee_target) in flee_query.iter_mut() {
        let current_pos = transform.translation.truncate();
        let direction = (flee_target.destination - current_pos).normalize_or_zero();
        let distance = current_pos.distance(flee_target.destination);

        if distance > 10.0 {
            let flee_speed = speed.0 * flee_target.flee_speed_multiplier;
            let movement = direction * flee_speed * time.delta_secs();
            transform.translation += movement.extend(0.0);
        } else {
            commands.entity(entity).remove::<FleeTarget>();
        }
    }
}