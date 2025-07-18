use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use leafwing_input_manager::prelude::*;
use std::f32::consts::PI;

use crate::components::*;
use crate::resources::*;
use crate::events::*;
use crate::states::*;

// Vision and detection constants
const DETECTION_BUILDUP_RATE: f32 = 2.0; // How fast detection builds up
const DETECTION_DECAY_RATE: f32 = 1.5;   // How fast detection decays when not visible
const FULL_DETECTION_THRESHOLD: f32 = 1.0; // When agent is fully detected
const VISION_CONE_SEGMENTS: usize = 16;   // Visual quality of vision cones

// Interaction constants
const INTERACTION_PULSE_SPEED: f32 = 3.0;
const INTERACTION_PULSE_AMPLITUDE: f32 = 0.2;
const INTERACTION_PULSE_BASE: f32 = 0.8;
const INTERACTION_RANGE_ALPHA: f32 = 0.3;
const PROGRESS_BAR_WIDTH: f32 = 40.0;
const PROGRESS_BAR_HEIGHT: f32 = 6.0;
const INTERACTION_PROMPT_RADIUS: f32 = 35.0;

// Equipment constants
const INVENTORY_PANEL_WIDTH: f32 = 400.0;
const INVENTORY_PANEL_HEIGHT: f32 = 500.0;
const INVENTORY_ITEM_HEIGHT: f32 = 30.0;
const NOTIFICATION_DURATION: f32 = 3.0;

// Component to mark neurovector commands for processing
#[derive(Component)]
pub struct NeurovectorCommand {
    pub caster: Entity,
    pub target: Entity,
}

// Test mission setup - creates a simple playable scenario
pub fn spawn_test_mission(
    mut commands: Commands,
    mut mission_data: ResMut<MissionData>,
    mut global_data: ResMut<GlobalGameData>,
) {
    // Spawn 3 agents
    for i in 0..3 {
        let agent_entity = commands.spawn((
            SpriteBundle {
                sprite: Sprite {
                    color: Color::srgb(0.2, 0.8, 0.2),
                    custom_size: Some(Vec2::new(20.0, 20.0)),
                    ..default()
                },
                transform: Transform::from_translation(Vec3::new(
                    -200.0 + i as f32 * 50.0,
                    0.0,
                    1.0,
                )),
                ..default()
            },
            Agent::default(),
            Movement {
                target_position: None,
                path: vec![],
                current_path_index: 0,
            },
            Selectable {
                selected: false,
                selection_radius: 15.0,
            },
            AgentVision {
                range: 150.0,
                angle: PI / 3.0, // 60 degrees
                direction: Vec2::new(1.0, 0.0),
                can_see: vec![],
                detection_buildup: 0.0,
            },
            NeurovectorCapability::default(),
            RigidBody::Dynamic,
            Collider::ball(10.0),
            Velocity::default(),
            Damping { linear_damping: 10.0, angular_damping: 10.0 },
        )).id();
        
        global_data.available_agents.push(agent_entity);
    }

    // Spawn some civilians
    for i in 0..5 {
        commands.spawn((
            SpriteBundle {
                sprite: Sprite {
                    color: Color::srgb(0.8, 0.8, 0.2),
                    custom_size: Some(Vec2::new(15.0, 15.0)),
                    ..default()
                },
                transform: Transform::from_translation(Vec3::new(
                    100.0 + i as f32 * 60.0,
                    100.0 + (i as f32 * 20.0).sin() * 50.0,
                    1.0,
                )),
                ..default()
            },
            Civilian {
                health: 50.0,
                occupation: OccupationType::Civilian,
                security_clearance: SecurityLevel::None,
                neurovector_target: true,
                controlled_by: None,
                awareness_level: 0.0,
            },
            Movement {
                target_position: None,
                path: vec![],
                current_path_index: 0,
            },
            Selectable {
                selected: false,
                selection_radius: 10.0,
            },
            RigidBody::Dynamic,
            Collider::ball(7.5),
            Velocity::default(),
            Damping { linear_damping: 10.0, angular_damping: 10.0 },
        ));
    }

    // Spawn an enemy guard
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::srgb(0.8, 0.2, 0.2),
                custom_size: Some(Vec2::new(18.0, 18.0)),
                ..default()
            },
            transform: Transform::from_translation(Vec3::new(200.0, -100.0, 1.0)),
            ..default()
        },
        Enemy {
            health: 100.0,
            patrol_route: vec![
                Vec2::new(200.0, -100.0),
                Vec2::new(300.0, -100.0),
                Vec2::new(300.0, 50.0),
                Vec2::new(200.0, 50.0),
            ],
            current_patrol_index: 0,
            alert_level: AlertLevel::Green,
            detection_range: 100.0,
            last_known_target: None,
        },
        Movement {
            target_position: Some(Vec2::new(300.0, -100.0)),
            path: vec![],
            current_path_index: 0,
        },
        AgentVision {
            range: 120.0,
            angle: PI / 4.0, // 45 degrees
            direction: Vec2::new(1.0, 0.0),
            can_see: vec![],
            detection_buildup: 0.0,
        },
        RigidBody::Dynamic,
        Collider::ball(9.0),
        Velocity::default(),
        Damping { linear_damping: 10.0, angular_damping: 10.0 },
    ));

    // Create a simple objective
    let objective_entity = commands.spawn(MissionObjective {
        objective_type: ObjectiveType::Infiltrate(Vec2::new(350.0, 0.0)),
        is_primary: true,
        completed: false,
        target_entity: None,
        target_position: Some(Vec2::new(350.0, 0.0)),
    }).id();

    mission_data.objectives.push(objective_entity);
    
    // Visual marker for objective
    commands.spawn(SpriteBundle {
        sprite: Sprite {
            color: Color::srgb(0.8, 0.2, 0.8),
            custom_size: Some(Vec2::new(30.0, 30.0)),
            ..default()
        },
        transform: Transform::from_translation(Vec3::new(350.0, 0.0, 0.5)),
        ..default()
    });

    info!("Test mission spawned with 3 agents, 5 civilians, 1 enemy, and 1 objective");
}

// Pause system - core mechanic for tactical gameplay
pub fn handle_pause_input(
    input: Query<&ActionState<PlayerAction>>,
    mut mission_state: ResMut<NextState<MissionState>>,
    mut mission_data: ResMut<MissionData>,
    current_mission_state: Res<State<MissionState>>,
) {
    if let Ok(action_state) = input.get_single() {
        if action_state.just_pressed(&PlayerAction::Pause) {
            match current_mission_state.get() {
                MissionState::Active => {
                    mission_state.set(MissionState::Paused);
                    mission_data.time_scale = 0.0;
                    info!("Mission paused");
                }
                MissionState::Paused => {
                    mission_state.set(MissionState::Active);
                    mission_data.time_scale = 1.0;
                    info!("Mission resumed");
                }
                _ => {}
            }
        }
    }
}

// Basic camera movement
pub fn camera_movement(
    input: Res<ButtonInput<KeyCode>>,
    mut camera_query: Query<&mut Transform, With<Camera>>,
    time: Res<Time>,
) {
    if let Ok(mut transform) = camera_query.get_single_mut() {
        let mut direction = Vec3::ZERO;
        let speed = 400.0;

        if input.pressed(KeyCode::ArrowUp) || input.pressed(KeyCode::KeyW) {
            direction.y += 1.0;
        }
        if input.pressed(KeyCode::ArrowDown) || input.pressed(KeyCode::KeyS) {
            direction.y -= 1.0;
        }
        if input.pressed(KeyCode::ArrowLeft) || input.pressed(KeyCode::KeyA) {
            direction.x -= 1.0;
        }
        if input.pressed(KeyCode::ArrowRight) || input.pressed(KeyCode::KeyD) {
            direction.x += 1.0;
        }

        if direction != Vec3::ZERO {
            direction = direction.normalize();
            transform.translation += direction * speed * time.delta_seconds();
        }
    }
}

// Agent selection system
pub fn selection_system(
    mut selection_state: ResMut<SelectionState>,
    input: Query<&ActionState<PlayerAction>>,
    mut selectable_query: Query<(Entity, &mut Selectable, &Transform), With<Agent>>,
    windows: Query<&Window>,
    cameras: Query<(&Camera, &GlobalTransform)>,
) {
    if let Ok(action_state) = input.get_single() {
        if action_state.just_pressed(&PlayerAction::Select) {
            if let Some(world_position) = get_world_mouse_position(&windows, &cameras) {
                // Clear previous selection
                for (_, mut selectable, _) in selectable_query.iter_mut() {
                    selectable.selected = false;
                }
                selection_state.selected_agents.clear();

                // Find closest selectable agent
                let mut closest_distance = f32::INFINITY;
                let mut closest_entity = None;

                for (entity, selectable, transform) in selectable_query.iter() {
                    let distance = world_position.distance(transform.translation.truncate());
                    if distance < selectable.selection_radius && distance < closest_distance {
                        closest_distance = distance;
                        closest_entity = Some(entity);
                    }
                }

                // Select the closest entity
                if let Some(entity) = closest_entity {
                    if let Ok((_, mut selectable, _)) = selectable_query.get_mut(entity) {
                        selectable.selected = true;
                        selection_state.selected_agents.push(entity);
                        info!("Agent selected");
                    }
                }
            }
        }
    }
}

// Helper function to execute movement
fn execute_movement(
    movement: &mut Movement,
    transform: &mut Transform,
    velocity: &mut Velocity,
    move_speed: f32,
    time: &Time,
) {
    if let Some(target) = movement.target_position {
        let current_pos = transform.translation.truncate();
        let direction = (target - current_pos).normalize_or_zero();
        let distance = current_pos.distance(target);

        if distance > 5.0 {
            let move_force = direction * move_speed * time.delta_seconds();
            velocity.linvel = move_force;
        } else {
            movement.target_position = None;
            velocity.linvel = Vec2::ZERO;
        }
    }
}

// Agent movement system - now handles both agents and controlled civilians
pub fn agent_movement_system(
    mut agent_query: Query<(&mut Movement, &mut Transform, &mut Velocity, &Agent), Without<Civilian>>,
    mut civilian_query: Query<(&mut Movement, &mut Transform, &mut Velocity, &Civilian), (Without<Agent>, With<Civilian>)>,
    input: Query<&ActionState<PlayerAction>>,
    neurovector_targeting: Res<NeurovectorTargeting>,
    mission_data: Res<MissionData>,
    time: Res<Time>,
    windows: Query<&Window>,
    cameras: Query<(&Camera, &GlobalTransform)>,
) {
    if mission_data.time_scale == 0.0 {
        return; // Don't move when paused
    }

    // Don't handle movement clicks when in neurovector targeting mode
    if neurovector_targeting.targeting_mode {
        // Still execute existing movement for agents
        for (mut movement, mut transform, mut velocity, agent) in agent_query.iter_mut() {
            execute_movement(&mut movement, &mut transform, &mut velocity, agent.movement_speed, &time);
        }
        
        // And for controlled civilians
        for (mut movement, mut transform, mut velocity, _) in civilian_query.iter_mut() {
            execute_movement(&mut movement, &mut transform, &mut velocity, 100.0, &time); // Civilian movement speed
        }
        return;
    }

    // Handle movement orders
    if let Ok(action_state) = input.get_single() {
        if action_state.just_pressed(&PlayerAction::Move) {
            if let Some(world_position) = get_world_mouse_position(&windows, &cameras) {
                // Move all selected agents
                for (mut movement, _, _, _) in agent_query.iter_mut() {
                    movement.target_position = Some(world_position);
                }
                
                // Also move any controlled civilians
                for (mut movement, _, _, civilian) in civilian_query.iter_mut() {
                    if civilian.controlled_by.is_some() {
                        movement.target_position = Some(world_position);
                    }
                }
            }
        }
    }

    // Execute movement for agents
    for (mut movement, mut transform, mut velocity, agent) in agent_query.iter_mut() {
        execute_movement(&mut movement, &mut transform, &mut velocity, agent.movement_speed, &time);
    }
    
    // Execute movement for controlled civilians
    for (mut movement, mut transform, mut velocity, _) in civilian_query.iter_mut() {
        execute_movement(&mut movement, &mut transform, &mut velocity, 100.0, &time); // Civilian movement speed
    }
}

// Basic agent action processing
pub fn agent_action_system(
    mut agent_query: Query<(&mut Agent, &Transform)>,
    mut action_events: EventReader<AgentActionEvent>,
    mission_data: Res<MissionData>,
) {
    if mission_data.time_scale == 0.0 {
        return; // Don't process actions when paused
    }

    for event in action_events.read() {
        if let Ok((mut agent, transform)) = agent_query.get_mut(event.agent) {
            match &event.action {
                AgentAction::TakeDamage(damage) => {
                    agent.health = (agent.health - damage).max(0.0);
                    if agent.health <= 0.0 {
                        info!("Agent at {:?} died", transform.translation);
                    }
                }
                AgentAction::Heal(amount) => {
                    agent.health = (agent.health + amount).min(agent.max_health);
                }
                AgentAction::Die => {
                    agent.health = 0.0;
                }
                _ => {
                    // Other actions will be implemented as needed
                }
            }
        }
    }
}

// Neurovector targeting system - handles target selection UI
pub fn neurovector_targeting_system(
    mut commands: Commands,
    input: Query<&ActionState<PlayerAction>>,
    mut neurovector_targeting: ResMut<NeurovectorTargeting>,
    mut selection_state: ResMut<SelectionState>,
    agent_query: Query<(Entity, &Transform, &NeurovectorCapability), With<Agent>>,
    civilian_query: Query<(Entity, &Transform, &Civilian)>,
    keyboard: Res<ButtonInput<KeyCode>>,
    windows: Query<&Window>,
    cameras: Query<(&Camera, &GlobalTransform)>,
) {
    if let Ok(action_state) = input.get_single() {
        // Toggle neurovector targeting mode with 'N' key
        if keyboard.just_pressed(KeyCode::KeyN) {
            if !neurovector_targeting.targeting_mode {
                // Enter targeting mode if we have a selected agent with neurovector capability
                if let Some(&selected_agent) = selection_state.selected_agents.first() {
                    if let Ok((_, _, neurovector_cap)) = agent_query.get(selected_agent) {
                        if neurovector_cap.current_cooldown <= 0.0 {
                            neurovector_targeting.targeting_mode = true;
                            neurovector_targeting.active_agent = Some(selected_agent);
                            info!("Neurovector targeting mode activated");
                        } else {
                            info!("Neurovector on cooldown: {:.1}s remaining", neurovector_cap.current_cooldown);
                        }
                    }
                }
            } else {
                // Exit targeting mode
                neurovector_targeting.targeting_mode = false;
                neurovector_targeting.active_agent = None;
                neurovector_targeting.valid_targets.clear();
                info!("Neurovector targeting mode deactivated");
            }
        }

        // Cancel targeting with escape
        if keyboard.just_pressed(KeyCode::Escape) && neurovector_targeting.targeting_mode {
            neurovector_targeting.targeting_mode = false;
            neurovector_targeting.active_agent = None;
            neurovector_targeting.valid_targets.clear();
            info!("Neurovector targeting cancelled");
        }

        // Handle target selection when in targeting mode
        if neurovector_targeting.targeting_mode {
            if let Some(active_agent) = neurovector_targeting.active_agent {
                if let Ok((_, agent_transform, neurovector_cap)) = agent_query.get(active_agent) {
                    // Update valid targets based on range and line of sight
                    neurovector_targeting.valid_targets.clear();
                    
                    for (civilian_entity, civilian_transform, civilian) in civilian_query.iter() {
                        if civilian.controlled_by.is_some() {
                            continue; // Skip already controlled civilians
                        }
                        
                        let distance = agent_transform.translation.truncate()
                            .distance(civilian_transform.translation.truncate());
                        
                        if distance <= neurovector_cap.range {
                            // TODO: Add line of sight check here
                            neurovector_targeting.valid_targets.push(civilian_entity);
                        }
                    }

                    // Handle target selection click
                    if action_state.just_pressed(&PlayerAction::Select) {
                        if let Some(mouse_pos) = get_world_mouse_position(&windows, &cameras) {
                            // Find closest valid target to mouse
                            let mut closest_target = None;
                            let mut closest_distance = f32::INFINITY;
                            
                            for &target_entity in &neurovector_targeting.valid_targets {
                                if let Ok((_, target_transform, _)) = civilian_query.get(target_entity) {
                                    let distance = mouse_pos.distance(target_transform.translation.truncate());
                                    if distance < 25.0 && distance < closest_distance {
                                        closest_distance = distance;
                                        closest_target = Some(target_entity);
                                    }
                                }
                            }

                            if let Some(target) = closest_target {
                                // Execute neurovector attack
                                commands.spawn((
                                    NeurovectorCommand {
                                        caster: active_agent,
                                        target,
                                    },
                                ));
                                
                                neurovector_targeting.targeting_mode = false;
                                neurovector_targeting.active_agent = None;
                                neurovector_targeting.valid_targets.clear();
                                info!("Neurovector executed on target");
                            }
                        }
                    }
                }
            }
        }
    }
}

// Neurovector execution system - processes the actual mind control
pub fn neurovector_system(
    mut commands: Commands,
    neurovector_commands: Query<(Entity, &NeurovectorCommand)>,
    mut agent_query: Query<&mut NeurovectorCapability, With<Agent>>,
    mut civilian_query: Query<&mut Civilian>,
    mut neurovector_events: EventWriter<NeurovectorEvent>,
    mission_data: Res<MissionData>,
) {
    if mission_data.time_scale == 0.0 {
        return; // Don't process when paused
    }

    for (command_entity, command) in neurovector_commands.iter() {
        if let Ok(mut neurovector_cap) = agent_query.get_mut(command.caster) {
            if let Ok(mut civilian) = civilian_query.get_mut(command.target) {
                // Check if we can control more targets
                if neurovector_cap.controlled_entities.len() < neurovector_cap.max_targets as usize {
                    // Execute mind control
                    civilian.controlled_by = Some(command.caster);
                    neurovector_cap.controlled_entities.push(command.target);
                    neurovector_cap.current_cooldown = neurovector_cap.cooldown;
                    
                    neurovector_events.send(NeurovectorEvent {
                        caster: command.caster,
                        target: command.target,
                        success: true,
                    });
                    
                    info!("Civilian successfully controlled via neurovector");
                } else {
                    neurovector_events.send(NeurovectorEvent {
                        caster: command.caster,
                        target: command.target,
                        success: false,
                    });
                    
                    info!("Neurovector failed - max targets reached");
                }
            }
        }
        
        // Remove the command entity
        commands.entity(command_entity).despawn();
    }
}

// Neurovector cooldown system
pub fn neurovector_cooldown_system(
    mut agent_query: Query<&mut NeurovectorCapability, With<Agent>>,
    time: Res<Time>,
    mission_data: Res<MissionData>,
) {
    if mission_data.time_scale == 0.0 {
        return; // Don't update cooldowns when paused
    }

    for mut neurovector_cap in agent_query.iter_mut() {
        if neurovector_cap.current_cooldown > 0.0 {
            neurovector_cap.current_cooldown -= time.delta_seconds();
            if neurovector_cap.current_cooldown <= 0.0 {
                neurovector_cap.current_cooldown = 0.0;
            }
        }
    }
}

// Neurovector visual feedback system
pub fn neurovector_visual_system(
    mut gizmos: Gizmos,
    neurovector_targeting: Res<NeurovectorTargeting>,
    agent_query: Query<(&Transform, &NeurovectorCapability), With<Agent>>,
    civilian_query: Query<(&Transform, &Civilian)>,
    selection_state: Res<SelectionState>,
) {
    // Show neurovector range for selected agents
    for &selected_agent in &selection_state.selected_agents {
        if let Ok((agent_transform, neurovector_cap)) = agent_query.get(selected_agent) {
            let agent_pos = agent_transform.translation.truncate();
            
            // Draw range circle
            let range_color = if neurovector_cap.current_cooldown > 0.0 {
                Color::srgb(0.8, 0.3, 0.3) // Red when on cooldown
            } else {
                Color::srgb(0.3, 0.3, 0.8) // Blue when available
            };
            
            gizmos.circle_2d(agent_pos, neurovector_cap.range, range_color);
        }
    }
    
    // Highlight valid targets when in targeting mode
    if neurovector_targeting.targeting_mode {
        for &target_entity in &neurovector_targeting.valid_targets {
            if let Ok((target_transform, _)) = civilian_query.get(target_entity) {
                let target_pos = target_transform.translation.truncate();
                
                // Draw targeting indicator around valid targets
                gizmos.circle_2d(target_pos, 20.0, Color::srgb(0.8, 0.8, 0.3));
                gizmos.circle_2d(target_pos, 15.0, Color::srgb(1.0, 1.0, 0.5));
            }
        }
    }
    
    // Show control lines between agents and controlled civilians
    for (agent_transform, neurovector_cap) in agent_query.iter() {
        let agent_pos = agent_transform.translation.truncate();
        
        for &controlled_entity in &neurovector_cap.controlled_entities {
            if let Ok((civilian_transform, _)) = civilian_query.get(controlled_entity) {
                let civilian_pos = civilian_transform.translation.truncate();
                
                // Draw control connection line
                gizmos.line_2d(
                    agent_pos,
                    civilian_pos,
                    Color::srgb(0.8, 0.3, 0.8),
                );
            }
        }
    }
}

// Visual feedback system for controlled civilians
pub fn controlled_civilian_visual_system(
    mut civilian_query: Query<(&mut Sprite, &Civilian), With<Civilian>>,
) {
    for (mut sprite, civilian) in civilian_query.iter_mut() {
        if civilian.controlled_by.is_some() {
            // Change color to purple when controlled
            sprite.color = Color::srgb(0.8, 0.3, 0.8);
        } else {
            // Normal yellow when not controlled
            sprite.color = Color::srgb(0.8, 0.8, 0.2);
        }
    }
}

// Mission timer and time management
pub fn mission_timer_system(
    mut mission_data: ResMut<MissionData>,
    time: Res<Time>,
    mut mission_events: EventWriter<MissionEvent>,
) {
    if mission_data.time_scale == 0.0 {
        return; // Don't advance timer when paused
    }

    mission_data.mission_timer += time.delta_seconds() * mission_data.time_scale;

    // Check for time limit
    if let Some(time_limit) = mission_data.time_limit {
        if mission_data.mission_timer >= time_limit {
            mission_events.send(MissionEvent {
                event_type: MissionEventType::TimeExpired,
            });
            mission_data.mission_active = false;
            info!("Mission failed - time expired");
        }
    }
}

// Basic visibility system
pub fn visibility_system(
    mut visibility_query: Query<(&mut AgentVision, &Transform)>,
    transform_query: Query<&Transform, Without<AgentVision>>,
    mission_data: Res<MissionData>,
) {
    if mission_data.time_scale == 0.0 {
        return; // Don't update visibility when paused
    }

    for (mut visibility, viewer_transform) in visibility_query.iter_mut() {
        visibility.can_see.clear();
        let viewer_pos = viewer_transform.translation.truncate();

        for (entity, target_transform) in transform_query.iter().enumerate() {
            let target_pos = target_transform.translation.truncate();
            let distance = viewer_pos.distance(target_pos);

            if distance <= visibility.range {
                let direction_to_target = (target_pos - viewer_pos).normalize_or_zero();
                let dot_product = visibility.direction.dot(direction_to_target);
                let angle_to_target = dot_product.acos();

                if angle_to_target <= visibility.angle / 2.0 {
                    // Simple line of sight - in a real game you'd do raycasting
                    // For now, assume all targets in range and angle are visible
                    // visibility.can_see.push(Entity::from_raw(entity as u32));
                }
            }
        }
    }
}

// Alert system for security responses
pub fn alert_system(
    mut mission_data: ResMut<MissionData>,
    mut alert_events: EventReader<AlertEvent>,
    time: Res<Time>,
) {
    if mission_data.time_scale == 0.0 {
        return; // Don't process alerts when paused
    }

    // Process new alert events
    for alert_event in alert_events.read() {
        match alert_event.new_level {
            AlertLevel::Yellow => {
                if matches!(mission_data.current_alert_level, AlertLevel::Green) {
                    mission_data.current_alert_level = AlertLevel::Yellow;
                    mission_data.alert_decay_timer = 30.0; // 30 seconds to decay
                    info!("Alert level raised to YELLOW");
                }
            }
            AlertLevel::Orange => {
                if !matches!(mission_data.current_alert_level, AlertLevel::Red) {
                    mission_data.current_alert_level = AlertLevel::Orange;
                    mission_data.alert_decay_timer = 60.0; // 60 seconds to decay
                    info!("Alert level raised to ORANGE");
                }
            }
            AlertLevel::Red => {
                mission_data.current_alert_level = AlertLevel::Red;
                mission_data.alert_decay_timer = 120.0; // 2 minutes to decay
                info!("Alert level raised to RED - FULL ALERT");
            }
            _ => {}
        }
    }

    // Handle alert decay
    if mission_data.alert_decay_timer > 0.0 {
        mission_data.alert_decay_timer -= time.delta_seconds();
        
        if mission_data.alert_decay_timer <= 0.0 {
            mission_data.current_alert_level = match mission_data.current_alert_level {
                AlertLevel::Red => AlertLevel::Orange,
                AlertLevel::Orange => AlertLevel::Yellow,
                AlertLevel::Yellow => AlertLevel::Green,
                AlertLevel::Green => AlertLevel::Green,
            };
            
            if !matches!(mission_data.current_alert_level, AlertLevel::Green) {
                mission_data.alert_decay_timer = 30.0; // Continue decay
            }
            
            info!("Alert level decayed to {:?}", mission_data.current_alert_level);
        }
    }
}

// Pause UI system
pub fn pause_ui_system(
    mut commands: Commands,
    mission_data: Res<MissionData>,
    ui_query: Query<Entity, With<Node>>,
) {
    // Clear existing UI
    for entity in ui_query.iter() {
        commands.entity(entity).despawn_recursive();
    }

    // Show pause overlay
    commands.spawn(NodeBundle {
        style: Style {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            position_type: PositionType::Absolute,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        background_color: Color::srgba(0.0, 0.0, 0.0, 0.5).into(),
        ..default()
    }).with_children(|parent| {
        parent.spawn(TextBundle::from_section(
            "PAUSED\nSpace to resume",
            TextStyle {
                font_size: 32.0,
                color: Color::WHITE,
                ..default()
            },
        ));
    });
}

// Process queued orders when paused
pub fn queued_orders_system(
    mut selection_state: ResMut<SelectionState>,
    mission_data: Res<MissionData>,
) {
    // In a paused state, orders are queued for execution when unpaused
    // This system would process the queue when the game resumes
    // For now, it's a placeholder for future implementation
}

// Utility function to convert mouse position to world coordinates
fn get_world_mouse_position(
    windows: &Query<&Window>,
    cameras: &Query<(&Camera, &GlobalTransform)>,
) -> Option<Vec2> {
    let window = windows.get_single().ok()?;
    let (camera, camera_transform) = cameras.get_single().ok()?;
    
    window.cursor_position()
        .and_then(|cursor| camera.viewport_to_world_2d(camera_transform, cursor))
}

// Interaction detection system - finds terminals near selected agents
pub fn interaction_detection_system(
    mut interaction_state: ResMut<InteractionState>,
    selection_state: Res<SelectionState>,
    agent_query: Query<&Transform, With<Agent>>,
    terminal_query: Query<(Entity, &Transform, &InteractableTerminal)>,
) {
    interaction_state.available_terminals.clear();

    // Check for terminals near selected agents
    for &selected_agent in &selection_state.selected_agents {
        if let Ok(agent_transform) = agent_query.get(selected_agent) {
            let agent_pos = agent_transform.translation.truncate();

            for (terminal_entity, terminal_transform, terminal) in terminal_query.iter() {
                if terminal.is_accessed {
                    continue; // Skip already accessed terminals
                }

                let terminal_pos = terminal_transform.translation.truncate();
                let distance = agent_pos.distance(terminal_pos);

                if distance <= terminal.interaction_range {
                    if !interaction_state.available_terminals.contains(&terminal_entity) {
                        interaction_state.available_terminals.push(terminal_entity);
                    }
                }
            }
        }
    }
}

// Interaction system - handles starting and managing interactions
pub fn interaction_system(
    mut commands: Commands,
    input: Query<&ActionState<PlayerAction>>,
    mut interaction_events: EventWriter<InteractionEvent>,
    interaction_state: Res<InteractionState>,
    selection_state: Res<SelectionState>,
    terminal_query: Query<(Entity, &Transform, &InteractableTerminal)>,
    agent_query: Query<&Transform, With<Agent>>,
    active_interactions: Query<&InteractionPrompt>,
    keyboard: Res<ButtonInput<KeyCode>>,
    windows: Query<&Window>,
    cameras: Query<(&Camera, &GlobalTransform)>,
) {
    if let Ok(action_state) = input.get_single() {
        // Handle interaction input with 'E' key
        if keyboard.just_pressed(KeyCode::KeyE) {
            // Check if we have a selected agent and available terminals
            if let Some(&selected_agent) = selection_state.selected_agents.first() {
                if let Ok(agent_transform) = agent_query.get(selected_agent) {
                    let agent_pos = agent_transform.translation.truncate();

                    // Find closest available terminal
                    let mut closest_terminal = None;
                    let mut closest_distance = f32::INFINITY;

                    for &terminal_entity in &interaction_state.available_terminals {
                        if let Ok((_, terminal_transform, terminal)) = terminal_query.get(terminal_entity) {
                            let distance = agent_pos.distance(terminal_transform.translation.truncate());
                            if distance < closest_distance {
                                closest_distance = distance;
                                closest_terminal = Some(terminal_entity);
                            }
                        }
                    }

                    // Start interaction with closest terminal
                    if let Some(terminal_entity) = closest_terminal {
                        if let Ok((_, _, terminal)) = terminal_query.get(terminal_entity) {
                            // Check if agent is already interacting
                            let already_interacting = active_interactions.iter()
                                .any(|prompt| prompt.interacting_agent == selected_agent);

                            if !already_interacting {
                                // Create interaction prompt
                                commands.spawn(InteractionPrompt {
                                    target_terminal: terminal_entity,
                                    interacting_agent: selected_agent,
                                    progress: 0.0,
                                    total_time: terminal.access_time,
                                });

                                interaction_events.send(InteractionEvent {
                                    agent: selected_agent,
                                    terminal: terminal_entity,
                                    interaction_type: InteractionEventType::StartInteraction,
                                });

                                info!("Started interaction with {:?} terminal", terminal.terminal_type);
                            }
                        }
                    }
                }
            }
        }

        // Cancel interaction with Escape
        if keyboard.just_pressed(KeyCode::Escape) {
            for prompt in active_interactions.iter() {
                interaction_events.send(InteractionEvent {
                    agent: prompt.interacting_agent,
                    terminal: prompt.target_terminal,
                    interaction_type: InteractionEventType::CancelInteraction,
                });
            }
        }
    }
}

// Interaction progress system - handles timing and completion
pub fn interaction_progress_system(
    mut commands: Commands,
    mut interaction_prompts: Query<(Entity, &mut InteractionPrompt)>,
    mut terminal_query: Query<&mut InteractableTerminal>,
    mut interaction_events: EventWriter<InteractionEvent>,
    mut completion_events: EventWriter<InteractionCompleteEvent>,
    time: Res<Time>,
    mission_data: Res<MissionData>,
) {
    if mission_data.time_scale == 0.0 {
        return; // Don't progress when paused
    }

    for (prompt_entity, mut prompt) in interaction_prompts.iter_mut() {
        prompt.progress += time.delta_seconds();

        if prompt.progress >= prompt.total_time {
            // Interaction complete
            if let Ok(mut terminal) = terminal_query.get_mut(prompt.target_terminal) {
                terminal.is_accessed = true;

                completion_events.send(InteractionCompleteEvent {
                    agent: prompt.interacting_agent,
                    terminal: prompt.target_terminal,
                    rewards: terminal.loot_table.clone(),
                });

                interaction_events.send(InteractionEvent {
                    agent: prompt.interacting_agent,
                    terminal: prompt.target_terminal,
                    interaction_type: InteractionEventType::CompleteInteraction,
                });

                info!("Interaction completed! Rewards: {:?}", terminal.loot_table);
            }

            // Remove the interaction prompt
            commands.entity(prompt_entity).despawn();
        }
    }

    // Handle interaction event cleanup
    /*
    // error[E0599]: no method named `read` found for struct `bevy::prelude::EventWriter` in the current scope: method not found in `EventWriter<'_, InteractionEvent>`
    for mut event in interaction_events.read() {
        match event.interaction_type {
            InteractionEventType::CancelInteraction => {
                // Find and remove matching interaction prompts
                for (prompt_entity, prompt) in interaction_prompts.iter() {
                    if prompt.interacting_agent == event.agent && prompt.target_terminal == event.terminal {
                        commands.entity(prompt_entity).despawn();
                        info!("Interaction cancelled");
                        break;
                    }
                }
            }
            _ => {}
        }
    }
    */
}

// Interaction visual system - shows prompts and progress
pub fn interaction_visual_system(
    mut gizmos: Gizmos,
    time: Res<Time>,
    interaction_state: Res<InteractionState>,
    selection_state: Res<SelectionState>,
    terminal_query: Query<(&Transform, &InteractableTerminal)>,
    agent_query: Query<&Transform, With<Agent>>,
    interaction_prompts: Query<&InteractionPrompt>,
) {
    // Show interaction ranges and prompts for available terminals
    for &terminal_entity in &interaction_state.available_terminals {
        if let Ok((terminal_transform, terminal)) = terminal_query.get(terminal_entity) {
            let terminal_pos = terminal_transform.translation.truncate();

            // Draw interaction range
            let range_color = match terminal.priority_color {
                PriorityColor::Critical => Color::srgba(0.9, 0.2, 0.2, 0.3),
                PriorityColor::Secondary => Color::srgba(0.2, 0.5, 0.9, 0.3),
                PriorityColor::Optional => Color::srgba(0.2, 0.8, 0.3, 0.3),
            };

            gizmos.circle_2d(terminal_pos, terminal.interaction_range, range_color);

            // Show "E to interact" indicator for terminals in range
            let indicator_color = match terminal.priority_color {
                PriorityColor::Critical => Color::srgb(1.0, 0.3, 0.3),
                PriorityColor::Secondary => Color::srgb(0.3, 0.6, 1.0),
                PriorityColor::Optional => Color::srgb(0.3, 1.0, 0.4),
            };

            // Pulsing effect for interaction prompt
            let pulse_factor = (time.elapsed_seconds() * INTERACTION_PULSE_SPEED).sin() 
                * INTERACTION_PULSE_AMPLITUDE + INTERACTION_PULSE_BASE;

            gizmos.circle_2d(terminal_pos, INTERACTION_PROMPT_RADIUS * pulse_factor, indicator_color);
        }
    }

    // Show interaction progress bars
    for prompt in interaction_prompts.iter() {
        if let Ok((terminal_transform, _)) = terminal_query.get(prompt.target_terminal) {
            let terminal_pos = terminal_transform.translation.truncate();
            let progress = prompt.progress / prompt.total_time;

            // Draw progress bar above terminal
            let bar_width = 40.0;
            let bar_height = 6.0;
            let bar_pos = terminal_pos + Vec2::new(0.0, 40.0);

            // Background
            gizmos.rect_2d(
                bar_pos,
                0.0,
                Vec2::new(bar_width, bar_height),
                Color::srgb(0.3, 0.3, 0.3),
            );

            // Progress fill
            gizmos.rect_2d(
                bar_pos - Vec2::new((bar_width * (1.0 - progress)) / 2.0, 0.0),
                0.0,
                Vec2::new(bar_width * progress, bar_height),
                Color::srgb(0.2, 0.8, 0.4),
            );
        }
    }
}

// Enemy vision cone visual system
pub fn enemy_vision_visual_system(
    mut gizmos: Gizmos,
    enemy_query: Query<(&Transform, &AgentVision, &Enemy)>,
    mission_data: Res<MissionData>,
) {
    for (transform, vision, enemy) in enemy_query.iter() {
        let enemy_pos = transform.translation.truncate();
        
        // Determine vision cone color based on alert level
        let cone_color = match enemy.alert_level {
            AlertLevel::Green => Color::srgba(1.0, 1.0, 0.3, 0.2),   // Yellow - normal patrol
            AlertLevel::Yellow => Color::srgba(1.0, 0.7, 0.0, 0.3),  // Orange - suspicious
            AlertLevel::Orange => Color::srgba(1.0, 0.4, 0.0, 0.4),  // Dark orange - searching
            AlertLevel::Red => Color::srgba(1.0, 0.2, 0.2, 0.5),     // Red - full alert
        };

        // Draw vision cone using triangle fan
        draw_vision_cone(&mut gizmos, enemy_pos, vision, cone_color);
        
        // Draw detection buildup indicator
        if vision.detection_buildup > 0.0 {
            let detection_color = Color::srgb(
                1.0, 
                1.0 - vision.detection_buildup, 
                1.0 - vision.detection_buildup
            );
            
            // Draw detection progress circle above enemy
            let detection_pos = enemy_pos + Vec2::new(0.0, 30.0);
            let detection_radius = 8.0 + (vision.detection_buildup * 12.0);
            gizmos.circle_2d(detection_pos, detection_radius, detection_color);
        }
    }
}

// Helper function to draw vision cone
fn draw_vision_cone(gizmos: &mut Gizmos, position: Vec2, vision: &AgentVision, color: Color) {
    let half_angle = vision.angle / 2.0;
    let base_direction = vision.direction;
    
    // Calculate cone edges
    let left_direction = Vec2::new(
        base_direction.x * half_angle.cos() - base_direction.y * half_angle.sin(),
        base_direction.x * half_angle.sin() + base_direction.y * half_angle.cos(),
    );
    
    let right_direction = Vec2::new(
        base_direction.x * half_angle.cos() + base_direction.y * half_angle.sin(),
        -base_direction.x * half_angle.sin() + base_direction.y * half_angle.cos(),
    );
    
    // Draw cone outline
    let left_end = position + left_direction * vision.range;
    let right_end = position + right_direction * vision.range;
    
    gizmos.line_2d(position, left_end, color);
    gizmos.line_2d(position, right_end, color);
    
    // Draw arc for the cone end
    for i in 0..VISION_CONE_SEGMENTS {
        let t1 = i as f32 / VISION_CONE_SEGMENTS as f32;
        let t2 = (i + 1) as f32 / VISION_CONE_SEGMENTS as f32;
        
        let angle1 = -half_angle + (vision.angle * t1);
        let angle2 = -half_angle + (vision.angle * t2);
        
        let dir1 = Vec2::new(
            base_direction.x * angle1.cos() - base_direction.y * angle1.sin(),
            base_direction.x * angle1.sin() + base_direction.y * angle1.cos(),
        );
        
        let dir2 = Vec2::new(
            base_direction.x * angle2.cos() - base_direction.y * angle2.sin(),
            base_direction.x * angle2.sin() + base_direction.y * angle2.cos(),
        );
        
        let point1 = position + dir1 * vision.range;
        let point2 = position + dir2 * vision.range;
        
        gizmos.line_2d(point1, point2, color);
    }
}

// Equipment inventory management system
pub fn inventory_management_system(
    mut completion_events: EventReader<InteractionCompleteEvent>,
    mut agent_query: Query<&mut EquipmentInventory, With<Agent>>,
    mut inventory_state: ResMut<InventoryState>,
) {
    // Process completed interactions and add rewards to inventory
    for event in completion_events.read() {
        if let Ok(mut inventory) = agent_query.get_mut(event.agent) {
            for reward in &event.rewards {
                match reward {
                    InteractionReward::Equipment(equipment) => {
                        match equipment {
                            Equipment::Weapon(weapon) => {
                                inventory.weapons.push(weapon.clone());
                                inventory_state.recent_acquisitions.push(format!("Acquired weapon: {:?}", weapon));
                            }
                            Equipment::Tool(tool) => {
                                inventory.tools.push(tool.clone());
                                inventory_state.recent_acquisitions.push(format!("Acquired tool: {:?}", tool));
                            }
                            Equipment::Armor(_armor) => {
                                // Armor would be handled here
                                inventory_state.recent_acquisitions.push("Acquired armor".to_string());
                            }
                        }
                    }
                    InteractionReward::SkillMatrix(skill) => {
                        inventory.skill_matrices.push(skill.clone());
                        inventory_state.recent_acquisitions.push(format!("Acquired skill: {:?}", skill));
                    }
                    InteractionReward::Currency(amount) => {
                        inventory.currency += amount;
                        inventory_state.recent_acquisitions.push(format!("Credits: +{}", amount));
                    }
                    InteractionReward::Intel(document) => {
                        inventory.intel_documents.push(document.clone());
                        inventory_state.recent_acquisitions.push("New intel acquired".to_string());
                    }
                    InteractionReward::AccessCard(level) => {
                        if !inventory.access_cards.contains(level) {
                            inventory.access_cards.push(*level);
                            inventory_state.recent_acquisitions.push(format!("Access card: {:?}", level));
                        }
                    }
                    InteractionReward::ObjectiveProgress => {
                        inventory_state.recent_acquisitions.push("Objective completed!".to_string());
                    }
                }
            }
            
            info!("Equipment added to inventory for agent {:?}", event.agent);
        }
    }
}

// Inventory UI toggle system
pub fn inventory_ui_system(
    input: Query<&ActionState<PlayerAction>>,
    mut inventory_state: ResMut<InventoryState>,
    selection_state: Res<SelectionState>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    if let Ok(_action_state) = input.get_single() {
        // Toggle inventory with 'I' key
        if keyboard.just_pressed(KeyCode::KeyI) {
            inventory_state.ui_open = !inventory_state.ui_open;
            
            // Set selected agent if inventory is opening
            if inventory_state.ui_open {
                inventory_state.selected_agent = selection_state.selected_agents.first().copied();
            }
            
            info!("Inventory UI {}", if inventory_state.ui_open { "opened" } else { "closed" });
        }
        
        // Close with Escape
        if keyboard.just_pressed(KeyCode::Escape) && inventory_state.ui_open {
            inventory_state.ui_open = false;
        }
    }
}

// Inventory UI rendering system
pub fn inventory_ui_render_system(
    mut commands: Commands,
    inventory_state: Res<InventoryState>,
    agent_query: Query<&EquipmentInventory, With<Agent>>,
    ui_query: Query<Entity, (With<Node>, Without<Camera>)>,
) {
    // Clear existing UI
    for entity in ui_query.iter() {
        commands.entity(entity).despawn_recursive();
    }
    
    if !inventory_state.ui_open {
        return;
    }
    
    // Get inventory data
    let inventory = if let Some(agent_entity) = inventory_state.selected_agent {
        agent_query.get(agent_entity).ok()
    } else {
        None
    };
    
    // Create inventory panel
    commands.spawn(NodeBundle {
        style: Style {
            width: Val::Px(INVENTORY_PANEL_WIDTH),
            height: Val::Px(INVENTORY_PANEL_HEIGHT),
            position_type: PositionType::Absolute,
            left: Val::Px(50.0),
            top: Val::Px(50.0),
            flex_direction: FlexDirection::Column,
            padding: UiRect::all(Val::Px(10.0)),
            ..default()
        },
        background_color: Color::srgba(0.1, 0.1, 0.1, 0.9).into(),
        ..default()
    }).with_children(|parent| {
        // Title
        parent.spawn(TextBundle::from_section(
            "AGENT INVENTORY",
            TextStyle {
                font_size: 24.0,
                color: Color::WHITE,
                ..default()
            },
        ));
        
        if let Some(inv) = inventory {
            // Currency display
            parent.spawn(TextBundle::from_section(
                format!("Credits: {}", inv.currency),
                TextStyle {
                    font_size: 18.0,
                    color: Color::srgb(0.8, 0.8, 0.2),
                    ..default()
                },
            ));
            
            // Weapons section
            if !inv.weapons.is_empty() {
                parent.spawn(TextBundle::from_section(
                    "WEAPONS:",
                    TextStyle {
                        font_size: 16.0,
                        color: Color::srgb(0.8, 0.3, 0.3),
                        ..default()
                    },
                ));
                
                for weapon in &inv.weapons {
                    parent.spawn(TextBundle::from_section(
                        format!("• {:?}", weapon),
                        TextStyle {
                            font_size: 14.0,
                            color: Color::WHITE,
                            ..default()
                        },
                    ));
                }
            }
            
            // Tools section
            if !inv.tools.is_empty() {
                parent.spawn(TextBundle::from_section(
                    "TOOLS:",
                    TextStyle {
                        font_size: 16.0,
                        color: Color::srgb(0.3, 0.8, 0.3),
                        ..default()
                    },
                ));
                
                for tool in &inv.tools {
                    parent.spawn(TextBundle::from_section(
                        format!("• {:?}", tool),
                        TextStyle {
                            font_size: 14.0,
                            color: Color::WHITE,
                            ..default()
                        },
                    ));
                }
            }
            
            // Cybernetics section
            if !inv.cybernetics.is_empty() {
                parent.spawn(TextBundle::from_section(
                    "CYBERNETICS:",
                    TextStyle {
                        font_size: 16.0,
                        color: Color::srgb(0.3, 0.3, 0.8),
                        ..default()
                    },
                ));
                
                for cybernetic in &inv.cybernetics {
                    parent.spawn(TextBundle::from_section(
                        format!("• {:?}", cybernetic),
                        TextStyle {
                            font_size: 14.0,
                            color: Color::WHITE,
                            ..default()
                        },
                    ));
                }
            }
            
            // Skill matrices section
            if !inv.skill_matrices.is_empty() {
                parent.spawn(TextBundle::from_section(
                    "SKILL MATRICES:",
                    TextStyle {
                        font_size: 16.0,
                        color: Color::srgb(0.8, 0.3, 0.8),
                        ..default()
                    },
                ));
                
                for skill in &inv.skill_matrices {
                    parent.spawn(TextBundle::from_section(
                        format!("• {:?}", skill),
                        TextStyle {
                            font_size: 14.0,
                            color: Color::WHITE,
                            ..default()
                        },
                    ));
                }
            }
            
            // Intel documents section
            if !inv.intel_documents.is_empty() {
                parent.spawn(TextBundle::from_section(
                    "INTEL DOCUMENTS:",
                    TextStyle {
                        font_size: 16.0,
                        color: Color::srgb(0.8, 0.8, 0.3),
                        ..default()
                    },
                ));
                
                for (i, document) in inv.intel_documents.iter().enumerate() {
                    let preview = if document.len() > 50 {
                        format!("{}...", &document[..47])
                    } else {
                        document.clone()
                    };
                    
                    parent.spawn(TextBundle::from_section(
                        format!("• Document {}: {}", i + 1, preview),
                        TextStyle {
                            font_size: 12.0,
                            color: Color::WHITE,
                            ..default()
                        },
                    ));
                }
            }
        } else {
            parent.spawn(TextBundle::from_section(
                "No agent selected",
                TextStyle {
                    font_size: 16.0,
                    color: Color::srgb(0.8, 0.3, 0.3),
                    ..default()
                },
            ));
        }
        
        // Instructions
        parent.spawn(TextBundle::from_section(
            "\nPress 'I' to close inventory",
            TextStyle {
                font_size: 12.0,
                color: Color::srgb(0.7, 0.7, 0.7),
                ..default()
            },
        ));
    });
}

// Notification system for equipment acquisitions
pub fn equipment_notification_system(
    mut commands: Commands,
    mut inventory_state: ResMut<InventoryState>,
) {
    // Display recent acquisitions as notifications
    if !inventory_state.recent_acquisitions.is_empty() {
        // Find a position that doesn't conflict with inventory panel
        let notification_x = if inventory_state.ui_open { 
            INVENTORY_PANEL_WIDTH + 70.0 
        } else { 
            50.0 
        };
        
        commands.spawn(NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                left: Val::Px(notification_x),
                top: Val::Px(50.0),
                flex_direction: FlexDirection::Column,
                ..default()
            },
            ..default()
        }).with_children(|parent| {
            for (i, notification) in inventory_state.recent_acquisitions.iter().enumerate() {
                if i < 5 { // Limit to 5 recent notifications
                    parent.spawn(NodeBundle {
                        style: Style {
                            padding: UiRect::all(Val::Px(8.0)),
                            margin: UiRect::bottom(Val::Px(4.0)),
                            ..default()
                        },
                        background_color: Color::srgba(0.2, 0.8, 0.2, 0.8).into(),
                        ..default()
                    }).with_children(|notification_parent| {
                        notification_parent.spawn(TextBundle::from_section(
                            notification,
                            TextStyle {
                                font_size: 14.0,
                                color: Color::WHITE,
                                ..default()
                            },
                        ));
                    });
                }
            }
        });
        
        // Clear notifications after displaying (simplified)
        if inventory_state.recent_acquisitions.len() > 10 {
            inventory_state.recent_acquisitions.clear();
        }
    }
}