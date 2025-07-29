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

    if keyboard.just_pressed(KeyCode::KeyI) {
        inventory_state.ui_open = !inventory_state.ui_open;
        if inventory_state.ui_open {
            inventory_state.selected_agent = selection.selected.first().copied();
        }
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

    // Continuous attack system handles all combat/movement
    handle_continuous_attack(
        &mut continuous_attack,
        &mouse,
        &windows,
        &cameras,
        &mut action_events,
        &selection,
        &game_mode,
        &target_query,
        &agent_query,
        &time,
    );
}

/*
    // FIXED: Direct mouse detection for movement (only when not in combat mode or not attacking)
    if mouse.just_pressed(MouseButton::Right) {
        if let Some(world_pos) = get_world_mouse_position(&windows, &cameras) {
            // Send movement commands for all selected agents
            for &entity in &selection.selected {
                action_events.write(ActionEvent {
                    entity,
                    action: Action::MoveTo(world_pos),
                });
            }
        }
    }
*/

fn handle_continuous_attack(
    continuous_attack: &mut ContinuousAttackState,
    mouse: &ButtonInput<MouseButton>,
    windows: &Query<&Window>,
    cameras: &Query<(&Camera, &GlobalTransform)>,
    action_events: &mut EventWriter<ActionEvent>,
    selection: &SelectionState,
    game_mode: &GameMode,
    target_query: &Query<(Entity, &Transform, &Health), Or<(With<Enemy>, With<Vehicle>)>>,
    agent_query: &Query<(&Transform, &Inventory), With<Agent>>,
    time: &Time,
) {
    let current_time = time.elapsed_secs();
    
    // Combat mode is default - check if we're NOT in other modes
    let combat_enabled = match &game_mode.targeting {
        Some(TargetingMode::Neurovector { .. }) => false,
        Some(TargetingMode::Scanning { .. }) => false, // Add this variant to your TargetingMode enum
        _ => true, // Combat is default
    };
    
    if !combat_enabled {
        continuous_attack.attacking = false;
        continuous_attack.current_target = None;
        return;
    }
    
    // Right mouse button for continuous attack
    if mouse.just_pressed(MouseButton::Right) {
        if let Some(target) = find_target_under_mouse(windows, cameras, target_query, agent_query, selection) {
            continuous_attack.attacking = true;
            continuous_attack.current_target = Some(target);
            continuous_attack.record_attack(current_time);
            
            // Initial attack
            for &agent in &selection.selected {
                action_events.write(ActionEvent {
                    entity: agent,
                    action: Action::Attack(target),
                });
            }
        } else if let Some(world_pos) = get_world_mouse_position(windows, cameras) {
            // No target - move command
            for &entity in &selection.selected {
                action_events.write(ActionEvent {
                    entity,
                    action: Action::MoveTo(world_pos),
                });
            }
        }
    }
    
    // Continue attacking while held
    if mouse.pressed(MouseButton::Right) && continuous_attack.attacking {
        if let Some(target) = continuous_attack.current_target {
            if continuous_attack.can_attack(current_time) {
                if let Ok((_, _, health)) = target_query.get(target) {
                    if health.0 > 0.0 {
                        continuous_attack.record_attack(current_time);
                        for &agent in &selection.selected {
                            action_events.write(ActionEvent {
                                entity: agent,
                                action: Action::Attack(target),
                            });
                        }
                    } else {
                        continuous_attack.attacking = false;
                        continuous_attack.current_target = None;
                    }
                }
            }
        }
    }
    
    // Stop on release
    if mouse.just_released(MouseButton::Right) {
        continuous_attack.attacking = false;
        continuous_attack.current_target = None;
    }
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