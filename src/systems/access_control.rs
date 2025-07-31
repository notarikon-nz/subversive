// src/systems/access_control.rs - Gates and doors with motion sensors and access cards
use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use crate::core::*;
use crate::systems::scanner::{Scannable};
use crate::systems::pathfinding::{PathfindingObstacle};
use crate::systems::interaction_prompts::{InteractionPrompt, InteractionSprites, InteractionType};
use crate::core::hackable::{setup_hackable_door};
use serde::{Deserialize, Serialize};

// === COMPONENTS ===
#[derive(Component)]
pub struct Gate {
    pub is_open: bool,
    pub requires_vehicle: bool,
    pub open_distance: f32,
    pub close_distance: f32,
    pub access_level: Option<u8>,
    pub open_timer: f32,
    pub auto_close_delay: f32,
}

#[derive(Component)]
pub struct Door {
    pub is_open: bool,
    pub requires_person: bool,
    pub open_distance: f32,
    pub close_distance: f32,
    pub access_level: Option<u8>,
    pub open_timer: f32,
    pub auto_close_delay: f32,
}

#[derive(Component)]
pub struct MotionSensor {
    pub detection_range: f32,
    pub target_type: SensorTarget,
    pub active: bool,
}

#[derive(Component)]
pub struct AccessCard {
    pub level: u8,
    pub card_type: CardType,
}

#[derive(Component)]
pub struct AccessReader {
    pub required_level: u8,
    pub active: bool,
}

#[derive(Debug, Clone, Copy)]
pub enum SensorTarget {
    Vehicle,
    Person,
    Any,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum CardType {
    Basic,
    Security,
    Executive,
    Master,
}

#[derive(Debug, Clone, Copy)]
pub enum AccessResult {
    Granted,
    Denied,
    RequiresCard,
    RequiresHacking,
}

// === EVENTS ===
#[derive(Event)]
pub struct AccessEvent {
    pub entity: Entity,
    pub agent: Entity,
    pub result: AccessResult,
}

#[derive(Event)]
pub struct GateStateChange {
    pub gate: Entity,
    pub opened: bool,
}

#[derive(Event)]
pub struct DoorStateChange {
    pub door: Entity,
    pub opened: bool,
}

// === SPAWNING FUNCTIONS ===
pub fn spawn_gate(
    commands: &mut Commands,
    position: Vec2,
    requires_vehicle: bool,
    access_level: Option<u8>,
) -> Entity {
    let entity = commands.spawn((
        Sprite {
            color: Color::srgb(0.6, 0.4, 0.2),
            custom_size: Some(Vec2::new(8.0, 60.0)),
            ..default()
        },
        Transform::from_translation(position.extend(1.0)),
        Gate {
            is_open: false,
            requires_vehicle,
            open_distance: 80.0,
            close_distance: 120.0,
            access_level,
            open_timer: 0.0,
            auto_close_delay: 5.0,
        },
        MotionSensor {
            detection_range: 80.0,
            target_type: if requires_vehicle { SensorTarget::Vehicle } else { SensorTarget::Any },
            active: true,
        },
        RigidBody::Fixed,
        Collider::cuboid(4.0, 30.0),
        CollisionGroups::new(TERMINAL_GROUP, Group::ALL),
        Selectable { radius: 20.0 },
        Scannable,
        PathfindingObstacle {
            radius: 30.0,
            blocks_movement: true,
        },
    )).id();

    // Add access reader if required
    if access_level.is_some() {
        commands.entity(entity).insert(AccessReader {
            required_level: access_level.unwrap_or(1),
            active: true,
        });
    }

    // Make hackable if secured
    if access_level.is_some() {
        setup_hackable_door(commands, entity);
    }

    entity
}

pub fn spawn_door(
    commands: &mut Commands,
    position: Vec2,
    requires_person: bool,
    access_level: Option<u8>,
) -> Entity {
    let entity = commands.spawn((
        Sprite {
            color: Color::srgb(0.4, 0.4, 0.6),
            custom_size: Some(Vec2::new(8.0, 32.0)),
            ..default()
        },
        Transform::from_translation(position.extend(1.0)),
        Door {
            is_open: false,
            requires_person,
            open_distance: 40.0,
            close_distance: 60.0,
            access_level,
            open_timer: 0.0,
            auto_close_delay: 3.0,
        },
        MotionSensor {
            detection_range: 40.0,
            target_type: if requires_person { SensorTarget::Person } else { SensorTarget::Any },
            active: true,
        },
        RigidBody::Fixed,
        Collider::cuboid(4.0, 16.0),
        CollisionGroups::new(TERMINAL_GROUP, Group::ALL),
        Selectable { radius: 15.0 },
        Scannable,
        PathfindingObstacle {
            radius: 16.0,
            blocks_movement: true,
        },
    )).id();

    // Add access reader if required
    if access_level.is_some() {
        commands.entity(entity).insert(AccessReader {
            required_level: access_level.unwrap_or(1),
            active: true,
        });
    }

    // Make hackable if secured
    if access_level.is_some() {
        setup_hackable_door(commands, entity);
    }

    entity
}

pub fn spawn_access_card(
    commands: &mut Commands,
    position: Vec2,
    level: u8,
    card_type: CardType,
) -> Entity {
    let color = match card_type {
        CardType::Basic => Color::srgb(0.7, 0.7, 0.7),
        CardType::Security => Color::srgb(0.8, 0.2, 0.2),
        CardType::Executive => Color::srgb(0.2, 0.2, 0.8),
        CardType::Master => Color::srgb(0.8, 0.8, 0.2),
    };

    commands.spawn((
        Sprite {
            color,
            custom_size: Some(Vec2::new(12.0, 8.0)),
            ..default()
        },
        Transform::from_translation(position.extend(1.0)),
        AccessCard { level, card_type },
        Selectable { radius: 10.0 },
        Scannable,
    )).id()
}

// === MOTION SENSOR SYSTEM ===
pub fn motion_sensor_system(
    mut gate_query: Query<(Entity, &mut Gate, &Transform, &MotionSensor), Without<Door>>,
    mut door_query: Query<(Entity, &mut Door, &Transform, &MotionSensor), Without<Gate>>,
    vehicle_query: Query<&Transform, (With<Vehicle>, Without<Gate>, Without<Door>)>,
    person_query: Query<&Transform, (Or<(With<Agent>, With<Civilian>)>, Without<Gate>, Without<Door>)>,
    mut gate_events: EventWriter<GateStateChange>,
    mut door_events: EventWriter<DoorStateChange>,
    time: Res<Time>,
) {
    // Handle gates
    for (entity, mut gate, gate_transform, sensor) in gate_query.iter_mut() {
        if !sensor.active { continue; }

        let gate_pos = gate_transform.translation.truncate();
        let mut should_open = false;

        // Check for appropriate targets
        match sensor.target_type {
            SensorTarget::Vehicle => {
                should_open = vehicle_query.iter().any(|transform| {
                    gate_pos.distance(transform.translation.truncate()) <= sensor.detection_range
                });
            },
            SensorTarget::Person => {
                should_open = person_query.iter().any(|transform| {
                    gate_pos.distance(transform.translation.truncate()) <= sensor.detection_range
                });
            },
            SensorTarget::Any => {
                should_open = vehicle_query.iter().any(|transform| {
                    gate_pos.distance(transform.translation.truncate()) <= sensor.detection_range
                }) || person_query.iter().any(|transform| {
                    gate_pos.distance(transform.translation.truncate()) <= sensor.detection_range
                });
            },
        }

        // Update gate state
        if should_open && !gate.is_open && gate.access_level.is_none() {
            gate.is_open = true;
            gate.open_timer = 0.0;
            gate_events.write(GateStateChange { gate: entity, opened: true });
        } else if gate.is_open {
            gate.open_timer += time.delta_secs();
            if gate.open_timer >= gate.auto_close_delay && !should_open {
                gate.is_open = false;
                gate_events.write(GateStateChange { gate: entity, opened: false });
            }
        }
    }

    // Handle doors (similar logic)
    for (entity, mut door, door_transform, sensor) in door_query.iter_mut() {
        if !sensor.active { continue; }

        let door_pos = door_transform.translation.truncate();
        let mut should_open = false;

        match sensor.target_type {
            SensorTarget::Person => {
                should_open = person_query.iter().any(|transform| {
                    door_pos.distance(transform.translation.truncate()) <= sensor.detection_range
                });
            },
            SensorTarget::Any => {
                should_open = person_query.iter().any(|transform| {
                    door_pos.distance(transform.translation.truncate()) <= sensor.detection_range
                });
            },
            _ => {},
        }

        if should_open && !door.is_open && door.access_level.is_none() {
            door.is_open = true;
            door.open_timer = 0.0;
            door_events.write(DoorStateChange { door: entity, opened: true });
        } else if door.is_open {
            door.open_timer += time.delta_secs();
            if door.open_timer >= door.auto_close_delay && !should_open {
                door.is_open = false;
                door_events.write(DoorStateChange { door: entity, opened: false });
            }
        }
    }
}

// === ACCESS CONTROL SYSTEM ===
pub fn access_control_system(
    mut action_events: EventReader<ActionEvent>,
    mut access_events: EventWriter<AccessEvent>,
    mut gate_query: Query<(Entity, &mut Gate, &Transform, Option<&AccessReader>), Without<Door>>,
    mut door_query: Query<(Entity, &mut Door, &Transform, Option<&AccessReader>), Without<Gate>>,
    agent_query: Query<(&Transform, &Inventory), With<Agent>>,
    mut gate_events: EventWriter<GateStateChange>,
    mut door_events: EventWriter<DoorStateChange>,
) {
    for event in action_events.read() {
        if let Action::InteractWith(target) = event.action {
            // Check if target is a gate
            if let Ok((gate_entity, mut gate, gate_transform, access_reader)) = gate_query.get_mut(target) {
                if let Ok((agent_transform, inventory)) = agent_query.get(event.entity) {
                    let result = check_access(inventory, access_reader);
                    
                    if matches!(result, AccessResult::Granted) {
                        gate.is_open = !gate.is_open;
                        gate_events.write(GateStateChange { 
                            gate: gate_entity, 
                            opened: gate.is_open 
                        });
                    }
                    
                    access_events.write(AccessEvent {
                        entity: gate_entity,
                        agent: event.entity,
                        result,
                    });
                }
            }
            
            // Check if target is a door
            if let Ok((door_entity, mut door, door_transform, access_reader)) = door_query.get_mut(target) {
                if let Ok((agent_transform, inventory)) = agent_query.get(event.entity) {
                    let result = check_access(inventory, access_reader);
                    
                    if matches!(result, AccessResult::Granted) {
                        door.is_open = !door.is_open;
                        door_events.write(DoorStateChange { 
                            door: door_entity, 
                            opened: door.is_open 
                        });
                    }
                    
                    access_events.write(AccessEvent {
                        entity: door_entity,
                        agent: event.entity,
                        result,
                    });
                }
            }
        }
    }
}

// === VISUAL AND AUDIO FEEDBACK ===
pub fn gate_door_audio_system(
    mut gate_events: EventReader<GateStateChange>,
    mut door_events: EventReader<DoorStateChange>,
    mut audio_events: EventWriter<AudioEvent>,
) {
    for event in gate_events.read() {
        audio_events.write(AudioEvent {
            sound: if event.opened { AudioType::TerminalAccess } else { AudioType::Alert },
            volume: 0.4,
        });
    }
    
    for event in door_events.read() {
        audio_events.write(AudioEvent {
            sound: if event.opened { AudioType::TerminalAccess } else { AudioType::Alert },
            volume: 0.3,
        });
    }
}

pub fn gate_door_visual_system(
    mut gate_query: Query<(&Gate, &mut Sprite, &mut Collider), (With<Gate>, Without<Door>)>,
    mut door_query: Query<(&Door, &mut Sprite, &mut Collider), (With<Door>, Without<Gate>)>,
) {
    // Update gate visuals and physics
    for (gate, mut sprite, mut collider) in gate_query.iter_mut() {
        if gate.is_open {
            sprite.color = Color::srgb(0.3, 0.6, 0.3); // Green when open
            *collider = Collider::cuboid(0.1, 0.1); // Make passable
        } else {
            sprite.color = if gate.access_level.is_some() {
                Color::srgb(0.8, 0.2, 0.2) // Red when locked
            } else {
                Color::srgb(0.6, 0.4, 0.2) // Brown when closed but unlocked
            };
            *collider = Collider::cuboid(4.0, 30.0); // Solid barrier
        }
    }
    
    // Update door visuals and physics
    for (door, mut sprite, mut collider) in door_query.iter_mut() {
        if door.is_open {
            sprite.color = Color::srgb(0.3, 0.6, 0.3); // Green when open
            *collider = Collider::cuboid(0.1, 0.1); // Make passable
        } else {
            sprite.color = if door.access_level.is_some() {
                Color::srgb(0.8, 0.2, 0.2) // Red when locked
            } else {
                Color::srgb(0.4, 0.4, 0.6) // Blue when closed but unlocked
            };
            *collider = Collider::cuboid(4.0, 16.0); // Solid barrier
        }
    }
}

// === INTERACTION PROMPTS ===
pub fn access_control_prompts(
    mut commands: Commands,
    interaction_sprites: Res<InteractionSprites>,
    selection: Res<SelectionState>,
    agent_query: Query<(&Transform, &Inventory), With<Agent>>,
    gate_query: Query<(Entity, &Transform, &Gate, Option<&AccessReader>), Without<Door>>,
    door_query: Query<(Entity, &Transform, &Door, Option<&AccessReader>), Without<Gate>>,
    existing_prompts: Query<Entity, (With<InteractionPrompt>, Without<MarkedForDespawn>)>,
    game_mode: Res<GameMode>,
) {
    if game_mode.paused { return; }

    // Clean up existing prompts
    for entity in existing_prompts.iter() {
        commands.entity(entity).insert(MarkedForDespawn);
    }

    for &selected_agent in &selection.selected {
        if let Ok((agent_transform, inventory)) = agent_query.get(selected_agent) {
            let agent_pos = agent_transform.translation.truncate();

            // Check gates
            for (entity, transform, gate, access_reader) in gate_query.iter() {
                let gate_pos = transform.translation.truncate();
                let distance = agent_pos.distance(gate_pos);
                
                if distance <= 50.0 {
                    let prompt_pos = gate_pos + Vec2::new(0.0, 35.0);
                    let access_result = check_access(inventory, access_reader);
                    
                    spawn_access_prompt(
                        &mut commands,
                        &interaction_sprites,
                        prompt_pos,
                        entity,
                        access_result,
                        "Gate",
                    );
                }
            }

            // Check doors
            for (entity, transform, door, access_reader) in door_query.iter() {
                let door_pos = transform.translation.truncate();
                let distance = agent_pos.distance(door_pos);
                
                if distance <= 30.0 {
                    let prompt_pos = door_pos + Vec2::new(0.0, 20.0);
                    let access_result = check_access(inventory, access_reader);
                    
                    spawn_access_prompt(
                        &mut commands,
                        &interaction_sprites,
                        prompt_pos,
                        entity,
                        access_result,
                        "Door",
                    );
                }
            }
        }
    }
}

// === HELPER FUNCTIONS ===
fn check_access(inventory: &Inventory, access_reader: Option<&AccessReader>) -> AccessResult {
    if let Some(reader) = access_reader {
        if !reader.active {
            return AccessResult::Denied;
        }
        
        // Check for access cards in inventory items (simplified - check if agent has any access card)
        let has_access_card = inventory.items.iter().any(|item| {
            matches!(item, crate::core::components::OriginalInventoryItem::AccessCard { .. })
        });
        
        if has_access_card {
            AccessResult::Granted
        } else if can_hack_access_system(inventory) {
            AccessResult::RequiresHacking
        } else {
            AccessResult::RequiresCard
        }
    } else {
        AccessResult::Granted // No access control
    }
}

fn can_hack_access_system(inventory: &Inventory) -> bool {
    inventory.equipped_tools.iter().any(|tool| {
        matches!(tool, ToolType::Hacker)
    })
}

fn spawn_access_prompt(
    commands: &mut Commands,
    sprites: &InteractionSprites,
    position: Vec2,
    target_entity: Entity,
    access_result: AccessResult,
    entity_type: &str,
) {
    let (prompt_type, color, tooltip) = match access_result {
        AccessResult::Granted => (
            InteractionType::Interact,
            Color::srgb(0.2, 0.8, 0.2),
            format!("Open {}", entity_type),
        ),
        AccessResult::RequiresHacking => (
            InteractionType::Interact,
            Color::srgb(0.2, 0.8, 0.8),
            format!("Hack {}", entity_type),
        ),
        AccessResult::RequiresCard => (
            InteractionType::Unavailable,
            Color::srgb(0.8, 0.8, 0.2),
            "Requires Access Card".to_string(),
        ),
        AccessResult::Denied => (
            InteractionType::Unavailable,
            Color::srgb(0.8, 0.2, 0.2),
            "Access Denied".to_string(),
        ),
    };

    let sprite_handle = match prompt_type {
        InteractionType::Interact => &sprites.key_e,
        InteractionType::Unavailable => &sprites.key_question,
        _ => &sprites.key_e,
    };

    // Background
    commands.spawn((
        Sprite {
            color: Color::srgba(0.0, 0.0, 0.0, 0.7),
            custom_size: Some(Vec2::new(24.0, 24.0)),
            ..default()
        },
        Transform::from_translation(position.extend(100.0)),
        InteractionPrompt {
            target_entity,
            prompt_type,
        },
    ));

    // Key sprite
    commands.spawn((
        Sprite {
            image: sprite_handle.clone(),
            color,
            custom_size: Some(Vec2::new(32.0, 32.0)),
            ..default()
        },
        Transform::from_translation(position.extend(101.0)),
        InteractionPrompt {
            target_entity,
            prompt_type,
        },
    ));
}

// === INTEGRATION EXAMPLES ===
pub fn setup_secure_facility(
    commands: &mut Commands,
    center: Vec2,
) {
    // Main entrance gate (requires vehicle)
    spawn_gate(commands, center + Vec2::new(-100.0, 0.0), true, Some(2));
    
    // Personnel door (requires person + access card)
    spawn_door(commands, center + Vec2::new(-120.0, 0.0), true, Some(1));
    
    // Emergency exit (no access control)
    spawn_door(commands, center + Vec2::new(100.0, 50.0), true, None);
    
    // High security area
    spawn_door(commands, center + Vec2::new(0.0, 80.0), true, Some(3));
    
    // Access cards
    spawn_access_card(commands, center + Vec2::new(-50.0, -20.0), 1, CardType::Basic);
    spawn_access_card(commands, center + Vec2::new(50.0, -20.0), 2, CardType::Security);
}

