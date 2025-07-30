// src/systems/financial_hacking.rs - ATMs and Billboards
use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use crate::core::*;
use crate::systems::pathfinding::PathfindingObstacle;
use crate::systems::scanner::Scannable;
use serde::{Deserialize, Serialize};
use crate::systems::interaction_prompts::{InteractionPrompt, InteractionSprites, InteractionType};
use crate::systems::minimap::{MinimapSettings};


// === COMPONENTS ===
#[derive(Component)]
pub struct ATM {
    pub bank_id: String,
    pub max_withdrawal: u32,
    pub current_balance: u32,
    pub requires_account_data: bool,
}

#[derive(Component)]
pub struct Billboard {
    pub influence_radius: f32,
    pub persuasion_bonus: f32,
    pub active: bool,
}

#[derive(Component)]
pub struct BankAccount {
    pub account_number: String,
    pub bank_id: String,
    pub balance: u32,
    pub access_code: String,
}

// === RESOURCES ===
#[derive(Resource, Default)]
pub struct BankingNetwork {
    pub banks: Vec<Bank>,
    pub stolen_accounts: Vec<StolenAccountData>,
}

#[derive(Clone)]
pub struct Bank {
    pub id: String,
    pub name: String,
    pub total_funds: u32,
    pub security_level: u8,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct StolenAccountData {
    pub account_number: String,
    pub bank_id: String,
    pub balance: u32,
    pub source: String, // Where we got this data from
}

// === SPAWNING FUNCTIONS ===
pub fn spawn_atm(
    commands: &mut Commands,
    position: Vec2,
    bank_id: String,
    network_id: Option<String>,
    power_grid: &mut Option<ResMut<PowerGrid>>,
) -> Entity {
    let entity = commands.spawn((
        Sprite {
            color: Color::srgb(0.3, 0.5, 0.7),
            custom_size: Some(Vec2::new(16.0, 24.0)),
            ..default()
        },
        Transform::from_translation(position.extend(1.0)),
        ATM {
            bank_id: bank_id.clone(),
            max_withdrawal: 5000,
            current_balance: 50000,
            requires_account_data: true,
        },
        RigidBody::Fixed,
        Collider::cuboid(8.0, 12.0),
        CollisionGroups::new(TERMINAL_GROUP, Group::ALL),
        Selectable { radius: 20.0 },
        Scannable,
        PathfindingObstacle {
            radius: 12.0,
            blocks_movement: true,
        },
    )).id();

    // Make hackable - ATMs are like advanced terminals
    if let (Some(network_id), Some(power_grid)) = (network_id, power_grid) {
        make_hackable_networked(commands, entity, DeviceType::Terminal, network_id, power_grid);
    } else {
        let mut hackable = Hackable::new(DeviceType::Terminal);
        hackable.security_level = 3; // ATMs are more secure
        hackable.hack_time = 6.0;
        hackable.requires_tool = Some(HackTool::AdvancedHacker);
        commands.entity(entity).insert((hackable, DeviceState::new(DeviceType::Terminal)));
    }

    entity
}

pub fn spawn_billboard(
    commands: &mut Commands,
    position: Vec2,
    network_id: Option<String>,
    power_grid: &mut Option<ResMut<PowerGrid>>,
) -> Entity {
    let entity = commands.spawn((
        Sprite {
            color: Color::srgb(0.8, 0.6, 0.2),
            custom_size: Some(Vec2::new(40.0, 20.0)),
            ..default()
        },
        Transform::from_translation(position.extend(1.0)),
        Billboard {
            influence_radius: 100.0,
            persuasion_bonus: 0.3,
            active: true,
        },
        RigidBody::Fixed,
        Collider::cuboid(20.0, 10.0),
        CollisionGroups::new(TERMINAL_GROUP, Group::ALL),
        Selectable { radius: 25.0 },
        Scannable,
        PathfindingObstacle {
            radius: 22.0,
            blocks_movement: true,
        },
    )).id();

    // Billboards are easier to hack
    if let (Some(network_id), Some(power_grid)) = (network_id, power_grid) {
        make_hackable_networked(commands, entity, DeviceType::Terminal, network_id, power_grid);
    } else {
        let mut hackable = Hackable::new(DeviceType::Terminal);
        hackable.security_level = 2;
        hackable.hack_time = 3.0;
        hackable.requires_tool = Some(HackTool::BasicHacker);
        commands.entity(entity).insert((hackable, DeviceState::new(DeviceType::Terminal)));
    }

    entity
}

// === ATM HACKING SYSTEM ===
pub fn atm_hacking_system(
    mut hack_completed: EventReader<HackCompletedEvent>,
    mut agent_query: Query<&mut Inventory, With<Agent>>,
    mut atm_query: Query<&mut ATM>,
    banking_network: Res<BankingNetwork>,
    mut audio_events: EventWriter<AudioEvent>,
) {
    for event in hack_completed.read() {
        // Check if this was an ATM hack
        if let Ok(mut atm) = atm_query.get_mut(event.target) {
            if let Ok(mut inventory) = agent_query.get_mut(event.agent) {
                let withdrawal_amount = calculate_atm_withdrawal(&atm, &inventory, &banking_network);
                
                if withdrawal_amount > 0 {
                    inventory.add_currency(withdrawal_amount);
                    atm.current_balance = atm.current_balance.saturating_sub(withdrawal_amount);
                    
                    // Success audio
                    audio_events.write(AudioEvent {
                        sound: AudioType::AccessGranted,
                        volume: 0.6,
                    });
                    
                    info!("ATM hack successful! Withdrew ${}", withdrawal_amount);
                } else {
                    // Failure audio
                    audio_events.write(AudioEvent {
                        sound: AudioType::AccessDenied,
                        volume: 0.4,
                    });
                    
                    info!("ATM hack failed - no valid account data or insufficient funds");
                }
            }
        }
    }
}

// === BILLBOARD INFLUENCE SYSTEM ===
pub fn billboard_influence_system(
    billboard_query: Query<(&Transform, &Billboard, &DeviceState)>,
    mut civilian_query: Query<(Entity, &Transform, &mut NeurovectorTarget), With<Civilian>>,
    mut persuasion_bonus: Local<f32>,
) {
    *persuasion_bonus = 0.0;
    
    // Calculate total billboard influence
    for (billboard_transform, billboard, device_state) in billboard_query.iter() {
        if !billboard.active || !device_state.operational || !device_state.powered {
            continue;
        }
        
        let billboard_pos = billboard_transform.translation.truncate();
        
        // Apply influence to nearby civilians
        for (_, civilian_transform, mut neurovector_target) in civilian_query.iter_mut() {
            let civilian_pos = civilian_transform.translation.truncate();
            let distance = billboard_pos.distance(civilian_pos);
            
            if distance <= billboard.influence_radius {
                // Closer = stronger influence
                let influence_strength = 1.0 - (distance / billboard.influence_radius);
                *persuasion_bonus += billboard.persuasion_bonus * influence_strength;
            }
        }
    }
    
    // Store bonus for neurovector system to use
    // This would integrate with your existing neurovector system
}

// === ACCOUNT DATA ACQUISITION ===
pub fn terminal_account_data_system(
    mut hack_completed: EventReader<HackCompletedEvent>,
    mut agent_query: Query<&mut Inventory, With<Agent>>,
    terminal_query: Query<&Terminal>,
    mut banking_network: ResMut<BankingNetwork>,
) {
    for event in hack_completed.read() {
        // Check if this was a terminal hack that might have account data
        if let Ok(terminal) = terminal_query.get(event.target) {
            if matches!(terminal.terminal_type, TerminalType::Intel) {
                if let Ok(mut inventory) = agent_query.get_mut(event.agent) {
                    // 30% chance to find account data in intel terminals
                    if rand::random::<f32>() < 0.3 {
                        let account_data = generate_stolen_account_data();
                        banking_network.stolen_accounts.push(account_data.clone());
                        
                        // Add to inventory as intel
                        inventory.add_intel(format!("Bank Account: {} - ${}", 
                                                  account_data.account_number, 
                                                  account_data.balance));
                        
                        info!("Found bank account data: {} (${}) from {}",
                              account_data.account_number,
                              account_data.balance,
                              account_data.source);
                    }
                }
            }
        }
    }
}

// === ENHANCED INTERACTION PROMPTS ===
pub fn financial_interaction_prompts(
    mut commands: Commands,
    interaction_sprites: Res<InteractionSprites>,
    selection: Res<SelectionState>,
    agent_query: Query<(&Transform, &Inventory), With<Agent>>,
    atm_query: Query<(Entity, &Transform, &ATM, &Hackable, &DeviceState)>,
    billboard_query: Query<(Entity, &Transform, &Billboard, &Hackable, &DeviceState)>,
    existing_prompts: Query<Entity, With<InteractionPrompt>>,
    banking_network: Res<BankingNetwork>,
    game_mode: Res<GameMode>,
) {
    if game_mode.paused { return; }

    for &selected_agent in &selection.selected {
        if let Ok((agent_transform, inventory)) = agent_query.get(selected_agent) {
            let agent_pos = agent_transform.translation.truncate();

            // ATM prompts
            for (entity, transform, atm, hackable, device_state) in atm_query.iter() {
                let atm_pos = transform.translation.truncate();
                let distance = agent_pos.distance(atm_pos);
                
                if distance <= 40.0 {
                    let can_hack = check_hack_tool_available(inventory, hackable);
                    let has_account_data = has_valid_account_data(inventory, &banking_network, &atm.bank_id);
                    let is_operational = device_state.powered && device_state.operational;
                    
                    let (prompt_type, color, tooltip) = if hackable.is_hacked {
                        (InteractionType::Unavailable, Color::srgb(0.2, 0.8, 0.2), "ATM Already Hacked")
                    } else if can_hack && is_operational {
                        if has_account_data {
                            (InteractionType::Interact, Color::srgb(0.2, 0.8, 0.2), "Hack ATM (Account Data Available)")
                        } else {
                            (InteractionType::Interact, Color::srgb(0.8, 0.6, 0.2), "Hack ATM (No Account Data)")
                        }
                    } else if !can_hack {
                        (InteractionType::Unavailable, Color::srgb(0.8, 0.2, 0.2), "Requires Advanced Hacker")
                    } else {
                        (InteractionType::Unavailable, Color::srgb(0.6, 0.6, 0.2), "ATM Offline")
                    };
                    
                    spawn_financial_prompt(&mut commands, &interaction_sprites, 
                                         atm_pos + Vec2::new(0.0, 30.0), entity, 
                                         prompt_type, color, tooltip);
                }
            }

            // Billboard prompts
            for (entity, transform, billboard, hackable, device_state) in billboard_query.iter() {
                let billboard_pos = transform.translation.truncate();
                let distance = agent_pos.distance(billboard_pos);
                
                if distance <= 50.0 {
                    let can_hack = check_hack_tool_available(inventory, hackable);
                    let is_operational = device_state.powered && device_state.operational;
                    
                    let (prompt_type, color, tooltip) = if hackable.is_hacked {
                        (InteractionType::Unavailable, Color::srgb(0.2, 0.8, 0.2), "Billboard Hacked")
                    } else if can_hack && is_operational {
                        (InteractionType::Interact, Color::srgb(0.8, 0.2, 0.8), "Hack Billboard")
                    } else if !can_hack {
                        (InteractionType::Unavailable, Color::srgb(0.8, 0.2, 0.2), "Requires Hacker Tool")
                    } else {
                        (InteractionType::Unavailable, Color::srgb(0.6, 0.6, 0.2), "Billboard Offline")
                    };
                    
                    spawn_financial_prompt(&mut commands, &interaction_sprites,
                                         billboard_pos + Vec2::new(0.0, 35.0), entity,
                                         prompt_type, color, tooltip);
                }
            }
        }
    }
}

// === MINIMAP INTEGRATION ===
pub fn update_financial_minimap(
    settings: Res<MinimapSettings>,
    mut minimap_dots: Local<Vec<(Entity, Vec2, Color)>>,
    atm_query: Query<(Entity, &Transform), With<ATM>>,
    billboard_query: Query<(Entity, &Transform), With<Billboard>>,
    camera: Query<&Transform, (With<Camera2d>, Without<ATM>, Without<Billboard>)>,
) {
    minimap_dots.clear();
    
    let camera_pos = camera.single().map(|t| t.translation.truncate()).unwrap_or(Vec2::ZERO);
    let range = settings.range;
    
    // Add ATMs (blue like terminals)
    for (entity, transform) in atm_query.iter() {
        let world_pos = transform.translation.truncate();
        if world_pos.distance(camera_pos) <= range {
            if let Some(minimap_pos) = world_to_minimap_pos(world_pos, camera_pos, range, settings.size) {
                minimap_dots.push((entity, minimap_pos, Color::srgb(0.2, 0.6, 1.0)));
            }
        }
    }
    
    // Add Billboards (purple)
    for (entity, transform) in billboard_query.iter() {
        let world_pos = transform.translation.truncate();
        if world_pos.distance(camera_pos) <= range {
            if let Some(minimap_pos) = world_to_minimap_pos(world_pos, camera_pos, range, settings.size) {
                minimap_dots.push((entity, minimap_pos, Color::srgb(0.8, 0.2, 0.8)));
            }
        }
    }
}

// === HELPER FUNCTIONS ===
fn calculate_atm_withdrawal(atm: &ATM, inventory: &Inventory, banking_network: &BankingNetwork) -> u32 {
    if atm.requires_account_data {
        // Find valid account for this bank
        let valid_account = banking_network.stolen_accounts.iter()
            .find(|account| account.bank_id == atm.bank_id);
        
        if let Some(account) = valid_account {
            let max_withdrawal = atm.max_withdrawal.min(atm.current_balance).min(account.balance);
            // Reduce by 10-20% for "transaction fees" and realism
            (max_withdrawal as f32 * (0.8 + rand::random::<f32>() * 0.2)) as u32
        } else {
            // No valid account data - can only get small amount
            (500 + rand::random::<u32>() % 1000).min(atm.current_balance)
        }
    } else {
        // Unsecured ATM - full access
        atm.max_withdrawal.min(atm.current_balance)
    }
}

fn has_valid_account_data(inventory: &Inventory, banking_network: &BankingNetwork, bank_id: &str) -> bool {
    // Check if we have stolen account data for this bank
    banking_network.stolen_accounts.iter().any(|account| account.bank_id == bank_id) ||
    // Check if we have bank intel documents
    inventory.intel_documents.iter().any(|doc| doc.contains(bank_id))
}

fn generate_stolen_account_data() -> StolenAccountData {
    let banks = ["MegaBank", "CyberCredit", "DataVault Financial", "NeoTokyo Savings"];
    let bank_id = banks[rand::random::<usize>() % banks.len()].to_string();
    
    StolenAccountData {
        account_number: format!("{:08}", rand::random::<u32>() % 100000000),
        bank_id,
        balance: 1000 + rand::random::<u32>() % 50000,
        source: "Corporate Terminal".to_string(),
    }
}

fn spawn_financial_prompt(
    commands: &mut Commands,
    sprites: &InteractionSprites,
    position: Vec2,
    target_entity: Entity,
    prompt_type: InteractionType,
    color: Color,
    tooltip: &str,
) {
    let sprite_handle = match prompt_type {
        InteractionType::Interact => &sprites.key_e,
        InteractionType::Unavailable => &sprites.key_question,
        _ => &sprites.key_e,
    };

    // Background
    commands.spawn((
        Sprite {
            color: Color::srgba(0.0, 0.0, 0.0, 0.7),
            custom_size: Some(Vec2::new(28.0, 28.0)),
            ..default()
        },
        Transform::from_translation(position.extend(100.0)),
        InteractionPrompt { target_entity, prompt_type },
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
        InteractionPrompt { target_entity, prompt_type },
    ));
}

fn world_to_minimap_pos(world_pos: Vec2, center_pos: Vec2, range: f32, minimap_size: f32) -> Option<Vec2> {
    let relative_pos = world_pos - center_pos;
    let distance = relative_pos.length();
    
    if distance > range {
        return None;
    }
    
    let normalized = relative_pos / range;
    let minimap_pos = normalized * (minimap_size * 0.4);
    Some(minimap_pos)
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

// === SETUP HELPER ===
pub fn setup_financial_district(
    commands: &mut Commands,
    mut power_grid: ResMut<PowerGrid>,
    mut banking_network: ResMut<BankingNetwork>,
    center: Vec2,
) {
    let network_id = "financial_district".to_string();
    
    // Set up banking network
    banking_network.banks.push(Bank {
        id: "MegaBank".to_string(),
        name: "MegaBank Corp".to_string(),
        total_funds: 1000000,
        security_level: 4,
    });
    
    banking_network.banks.push(Bank {
        id: "CyberCredit".to_string(),
        name: "CyberCredit Union".to_string(),
        total_funds: 500000,
        security_level: 3,
    });
    
    // Spawn ATMs
    let mut power_grid_option = Some(power_grid);
    spawn_atm(commands, center + Vec2::new(-50.0, 100.0), 
             "MegaBank".to_string(), Some(network_id.clone()), &mut power_grid_option);
    spawn_atm(commands, center + Vec2::new(150.0, -80.0), 
             "CyberCredit".to_string(), Some(network_id.clone()), &mut power_grid_option);
    
    // Spawn Billboards
    spawn_billboard(commands, center + Vec2::new(0.0, 200.0), 
                   Some(network_id.clone()), &mut power_grid_option);
    spawn_billboard(commands, center + Vec2::new(300.0, 0.0), 
                   Some(network_id), &mut power_grid_option);
}