use bevy::prelude::*;
use crate::core::*;

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
    let interaction_range = 80.0;
    
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
