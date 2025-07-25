// src/core/resources.rs - Game resources and state
use bevy::prelude::*;
use crate::core::{TargetingMode, AlertLevel, AttachmentSlot};

#[derive(Resource)]
pub struct MissionLaunchData {
    pub city_id: String,
    pub region_id: usize,
}

// === GAME MODE ===
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

// === UI STATE ===
#[derive(Resource, Default)]
pub struct UIState {
    pub global_map_open: bool,
    pub inventory_open: bool,
    pub pause_open: bool,
    pub post_mission_open: bool,
    pub fps_visible: bool,
}

// === MISSION DATA ===
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

// === POST MISSION ===
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

#[derive(Resource, Default)]
pub struct PostMissionProcessed(pub bool);

// === INVENTORY STATE ===
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

// === MANUFACTURING ===
#[derive(Resource, Default)]
pub struct ManufactureState {
    pub selected_agent_idx: usize,
    pub selected_weapon_idx: usize,
    pub selected_slot: Option<AttachmentSlot>,
    pub selected_attachments: std::collections::HashMap<AttachmentSlot, String>,
}

// === GLOBAL MAP UI ===
#[derive(Component)]
pub struct GlobalMapUI;

#[derive(Resource)]
pub struct ShouldRestart;

// === FPS COUNTER ===
#[derive(Component)]
pub struct FpsText;

// === DAY NIGHT CYCLE ===
#[derive(Resource)]
pub struct DayNightCycle {
    pub time_of_day: f32, // 0.0 to 24.0 hours
    pub cycle_speed: f32, // Real seconds per game hour
    pub current_period: TimeOfDay,
}

impl Default for DayNightCycle {
    fn default() -> Self {
        Self {
            time_of_day: 12.0, // Start at noon
            cycle_speed: 2.0,   // 2 real seconds = 1 game hour
            current_period: TimeOfDay::Day,
        }
    }
}

impl DayNightCycle {
    pub fn advance_time(&mut self, delta_secs: f32) {
        self.time_of_day += delta_secs / self.cycle_speed;
        if self.time_of_day >= 24.0 {
            self.time_of_day -= 24.0;
        }
        
        self.current_period = match self.time_of_day {
            t if t >= 6.0 && t < 18.0 => TimeOfDay::Day,
            t if t >= 18.0 && t < 20.0 => TimeOfDay::Dusk,
            t if t >= 20.0 || t < 4.0 => TimeOfDay::Night,
            _ => TimeOfDay::Dawn,
        };
    }
    
    pub fn get_ambient_light(&self) -> Color {
        match self.current_period {
            TimeOfDay::Day => Color::srgb(1.0, 1.0, 1.0),
            TimeOfDay::Dusk => Color::srgb(0.9, 0.7, 0.5),
            TimeOfDay::Night => Color::srgb(0.3, 0.3, 0.5),
            TimeOfDay::Dawn => Color::srgb(0.8, 0.8, 0.9),
        }
    }
    
    pub fn get_overlay_color(&self) -> Color {
        // Semi-transparent overlay that darkens the scene
        match self.current_period {
            TimeOfDay::Day => Color::srgba(1.0, 1.0, 0.9, 0.0),      // No overlay during day
            TimeOfDay::Dusk => Color::srgba(0.8, 0.4, 0.2, 0.2),     // Warm orange tint
            TimeOfDay::Night => Color::srgba(0.1, 0.1, 0.3, 0.6),    // Dark blue overlay
            TimeOfDay::Dawn => Color::srgba(0.8, 0.6, 0.8, 0.3),     // Light purple tint
        }
    }
    
    pub fn get_visibility_modifier(&self) -> f32 {
        match self.current_period {
            TimeOfDay::Day => 1.0,
            TimeOfDay::Dusk => 0.8,
            TimeOfDay::Night => 0.5,
            TimeOfDay::Dawn => 0.7,
        }
    }
    
    pub fn get_time_string(&self) -> String {
        let hours = self.time_of_day as u32;
        let minutes = ((self.time_of_day - hours as f32) * 60.0) as u32;
        format!("{:02}:{:02}", hours, minutes)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TimeOfDay {
    Day,
    Dusk,
    Night,
    Dawn,
}