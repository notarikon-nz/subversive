use bevy::prelude::*;
use crate::core::*;
use crate::systems::scanner::*;
use crate::systems::npc_barks::*;
use crate::systems::isometric_camera::{IsometricCamera};

pub fn get_isometric_world_mouse_position(
    windows: &Query<&Window>,
    cameras: &Query<(&Camera, &GlobalTransform, &IsometricCamera)>,
) -> Option<Vec2> {
    let window = windows.single().ok()?;
    let (camera, camera_transform, _) = cameras.single().ok()?;
    let cursor_pos = window.cursor_position()?;
    
    // Convert cursor position to world coordinates for isometric camera
    camera.viewport_to_world_2d(camera_transform, cursor_pos).ok()
}

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
        info!("Interaction Requested");
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

fn find_target_under_mouse(
    windows: &Query<&Window>,
    cameras: &Query<(&Camera, &GlobalTransform)>,
    target_query: &Query<(Entity, &Transform, &Health), Or<(With<Enemy>, With<Vehicle>)>>,
    agent_query: &Query<(&Transform, &Inventory), With<Agent>>,
    selection: &SelectionState,
) -> Option<Entity> {
    let mouse_pos = get_world_mouse_position(windows, cameras)?;

    // Get the primary selected agent to determine range
    let primary_agent = selection.selected.first()?;
    let (agent_transform, inventory) = agent_query.get(*primary_agent).ok()?;
    let agent_pos = agent_transform.translation.truncate();
    let range = get_weapon_range_simple(inventory);

    // Find the closest valid target under the mouse
    target_query.iter()
        .filter(|(_, _, health)| health.0 > 0.0) // Target must be alive
        .filter(|(_, transform, _)| {
            // Target must be in range of primary agent
            agent_pos.distance(transform.translation.truncate()) <= range
        })
        .filter(|(_, transform, _)| {
            // Target must be close to mouse cursor
            mouse_pos.distance(transform.translation.truncate()) < 35.0
        })
        .min_by(|(_, a_transform, _), (_, b_transform, _)| {
            let a_dist = mouse_pos.distance(a_transform.translation.truncate());
            let b_dist = mouse_pos.distance(b_transform.translation.truncate());
            a_dist.partial_cmp(&b_dist).unwrap_or(std::cmp::Ordering::Equal)
        })
        .map(|(entity, _, _)| entity)
}

pub fn get_weapon_range_simple(inventory: &Inventory) -> f32 {
    let base_range = 150.0;
    if let Some(weapon_config) = &inventory.equipped_weapon {
        let stats = weapon_config.stats();
        (base_range * (1.0_f32 + stats.range as f32 * 0.1_f32)).max(50.0_f32)
    } else {
        base_range
    }
}

fn toggle_neurovector_targeting(game_mode: &mut GameMode, agent: Entity) {
    match &game_mode.targeting {
        Some(TargetingMode::Neurovector { .. }) => {
            game_mode.targeting = None;
        }
        _ => {
            game_mode.targeting = Some(TargetingMode::Neurovector { agent });
        }
    }
}

// In toggle_combat_targeting:
fn toggle_combat_targeting(game_mode: &mut GameMode, agent: Entity) {
    match &game_mode.targeting {
        Some(TargetingMode::Combat { .. }) => {
            game_mode.targeting = None;
        }
        _ => {
            game_mode.targeting = Some(TargetingMode::Combat { agent });
        }
    }
}