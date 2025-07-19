// src/core/mod.rs - Add GOAP module
use bevy::prelude::*;
use leafwing_input_manager::prelude::*;
use serde::{Deserialize, Serialize};

pub mod events;
pub use events::*;

pub mod audio;
pub use audio::*;

pub mod sprites;
pub use sprites::*;

pub mod goap; // Add this line
pub use goap::*;

// Weapon Attachment System
pub mod attachments;
pub use attachments::*;

// Re-export hub types for convenience
pub use crate::systems::ui::hub::{HubState, HubTab};

// === STATES ===
#[derive(States, Debug, Clone, PartialEq, Eq, Hash)]
pub enum GameState {
    GlobalMap,
    Mission,
    PostMission,
}

impl Default for GameState {
    fn default() -> Self { GameState::GlobalMap }
}

// === SQUAD GOALS ===
#[derive(Component)]
pub struct SelectionBox {
    pub start: Vec2,
    pub end: Vec2,
}

#[derive(Resource, Default)]
pub struct SelectionDrag {
    pub dragging: bool,
    pub start_pos: Vec2,
    pub current_pos: Vec2,
}

// === RESOURCES ===
#[derive(Resource)]
pub struct GameMode {
    pub paused: bool,
    pub targeting: Option<TargetingMode>,
}

impl Default for GameMode {
    fn default() -> Self {
        Self { paused: false, targeting: None }
    }
}

#[derive(Resource, Default)]
pub struct UIState {
    pub global_map_open: bool,
    pub inventory_open: bool,
    pub pause_open: bool,
    pub post_mission_open: bool,
    pub fps_visible: bool,
}

#[derive(Resource, Default)]
pub struct PostMissionProcessed(pub bool);

#[derive(Component)]
pub struct FpsText;

#[derive(Debug)]
pub enum TargetingMode {
    Neurovector { agent: Entity },
    Combat { agent: Entity },
}

#[derive(Resource, Default)]
pub struct SelectionState {
    pub selected: Vec<Entity>,
}

#[derive(Resource)]
pub struct MissionData {
    pub timer: f32,
    pub alert_level: AlertLevel,
    pub objectives_completed: u32,
    pub total_objectives: u32,
    pub enemies_killed: u32,
    pub terminals_accessed: u32,
    pub time_limit: f32,
}

impl Default for MissionData {
    fn default() -> Self {
        Self { 
            timer: 0.0, 
            alert_level: AlertLevel::Green,
            objectives_completed: 0,
            total_objectives: 1,
            enemies_killed: 0,
            terminals_accessed: 0,
            time_limit: 300.0, // 5 minutes
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AlertLevel {
    Green,
    Yellow,
    Orange,
    Red,
}

// === INPUT ===
#[derive(Actionlike, PartialEq, Eq, Hash, Clone, Copy, Debug, Reflect)]
pub enum PlayerAction {
    Pause,
    Select,
    Move,
    Neurovector,
    Combat,
    Interact,
    Inventory,
}

// === CORE COMPONENTS ===
#[derive(Component)]
pub struct Agent {
    pub experience: u32,
    pub level: u8,
}

impl Default for Agent {
    fn default() -> Self {
        Self { experience: 0, level: 1 }
    }
}

#[derive(Component)]
pub struct Civilian;

#[derive(Component)]
pub struct Enemy;

#[derive(Component)]
pub struct Health(pub f32);

#[derive(Component)]
pub struct Dead;

#[derive(Component)]
pub struct MovementSpeed(pub f32);

#[derive(Component)]
pub struct Controllable;

#[derive(Component)]
pub struct Selected;

#[derive(Component)]
pub struct Selectable {
    pub radius: f32,
}

#[derive(Component)]
pub struct NeurovectorTarget;

#[derive(Component)]
pub struct NeurovectorControlled {
    pub controller: Entity,
}

#[derive(Component)]
pub struct MoveTarget {
    pub position: Vec2,
}

#[derive(Component)]
pub struct Objective;

#[derive(Resource)]
pub struct PostMissionResults {
    pub success: bool,
    pub time_taken: f32,
    pub enemies_killed: u32,
    pub terminals_accessed: u32,
    pub credits_earned: u32,
    pub alert_level: AlertLevel,
}

impl Default for PostMissionResults {
    fn default() -> Self {
        Self {
            success: false,
            time_taken: 0.0,
            enemies_killed: 0,
            terminals_accessed: 0,
            credits_earned: 0,
            alert_level: AlertLevel::Green,
        }
    }
}

#[derive(Resource, Clone)]
pub struct GlobalData {
    pub credits: u32,
    pub selected_region: usize,
    pub regions: Vec<Region>,
    pub agent_levels: [u8; 3],
    pub agent_experience: [u32; 3],
    pub current_day: u32,
    pub agent_recovery: [u32; 3],
}

impl Default for GlobalData {
    fn default() -> Self {
        Self {
            credits: 1000,
            selected_region: 0,
            regions: vec![
                Region { 
                    name: "Neo-Tokyo Central".to_string(), 
                    threat_level: 1,
                    alert_level: AlertLevel::Green,
                    alert_decay_timer: 0,
                },
                Region { 
                    name: "Corporate District".to_string(), 
                    threat_level: 2,
                    alert_level: AlertLevel::Green,
                    alert_decay_timer: 0,
                },
                Region { 
                    name: "Underground Labs".to_string(), 
                    threat_level: 3,
                    alert_level: AlertLevel::Green,
                    alert_decay_timer: 0,
                },
            ],
            agent_levels: [1, 1, 1],
            agent_experience: [0, 0, 0],
            current_day: 1,
            agent_recovery: [0, 0, 0],
        }
    }
}

pub fn experience_for_level(level: u8) -> u32 {
    (level as u32) * 100
}

#[derive(Clone)]
pub struct Region {
    pub name: String,
    pub threat_level: u8,
    pub alert_level: AlertLevel,
    pub alert_decay_timer: u32,
}

impl Region {
    pub fn raise_alert(&mut self, current_day: u32) {
        self.alert_level = match self.alert_level {
            AlertLevel::Green => AlertLevel::Yellow,
            AlertLevel::Yellow => AlertLevel::Orange,
            AlertLevel::Orange => AlertLevel::Red,
            AlertLevel::Red => AlertLevel::Red,
        };
        
        self.alert_decay_timer = current_day + match self.alert_level {
            AlertLevel::Yellow => 3,
            AlertLevel::Orange => 7,
            AlertLevel::Red => 14,
            AlertLevel::Green => 0,
        };
    }
    
    pub fn update_alert(&mut self, current_day: u32) {
        if current_day >= self.alert_decay_timer && self.alert_level != AlertLevel::Green {
            self.alert_level = match self.alert_level {
                AlertLevel::Red => AlertLevel::Orange,
                AlertLevel::Orange => AlertLevel::Yellow,
                AlertLevel::Yellow => AlertLevel::Green,
                AlertLevel::Green => AlertLevel::Green,
            };
            
            if self.alert_level != AlertLevel::Green {
                self.alert_decay_timer = current_day + match self.alert_level {
                    AlertLevel::Yellow => 3,
                    AlertLevel::Orange => 7,
                    _ => 0,
                };
            }
        }
    }
    
    pub fn mission_difficulty_modifier(&self) -> f32 {
        match self.alert_level {
            AlertLevel::Green => 1.0,
            AlertLevel::Yellow => 1.3,
            AlertLevel::Orange => 1.6,
            AlertLevel::Red => 2.0,
        }
    }
}

#[derive(Component)]
pub struct GlobalMapUI;

#[derive(Resource)]
pub struct ShouldRestart;

// === COMPLEX COMPONENTS ===
#[derive(Component)]
pub struct Vision {
    pub range: f32,
    pub angle: f32,
    pub direction: Vec2,
}

impl Vision {
    pub fn new(range: f32, angle_degrees: f32) -> Self {
        Self {
            range,
            angle: angle_degrees.to_radians(),
            direction: Vec2::X,
        }
    }
}

#[derive(Component)]
pub struct NeurovectorCapability {
    pub range: f32,
    pub max_targets: u8,
    pub cooldown: f32,
    pub current_cooldown: f32,
    pub controlled: Vec<Entity>,
}

impl Default for NeurovectorCapability {
    fn default() -> Self {
        Self {
            range: 200.0,
            max_targets: 3,
            cooldown: 5.0,
            current_cooldown: 0.0,
            controlled: Vec::new(),
        }
    }
}

#[derive(Component)]
pub struct Patrol {
    pub points: Vec<Vec2>,
    pub current_index: usize,
}

impl Patrol {
    pub fn new(points: Vec<Vec2>) -> Self {
        Self { points, current_index: 0 }
    }
    
    pub fn current_target(&self) -> Option<Vec2> {
        self.points.get(self.current_index).copied()
    }
    
    pub fn advance(&mut self) {
        if !self.points.is_empty() {
            self.current_index = (self.current_index + 1) % self.points.len();
        }
    }
}

#[derive(Component)]
pub struct Terminal {
    pub terminal_type: TerminalType,
    pub range: f32,
    pub accessed: bool,
}

#[derive(Debug, Clone)]
pub enum TerminalType {
    Objective,
    Equipment,
    Intel,
}

#[derive(Component)]
pub struct InventoryUI;

#[derive(Resource)]
pub struct InventoryState {
    pub ui_open: bool,
    pub selected_agent: Option<Entity>,
}

impl Default for InventoryState {
    fn default() -> Self {
        Self {
            ui_open: false,
            selected_agent: None,
        }
    }
}

// Updated the Inventory struct to support weapon configs:
#[derive(Component, Default)]
pub struct Inventory {
    pub weapons: Vec<WeaponConfig>,  // Changed from WeaponType to WeaponConfig
    pub tools: Vec<ToolType>,
    pub currency: u32,
    pub equipped_weapon: Option<WeaponConfig>,  // Changed from WeaponType
    pub equipped_tools: Vec<ToolType>,
    pub cybernetics: Vec<CyberneticType>,
    pub intel_documents: Vec<String>,
}

#[derive(Component)]
pub struct InventoryVersion(pub u32);

impl Inventory {
    pub fn add_weapon(&mut self, weapon: WeaponType) {
        let config = WeaponConfig::new(weapon);
        if self.equipped_weapon.is_none() {
            self.equipped_weapon = Some(config.clone());
        }
        self.weapons.push(config);
    }
    
    pub fn add_weapon_config(&mut self, config: WeaponConfig) {
        if self.equipped_weapon.is_none() {
            self.equipped_weapon = Some(config.clone());
        }
        self.weapons.push(config);
    }
    
    // Keep existing methods unchanged
    pub fn add_currency(&mut self, amount: u32) {
        self.currency += amount;
    }
    
    pub fn add_tool(&mut self, tool: ToolType) {
        if self.equipped_tools.len() < 2 {
            self.equipped_tools.push(tool.clone());
        }
        self.tools.push(tool);
    }
    
    pub fn add_cybernetic(&mut self, cybernetic: CyberneticType) {
        self.cybernetics.push(cybernetic);
    }
    
    pub fn add_intel(&mut self, document: String) {
        self.intel_documents.push(document);
    }
}

#[derive(Debug, Clone)]
pub enum CyberneticType {
    Neurovector,
    CombatEnhancer,
    StealthModule,
    TechInterface,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WeaponType {
    Pistol,
    Rifle,
    Minigun,
    Flamethrower,
}

#[derive(Debug, Clone)]
pub enum ToolType {
    Hacker,
    Scanner,
    Lockpick,
    MedKit,
}

// === UTILITY FUNCTIONS ===
pub fn get_world_mouse_position(
    windows: &Query<&Window>,
    cameras: &Query<(&Camera, &GlobalTransform)>,
) -> Option<Vec2> {
    let window = windows.get_single().ok()?;
    let (camera, camera_transform) = cameras.get_single().ok()?;
    
    window.cursor_position()
        .and_then(|cursor| camera.viewport_to_world_2d(camera_transform, cursor))
}

// === GOAP FEATURES ===
#[derive(Event)]
pub struct AlertEvent {
    pub alerter: Entity,
    pub position: Vec2,
    pub alert_type: AlertType,
}

#[derive(Debug, Clone)]
pub enum AlertType {
    CallForHelp,
    GunshotHeard,
    EnemySpotted,
}

#[derive(Resource, Default)]
pub struct ManufactureState {
    pub selected_agent_idx: usize,
    pub selected_weapon_idx: usize,
    pub selected_slot: Option<AttachmentSlot>,
    pub selected_attachments: std::collections::HashMap<AttachmentSlot, String>, // NEW: Per-slot selection
}