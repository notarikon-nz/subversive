// src/systems/quicksave.rs - Minimal mission quicksave
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::fs;
use crate::core::*;
use crate::systems::scenes::*;

const QUICKSAVE_FILE: &str = "quicksave.json";

#[derive(Serialize, Deserialize)]
struct QuickSave {
    timer: f32,
    objectives: u32,
    kills: u32,
    terminals: u32,
}

pub fn quicksave_system(
    input: Res<ButtonInput<KeyCode>>,
    game_state: Res<State<GameState>>,
    game_mode: Res<GameMode>,
    mut mission_data: ResMut<MissionData>,
    mut commands: Commands,
    entities: Query<Entity, (Without<Camera>, Without<Window>)>,
    sprites: Res<GameSprites>,
    global_data: Res<GlobalData>,
) {
    debug!("quicksave_system");
    if *game_state.get() != GameState::Mission || game_mode.paused {
        return;
    }

    // F5: Quick save
    if input.just_pressed(KeyCode::F5) {
        let save = QuickSave {
            timer: mission_data.timer,
            objectives: mission_data.objectives_completed,
            kills: mission_data.enemies_killed,
            terminals: mission_data.terminals_accessed,
        };
        
        if let Ok(json) = serde_json::to_string(&save) {
            if fs::write(QUICKSAVE_FILE, json).is_ok() {
                info!("Mission quicksaved");
            }
        }
    }

    // F8: Quick load
    if input.just_pressed(KeyCode::F8) {
        if let Ok(content) = fs::read_to_string(QUICKSAVE_FILE) {
            if let Ok(save) = serde_json::from_str::<QuickSave>(&content) {
                // Clear current mission
                for entity in entities.iter() {
                    commands.entity(entity).insert(MarkedForDespawn);
                }
                
                // Restart with saved progress
                match crate::systems::scenes::load_scene("mission1") {
                    Some(scene) => {
                        crate::systems::scenes::spawn_from_scene(&mut commands, &scene, &global_data, &sprites);
                    },
                    None => {
                        error!("Failed to load scene for quickload. Using fallback.");
                        // spawn_fallback_mission(&mut commands, &*global_data, &sprites);
                    }
                }
                
                // Restore progress
                mission_data.timer = save.timer;
                mission_data.objectives_completed = save.objectives;
                mission_data.enemies_killed = save.kills;
                mission_data.terminals_accessed = save.terminals;
                
                info!("Mission quickloaded");
            }
        }
    }
}