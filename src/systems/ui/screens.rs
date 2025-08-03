// src/systems/ui/screens.rs - All the screen UIs updated for Bevy 0.16
use bevy::prelude::*;
use crate::systems::ui::*;

// Add a marker for inventory UI refresh
#[derive(Resource, Default)]
pub struct InventoryUIState {
    pub needs_refresh: bool,
    pub last_selected_agent: Option<Entity>,
}

// Re-export components for compatibility
#[derive(Component)]
pub struct InventoryUI;
