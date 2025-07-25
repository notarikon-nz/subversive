// src/systems/formations.rs
use bevy::prelude::*;
use crate::core::*;

pub fn formation_input_system(
    mut commands: Commands,
    input: Res<ButtonInput<KeyCode>>,
    mut formation_state: ResMut<FormationState>,
    selection: Res<SelectionState>,
    mut formation_query: Query<&mut Formation>,
    member_query: Query<Entity, With<FormationMember>>,
) {
    if selection.selected.len() < 2 { return; }

    let formation_type = if input.just_pressed(KeyCode::Digit1) && input.pressed(KeyCode::ShiftLeft) {
        Some(FormationType::Line)
    } else if input.just_pressed(KeyCode::Digit2) && input.pressed(KeyCode::ShiftLeft) {
        Some(FormationType::Wedge)
    } else if input.just_pressed(KeyCode::Digit3) && input.pressed(KeyCode::ShiftLeft) {
        Some(FormationType::Column)
    } else if input.just_pressed(KeyCode::Digit4) && input.pressed(KeyCode::ShiftLeft) {
        Some(FormationType::Box)
    } else {
        None
    };

    if let Some(ftype) = formation_type {
        if let Some(formation_entity) = formation_state.active_formation {
            if let Ok(mut formation) = formation_query.get_mut(formation_entity) {
                formation.formation_type = ftype;
                return;
            }
        }

        let leader = selection.selected[0];
        let mut formation = Formation::new(ftype, leader);
        formation.members = selection.selected.clone();
        
        let formation_entity = commands.spawn(formation).id();
        formation_state.active_formation = Some(formation_entity);
        
        for (i, &entity) in selection.selected.iter().enumerate() {
            commands.entity(entity).insert(FormationMember {
                formation_entity,
                position_index: i,
            });
        }
    }

    if input.just_pressed(KeyCode::KeyG) {
        for entity in member_query.iter() {
            commands.entity(entity).remove::<FormationMember>();
        }
        if let Some(formation_entity) = formation_state.active_formation {
            // commands.entity(formation_entity).despawn();
            commands.entity(formation_entity).insert(MarkedForDespawn);
        }
        formation_state.active_formation = None;
    }
}

pub fn formation_movement_system(
    mut formation_query: Query<&mut Formation>,
    mut action_events: EventWriter<ActionEvent>,
    mut last_leader_positions: Local<std::collections::HashMap<Entity, Vec2>>,
    leader_query: Query<&Transform, With<Agent>>,
) {
    for mut formation in formation_query.iter_mut() {
        if let Ok(leader_transform) = leader_query.get(formation.leader) {
            let current_pos = leader_transform.translation.truncate();
            let last_pos = last_leader_positions.get(&formation.leader).copied();
            
            if last_pos.is_none() || last_pos.unwrap().distance(current_pos) > 5.0 {
                formation.calculate_positions(current_pos);
                last_leader_positions.insert(formation.leader, current_pos);
                
                for (i, &member) in formation.members.iter().enumerate().skip(1) {
                    if let Some(&formation_pos) = formation.positions.get(i) {
                        action_events.write(ActionEvent {
                            entity: member,
                            action: Action::MoveTo(formation_pos),
                        });
                    }
                }
            }
        }
    }
}

pub fn formation_visual_system(
    gizmos: Gizmos,
    formation_query: Query<&Formation>,
    formation_state: Res<FormationState>,
) {
    if let Some(formation_entity) = formation_state.active_formation {
        if let Ok(formation) = formation_query.get(formation_entity) {
            for (i, &pos) in formation.positions.iter().enumerate() {
                let color = if i == 0 { 
                    Color::srgb(0.8, 0.8, 0.2) 
                } else { 
                    Color::srgba(0.2, 0.8, 0.2, 0.6) 
                };
            }
        }
    }
}