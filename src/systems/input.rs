use bevy::prelude::*;
use crate::core::*;
use crate::systems::scanner::*;
use crate::systems::npc_barks::*;

// Simplified input handler - remove duplicate movement handling
pub fn handle_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    cameras: Query<(&Camera, &GlobalTransform)>,
    mut game_mode: ResMut<GameMode>,
    mut inventory_state: ResMut<InventoryState>,
    mut ui_state: ResMut<UIState>,
    mut action_events: EventWriter<ActionEvent>,
    mut continuous_attack: ResMut<ContinuousAttackState>,
    selection: Res<SelectionState>,
    time: Res<Time>,
    mut scanner_state: ResMut<ScannerState>,
    scannable_query: Query<(Entity, &Transform), (With<Scannable>, Without<ChatBubble>, Without<MarkedForDespawn>)>,
    target_query: Query<(Entity, &Transform, &Health), Or<(With<Enemy>, With<Vehicle>)>>,
    agent_query: Query<(&Transform, &Inventory), With<Agent>>,
) {
    // Toggle FPS counter
    if keyboard.just_pressed(KeyCode::F3) {
        ui_state.fps_visible = !ui_state.fps_visible;
    }

    // Basic controls
    if keyboard.just_pressed(KeyCode::Space) {
        game_mode.paused = !game_mode.paused;
    }

    // Mode toggles - always return to combat mode when exiting
    if keyboard.just_pressed(KeyCode::KeyN) {
        if let Some(&agent) = selection.selected.first() {
            match &game_mode.targeting {
                Some(TargetingMode::Neurovector { .. }) => game_mode.targeting = None,
                _ => game_mode.targeting = Some(TargetingMode::Neurovector { agent }),
            }
        }
    }

    if keyboard.just_pressed(KeyCode::KeyS) {
        match &game_mode.targeting {
            Some(TargetingMode::Scanning { .. }) => game_mode.targeting = None,
            _ => game_mode.targeting = Some(TargetingMode::Scanning),
        }
    }

    // Agent selection with 1, 2, 3
    handle_agent_selection(&keyboard, &selection, &mut action_events);

    // Other controls...
    if keyboard.just_pressed(KeyCode::KeyE) {
        if let Some(&agent) = selection.selected.first() {
            action_events.write(ActionEvent {
                entity: agent,
                action: Action::InteractWith(agent),
            });
        }
    }

    if keyboard.just_pressed(KeyCode::KeyR) {
        if let Some(&agent) = selection.selected.first() {
            action_events.write(ActionEvent {
                entity: agent,
                action: Action::Reload,
            });
        }
    }

    // Handle scanner if in scanner mode
    if matches!(game_mode.targeting, Some(TargetingMode::Scanning)) {
        handle_scanner_input(&keyboard, &mouse, &windows, &cameras, &mut scanner_state, &scannable_query);
    }

    if game_mode.paused { return; }
}

// Add agent selection handler
fn handle_agent_selection(
    keyboard: &ButtonInput<KeyCode>,
    selection: &SelectionState,
    action_events: &mut EventWriter<ActionEvent>,
) {
    use std::time::Instant;

    // Static to track double-tap timing
    static mut LAST_KEY_TIME: [Option<Instant>; 3] = [None; 3];
    const DOUBLE_TAP_TIME: std::time::Duration = std::time::Duration::from_millis(300);

    let keys = [KeyCode::Digit1, KeyCode::Digit2, KeyCode::Digit3];

    for (idx, &key) in keys.iter().enumerate() {
        if keyboard.just_pressed(key) {
            let now = Instant::now();
            let is_double_tap = unsafe {
                LAST_KEY_TIME[idx].map_or(false, |t| now.duration_since(t) < DOUBLE_TAP_TIME)
            };

            unsafe { LAST_KEY_TIME[idx] = Some(now); }

            if is_double_tap {
                // Double tap - center camera on agent but don't change selection
                action_events.write(ActionEvent {
                    entity: Entity::PLACEHOLDER, // You'll need to handle this in camera system
                    action: Action::CenterCameraOnAgent(idx),
                });
            } else {
                // Single tap - select agent
                action_events.write(ActionEvent {
                    entity: Entity::PLACEHOLDER,
                    action: Action::SelectAgent(idx),
                });
            }
        }
    }
}

