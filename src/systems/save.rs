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
}

#[derive(Serialize, Deserialize)]
pub struct SaveRegion {
    pub name: String,
    pub threat_level: u8,
    pub alert_level: u8, // Serialize as u8 instead of enum
    pub alert_decay_timer: u32,
}

impl From<&GlobalData> for SaveData {
    fn from(data: &GlobalData) -> Self {
        Self {
            credits: data.credits,
            current_day: data.current_day,
            agent_levels: data.agent_levels,
            agent_experience: data.agent_experience,
            agent_recovery: data.agent_recovery,
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

impl From<SaveData> for GlobalData {
    fn from(save: SaveData) -> Self {
        Self {
            credits: save.credits,
            selected_region: 0,
            current_day: save.current_day,
            agent_levels: save.agent_levels,
            agent_experience: save.agent_experience,
            agent_recovery: save.agent_recovery,
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
        }
    }
}

pub fn save_game(global_data: &GlobalData) {
    let save_data = SaveData::from(global_data);
    if let Ok(json) = serde_json::to_string_pretty(&save_data) {
        if fs::write(SAVE_FILE, json).is_ok() {
            info!("Game saved successfully");
        } else {
            warn!("Failed to save game");
        }
    }
}

pub fn load_game() -> Option<GlobalData> {
    fs::read_to_string(SAVE_FILE)
        .ok()
        .and_then(|content| serde_json::from_str::<SaveData>(&content).ok())
        .map(GlobalData::from)
}

pub fn auto_save_system(
    global_data: Res<GlobalData>,
    mut last_day: Local<u32>,
) {
    if global_data.current_day != *last_day && global_data.current_day > 1 {
        save_game(&global_data);
        *last_day = global_data.current_day;
    }
}

pub fn save_input_system(
    input: Res<ButtonInput<KeyCode>>,
    global_data: Res<GlobalData>,
    game_state: Res<State<GameState>>,
) {
    if input.just_pressed(KeyCode::F5) && *game_state.get() == GameState::GlobalMap {
        save_game(&global_data);
    }
}