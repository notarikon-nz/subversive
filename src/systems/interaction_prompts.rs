// src/systems/interaction_prompts.rs
use bevy::prelude::*;
use crate::core::*;

#[derive(Component)]
pub struct DeviceTooltip {
    pub target_entity: Entity,
    pub device_name: String,
}

#[derive(Resource)]
pub struct InteractionSprites {
    pub key_e: Handle<Image>,
    pub key_f: Handle<Image>,
    pub key_r: Handle<Image>,
    pub key_question: Handle<Image>,
    pub security_bar: Handle<Image>,
    pub security_bar_filled: Handle<Image>,
}

#[derive(Component)]
pub struct InteractionPrompt {
    pub target_entity: Entity,
    pub prompt_type: InteractionType,
}

#[derive(Component)]
pub struct SecurityLevelDisplay {
    pub target_entity: Entity,
    pub level: u8,
}

#[derive(PartialEq, Copy, Clone, Debug)]
pub enum InteractionType {
    Interact,    // E key
    Attack,      // F key  
    Reload,      // R key
    Unavailable, // ? symbol
}

pub fn load_interaction_sprites(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    let interaction_sprites = InteractionSprites {
        key_e: asset_server.load("sprites/ui/key_e.png"),
        key_f: asset_server.load("sprites/ui/key_f.png"),
        key_r: asset_server.load("sprites/ui/key_r.png"),
        key_question: asset_server.load("sprites/ui/key_question.png"),
        security_bar: asset_server.load("sprites/ui/security_bar.png"),
        security_bar_filled: asset_server.load("sprites/ui/security_bar_filled.png"),
    };
    
    commands.insert_resource(interaction_sprites);
}

// Replace the old gizmos-based interaction system
pub fn interaction_prompt_system(
    mut commands: Commands,
    interaction_sprites: Res<InteractionSprites>,
    selection: Res<SelectionState>,
    inventory_query: Query<(&Transform, &Inventory), With<Agent>>,
    terminal_query: Query<(Entity, &Transform, &Terminal, Option<&LoreSource>)>,
    hackable_query: Query<(Entity, &Transform, &Hackable, &DeviceState)>,
    
    // Cleanup old prompts
    existing_prompts: Query<Entity, Or<(With<InteractionPrompt>, With<DeviceTooltip>, Without<MarkedForDespawn>)>>,
    game_mode: Res<GameMode>,
) {
    if game_mode.paused { return; }

    // Clean up existing prompts
    for entity in existing_prompts.iter() {
        commands.entity(entity).insert(MarkedForDespawn);
    }

    // Show prompts for selected agents
    for &selected_agent in &selection.selected {
        if let Ok((agent_transform, inventory)) = inventory_query.get(selected_agent) {
            let agent_pos = agent_transform.translation.truncate();

            // Show terminal prompts
            spawn_terminal_prompts(
                &mut commands,
                &interaction_sprites,
                agent_pos,
                &terminal_query,
            );
            
            // Show hackable device prompts
            spawn_hackable_prompts(
                &mut commands,
                &interaction_sprites,
                agent_pos,
                inventory,
                &hackable_query,
            );
        }
    }
}

fn get_device_name(device_type: &DeviceType) -> &'static str {
    match device_type {
        DeviceType::Camera => "Security Camera",
        DeviceType::Turret => "Auto Turret",
        DeviceType::Drone => "Security Drone", 
        DeviceType::Door => "Electronic Door",
        DeviceType::Elevator => "Elevator",
        DeviceType::Vehicle => "Vehicle System",
        DeviceType::PowerStation => "Power Station",
        DeviceType::StreetLight => "Street Light",
        DeviceType::TrafficLight => "Traffic Light",
        DeviceType::Terminal => "Data Terminal",
        DeviceType::SecuritySystem => "Security System",
        DeviceType::AlarmPanel => "Alarm Panel",
        DeviceType::ATM => "ATM Machine",
        DeviceType::Billboard => "Digital Billboard",
    }
}

fn get_device_color(device_type: &DeviceType) -> Color {
    match device_type {
        DeviceType::Camera | DeviceType::Drone => Color::srgb(0.2, 0.6, 0.8),      // Blue - surveillance
        DeviceType::Turret | DeviceType::SecuritySystem => Color::srgb(0.8, 0.2, 0.2), // Red - defensive
        DeviceType::Door | DeviceType::Elevator => Color::srgb(0.6, 0.4, 0.2),     // Brown - access
        DeviceType::PowerStation | DeviceType::StreetLight => Color::srgb(0.8, 0.8, 0.2), // Yellow - power
        DeviceType::TrafficLight => Color::srgb(0.8, 0.6, 0.2),                   // Orange - traffic
        DeviceType::ATM | DeviceType::Terminal => Color::srgb(0.2, 0.8, 0.6),      // Green - data
        DeviceType::AlarmPanel => Color::srgb(0.8, 0.4, 0.2),                     // Orange - alerts
        DeviceType::Billboard => Color::srgb(0.6, 0.2, 0.8),                      // Purple - media
        DeviceType::Vehicle => Color::srgb(0.4, 0.4, 0.4),                        // Gray - vehicles
    }
}

fn spawn_device_tooltip(
    commands: &mut Commands,
    position: Vec2,
    device_type: &DeviceType,
    security_level: u8,
    target_entity: Entity,
) {
    let device_name = get_device_name(device_type);
    let tooltip_text = format!("{} (Sec:{})", device_name, security_level);
    let text_width = tooltip_text.len() as f32 * 6.0 + 8.0;
    
    // Tooltip background
    commands.spawn((
        Sprite {
            color: Color::srgba(0.0, 0.0, 0.0, 0.8),
            custom_size: Some(Vec2::new(text_width, 16.0)),
            ..default()
        },
        Transform::from_translation(position.extend(102.0)),
        GlobalTransform::default(),
        DeviceTooltip {
            target_entity,
            device_name: tooltip_text.clone(),
        },
    ));
    
    // Tooltip border
    commands.spawn((
        Sprite {
            color: get_device_color(device_type),
            custom_size: Some(Vec2::new(text_width + 2.0, 18.0)),
            ..default()
        },
        Transform::from_translation(position.extend(101.0)),
        GlobalTransform::default(),
        DeviceTooltip {
            target_entity,
            device_name: tooltip_text,
        },
    ));
}

fn spawn_terminal_prompts(
    commands: &mut Commands,
    sprites: &InteractionSprites,
    agent_pos: Vec2,
    terminal_query: &Query<(Entity, &Transform, &Terminal, Option<&LoreSource>)>,
) {
    for (entity, terminal_transform, terminal, lore_source) in terminal_query.iter() {
        if terminal.accessed && lore_source.map_or(true, |ls| ls.accessed) { 
            continue; 
        }

        let terminal_pos = terminal_transform.translation.truncate();
        let distance = agent_pos.distance(terminal_pos);

        if distance <= terminal.range {
            let prompt_pos = terminal_pos + Vec2::new(0.0, 30.0);
            
            spawn_key_prompt(
                commands,
                sprites,
                prompt_pos,
                InteractionType::Interact,
                entity,
                get_terminal_color(&terminal.terminal_type),
            );
        }
    }
}

fn spawn_hackable_prompts(
    commands: &mut Commands,
    sprites: &InteractionSprites,
    agent_pos: Vec2,
    inventory: &Inventory,
    hackable_query: &Query<(Entity, &Transform, &Hackable, &DeviceState)>,
) {
    for (entity, transform, hackable, device_state) in hackable_query.iter() {
        if hackable.is_hacked { continue; }
        
        let device_pos = transform.translation.truncate();
        let distance = agent_pos.distance(device_pos);
        let interaction_range = 80.0;

        if distance <= interaction_range {
            let has_tool = check_hack_tool_available(inventory, hackable);
            let is_operational = device_state.powered && device_state.operational;
            
            let prompt_pos = device_pos + Vec2::new(0.0, 30.0);
            
            if has_tool && is_operational {
                spawn_key_prompt(
                    commands,
                    sprites,
                    prompt_pos,
                    InteractionType::Interact,
                    entity,
                    Color::srgb(0.2, 0.8, 0.8), // Cyan for hackable
                );
                
                // Add security level display
                spawn_device_tooltip(
                    commands,
                    device_pos + Vec2::new(0.0, 45.0),
                    &hackable.device_type,
                    hackable.security_level,
                    entity,
                );
            } else {
                spawn_key_prompt(
                    commands,
                    sprites,
                    prompt_pos,
                    InteractionType::Unavailable,
                    entity,
                    Color::srgb(0.8, 0.2, 0.2), // Red for unavailable
                );

                // Show tooltip for unavailable devices too
                spawn_device_tooltip(
                    commands,
                    device_pos + Vec2::new(0.0, 45.0),
                    &hackable.device_type,
                    hackable.security_level,
                    entity,
                );                
            }
        }
    }
}

fn spawn_key_prompt(
    commands: &mut Commands,
    sprites: &InteractionSprites,
    position: Vec2,
    prompt_type: InteractionType,
    target_entity: Entity,
    tint_color: Color,
) {
    let sprite_handle = match prompt_type {
        InteractionType::Interact => &sprites.key_e,
        InteractionType::Attack => &sprites.key_f,
        InteractionType::Reload => &sprites.key_r,
        InteractionType::Unavailable => &sprites.key_question,
    };

    // Key sprite
    commands.spawn((
        Sprite {
            image: sprite_handle.clone(),
            color: tint_color,
            custom_size: Some(Vec2::new(32.0, 32.0)),
            ..default()
        },
        Transform::from_translation(position.extend(101.0)),
        GlobalTransform::default(),
        InteractionPrompt {
            target_entity,
            prompt_type,
        },
    ));
}

fn spawn_security_display(
    commands: &mut Commands,
    sprites: &InteractionSprites,
    position: Vec2,
    security_level: u8,
    target_entity: Entity,
) {
    let bar_width = 1.0;
    let bar_height = 4.0;
    let bar_spacing = 1.0;
    
    for i in 0..5 {
        let bar_pos = position + Vec2::new(i as f32 * (bar_width + bar_spacing), 0.0);
        let is_filled = i < security_level;
        
        let (sprite_handle, color) = if is_filled {
            (&sprites.security_bar_filled, get_security_color(security_level))
        } else {
            (&sprites.security_bar, Color::srgb(0.3, 0.3, 0.3))
        };

        commands.spawn((
            Sprite {
                image: sprite_handle.clone(),
                color,
                custom_size: Some(Vec2::new(bar_width, bar_height)),
                ..default()
            },
            Transform::from_translation(bar_pos.extend(100.0)),
            GlobalTransform::default(),
            SecurityLevelDisplay {
                target_entity,
                level: security_level,
            },
        ));
    }
}

fn get_terminal_color(terminal_type: &TerminalType) -> Color {
    match terminal_type {
        TerminalType::Objective => Color::srgb(0.9, 0.2, 0.2),  // Red
        TerminalType::Equipment => Color::srgb(0.2, 0.5, 0.9),  // Blue  
        TerminalType::Intel => Color::srgb(0.2, 0.8, 0.3),     // Green
    }
}

fn get_security_color(security_level: u8) -> Color {
    match security_level {
        1..=2 => Color::srgb(0.2, 0.8, 0.2), // Green - easy
        3 => Color::srgb(0.8, 0.8, 0.2),     // Yellow - medium
        4..=5 => Color::srgb(0.8, 0.2, 0.2), // Red - hard
        _ => Color::WHITE,
    }
}

fn check_hack_tool_available(inventory: &Inventory, hackable: &Hackable) -> bool {
    match &hackable.requires_tool {
        Some(required_tool) => {
            inventory.equipped_tools.iter().any(|tool| {
                matches!((tool, required_tool), 
                    (ToolType::Hacker, HackTool::BasicHacker) |
                    (ToolType::Hacker, HackTool::AdvancedHacker)
                )
            })
        },
        None => true,
    }
}

// Animation system for prompts (optional - adds polish)
pub fn animate_interaction_prompts(
    time: Res<Time>,
    mut prompt_query: Query<&mut Transform, With<InteractionPrompt>>,
) {
    let pulse = (time.elapsed_secs() * 3.0).sin() * 0.1 + 1.0;
    
    for mut transform in prompt_query.iter_mut() {
        transform.scale = Vec3::splat(pulse);
    }
}

// Cleanup system to remove prompts when targets are despawned
pub fn cleanup_orphaned_prompts(
    mut commands: Commands,
    prompt_query: Query<(Entity, &InteractionPrompt), Without<MarkedForDespawn>>,
    tooltip_query: Query<(Entity, &DeviceTooltip), Without<MarkedForDespawn>>, 
    entity_query: Query<Entity>, // All entities to check if targets still exist
) {
    // Check interaction prompts
    for (prompt_entity, prompt) in prompt_query.iter() {
        if entity_query.get(prompt.target_entity).is_err() {
            commands.entity(prompt_entity).insert(MarkedForDespawn);
        }
    }
    
    for (tooltip_entity, tooltip) in tooltip_query.iter() {
        if entity_query.get(tooltip.target_entity).is_err() {
            commands.entity(tooltip_entity).insert(MarkedForDespawn);
        }
    }
}

use crate::systems::access_control::{AccessCard};

pub fn access_card_pickup_system(
    mut commands: Commands,
    mut action_events: EventReader<ActionEvent>,
    mut agent_query: Query<&mut Inventory, With<Agent>>,
    card_query: Query<(Entity, &AccessCard), Without<Agent>>,
    mut audio_events: EventWriter<AudioEvent>,
) {
    for event in action_events.read() {
        if let Action::InteractWith(target) = event.action {
            // Check if target is an access card
            if let Ok((card_entity, access_card)) = card_query.get(target) {
                if let Ok(mut inventory) = agent_query.get_mut(event.entity) {
                    // Add card to inventory
                    inventory.add_access_card(access_card.level, access_card.card_type);
                    
                    // Remove card from world
                    commands.entity(card_entity).insert(MarkedForDespawn);
                    
                    // Play pickup sound
                    audio_events.write(AudioEvent {
                        sound: AudioType::CardSwipe,
                        volume: 0.4,
                    });
                    
                    info!("Agent picked up {:?} access card (level {})", 
                          access_card.card_type, access_card.level);
                }
            }
        }
    }
}

pub fn access_card_interaction_prompts(
    mut commands: Commands,
    interaction_sprites: Res<InteractionSprites>,
    selection: Res<SelectionState>,
    agent_query: Query<&Transform, With<Agent>>,
    card_query: Query<(Entity, &Transform, &AccessCard)>,
    existing_prompts: Query<Entity, With<InteractionPrompt>>,
    game_mode: Res<GameMode>,
) {
    if game_mode.paused { return; }

    for &selected_agent in &selection.selected {
        if let Ok(agent_transform) = agent_query.get(selected_agent) {
            let agent_pos = agent_transform.translation.truncate();

            // Check for nearby access cards
            for (card_entity, card_transform, access_card) in card_query.iter() {
                let card_pos = card_transform.translation.truncate();
                let distance = agent_pos.distance(card_pos);
                
                if distance <= 30.0 {
                    let prompt_pos = card_pos + Vec2::new(0.0, 15.0);
                    
                    // Background
                    commands.spawn((
                        Sprite {
                            color: Color::srgba(0.0, 0.0, 0.0, 0.7),
                            custom_size: Some(Vec2::new(20.0, 20.0)),
                            ..default()
                        },
                        Transform::from_translation(prompt_pos.extend(100.0)),
                        InteractionPrompt {
                            target_entity: card_entity,
                            prompt_type: InteractionType::Interact,
                        },
                    ));

                    // Key sprite
                    commands.spawn((
                        Sprite {
                            image: interaction_sprites.key_e.clone(),
                            color: Color::srgb(0.8, 0.8, 0.2),
                            custom_size: Some(Vec2::new(24.0, 24.0)),
                            ..default()
                        },
                        Transform::from_translation(prompt_pos.extend(101.0)),
                        InteractionPrompt {
                            target_entity: card_entity,
                            prompt_type: InteractionType::Interact,
                        },
                    ));
                }
            }
        }
    }
}