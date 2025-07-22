// src/core/missions.rs - Mission briefing and evaluation systems
use bevy::prelude::*;
use crate::core::*;
use rand::{thread_rng, Rng};

// === MISSION BRIEFING ===
#[derive(Clone)]
pub struct MissionBriefing {
    pub region_id: usize,
    pub objectives: Vec<MissionObjective>,
    pub resistance: ResistanceProfile,
    pub environment: EnvironmentData,
    pub rewards: MissionRewards,
    pub risks: RiskAssessment,
}

#[derive(Clone)]
pub struct MissionObjective {
    pub name: String,
    pub description: String,
    pub objective_type: ObjectiveType,
    pub required: bool,
    pub difficulty: u8, // 1-5
}

#[derive(Clone)]
pub enum ObjectiveType {
    Eliminate,
    Extract,
    Hack,
    Infiltrate,
    Survive,
}

#[derive(Clone)]
pub struct ResistanceProfile {
    pub enemy_count: u8,
    pub patrol_density: f32,
    pub security_level: u8, // 1-5
    pub enemy_types: Vec<EnemyType>,
    pub alert_sensitivity: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum EnemyType {
    Guard,
    Patrol,
    Elite,
    Cyborg,
}

#[derive(Clone)]
pub struct EnvironmentData {
    pub terrain: TerrainType,
    pub visibility: f32, // 0.0-1.0
    pub cover_density: f32,
    pub civilian_presence: u8,
    pub time_of_day: TimeOfDay,
}

#[derive(Debug, Clone)]
pub enum TerrainType {
    Urban,
    Corporate,
    Industrial,
    Underground,
}

#[derive(Clone)]
pub struct MissionRewards {
    pub base_credits: u32,
    pub bonus_credits: u32,
    pub equipment_chance: f32,
    pub intel_value: u8,
    pub experience_modifier: f32,
}

#[derive(Clone)]
pub struct RiskAssessment {
    pub casualty_risk: RiskLevel,
    pub detection_risk: RiskLevel,
    pub equipment_loss_risk: RiskLevel,
    pub mission_failure_chance: f32,
    pub recommended_agent_level: u8,
    pub recommended_loadout: Vec<String>,
}

#[derive(Clone)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Extreme,
}

impl RiskLevel {
    pub fn color(&self) -> Color {
        match self {
            RiskLevel::Low => Color::srgb(0.2, 0.8, 0.2),
            RiskLevel::Medium => Color::srgb(0.8, 0.8, 0.2),
            RiskLevel::High => Color::srgb(0.8, 0.5, 0.2),
            RiskLevel::Extreme => Color::srgb(0.8, 0.2, 0.2),
        }
    }
    
    pub fn text(&self) -> &'static str {
        match self {
            RiskLevel::Low => "LOW",
            RiskLevel::Medium => "MEDIUM", 
            RiskLevel::High => "HIGH",
            RiskLevel::Extreme => "EXTREME",
        }
    }
}

// === MISSION STATE ===
#[derive(Resource, Default)]
pub struct MissionState {
    pub current_briefing: Option<MissionBriefing>,
    pub selected_objective: usize,
    pub deployment_confirmed: bool,
}

// === MISSION TRACKING ===
#[derive(Component)]
pub struct MissionTracker {
    pub objectives: Vec<ObjectiveStatus>,
    pub bonus_criteria_met: Vec<bool>,
    pub stealth_rating: f32, // 0.0 = detected, 1.0 = perfect stealth
    pub efficiency_rating: f32, // Based on time and method
}

#[derive(Clone)]
pub struct ObjectiveStatus {
    pub completed: bool,
    pub method: Option<CompletionMethod>,
    pub time_taken: f32,
}

#[derive(Clone)]
pub enum CompletionMethod {
    Stealth,
    Combat,
    Hacking,
    Diplomacy,
}

#[derive(Component)]
pub struct MissionModifiers {
    pub reinforcements_called: bool,
    pub alarm_triggered: bool,
    pub civilian_casualties: u8,
    pub equipment_recovered: Vec<String>,
    pub intel_gathered: Vec<String>,
}

// === MISSION PERFORMANCE ===
#[derive(Default)]
pub struct MissionPerformance {
    pub success: bool,
    pub stealth_rating: f32,
    pub efficiency_rating: f32,
    pub objective_completion: f32,
    pub stealth_bonus: u32,
    pub speed_bonus: u32,
    pub civilian_penalty: f32,
    pub research_progress: Option<String>,
    pub final_credits: u32,
}

// === EQUIPMENT RECOMMENDATIONS ===
#[derive(Clone)]
pub struct EquipmentRecommendation {
    pub item_type: RecommendationType,
    pub name: String,
    pub reason: String,
    pub priority: Priority,
}

#[derive(Clone)]
pub enum RecommendationType {
    Weapon,
    Tool,
    Attachment,
    Cybernetic,
}

#[derive(Clone)]
pub enum Priority {
    Low,
    Medium,
    High,
    Critical,
}

impl Priority {
    pub fn color(&self) -> Color {
        match self {
            Priority::Low => Color::srgb(0.6, 0.6, 0.6),
            Priority::Medium => Color::srgb(0.8, 0.8, 0.2),
            Priority::High => Color::srgb(0.8, 0.5, 0.2),
            Priority::Critical => Color::srgb(0.8, 0.2, 0.2),
        }
    }
}

// === BRIEFING CACHE ===
// Update the cache to use city_id instead of region_idx
#[derive(Resource, Default)]
pub struct MissionBriefingCache {
    pub cached_briefings: std::collections::HashMap<String, MissionBriefing>, // Changed from usize to String
    pub last_day_generated: u32,
}

impl MissionBriefingCache {
    pub fn get_or_generate_for_city(
        &mut self,
        city_id: &str,
        current_day: u32,
        global_data: &GlobalData,
        cities_db: &CitiesDatabase,
        cities_progress: &CitiesProgress,
    ) -> &MissionBriefing {
        // Invalidate cache if day changed
        if self.last_day_generated != current_day {
            self.cached_briefings.clear();
            self.last_day_generated = current_day;
        }
        
        // Get or generate briefing for this city
        self.cached_briefings.entry(city_id.to_string()).or_insert_with(|| {
            generate_mission_briefing_for_city(global_data, cities_db, cities_progress, city_id)
        })
    }
}

pub fn generate_mission_briefing_for_city(
    global_data: &GlobalData,
    cities_db: &CitiesDatabase,
    cities_progress: &CitiesProgress,
    city_id: &str,
) -> MissionBriefing {
    let city = cities_db.get_city(city_id).expect("City not found");
    
    let city_state = cities_progress.get_city_state(city_id);
    
    // Use a simple hash-based random generation for consistency
    let seed = city_id.len() + global_data.current_day as usize + city.corruption_level as usize;
    let pseudo_random = (seed * 1103515245 + 12345) % (1 << 31);
    let random_f32 = (pseudo_random as f32) / (1u32 << 31) as f32;
    
    // Convert city data to mission parameters
    let base_difficulty = city.corruption_level as f32 / 10.0; // 0.1 to 1.0
    let alert_modifier = match city_state.alert_level {
        AlertLevel::Green => 0.8,
        AlertLevel::Yellow => 1.0,
        AlertLevel::Orange => 1.3,
        AlertLevel::Red => 1.6,
    };
    
    let effective_difficulty = (base_difficulty * alert_modifier).clamp(0.1, 2.0);
    
    // Generate objectives based on city traits and corporation
    let objectives = generate_city_objectives(&city, effective_difficulty, seed);
    
    // Enemy resistance based on city properties
    let enemy_count = (8.0 + (city.population as f32 * 0.5) + (effective_difficulty * 10.0)) as u32;
    let security_level = ((city.corruption_level as f32 * 0.4) + (effective_difficulty * 3.0)) as u8;
    
    let resistance = ResistanceProfile {
        enemy_count: enemy_count.clamp(5, 25) as u8,
        security_level: security_level.clamp(1, 5),
        alert_sensitivity: 0.3 + (effective_difficulty * 0.4),
        patrol_density: 0.2 + (effective_difficulty * 0.3),
        enemy_types: vec![EnemyType::Guard, EnemyType::Patrol], // Use existing enemy types only
    };
    
    // Environment based on city traits  
    let environment = EnvironmentData {
        terrain: TerrainType::Urban, // Keep it simple for now
        cover_density: 0.4 + random_f32 * 0.4,
        visibility: 0.6 + random_f32 * 0.3,
        civilian_presence: (city.population / 3).clamp(0, 5) as u8, // Civilians are separate from enemies
        time_of_day: match (pseudo_random % 4) {
            0 => TimeOfDay::Dawn,
            1 => TimeOfDay::Day,
            2 => TimeOfDay::Dusk,
            _ => TimeOfDay::Night,
        },
    };
    
    // Risk assessment
    let casualty_risk = match (effective_difficulty * 4.0) as u32 {
        0 => RiskLevel::Low,
        1 => RiskLevel::Medium,
        2 => RiskLevel::High,
        _ => RiskLevel::Extreme,
    };
    let casualty_risk_val = casualty_risk.clone();
    
    let risks = RiskAssessment {
        casualty_risk,
        detection_risk: match city_state.alert_level {
            AlertLevel::Green => RiskLevel::Low,
            AlertLevel::Yellow => RiskLevel::Medium,
            AlertLevel::Orange => RiskLevel::High,
            AlertLevel::Red => RiskLevel::Extreme,
        },
        equipment_loss_risk: casualty_risk_val,
        mission_failure_chance: (0.1 + effective_difficulty * 0.3).clamp(0.05, 0.8),
        recommended_agent_level: (1 + (effective_difficulty * 4.0) as u8).clamp(1, 8),
        recommended_loadout: generate_city_loadout_recommendations(&city, &environment),
    };
    
    // Rewards scaled by difficulty and city wealth
    let base_credits = ((200.0 * effective_difficulty) as u32).clamp(100, 800);
    
    let rewards = MissionRewards {
        base_credits,
        bonus_credits: ((100.0 * effective_difficulty) as u32).clamp(50, 300),
        equipment_chance: (0.1 + effective_difficulty * 0.2).clamp(0.05, 0.4),
        experience_modifier: 1.0 + (effective_difficulty * 0.3),
        intel_value: ((effective_difficulty * 3.0) as u8).clamp(1, 5),
    };
    
    MissionBriefing {
        objectives,
        resistance,
        environment,
        risks,
        rewards,
        region_id: city_id.len() + seed, // Use city_id as region_id for now
    }
}

/*
    Eliminate,
    Extract,
    Hack,
    Infiltrate,
    Survive,
    */
fn generate_city_objectives(city: &City, difficulty: f32, seed: usize) -> Vec<MissionObjective> {
    let mut objectives = vec![
        MissionObjective {
            name: "Infiltrate Target Building".to_string(),
            description: format!("Gain access to {:?} corporate facility", city.controlling_corp),
            required: true,
            difficulty: (1 + (difficulty * 3.0) as u8).clamp(1, 5),
            objective_type: ObjectiveType::Infiltrate, // Add required field
        }
    ];
    
    // Use seed for deterministic randomness
    let random_chance = ((seed * 31) % 100) as f32 / 100.0;
    
    // Add optional data extraction objective
    if random_chance > 0.3 {
        objectives.push(MissionObjective {
            name: "Download Corporate Data".to_string(),
            description: "Access secure servers and extract intelligence".to_string(),
            required: false,
            difficulty: (2 + (difficulty * 2.0) as u8).clamp(1, 4),
            objective_type: ObjectiveType::Extract,
        });
    }
    
    // Add elimination objective for high corruption cities
    if city.corruption_level >= 7 && random_chance > 0.6 {
        objectives.push(MissionObjective {
            name: "Eliminate High-Value Target".to_string(),
            description: "Remove key corporate operative".to_string(),
            required: false,
            difficulty: (3 + (difficulty * 2.0) as u8).clamp(2, 5),
            objective_type: ObjectiveType::Eliminate,
        });
    }
    
    objectives
}

fn generate_city_loadout_recommendations(city: &City, environment: &EnvironmentData) -> Vec<String> {
    let mut recommendations = vec!["Basic Combat Gear".to_string()];
    
    // Simple recommendations based on corruption level
    if city.corruption_level >= 7 {
        recommendations.push("Heavy Armor - High security expected".to_string());
        recommendations.push("Advanced Weapons".to_string());
    } else if city.corruption_level >= 4 {
        recommendations.push("Stealth Equipment - Moderate security".to_string());
    }
    
    // High population means more civilians
    if city.population >= 4 {
        recommendations.push("Non-Lethal Options - High civilian presence".to_string());
    }
    
    if environment.visibility < 0.5 {
        recommendations.push("Night Vision Equipment".to_string());
    }
    
    recommendations
}