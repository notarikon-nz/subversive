use bevy::prelude::*;
use leafwing_input_manager::prelude::*;
use crate::core::*;

pub fn system(
    mut gizmos: Gizmos,
    input: Query<&ActionState<PlayerAction>>,
    mut action_events: EventReader<ActionEvent>,
    selection: Res<SelectionState>,
    agent_query: Query<&Transform, With<Agent>>,
    mut terminal_query: Query<(Entity, &Transform, &mut Terminal)>,
    game_mode: Res<GameMode>,
) {
    if game_mode.paused { return; }

    let Ok(action_state) = input.get_single() else { return; };
    let mut interaction_target = None;

    // Draw interaction prompts and detect interaction input
    for &selected_agent in &selection.selected {
        if let Ok(agent_transform) = agent_query.get(selected_agent) {
            let agent_pos = agent_transform.translation.truncate();

            for (terminal_entity, terminal_transform, terminal) in terminal_query.iter() {
                if terminal.accessed { continue; }

                let terminal_pos = terminal_transform.translation.truncate();
                let distance = agent_pos.distance(terminal_pos);

                if distance <= terminal.range {
                    // Draw interaction range
                    let color = match terminal.terminal_type {
                        TerminalType::Objective => Color::srgba(0.9, 0.2, 0.2, 0.3),
                        TerminalType::Equipment => Color::srgba(0.2, 0.5, 0.9, 0.3),
                        TerminalType::Intel => Color::srgba(0.2, 0.8, 0.3, 0.3),
                    };
                    
                    gizmos.circle_2d(terminal_pos, terminal.range, color);
                    
                    // Handle interaction input
                    if action_state.just_pressed(&PlayerAction::Interact) {
                        interaction_target = Some(terminal_entity);
                    }
                }
            }
        }
    }

    // Execute interaction if one was triggered
    if let Some(terminal_entity) = interaction_target {
        execute_terminal_interaction(&mut terminal_query, terminal_entity);
    }

    // Process interaction actions from events
    for event in action_events.read() {
        if let Action::InteractWith(terminal_entity) = event.action {
            execute_terminal_interaction(&mut terminal_query, terminal_entity);
        }
    }
}

fn execute_terminal_interaction(
    terminal_query: &mut Query<(Entity, &Transform, &mut Terminal)>,
    terminal_entity: Entity,
) {
    if let Ok((_, _, mut terminal)) = terminal_query.get_mut(terminal_entity) {
        terminal.accessed = true;
        match terminal.terminal_type {
            TerminalType::Objective => {
                info!("Objective terminal accessed - Mission progress!");
            }
            TerminalType::Equipment => {
                info!("Equipment terminal accessed - New gear acquired!");
            }
            TerminalType::Intel => {
                info!("Intel terminal accessed - New information acquired!");
            }
        }
    }
}