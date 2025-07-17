use bevy::prelude::*;
use leafwing_input_manager::prelude::*;
use crate::components::*;

#[derive(Resource)]
pub struct GlobalGameData {
    pub current_region: RegionId,
    pub regions: Vec<RegionState>,
    pub currency: u32,
    pub research_points: u32,
    pub calendar_day: u32,
    pub available_agents: Vec<Entity>,
}

impl Default for GlobalGameData {
    fn default() -> Self {
        Self {
            current_region: RegionId(0),
            regions: vec![RegionState::default()],
            currency: 10000,
            research_points: 0,
            calendar_day: 1,
            available_agents: vec![],
        }
    }
}

#[derive(Resource)]
pub struct MissionData {
    pub mission_timer: f32,
    pub time_limit: Option<f32>,
    pub objectives: Vec<Entity>,
    pub current_alert_level: AlertLevel,
    pub alert_decay_timer: f32,
    pub loot_collected: Vec<Equipment>,
    pub mission_active: bool,
    pub time_scale: f32, // 0.0 = paused, 1.0 = normal speed
}

impl Default for MissionData {
    fn default() -> Self {
        Self {
            mission_timer: 0.0,
            time_limit: Some(300.0), // 5 minutes
            objectives: vec![],
            current_alert_level: AlertLevel::Green,
            alert_decay_timer: 0.0,
            loot_collected: vec![],
            mission_active: true,
            time_scale: 1.0,
        }
    }
}

#[derive(Resource)]
pub struct SelectionState {
    pub selected_agents: Vec<Entity>,
    pub selection_box_start: Option<Vec2>,
    pub selection_box_end: Option<Vec2>,
    pub queued_orders: Vec<QueuedOrder>,
}

impl Default for SelectionState {
    fn default() -> Self {
        Self {
            selected_agents: vec![],
            selection_box_start: None,
            selection_box_end: None,
            queued_orders: vec![],
        }
    }
}

#[derive(Debug, Clone)]
pub struct QueuedOrder {
    pub agent: Entity,
    pub order: AgentOrder,
    pub execute_at: f32, // mission time
}

#[derive(Debug, Clone)]
pub enum AgentOrder {
    MoveTo(Vec2),
    Attack(Entity),
    Interact(Entity),
    UseNeurovector(Entity),
    Hold,
    Follow(Entity),
}

#[derive(Debug, Clone, Copy)]
pub struct RegionId(pub u8);

#[derive(Debug, Clone)]
pub struct RegionState {
    pub id: RegionId,
    pub name: String,
    pub security_state: AlertLevel,
    pub controlled_by: Option<Entity>,
    pub tax_income: u32,
    pub civilian_pool: u32,
    pub alert_decay_timer: f32,
}

impl Default for RegionState {
    fn default() -> Self {
        Self {
            id: RegionId(0),
            name: "Neo-Tokyo Central".to_string(),
            security_state: AlertLevel::Green,
            controlled_by: None,
            tax_income: 1000,
            civilian_pool: 50,
            alert_decay_timer: 0.0,
        }
    }
}

#[derive(Resource)]
pub struct NeurovectorTargeting {
    pub active_agent: Option<Entity>,
    pub targeting_mode: bool,
    pub valid_targets: Vec<Entity>,
    pub target_preview: Option<Entity>,
}

impl Default for NeurovectorTargeting {
    fn default() -> Self {
        Self {
            active_agent: None,
            targeting_mode: false,
            valid_targets: vec![],
            target_preview: None,
        }
    }
}

#[derive(Actionlike, PartialEq, Eq, Hash, Clone, Copy, Debug, Reflect)]
pub enum PlayerAction {
    Pause,
    Select,
    Move,
    Attack,
    UseNeurovector,
    CancelAction,
    CameraUp,
    CameraDown,
    CameraLeft,
    CameraRight,
}