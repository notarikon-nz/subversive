// src/core/mod.rs - Streamlined module organization
use bevy::prelude::*;

// === SUB-MODULES ===
pub mod events;
pub mod audio;
pub mod sprites;
pub mod goap; 
pub mod research;
pub mod attachments;
pub mod agent_upgrades;
pub mod fonts;
pub mod collision_groups;

// NEW: Split out focused modules
pub mod input;
pub mod game_state;
pub mod components;
pub mod resources;
pub mod entities;
pub mod missions;
pub mod weapons;
pub mod config;
pub mod scene_cache; 
pub mod factions;
pub mod lore;
pub mod hackable;
pub mod cities;
pub mod despawn;
pub mod spawn_damage_text;

// Re-exports for convenience
pub use events::*;
pub use audio::*;
pub use sprites::*;
pub use goap::*;
pub use research::*;
pub use attachments::*;
pub use agent_upgrades::*;
pub use fonts::*;
pub use collision_groups::*;

pub use input::*;
pub use game_state::*;
pub use components::*;
pub use resources::*;
pub use entities::*;
pub use missions::*;
pub use weapons::*;
pub use config::*;
pub use scene_cache::*;
pub use lore::*;
pub use hackable::*;
pub use cities::*;
pub use spawn_damage_text::*;


// === MISSING TYPES ===
// AgentLoadout definition
#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct AgentLoadout {
    pub weapon_configs: Vec<WeaponConfig>,
    pub equipped_weapon_idx: usize,
    pub tools: Vec<ToolType>,
    pub cybernetics: Vec<CyberneticType>,
    
}

impl Default for AgentLoadout {
    fn default() -> Self {
        Self {
            weapon_configs: vec![WeaponConfig::new(WeaponType::Rifle)],
            equipped_weapon_idx: 0,
            tools: vec![ToolType::Scanner],
            cybernetics: vec![],
        }
    }
}

// === UTILITY FUNCTIONS ===
pub fn get_world_mouse_position(
    windows: &Query<&Window>,
    cameras: &Query<(&Camera, &GlobalTransform)>,
) -> Option<Vec2> {
    let window = windows.single().ok()?;
    let (camera, camera_transform) = cameras.single().ok()?;
    let cursor_pos = window.cursor_position()?;
    
    // Flip Y coordinate to match UI coordinate system
    let flipped_cursor = Vec2::new(cursor_pos.x, cursor_pos.y);
    
    camera.viewport_to_world_2d(camera_transform, flipped_cursor).ok()
        
    // camera.viewport_to_world_2d(camera_transform, cursor_pos).ok()
}

pub fn get_global_map_mouse_position(
    windows: &Query<&Window>,
    cameras: &Query<(&Camera, &GlobalTransform)>,
) -> Option<Vec2> {
    let window = windows.single().ok()?;
    let (camera, camera_transform) = cameras.single().ok()?;
    let cursor_pos = window.cursor_position()?;
    
    // Flip Y coordinate to match UI coordinate system
    let flipped_cursor = Vec2::new(cursor_pos.x, window.height() - cursor_pos.y);
    
    camera.viewport_to_world_2d(camera_transform, flipped_cursor).ok()
        
    // camera.viewport_to_world_2d(camera_transform, cursor_pos).ok()
}

pub fn experience_for_level(level: u8) -> u32 {
    (level as u32) * 100
}
