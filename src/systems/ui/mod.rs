// src/systems/ui/mod.rs - Updated with UIBuilder
use bevy::prelude::*;
use crate::core::*;

pub mod world;
pub mod screens;
pub mod hub;
pub mod builder;
pub mod main_menu;
pub mod settings;
pub mod credits;
pub mod fps;
pub mod pause;
pub mod inventory;
pub mod post_mission;

pub use screens::*;
pub use hub::*;
pub use main_menu::*;
pub use fps::*;
pub use pause::*;
pub use inventory::*;
pub use post_mission::*;

pub fn cleanup_mission_ui(
    mut commands: Commands,
    mut inventory_state: ResMut<InventoryState>,
    mut game_mode: ResMut<GameMode>,
    inventory_ui_query: Query<Entity, (With<InventoryUI>, Without<MarkedForDespawn>)>,
    pause_ui_query: Query<Entity, (With<PauseScreen>, Without<MarkedForDespawn>)>,
) {
    inventory_state.ui_open = false;
    inventory_state.selected_agent = None;
    game_mode.targeting = None;
    game_mode.paused = false;
    
    for entity in inventory_ui_query.iter() {
        commands.entity(entity).insert(MarkedForDespawn);
    }
    
    for entity in pause_ui_query.iter() {
        commands.entity(entity).insert(MarkedForDespawn);
    }
}

pub fn cleanup_global_map_ui(
    mut commands: Commands,
    mut inventory_state: ResMut<InventoryState>,
    inventory_ui_query: Query<Entity, With<InventoryUI>>,
    post_mission_ui_query: Query<Entity, (With<PostMissionScreen>, Without<MarkedForDespawn>)>,
) {
    inventory_state.ui_open = false;
    inventory_state.selected_agent = None;
    
    for entity in inventory_ui_query.iter() {
        commands.entity(entity).insert(MarkedForDespawn);
    }
    
    for entity in post_mission_ui_query.iter() {
        commands.entity(entity).insert(MarkedForDespawn);
    }
}