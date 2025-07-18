use bevy::prelude::*;
use leafwing_input_manager::prelude::*;
use crate::core::*;

pub fn system(
    mut gizmos: Gizmos,
    input: Query<&ActionState<PlayerAction>>,
    mut action_events: EventReader<ActionEvent>,
    selection: Res<SelectionState>,
    agent_query: Query<&Transform, With<Agent>>,
    mut agent_inventory_query: Query<&mut Inventory, With<Agent>>,
    mut terminal_query: Query<(Entity, &Transform, &mut Terminal)>,
    mut mission_data: ResMut<MissionData>,
    game_mode: Res<GameMode>,
) {
    if game_mode.paused { return; }

    let Ok(action_state) = input.get_single() else { return; };
    let mut interaction_target = None;
    let mut selected_agent_entity = None;

    // Draw interaction prompts and detect interaction input
    for &selected_agent in &selection.selected {
        if let Ok(agent_transform) = agent_query.get(selected_agent) {
            let agent_pos = agent_transform.translation.truncate();

            for (terminal_entity, terminal_transform, terminal) in terminal_query.iter() {
                if terminal.accessed { continue; }

                let terminal_pos = terminal_transform.translation.truncate();
                let distance = agent_pos.distance(terminal_pos);

                if distance <= terminal.range {
                    let color = match terminal.terminal_type {
                        TerminalType::Objective => Color::srgba(0.9, 0.2, 0.2, 0.3),
                        TerminalType::Equipment => Color::srgba(0.2, 0.5, 0.9, 0.3),
                        TerminalType::Intel => Color::srgba(0.2, 0.8, 0.3, 0.3),
                    };
                    
                    gizmos.circle_2d(terminal_pos, terminal.range, color);
                    
                    if action_state.just_pressed(&PlayerAction::Interact) {
                        interaction_target = Some(terminal_entity);
                        selected_agent_entity = Some(selected_agent);
                    }
                }
            }
        }
    }

    if let (Some(terminal_entity), Some(agent_entity)) = (interaction_target, selected_agent_entity) {
        execute_terminal_interaction(&mut terminal_query, &mut agent_inventory_query, terminal_entity, agent_entity, &mut mission_data);
    }

    for event in action_events.read() {
        if let Action::InteractWith(terminal_entity) = event.action {
            execute_terminal_interaction(&mut terminal_query, &mut agent_inventory_query, terminal_entity, event.entity, &mut mission_data);
        }
    }
}

fn execute_terminal_interaction(
    terminal_query: &mut Query<(Entity, &Transform, &mut Terminal)>,
    agent_inventory_query: &mut Query<&mut Inventory, With<Agent>>,
    terminal_entity: Entity,
    agent_entity: Entity,
    mission_data: &mut ResMut<MissionData>,
) {
    if let Ok((_, _, mut terminal)) = terminal_query.get_mut(terminal_entity) {
        if let Ok(mut inventory) = agent_inventory_query.get_mut(agent_entity) {
            terminal.accessed = true;
            mission_data.terminals_accessed += 1;
            
            match terminal.terminal_type {
                TerminalType::Objective => {
                    inventory.add_currency(500);
                    mission_data.objectives_completed += 1;
                    info!("Objective completed! ({}/{})", mission_data.objectives_completed, mission_data.total_objectives);
                }
                TerminalType::Equipment => {
                    inventory.add_weapon(WeaponType::Rifle);
                    inventory.add_tool(ToolType::Hacker);
                    inventory.add_currency(200);
                    info!("Equipment acquired!");
                }
                TerminalType::Intel => {
                    inventory.add_intel("Corporate research logs indicate unusual neurovector activity...".to_string());
                    inventory.add_currency(50);
                    info!("Intel acquired!");
                }
            }
        }
    }
}