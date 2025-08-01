// src/systems/ai.rs - Updated to integrate with GOAP
use bevy::prelude::*;
use crate::core::*;
use crate::systems::death::*;

#[derive(Component)]
pub struct AIState {
    pub mode: AIMode,
    pub last_known_target: Option<Vec2>,
    pub investigation_timer: f32,
    pub alert_cooldown: f32,
    pub use_goap: bool, // Toggle between GOAP and legacy AI
    
    pub current_target: Option<Entity>,
    pub target_last_seen: f32,
}

#[derive(Debug, Clone)]
pub enum AIMode {
    Patrol,
    Investigate { location: Vec2 },
    Combat { target: Entity },
    Search { area: Vec2 },
    Panic,
}

impl Default for AIState {
    fn default() -> Self {
        Self {
            mode: AIMode::Patrol,
            last_known_target: None,
            current_target: None,      // Initialize as None
            target_last_seen: 0.0,    // Initialize as 0            
            investigation_timer: 0.0,
            alert_cooldown: 0.0,
            use_goap: true, // Enable GOAP by default for new enemies
        }
    }
}

// Alert coordination system for CallForHelp
pub fn alert_system(
    mut alert_events: EventReader<AlertEvent>,
    mut enemy_query: Query<(Entity, &Transform, &mut AIState, &mut GoapAgent), (With<Enemy>, Without<Dead>)>,
) {
    for alert_event in alert_events.read() {
        for (enemy_entity, enemy_transform, mut ai_state, mut goap_agent) in enemy_query.iter_mut() {
            // Skip the alerter itself
            if enemy_entity == alert_event.alerter {
                continue;
            }
            
            let distance = enemy_transform.translation.truncate().distance(alert_event.position);
            
            // Alert enemies within 200 units for CallForHelp
            let alert_range = match alert_event.alert_type {
                AlertType::CallForHelp => 200.0,
                AlertType::GunshotHeard => 150.0,
                AlertType::EnemySpotted => 250.0,
                _ => 200.0,
            };
            
            if distance <= alert_range {
                match alert_event.alert_type {
                    AlertType::CallForHelp => {
                        // Set last known target to the alert position
                        ai_state.last_known_target = Some(alert_event.position);
                        ai_state.mode = crate::systems::ai::AIMode::Investigate { 
                            location: alert_event.position 
                        };
                        
                        // Update GOAP world state
                        goap_agent.update_world_state(WorldKey::HeardSound, true);
                        goap_agent.update_world_state(WorldKey::IsAlert, true);
                        goap_agent.abort_plan(); // Force immediate replanning
                        
                        info!("Enemy {} responding to call for help from {} (distance: {:.1})", enemy_entity.index(), alert_event.alerter.index(), distance);
                    },
                    _ => {
                        // Handle other alert types in future expansions
                    }
                }
            }
        }
    }
}

// Keep the legacy AI system for backward compatibility
pub fn legacy_enemy_ai_system(
    mut enemy_query: Query<(Entity, &Transform, &mut AIState, &mut Vision, &mut Patrol), (With<Enemy>, Without<Dead>, Without<GoapAgent>, Without<Corpse>)>,
    agent_query: Query<(Entity, &Transform), With<Agent>>,
    mut audio_events: EventWriter<AudioEvent>,
    mut action_events: EventWriter<ActionEvent>,
    time: Res<Time>,
    game_mode: Res<GameMode>,
) {
    if game_mode.paused { return; }

    for (enemy_entity, enemy_transform, mut ai_state, mut vision, patrol) in enemy_query.iter_mut() {
        ai_state.alert_cooldown -= time.delta_secs();
        ai_state.investigation_timer -= time.delta_secs();


        // Update vision direction based on movement/patrol
        update_vision_direction(&mut vision, &ai_state, &patrol, enemy_transform);

        // Check for visible agents
        let visible_agent = check_line_of_sight(enemy_transform, &vision, &agent_query);
        
        // State machine
        match &mut ai_state.mode {
            AIMode::Patrol => {
                if let Some(agent_entity) = visible_agent {
                    // Store current position as last known
                    if let Ok((_, agent_transform)) = agent_query.get(agent_entity) {
                        ai_state.last_known_target = Some(agent_transform.translation.truncate());
                    }
                    
                    ai_state.mode = AIMode::Combat { target: agent_entity };
                    
                    // Alert sound
                    audio_events.write(AudioEvent {
                        sound: AudioType::Alert,
                        volume: 0.8,
                    });
                    
                    info!("Enemy spotted agent! Entering combat mode.");
                }
                // Patrol movement handled by movement system
            },
            
            AIMode::Combat { target } => {
                if let Some(spotted_agent) = visible_agent {
                    // Update last known position
                    if let Ok((_, agent_transform)) = agent_query.get(spotted_agent) {
                        ai_state.last_known_target = Some(agent_transform.translation.truncate());
                        
                        let distance = enemy_transform.translation.truncate()
                            .distance(agent_transform.translation.truncate());
                        
                        if distance <= 150.0 {
                            // In range - attack
                            action_events.write(ActionEvent {
                                entity: enemy_entity,
                                action: Action::Attack(spotted_agent),
                            });
                        } else {
                            // Too far - move closer
                            action_events.write(ActionEvent {
                                entity: enemy_entity,
                                action: Action::MoveTo(agent_transform.translation.truncate()),
                            });
                        }
                    }
                } else {
                    // Lost sight - investigate last known position
                    if let Some(last_pos) = ai_state.last_known_target {
                        ai_state.mode = AIMode::Investigate { location: last_pos };
                        ai_state.investigation_timer = 5.0;
                        
                        // Start moving to investigate
                        action_events.write(ActionEvent {
                            entity: enemy_entity,
                            action: Action::MoveTo(last_pos),
                        });
                        
                        info!("Lost sight of agent, investigating last known position");
                    } else {
                        // No last known position - return to patrol
                        ai_state.mode = AIMode::Patrol;
                        info!("No last known position, returning to patrol");
                    }
                }
            },
            
            AIMode::Investigate { location } => {
                // Check for new sightings during investigation
                if let Some(agent_entity) = visible_agent {
                    if let Ok((_, agent_transform)) = agent_query.get(agent_entity) {
                        ai_state.last_known_target = Some(agent_transform.translation.truncate());
                    }
                    ai_state.mode = AIMode::Combat { target: agent_entity };
                    info!("Spotted agent during investigation!");
                    return;
                }
                
                let distance = enemy_transform.translation.truncate().distance(*location);
                
                if distance > 20.0 {
                    // Still moving to investigation point - movement system handles this
                } else if ai_state.investigation_timer <= 0.0 {
                    // Investigation complete - return to patrol
                    ai_state.mode = AIMode::Patrol;
                    ai_state.last_known_target = None;
                    info!("Investigation complete, returning to patrol");
                }
            },
            
            AIMode::Search { area: _ } => {
                // Future expansion - search pattern around area
                if ai_state.investigation_timer <= 0.0 {
                    ai_state.mode = AIMode::Patrol;
                }
            },

            AIMode::Panic => {
                // Panic handled by morale system
            },

        }
    }
}

fn update_vision_direction(vision: &mut Vision, ai_state: &AIState, patrol: &Patrol, transform: &Transform) {
    match &ai_state.mode {
        AIMode::Patrol => {
            // Face the direction of the next patrol point
            if let Some(target) = patrol.current_target() {
                let current_pos = transform.translation.truncate();
                let direction = (target - current_pos).normalize_or_zero();
                if direction != Vec2::ZERO {
                    vision.direction = direction;
                }
            }
        },
        AIMode::Combat { .. } | AIMode::Investigate { .. } => {
            // Vision direction gets updated in movement when chasing/investigating
        },
        AIMode::Search { .. } => {
            // Future: implement search vision patterns
        },
        AIMode::Panic => {
            // Random direction when panicked
        },       
    }
}

fn check_line_of_sight(
    enemy_transform: &Transform,
    vision: &Vision,
    agent_query: &Query<(Entity, &Transform), With<Agent>>,
) -> Option<Entity> {
    let enemy_pos = enemy_transform.translation.truncate();
    
    for (agent_entity, agent_transform) in agent_query.iter() {
        let agent_pos = agent_transform.translation.truncate();
        let to_agent = agent_pos - enemy_pos;
        let distance = to_agent.length();
        
        if distance <= vision.range && distance > 1.0 { // Avoid division by zero
            let agent_direction = to_agent.normalize();
            let dot_product = vision.direction.dot(agent_direction);
            let angle_cos = (vision.angle / 2.0).cos();
            
            if dot_product >= angle_cos {
                // TODO: Add raycasting for obstacles when we have walls
                return Some(agent_entity);
            }
        }
    }
    
    None
}

// Update legacy sound detection system
pub fn sound_detection_system(
    mut enemy_query: Query<(Entity, &Transform, &mut AIState), (With<Enemy>, Without<Dead>)>,
    mut combat_events: EventReader<CombatEvent>,
    combat_transforms: Query<(&Transform, &Inventory), With<Agent>>,
) {
    // React to gunshots with attachment-modified detection range
    for combat_event in combat_events.read() {
        if let Ok((shooter_transform, inventory)) = combat_transforms.get(combat_event.attacker) {
            let gunshot_pos = shooter_transform.translation.truncate();
            
            // Calculate noise level from attachments
            let noise_modifier = if let Some(weapon_config) = &inventory.equipped_weapon {
                let stats = weapon_config.calculate_total_stats();
                1.0 + (stats.noise as f32 * 0.1) // Each noise point = 10% modifier
            } else {
                1.0
            };
            
            // Base detection range modified by noise
            let base_range = 200.0;
            let detection_range = (base_range * noise_modifier).max(50.0); // Minimum 50 units
            
            for (_, enemy_transform, mut ai_state) in enemy_query.iter_mut() {
                let distance = enemy_transform.translation.truncate().distance(gunshot_pos);
                
                if distance <= detection_range && ai_state.alert_cooldown <= 0.0 {
                    match ai_state.mode {
                        AIMode::Patrol => {
                            ai_state.mode = AIMode::Investigate { location: gunshot_pos };
                            ai_state.investigation_timer = 8.0;
                            ai_state.alert_cooldown = 3.0;
                        },
                        _ => {
                            // Already in alert state
                        }
                    }
                }
            }
        }
    }
}

// Update GOAP sound detection system
pub fn goap_sound_detection_system(
    mut enemy_query: Query<(Entity, &Transform, &mut GoapAgent), (With<Enemy>, Without<Dead>)>,
    mut combat_events: EventReader<CombatEvent>,
    combat_transforms: Query<(&Transform, &Inventory), With<Agent>>,
) {
    // React to gunshots by updating GOAP world state with attachment consideration
    for combat_event in combat_events.read() {
        if let Ok((shooter_transform, inventory)) = combat_transforms.get(combat_event.attacker) {
            let gunshot_pos = shooter_transform.translation.truncate();
            
            // Calculate noise level from attachments
            let noise_modifier = if let Some(weapon_config) = &inventory.equipped_weapon {
                let stats = weapon_config.calculate_total_stats();
                1.0 + (stats.noise as f32 * 0.1)
            } else {
                1.0
            };
            
            // Base detection range modified by noise
            let base_range = 200.0;
            let detection_range = (base_range * noise_modifier).max(50.0);
            
            for (_, enemy_transform, mut goap_agent) in enemy_query.iter_mut() {
                let distance = enemy_transform.translation.truncate().distance(gunshot_pos);
                
                if distance <= detection_range {
                    goap_agent.update_world_state(WorldKey::HeardSound, true);
                    goap_agent.abort_plan(); // Force replanning
                }
            }
        }
    }
}