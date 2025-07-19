// src/systems/ui/mod.rs - Just split the big file, nothing fancy
use bevy::prelude::*;
use crate::core::*;
use crate::systems::*;

use crate::core::{PostMissionProcessed, PostMissionResults, GlobalData, UIState, GameState, experience_for_level};


pub mod world;      // Gizmos and world-space UI (selection, vision cones)
pub mod screens;    // All fullscreen UIs (inventory, global map, post-mission, pause)

pub use world::*;
pub use screens::*;

// Add this system to handle state transitions
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
    
    // Despawn any open UI windows
    for entity in inventory_ui_query.iter() {
        commands.entity(entity).despawn_recursive();
    }
    
    for entity in pause_ui_query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

// Also add cleanup when entering global map
pub fn cleanup_global_map_ui(
    mut commands: Commands,
    mut inventory_state: ResMut<InventoryState>,
    inventory_ui_query: Query<Entity, With<InventoryUI>>,
    post_mission_ui_query: Query<Entity, With<PostMissionScreen>>,
) {
    // Make sure inventory is closed in global map
    inventory_state.ui_open = false;
    inventory_state.selected_agent = None;
    
    // Clean up any lingering UI
    for entity in inventory_ui_query.iter() {
        commands.entity(entity).despawn_recursive();
    }
    
    for entity in post_mission_ui_query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}
