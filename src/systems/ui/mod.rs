// src/systems/ui/mod.rs - Updated with UIBuilder
use bevy::prelude::*;
use crate::core::*;

pub mod world;
pub mod screens;
pub mod hub;
pub mod main_menu;
pub mod settings;
pub mod credits;
pub mod fps;
pub mod pause;
pub mod post_mission;
pub mod loading_system;

// 0.2.15
pub mod enhanced_inventory;
pub mod inventory_integration;
pub mod inventory_compatibility;


pub use screens::*;
pub use hub::*;
pub use main_menu::*;
pub use fps::*;
pub use pause::*;
pub use post_mission::*;
pub use loading_system::*;

pub fn cleanup_mission_ui(
    mut commands: Commands,
    mut inventory_state: ResMut<InventoryState>,
    mut game_mode: ResMut<GameMode>,
    inventory_ui_query: Query<Entity, (With<InventoryUI>, Without<MarkedForDespawn>)>,
) {
    inventory_state.ui_open = false;
    inventory_state.selected_agent = None;
    game_mode.targeting = None;
    game_mode.paused = false;

    for entity in inventory_ui_query.iter() {
        commands.entity(entity).insert(MarkedForDespawn);
    }
}

pub fn cleanup_global_map_ui(
    mut commands: Commands,
    mut inventory_state: ResMut<InventoryState>,
    inventory_ui_query: Query<Entity, (With<InventoryUI>, Without<MarkedForDespawn>)>,
) {
    inventory_state.ui_open = false;
    inventory_state.selected_agent = None;

    for entity in inventory_ui_query.iter() {
        commands.entity(entity).insert(MarkedForDespawn);
    }
}