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
    };
    
    commands.insert_resource(interaction_sprites);
}

fn get_terminal_color(terminal_type: &TerminalType) -> Color {
    match terminal_type {
        TerminalType::Objective => Color::srgb(0.9, 0.2, 0.2),  // Red
        TerminalType::Equipment => Color::srgb(0.2, 0.5, 0.9),  // Blue  
        TerminalType::Intel => Color::srgb(0.2, 0.8, 0.3),     // Green
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

// 0.2.12
// === TEMPORARY INTERACTION PROMPTS ===
pub fn research_interaction_prompts(
    commands: Commands,
    scientist_query: Query<(Entity, &Transform, &Scientist, &ScientistNPC), Without<Agent>>,
    agent_query: Query<&Transform, With<Agent>>,
    sprites: Res<GameSprites>,
) {
    for agent_transform in agent_query.iter() {
        let agent_pos = agent_transform.translation.truncate();
        
        for (scientist_entity, scientist_transform, scientist, npc) in scientist_query.iter() {
            let scientist_pos = scientist_transform.translation.truncate();
            let distance = agent_pos.distance(scientist_pos);
            
            if distance <= 50.0 {
                let prompt_text = if !npc.location_discovered {
                    "Press E to approach scientist"
                } else if scientist.is_recruited {
                    "Press E to talk with scientist"
                } else if npc.recruitment_difficulty > 0 {
                    "Press E to build rapport"
                } else {
                    "Press E to recruit scientist"
                };
                
                // Spawn interaction prompt (using your existing prompt system)
                // PLACEHOLDER
                // spawn_interaction_prompt(&mut commands, scientist_pos, prompt_text, &sprites, );
            }
        }
    }
}