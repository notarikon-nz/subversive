use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use leafwing_input_manager::prelude::*;
use std::f32::consts::PI;

use crate::components::*;
use crate::resources::*;
use crate::events::*;
use crate::states::*;

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