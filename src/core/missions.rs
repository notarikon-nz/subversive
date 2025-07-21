// src/core/missions.rs - Mission briefing and evaluation systems
use bevy::prelude::*;
use crate::core::{AlertLevel, TimeOfDay, GlobalData};

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
#[derive(Resource, Default)]
pub struct MissionBriefingCache {
    cached_briefings: std::collections::HashMap<(usize, u32), MissionBriefing>, // (region, day) -> briefing
}

impl MissionBriefingCache {
    pub fn get_or_generate(&mut self, global_data: &GlobalData, region_idx: usize) -> &MissionBriefing {
        let cache_key = (region_idx, global_data.current_day);
        
        if !self.cached_briefings.contains_key(&cache_key) {
            let briefing = generate_mission_briefing(global_data, region_idx);
            self.cached_briefings.insert(cache_key, briefing);
        }
        
        &self.cached_briefings[&cache_key]
    }
    
    pub fn clear_old_briefings(&mut self, current_day: u32) {
        // Remove briefings older than 3 days
        self.cached_briefings.retain(|(_, day), _| current_day - day <= 3);
    }
}

// === HELPER FUNCTIONS ===
pub fn generate_mission_briefing(global_data: &GlobalData, region_idx: usize) -> MissionBriefing {
    let region = &global_data.regions[region_idx];
    let difficulty_mod = region.mission_difficulty_modifier();
    
    // Generate objectives based on region
    let objectives = match region_idx {
        0 => vec![
            MissionObjective {
                name: "Access Corporate Terminal".to_string(),
                description: "Infiltrate the secured data terminal and extract financial records".to_string(),
                objective_type: ObjectiveType::Hack,
                required: true,
                difficulty: 2,
            },
            MissionObjective {
                name: "Minimize Casualties".to_string(),
                description: "Complete mission with minimal enemy engagement".to_string(),
                objective_type: ObjectiveType::Infiltrate,
                required: false,
                difficulty: 3,
            },
        ],
        1 => vec![
            MissionObjective {
                name: "Eliminate Security Chief".to_string(),
                description: "Neutralize the target and secure their access codes".to_string(),
                objective_type: ObjectiveType::Eliminate,
                required: true,
                difficulty: 4,
            },
            MissionObjective {
                name: "Extract Intel Documents".to_string(),
                description: "Recover classified research data from secure servers".to_string(),
                objective_type: ObjectiveType::Extract,
                required: true,
                difficulty: 3,
            },
        ],
        2 => vec![
            MissionObjective {
                name: "Survive the Underground".to_string(),
                description: "Navigate hostile territory and reach extraction point".to_string(),
                objective_type: ObjectiveType::Survive,
                required: true,
                difficulty: 5,
            },
            MissionObjective {
                name: "Destroy Research Equipment".to_string(),
                description: "Sabotage experimental neurovector prototypes".to_string(),
                objective_type: ObjectiveType::Eliminate,
                required: false,
                difficulty: 4,
            },
        ],
        _ => vec![],
    };
    
    // Generate resistance profile
    let resistance = ResistanceProfile {
        enemy_count: (3.0 + (region.threat_level as f32 * 2.0 * difficulty_mod)) as u8,
        patrol_density: 0.3 + (region.threat_level as f32 * 0.2),
        security_level: region.threat_level.min(5),
        enemy_types: match region.threat_level {
            1 => vec![EnemyType::Guard, EnemyType::Patrol],
            2 => vec![EnemyType::Guard, EnemyType::Patrol, EnemyType::Elite],
            3 => vec![EnemyType::Patrol, EnemyType::Elite, EnemyType::Cyborg],
            _ => vec![EnemyType::Elite, EnemyType::Cyborg],
        },
        alert_sensitivity: match region.alert_level {
            AlertLevel::Green => 0.3,
            AlertLevel::Yellow => 0.5,
            AlertLevel::Orange => 0.7,
            AlertLevel::Red => 0.9,
        },
    };
    
    // Generate environment
    let environment = EnvironmentData {
        terrain: match region_idx {
            0 => TerrainType::Urban,
            1 => TerrainType::Corporate,
            2 => TerrainType::Underground,
            _ => TerrainType::Industrial,
        },
        visibility: match region_idx {
            2 => 0.4, // Underground has poor visibility
            _ => 0.8,
        },
        cover_density: match region_idx {
            0 => 0.6, // Urban has good cover
            1 => 0.3, // Corporate is more open
            2 => 0.7, // Underground has lots of cover
            _ => 0.5,
        },
        civilian_presence: match region_idx {
            0 => 4, // High civilian presence in urban
            1 => 2, // Some in corporate
            2 => 0, // None underground
            _ => 1,
        },
        time_of_day: TimeOfDay::Night, // Most missions are at night
    };
    
    // Calculate risks
    let avg_agent_level = global_data.agent_levels.iter().sum::<u8>() as f32 / 3.0;
    let level_gap = (region.threat_level as f32) - avg_agent_level;
    
    let casualty_risk = match level_gap {
        x if x <= -1.0 => RiskLevel::Low,
        x if x <= 0.5 => RiskLevel::Medium,
        x if x <= 1.5 => RiskLevel::High,
        _ => RiskLevel::Extreme,
    };
    
    let detection_risk = match (resistance.alert_sensitivity, environment.cover_density) {
        (s, c) if s > 0.7 && c < 0.4 => RiskLevel::Extreme,
        (s, c) if s > 0.5 || c < 0.5 => RiskLevel::High,
        (s, _) if s > 0.3 => RiskLevel::Medium,
        _ => RiskLevel::Low,
    };
    
    let equipment_loss_risk = match difficulty_mod {
        x if x >= 2.0 => RiskLevel::High,
        x if x >= 1.5 => RiskLevel::Medium,
        _ => RiskLevel::Low,
    };
    
    let recommended_loadout = match detection_risk {
        RiskLevel::High | RiskLevel::Extreme => vec![
            "Suppressed Weapons".to_string(),
            "Scanner Tool".to_string(),
            "Stealth Cybernetics".to_string(),
        ],
        _ => vec![
            "Combat Rifle".to_string(),
            "Medkit".to_string(),
            "Hacker Tool".to_string(),
        ],
    };
    
    let risks = RiskAssessment {
        casualty_risk,
        detection_risk,
        equipment_loss_risk,
        mission_failure_chance: (level_gap * 0.2 + difficulty_mod * 0.1).clamp(0.05, 0.8),
        recommended_agent_level: region.threat_level + 1,
        recommended_loadout,
    };
    
    // Generate rewards
    let rewards = MissionRewards {
        base_credits: 1000 + (region.threat_level as u32 * 500),
        bonus_credits: 500,
        equipment_chance: 0.2 + (region.threat_level as f32 * 0.1),
        intel_value: region.threat_level.min(5),
        experience_modifier: difficulty_mod,
    };
    
    MissionBriefing {
        region_id: region_idx,
        objectives,
        resistance,
        environment,
        rewards,
        risks,
    }
}