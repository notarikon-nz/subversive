use bevy::prelude::*;
use leafwing_input_manager::prelude::*;
use crate::core::*;

pub fn handle_input(
    input: Query<&ActionState<PlayerAction>>,
    mut game_mode: ResMut<GameMode>,
    mut inventory_state: ResMut<InventoryState>,
    mut ui_state: ResMut<UIState>,
    mut action_events: EventWriter<ActionEvent>,
    selection: Res<SelectionState>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    cameras: Query<(&Camera, &GlobalTransform)>,
    game_state: Res<State<GameState>>,
) {
    // Toggle FPS counter with F3 key (works in all states)
    if keyboard.just_pressed(KeyCode::F3) {
        ui_state.fps_visible = !ui_state.fps_visible;
        info!("FPS counter {}", if ui_state.fps_visible { "enabled" } else { "disabled" });
    }

    // Only handle mission input during missions
    if *game_state.get() != GameState::Mission {
        return;
    }

    // Direct keyboard input for mission (bypass ActionState)
    if keyboard.just_pressed(KeyCode::Space) {
        game_mode.paused = !game_mode.paused;
        info!("Game {}", if game_mode.paused { "paused" } else { "resumed" });
    }

    if keyboard.just_pressed(KeyCode::KeyI) {
        inventory_state.ui_open = !inventory_state.ui_open;
        if inventory_state.ui_open {
            inventory_state.selected_agent = selection.selected.first().copied();
        }
        info!("Inventory {}", if inventory_state.ui_open { "opened" } else { "closed" });
    }

    if keyboard.just_pressed(KeyCode::KeyN) {
        if let Some(&agent) = selection.selected.first() {
            toggle_neurovector_targeting(&mut game_mode, agent);
        }
    }

    if keyboard.just_pressed(KeyCode::KeyF) {
        if let Some(&agent) = selection.selected.first() {
            toggle_combat_targeting(&mut game_mode, agent);
        }
    }

    // Fixed E key for interaction
    if keyboard.just_pressed(KeyCode::KeyE) {
        if let Some(&agent) = selection.selected.first() {
            action_events.send(ActionEvent {
                entity: agent,
                action: Action::InteractWith(agent),
            });
        }
    }

    if game_mode.paused { return; }


    // Handle mouse movement (keep working mouse input)
    if mouse.just_pressed(MouseButton::Right) {
        if let Some(world_pos) = get_world_mouse_position(&windows, &cameras) {
            for &entity in &selection.selected {
                action_events.send(ActionEvent {
                    entity,
                    action: Action::MoveTo(world_pos),
                });
            }
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
            info!("Neurovector targeting activated for agent");
        }
    }
}

// In toggle_combat_targeting:
fn toggle_combat_targeting(game_mode: &mut GameMode, agent: Entity) {
    match &game_mode.targeting {
        Some(TargetingMode::Combat { .. }) => {
            game_mode.targeting = None;
            info!("Combat targeting cancelled");
        }
        _ => {
            game_mode.targeting = Some(TargetingMode::Combat { agent });
            info!("Combat targeting activated for agent");
        }
    }
}