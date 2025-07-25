use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::fs;
use crate::core::*;

const SAVE_FILE: &str = "subversive_save.json";

#[derive(Serialize, Deserialize)]
pub struct SaveData {
    pub credits: u32,
    pub current_day: u32,
    pub agent_levels: [u8; 3],
    pub agent_experience: [u32; 3],
    pub agent_recovery: [u32; 3],
    pub regions: Vec<SaveRegion>,
    pub agent_loadouts: [AgentLoadout; 3],
    pub research_progress: ResearchProgress,
    pub cities_progress: CitiesProgress,
}

#[derive(Serialize, Deserialize)]
pub struct SaveRegion {
    pub name: String,
    pub threat_level: u8,
    pub alert_level: u8, // Serialize as u8 instead of enum
    pub alert_decay_timer: u32,
}

impl From<(&GlobalData)> for SaveData {
    fn from((data): (&GlobalData)) -> Self {
        Self {
            credits: data.credits,
            current_day: data.current_day,
            agent_levels: data.agent_levels,
            agent_experience: data.agent_experience,
            agent_recovery: data.agent_recovery,
            agent_loadouts: data.agent_loadouts.clone(),
            research_progress: data.research_progress.clone(),
            cities_progress: data.cities_progress.clone(),
            regions: data.regions.iter().map(|r| SaveRegion {
                name: r.name.clone(),
                threat_level: r.threat_level,
                alert_level: match r.alert_level {
                    AlertLevel::Green => 0,
                    AlertLevel::Yellow => 1,
                    AlertLevel::Orange => 2,
                    AlertLevel::Red => 3,
                },
                alert_decay_timer: r.alert_decay_timer,
            }).collect(),
        }
    }
}

impl From<SaveData> for (GlobalData) {
    fn from(save: SaveData) -> Self {
        let global_data = GlobalData {
            credits: save.credits,
            selected_region: 0,
            current_day: save.current_day,
            agent_levels: save.agent_levels,
            agent_experience: save.agent_experience,
            agent_recovery: save.agent_recovery,
            agent_loadouts: save.agent_loadouts,
            research_progress: save.research_progress,
            regions: save.regions.into_iter().map(|r| Region {
                name: r.name,
                threat_level: r.threat_level,
                alert_level: match r.alert_level {
                    0 => AlertLevel::Green,
                    1 => AlertLevel::Yellow,
                    2 => AlertLevel::Orange,
                    _ => AlertLevel::Red,
                },
                alert_decay_timer: r.alert_decay_timer,
            }).collect(),
            cities_progress: save.cities_progress.clone(),
        };
        
        (global_data)
    }
}

pub fn save_game_complete(
    global_data: &GlobalData,
    research_progress: &ResearchProgress,
) {
    // Create a mutable copy of global data with current research
    let mut updated_global_data = global_data.clone();
    updated_global_data.research_progress = research_progress.clone();
    
    let save_data = SaveData::from((&updated_global_data));
    if let Ok(json) = serde_json::to_string_pretty(&save_data) {
        if fs::write(SAVE_FILE, json).is_ok() {
            info!("Game saved successfully - {} research projects completed, {} cities unlocked", 
                  research_progress.completed.len(),
                  updated_global_data.cities_progress.unlocked_cities.len());
        } else {
            warn!("Failed to save game");
        }
    }
}

pub fn load_game() -> Option<(GlobalData)> {
    fs::read_to_string(SAVE_FILE)
        .ok()
        .and_then(|content| serde_json::from_str::<SaveData>(&content).ok())
        .map(|save_data| save_data.into())
}

// Legacy function for backward compatibility
pub fn save_game(global_data: &GlobalData) {
    // Create default cities progress if not available
    let default_cities_progress = CitiesProgress::new("new_york".to_string());
    save_game_complete(global_data, &global_data.research_progress);
}

// Update the save input system
pub fn save_input_system(
    input: Res<ButtonInput<KeyCode>>,
    global_data: Res<GlobalData>,
    research_progress: Res<ResearchProgress>,
    game_state: Res<State<GameState>>,
) {
    if input.just_pressed(KeyCode::F5) && *game_state.get() == GameState::GlobalMap {
        save_game_complete(&global_data, &research_progress);
    }
}

// Update auto_save_system
pub fn auto_save_system(
    global_data: Res<GlobalData>,
    research_progress: Res<ResearchProgress>,
    mut last_day: Local<u32>,
) {
    if global_data.current_day != *last_day && global_data.current_day > 1 {
        save_game_complete(&global_data, &research_progress);
        *last_day = global_data.current_day;
    }
}

// Add this system to handle post-mission saves
pub fn post_mission_save_system(
    mut processed: ResMut<PostMissionProcessed>,
    global_data: Res<GlobalData>,
    research_progress: Res<ResearchProgress>,
    cities_progress: Res<CitiesProgress>,
    post_mission: Res<PostMissionResults>,
) {
    if processed.0 && post_mission.success {
        save_game_complete(&global_data, &research_progress);
        info!("Auto-saved after successful mission completion");
        processed.0 = false; // Reset for next mission
    }
}