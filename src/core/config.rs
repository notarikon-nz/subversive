// src/core/config.rs - Game configuration and balancing
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Resource, Serialize, Deserialize, Clone)]
pub struct GameConfig {
    pub gameplay: GameplayConfig,
    pub combat: CombatConfig,
    pub ai: AIConfig,
    pub civilians: CiviliansConfig,
    pub neurovector: NeurovectorConfig,
    pub police: PoliceConfig,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct GameplayConfig {
    pub max_squad_size: usize,
    pub base_mission_time_limit: f32,
    pub starting_credits: u32,
    pub experience_per_level_multiplier: u32,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct CombatConfig {
    pub base_agent_health: f32,
    pub base_enemy_health: f32,
    pub base_weapon_damage: f32,
    pub base_weapon_accuracy: f32,
    pub base_weapon_range: f32,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct AIConfig {
    pub enemy_vision_range: f32,
    pub enemy_vision_angle: f32,
    pub patrol_speed: f32,
    pub alert_duration: f32,
    pub goap_planning_interval: f32,
    pub sound_detection_range: f32,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct CiviliansConfig {
    pub max_civilians: u32,
    pub spawn_interval_min: f32,
    pub spawn_interval_max: f32,
    pub base_morale: f32,
    pub panic_threshold: f32,
    pub movement_speed: f32,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct NeurovectorConfig {
    pub base_range: f32,
    pub max_targets: u8,
    pub cooldown: f32,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct PoliceConfig {
    pub heat_decay_rate: f32,
    pub spawn_threshold: f32,
    pub response_time: f32,
    pub max_units_per_spawn: u8,
}

impl GameConfig {
    pub fn load() -> Self {
        match std::fs::read_to_string("data/config/game.json") {
            Ok(content) => {
                serde_json::from_str(&content)
                    .map_err(|e| error!("Failed to parse game config: {}", e))
                    .unwrap_or_else(|_| Self::default())
            },
            Err(e) => {
                error!("Failed to load game config: {}", e);
                Self::default()
            }
        }
    }
}

impl Default for GameConfig {
    fn default() -> Self {
        Self {
            gameplay: GameplayConfig {
                max_squad_size: 3,
                base_mission_time_limit: 300.0,
                starting_credits: 1000,
                experience_per_level_multiplier: 100,
            },
            combat: CombatConfig {
                base_agent_health: 100.0,
                base_enemy_health: 100.0,
                base_weapon_damage: 35.0,
                base_weapon_accuracy: 0.8,
                base_weapon_range: 150.0,
            },
            ai: AIConfig {
                enemy_vision_range: 120.0,
                enemy_vision_angle: 45.0,
                patrol_speed: 120.0,
                alert_duration: 8.0,
                goap_planning_interval: 2.0,
                sound_detection_range: 200.0,
            },
            civilians: CiviliansConfig {
                max_civilians: 12,
                spawn_interval_min: 8.0,
                spawn_interval_max: 12.0,
                base_morale: 80.0,
                panic_threshold: 40.0,
                movement_speed: 100.0,
            },
            neurovector: NeurovectorConfig {
                base_range: 200.0,
                max_targets: 3,
                cooldown: 5.0,
            },
            police: PoliceConfig {
                heat_decay_rate: 2.0,
                spawn_threshold: 25.0,
                response_time: 30.0,
                max_units_per_spawn: 4,
            },
        }
    }
}