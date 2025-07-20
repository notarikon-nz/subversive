// src/systems/ui/mod.rs - Just split the big file, nothing fancy
use bevy::prelude::*;
use crate::core::*;

pub mod world;      // Gizmos and world-space UI (selection, vision cones)
pub mod screens;    // All fullscreen UIs (inventory, global map, post-mission, pause)
pub mod hub;

pub use world::*;
pub use screens::*;
pub use hub::*;

// handle state transitions
// otherwise our UI tends to stay open on the hub
pub fn cleanup_mission_ui(
    mut commands: Commands,
    mut inventory_state: ResMut<InventoryState>,
    mut game_mode: ResMut<GameMode>,
    inventory_ui_query: Query<Entity, With<InventoryUI>>,
    pause_ui_query: Query<Entity, With<PauseScreen>>,
) {
    // Close inventory
    inventory_state.ui_open = false;
    inventory_state.selected_agent = None;
    
    // Clear targeting modes
    game_mode.targeting = None;
    game_mode.paused = false;
    
    // Safe despawn UI windows
    for entity in inventory_ui_query.iter() {
        commands.safe_despawn_recursive(entity);
    }
    
    for entity in pause_ui_query.iter() {
        commands.safe_despawn_recursive(entity);
    }
}

pub fn cleanup_global_map_ui(
    mut commands: Commands,
    mut inventory_state: ResMut<InventoryState>,
    inventory_ui_query: Query<Entity, With<InventoryUI>>,
    post_mission_ui_query: Query<Entity, With<PostMissionScreen>>,
) {
    inventory_state.ui_open = false;
    inventory_state.selected_agent = None;
    
    // Safe despawn lingering UI
    for entity in inventory_ui_query.iter() {
        commands.safe_despawn_recursive(entity);
    }
    
    for entity in post_mission_ui_query.iter() {
        commands.safe_despawn_recursive(entity);
    }
}