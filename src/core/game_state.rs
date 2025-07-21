// src/core/game_state.rs - Game states and global data
use bevy::prelude::*;
use crate::core::{ResearchProgress};

// === GAME STATES ===
#[derive(States, Debug, Clone, PartialEq, Eq, Hash)]
pub enum GameState {
    GlobalMap,
    Mission,
    PostMission,
}

impl Default for GameState {
    fn default() -> Self { GameState::GlobalMap }
}

// === ALERT LEVELS ===
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AlertLevel {
    Green,
    Yellow,
    Orange,
    Red,
}

// === GLOBAL DATA ===
const MAX_SQUAD_SIZE: usize = 3;

#[derive(Resource, Clone)]
pub struct GlobalData {
    pub credits: u32,
    pub selected_region: usize,
    pub regions: Vec<Region>,
    pub agent_levels: [u8; MAX_SQUAD_SIZE],
    pub agent_experience: [u32; MAX_SQUAD_SIZE],
    pub current_day: u32,
    pub agent_recovery: [u32; MAX_SQUAD_SIZE],
    pub agent_loadouts: [crate::core::AgentLoadout; MAX_SQUAD_SIZE],
    pub research_progress: ResearchProgress,
}

impl GlobalData {
    pub fn get_agent_loadout(&self, agent_idx: usize) -> &crate::core::AgentLoadout {
        &self.agent_loadouts[agent_idx.min(2)]
    }
    
    pub fn save_agent_loadout(&mut self, agent_idx: usize, loadout: crate::core::AgentLoadout) {
        if agent_idx < 3 {
            self.agent_loadouts[agent_idx] = loadout;
            info!("Saved loadout for Agent {}", agent_idx + 1);
        }
    }
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
            agent_loadouts: [
                crate::core::AgentLoadout::default(),
                crate::core::AgentLoadout::default(), 
                crate::core::AgentLoadout::default()
            ],
            research_progress: ResearchProgress::default(),
        }
    }
}

// === REGIONS ===
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