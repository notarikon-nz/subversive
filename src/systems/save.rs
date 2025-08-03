use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::fs;
use std::collections::HashSet;
use crate::core::*;

const SAVE_FILE: &str = "subversive_save.json";

#[derive(Clone, Serialize, Deserialize)]
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
    pub recruited_scientists: Vec<Scientist>,
    pub research_facilities_discovered: HashSet<String>,
    pub alert_level: u8,
     // 0.2.17
    pub territory_manager: Option<TerritoryManager>,
    pub progression_tracker: Option<CampaignProgressionTracker>,
}

#[derive(Clone, Serialize, Deserialize)]
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
            recruited_scientists: data.recruited_scientists.clone(),
            research_facilities_discovered: data.research_facilities_discovered.clone(),
            alert_level: data.alert_level,
            // 0.2.17
            territory_manager: None,
            progression_tracker: None,
        }
    }
}

impl From<SaveData> for GlobalData {
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
            recruited_scientists: save.recruited_scientists,
            research_facilities_discovered: save.research_facilities_discovered,
            alert_level: save.alert_level,
        };

        global_data
    }
}

pub fn save_game_complete(
    global_data: &GlobalData,
    research_progress: &ResearchProgress,
    territory_manager: &TerritoryManager,
    progression_tracker: &CampaignProgressionTracker,
) {
    let mut updated_global_data = global_data.clone();
    updated_global_data.research_progress = research_progress.clone();

    let mut save_data = SaveData::from(&updated_global_data);

    // ADD TERRITORY DATA:
    save_data.territory_manager = Some(territory_manager.clone());
    save_data.progression_tracker = Some(progression_tracker.clone());

    if let Ok(json) = serde_json::to_string_pretty(&save_data) {
        if fs::write(SAVE_FILE, json).is_ok() {
            info!("Game saved successfully");
        } else {
            warn!("Failed to save game");
        }
    }
}

pub fn load_game() -> Option<(GlobalData, TerritoryManager, CampaignProgressionTracker)> {
    fs::read_to_string(SAVE_FILE)
        .ok()
        .and_then(|content| serde_json::from_str::<SaveData>(&content).ok())
        .map(|save_data| {
            let global_data = GlobalData::from(save_data.clone());
            let territory_manager = save_data.territory_manager.unwrap_or_default();
            let progression_tracker = save_data.progression_tracker.unwrap_or_default();
            (global_data, territory_manager, progression_tracker)
        })
}

// Used by Main Menu for Continue logic
pub fn save_game_exists() -> bool {
    std::path::Path::new(SAVE_FILE).exists()
}

pub fn save_input_system(
    input: Res<ButtonInput<KeyCode>>,
    global_data: Res<GlobalData>,
    research_progress: Res<ResearchProgress>,
    territory_manager: Res<TerritoryManager>,
    progression_tracker: Res<CampaignProgressionTracker>,
    game_state: Res<State<GameState>>,
) {
    if input.just_pressed(KeyCode::F5) && *game_state.get() == GameState::GlobalMap {
        save_game_complete(&global_data, &research_progress, &territory_manager, &progression_tracker);
    }
}

pub fn auto_save_system(
    global_data: Res<GlobalData>,
    research_progress: Res<ResearchProgress>,
    territory_manager: Res<TerritoryManager>,
    progression_tracker: Res<CampaignProgressionTracker>,
    mut last_day: Local<u32>,
) {
    if global_data.current_day != *last_day && global_data.current_day > 1 {
        save_game_complete(&global_data, &research_progress, &territory_manager, &progression_tracker);
        *last_day = global_data.current_day;
    }
}

pub fn post_mission_save_system(
    mut processed: ResMut<PostMissionProcessed>,
    global_data: Res<GlobalData>,
    research_progress: Res<ResearchProgress>,
    territory_manager: Res<TerritoryManager>,
    progression_tracker: Res<CampaignProgressionTracker>,
    cities_progress: Res<CitiesProgress>,
    post_mission: Res<PostMissionResults>,
) {
    if processed.0 && post_mission.success {
        save_game_complete(&global_data, &research_progress, &territory_manager, &progression_tracker);
        info!("Auto-saved after successful mission completion");
        processed.0 = false;
    }
}