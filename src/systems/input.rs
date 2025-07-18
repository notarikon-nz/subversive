use bevy::prelude::*;
use leafwing_input_manager::prelude::*;
use crate::core::*;

pub fn handle_input(
    input: Query<&ActionState<PlayerAction>>,
    mut game_mode: ResMut<GameMode>,
    mut action_events: EventWriter<ActionEvent>,
    selection: Res<SelectionState>,
    windows: Query<&Window>,
    cameras: Query<(&Camera, &GlobalTransform)>,
) {
    let Ok(action_state) = input.get_single() else { return; };

    if action_state.just_pressed(&PlayerAction::Pause) {
        game_mode.paused = !game_mode.paused;
        info!("Game {}", if game_mode.paused { "paused" } else { "resumed" });
    }

    if game_mode.paused { return; }

    if action_state.just_pressed(&PlayerAction::Move) {
        if let Some(world_pos) = get_world_mouse_position(&windows, &cameras) {
            for &entity in &selection.selected {
                action_events.send(ActionEvent {
                    entity,
                    action: Action::MoveTo(world_pos),
                });
            }
        }
    }

    if action_state.just_pressed(&PlayerAction::Neurovector) {
        if let Some(&agent) = selection.selected.first() {
            toggle_neurovector_targeting(&mut game_mode, agent);
        }
    }

    if action_state.just_pressed(&PlayerAction::Combat) {
        if let Some(&agent) = selection.selected.first() {
            toggle_combat_targeting(&mut game_mode, agent);
        }
    }
}

fn toggle_neurovector_targeting(game_mode: &mut GameMode, agent: Entity) {
    match &game_mode.targeting {
        Some(TargetingMode::Neurovector { .. }) => {
            game_mode.targeting = None;
            info!("Neurovector targeting cancelled");
        }
        _ => {
            game_mode.targeting = Some(TargetingMode::Neurovector { agent });
            info!("Neurovector targeting activated");
        }
    }
}

fn toggle_combat_targeting(game_mode: &mut GameMode, agent: Entity) {
    match &game_mode.targeting {
        Some(TargetingMode::Combat { .. }) => {
            game_mode.targeting = None;
            info!("Combat targeting cancelled");
        }
        _ => {
            game_mode.targeting = Some(TargetingMode::Combat { agent });
            info!("Combat targeting activated");
        }
    }
}