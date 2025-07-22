use bevy::prelude::*;
use crate::core::*;
use crate::core::lore::*;
use crate::core::hackable::*;

pub fn system(
    mut gizmos: Gizmos,
    mut action_events: EventReader<ActionEvent>,
    mut audio_events: EventWriter<AudioEvent>,
    mut lore_events: EventWriter<LoreAccessEvent>,
    mut hack_events: EventWriter<HackAttemptEvent>,
    selection: Res<SelectionState>,
    // Fixed: Use ParamSet to separate conflicting Inventory queries
    mut inventory_queries: ParamSet<(
        Query<(&Transform, &Inventory), With<Agent>>,
        Query<&mut Inventory, With<Agent>>,
    )>,
    mut terminal_query: Query<(Entity, &Transform, &mut Terminal, Option<&LoreSource>)>,
    hackable_query: Query<(Entity, &Transform, &Hackable, &DeviceState)>,
    mut mission_data: ResMut<MissionData>,
    game_mode: Res<GameMode>,
) {
    if game_mode.paused { return; }

    // Draw interaction prompts for selected agents
    for &selected_agent in &selection.selected {
        if let Ok((agent_transform, inventory)) = inventory_queries.p0().get(selected_agent) {
            let agent_pos = agent_transform.translation.truncate();

            // Show terminal interaction prompts
            draw_terminal_prompts(&mut gizmos, agent_pos, &terminal_query);
            
            // Show hackable device prompts
            draw_hackable_prompts(&mut gizmos, agent_pos, inventory, &hackable_query);
        }
    }

    // Process interaction events
    for event in action_events.read() {
        if let Action::InteractWith(_) = event.action {
            // Get agent position first
            let agent_pos = if let Ok((agent_transform, _)) = inventory_queries.p0().get(event.entity) {
                agent_transform.translation.truncate()
            } else {
                continue;
            };
            
            // Try terminal interaction first
            if let Some(terminal_entity) = find_closest_terminal(agent_pos, &terminal_query) {
                execute_terminal_interaction(
                    &mut terminal_query,
                    &mut inventory_queries.p1(),
                    terminal_entity,
                    event.entity,
                    &mut mission_data,
                    &mut audio_events,
                    &mut lore_events,
                );
                continue;
            }
            
            // Try hackable device interaction - get inventory again when needed
            if let Ok((_, inventory)) = inventory_queries.p0().get(event.entity) {
                if let Some(hackable_entity) = find_closest_hackable(agent_pos, inventory, &hackable_query) {
                    execute_hack_interaction(
                        hackable_entity,
                        event.entity,
                        inventory,
                        &mut hack_events,
                    );
                }
            }
        }
    }
}


fn draw_terminal_prompts(
    gizmos: &mut Gizmos,
    agent_pos: Vec2,
    terminal_query: &Query<(Entity, &Transform, &mut Terminal, Option<&LoreSource>)>,
) {
    for (_, terminal_transform, terminal, lore_source) in terminal_query.iter() {
        if terminal.accessed && lore_source.map_or(true, |ls| ls.accessed) { 
            continue; 
        }

        let terminal_pos = terminal_transform.translation.truncate();
        let distance = agent_pos.distance(terminal_pos);

        if distance <= terminal.range {
            let color = match terminal.terminal_type {
                TerminalType::Objective => Color::srgba(0.9, 0.2, 0.2, 0.4),
                TerminalType::Equipment => Color::srgba(0.2, 0.5, 0.9, 0.4),
                TerminalType::Intel => Color::srgba(0.2, 0.8, 0.3, 0.4),
            };
            
            // Draw interaction range
            gizmos.circle_2d(terminal_pos, terminal.range, color);
            
            // Draw "E" prompt
            draw_interaction_prompt(gizmos, terminal_pos, "E", Color::WHITE);
        }
    }
}

fn draw_hackable_prompts(
    gizmos: &mut Gizmos,
    agent_pos: Vec2,
    inventory: &Inventory,
    hackable_query: &Query<(Entity, &Transform, &Hackable, &DeviceState)>,
) {
    for (_, transform, hackable, device_state) in hackable_query.iter() {
        if hackable.is_hacked { continue; }
        
        let device_pos = transform.translation.truncate();
        let distance = agent_pos.distance(device_pos);
        let interaction_range = 40.0; // Standard hacking range

        if distance <= interaction_range {
            // Check if agent has required tool
            let has_tool = check_hack_tool_available(inventory, hackable);
            
            let (prompt_color, prompt_text) = if has_tool {
                (Color::srgb(0.2, 0.8, 0.8), "E") // Cyan = can hack
            } else {
                (Color::srgb(0.8, 0.2, 0.2), "?") // Red = missing tool
            };
            
            // Draw device outline
            let device_color = if device_state.powered && device_state.operational {
                Color::srgba(0.8, 0.8, 0.2, 0.4) // Yellow = hackable
            } else {
                Color::srgba(0.5, 0.5, 0.5, 0.4) // Gray = offline
            };
            
            gizmos.circle_2d(device_pos, interaction_range, device_color);
            
            // Draw interaction prompt
            draw_interaction_prompt(gizmos, device_pos, prompt_text, prompt_color);
            
            // Draw security level indicator
            draw_security_level(gizmos, device_pos, hackable.security_level);
        }
    }
}

fn draw_interaction_prompt(gizmos: &mut Gizmos, position: Vec2, text: &str, color: Color) {
    let prompt_pos = position + Vec2::new(0.0, 25.0);
    
    // Background
    gizmos.rect_2d(prompt_pos, Vec2::new(15.0, 15.0), Color::srgba(0.0, 0.0, 0.0, 0.8));
    
    // Border  
    gizmos.rect_2d(prompt_pos, Vec2::new(16.0, 16.0), color);
    
    // Simple text representation (just a colored circle for now)
    gizmos.circle_2d(prompt_pos, 5.0, color);
}

fn draw_security_level(gizmos: &mut Gizmos, position: Vec2, security_level: u8) {
    let indicator_pos = position + Vec2::new(15.0, 15.0);
    
    // Draw security level as colored bars
    for i in 0..5 {
        let bar_pos = indicator_pos + Vec2::new(i as f32 * 3.0, 0.0);
        let bar_color = if i < security_level {
            match security_level {
                1..=2 => Color::srgb(0.2, 0.8, 0.2), // Green = easy
                3 => Color::srgb(0.8, 0.8, 0.2),     // Yellow = medium  
                4..=5 => Color::srgb(0.8, 0.2, 0.2), // Red = hard
                _ => Color::WHITE,
            }
        } else {
            Color::srgb(0.3, 0.3, 0.3) // Gray = empty
        };
        
        gizmos.rect_2d(bar_pos, Vec2::new(2.0, 8.0), bar_color);
    }
}

fn find_closest_terminal(
    agent_pos: Vec2,
    terminal_query: &Query<(Entity, &Transform, &mut Terminal, Option<&LoreSource>)>,
) -> Option<Entity> {
    terminal_query.iter()
        .filter(|(_, _, terminal, lore_source)| {
            !terminal.accessed || lore_source.map_or(false, |ls| !ls.accessed)
        })
        .filter(|(_, transform, terminal, _)| {
            agent_pos.distance(transform.translation.truncate()) <= terminal.range
        })
        .min_by(|(_, a_transform, _, _), (_, b_transform, _, _)| {
            let a_dist = agent_pos.distance(a_transform.translation.truncate());
            let b_dist = agent_pos.distance(b_transform.translation.truncate());
            a_dist.partial_cmp(&b_dist).unwrap_or(std::cmp::Ordering::Equal)
        })
        .map(|(entity, _, _, _)| entity)
}

fn find_closest_hackable(
    agent_pos: Vec2,
    inventory: &Inventory,
    hackable_query: &Query<(Entity, &Transform, &Hackable, &DeviceState)>,
) -> Option<Entity> {
    let interaction_range = 40.0;
    
    hackable_query.iter()
        .filter(|(_, _, hackable, _)| !hackable.is_hacked)
        .filter(|(_, transform, _, _)| {
            agent_pos.distance(transform.translation.truncate()) <= interaction_range
        })
        .filter(|(_, _, hackable, _)| check_hack_tool_available(inventory, hackable))
        .min_by(|(_, a_transform, _, _), (_, b_transform, _, _)| {
            let a_dist = agent_pos.distance(a_transform.translation.truncate());
            let b_dist = agent_pos.distance(b_transform.translation.truncate());
            a_dist.partial_cmp(&b_dist).unwrap_or(std::cmp::Ordering::Equal)
        })
        .map(|(entity, _, _, _)| entity)
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
        None => true, // No tool required
    }
}

fn execute_terminal_interaction(
    terminal_query: &mut Query<(Entity, &Transform, &mut Terminal, Option<&LoreSource>)>,
    agent_inventory_query: &mut Query<&mut Inventory, With<Agent>>,
    terminal_entity: Entity,
    agent_entity: Entity,
    mission_data: &mut ResMut<MissionData>,
    audio_events: &mut EventWriter<AudioEvent>,
    lore_events: &mut EventWriter<LoreAccessEvent>,
) {
    if let Ok((_, _, mut terminal, lore_source)) = terminal_query.get_mut(terminal_entity) {
        let mut terminal_accessed = false;
        let mut lore_accessed = false;
        
        // Handle regular terminal interaction
        if !terminal.accessed {
            if let Ok(mut inventory) = agent_inventory_query.get_mut(agent_entity) {
                terminal.accessed = true;
                terminal_accessed = true;
                mission_data.terminals_accessed += 1;
                
                match terminal.terminal_type {
                    TerminalType::Objective => {
                        inventory.add_currency(500);
                        mission_data.objectives_completed += 1;
                        info!("Objective completed! ({}/{})", 
                              mission_data.objectives_completed, mission_data.total_objectives);
                    }
                    TerminalType::Equipment => {
                        inventory.add_weapon(WeaponType::Rifle);
                        inventory.add_tool(ToolType::Hacker);
                        inventory.add_currency(200);
                        info!("Equipment acquired!");
                    }
                    TerminalType::Intel => {
                        inventory.add_intel("Corporate research logs...".to_string());
                        inventory.add_currency(50);
                        info!("Intel acquired!");
                    }
                }
            }
        }
        
        // Handle lore interaction
        if let Some(lore_source) = lore_source {
            if !lore_source.accessed || !lore_source.one_time_use {
                lore_events.write(LoreAccessEvent {
                    agent: agent_entity,
                    source: terminal_entity,
                });
                lore_accessed = true;
            }
        }
        
        // Play sound if any interaction occurred
        if terminal_accessed || lore_accessed {
            audio_events.write(AudioEvent {
                sound: AudioType::TerminalAccess,
                volume: 0.6,
            });
        }
    }
}

fn execute_hack_interaction(
    hackable_entity: Entity,
    agent_entity: Entity,
    inventory: &Inventory,
    hack_events: &mut EventWriter<HackAttemptEvent>,
) {
    // Determine which tool to use - check for equipped tools using PartialEq
    let has_hacker = inventory.equipped_tools.iter().any(|tool| {
        matches!(tool, ToolType::Hacker)
    });
    
    if !has_hacker {
        info!("Hack failed: No hacker tool equipped");
        return;
    }
    
    let tool_used = HackTool::BasicHacker; // Default to basic for now
    
    hack_events.write(HackAttemptEvent {
        agent: agent_entity,
        target: hackable_entity,
        tool_used,
    });
    
    info!("Hack attempt initiated on entity {:?}", hackable_entity.index());
}
