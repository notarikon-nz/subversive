// src/core/mod.rs - Add GOAP module
use bevy::prelude::*;
use leafwing_input_manager::prelude::*;
use serde::{Deserialize, Serialize};

pub mod events;
pub mod audio;
pub mod sprites;
pub mod goap; 
pub mod research;
pub mod attachments;
pub mod agent_upgrades;
pub mod fonts;

pub use events::*;
pub use audio::*;
pub use sprites::*;
pub use goap::*;
pub use research::*;
pub use attachments::*;
pub use agent_upgrades::*;
pub use fonts::*;

pub use crate::systems::ui::hub::{HubState};

// === INPUT ===
// Updated for leafwing-input-manager 0.17.1 + Bevy 0.16
#[derive(Actionlike, PartialEq, Eq, Hash, Clone, Copy, Debug, Reflect)]
#[reflect(Hash, PartialEq)]  // Updated reflection traits
pub enum PlayerAction {
    Pause,
    Select,
    Move,
    Neurovector,
    Combat,
    Interact,
    Inventory,
    Reload,
}

// === STATES ===
#[derive(States, Debug, Clone, PartialEq, Eq, Hash)]
pub enum GameState {
    GlobalMap,
    Mission,
    PostMission,
}

impl Default for GameState {
    fn default() -> Self { GameState::GlobalMap }
}

// === SQUAD GOALS ===
#[derive(Component)]
pub struct SelectionBox {
    pub start: Vec2,
    pub end: Vec2,
}

#[derive(Resource, Default)]
pub struct SelectionDrag {
    pub dragging: bool,
    pub start_pos: Vec2,
    pub current_pos: Vec2,
}

// === RESOURCES ===
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

#[derive(Resource, Default)]
pub struct UIState {
    pub global_map_open: bool,
    pub inventory_open: bool,
    pub pause_open: bool,
    pub post_mission_open: bool,
    pub fps_visible: bool,
}

#[derive(Resource, Default)]
pub struct PostMissionProcessed(pub bool);

#[derive(Component)]
pub struct FpsText;

#[derive(Debug)]
pub enum TargetingMode {
    Neurovector { agent: Entity },
    Combat { agent: Entity },
}

#[derive(Resource, Default)]
pub struct SelectionState {
    pub selected: Vec<Entity>,
}

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

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AlertLevel {
    Green,
    Yellow,
    Orange,
    Red,
}

// === CORE COMPONENTS ===
#[derive(Component)]
pub struct Agent {
    pub experience: u32,
    pub level: u8,
}

impl Default for Agent {
    fn default() -> Self {
        Self { experience: 0, level: 1 }
    }
}

#[derive(Component)]
pub struct Civilian;

#[derive(Component)]
pub struct Enemy;

#[derive(Component)]
pub struct Health(pub f32);

#[derive(Component)]
pub struct Dead;

#[derive(Component)]
pub struct MovementSpeed(pub f32);

#[derive(Component)]
pub struct Controllable;

#[derive(Component)]
pub struct Selected;

#[derive(Component)]
pub struct Selectable {
    pub radius: f32,
}

#[derive(Component)]
pub struct NeurovectorTarget;

#[derive(Component)]
pub struct NeurovectorControlled {
    pub controller: Entity,
}

#[derive(Component)]
pub struct MoveTarget {
    pub position: Vec2,
}

#[derive(Component)]
pub struct Objective;

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
    pub agent_loadouts: [AgentLoadout; MAX_SQUAD_SIZE],
    pub research_progress: ResearchProgress,
}

impl GlobalData {
    pub fn get_agent_loadout(&self, agent_idx: usize) -> &AgentLoadout {
        &self.agent_loadouts[agent_idx.min(2)]
    }
    
    pub fn save_agent_loadout(&mut self, agent_idx: usize, loadout: AgentLoadout) {
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
                AgentLoadout::default(),
                AgentLoadout::default(), 
                AgentLoadout::default()
            ],
            research_progress: ResearchProgress::default(),
        }
    }
}

pub fn experience_for_level(level: u8) -> u32 {
    (level as u32) * 100
}

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

#[derive(Component)]
pub struct GlobalMapUI;

#[derive(Resource)]
pub struct ShouldRestart;

// === COMPLEX COMPONENTS ===
#[derive(Component)]
pub struct Vision {
    pub range: f32,
    pub angle: f32,
    pub direction: Vec2,
}

impl Vision {
    pub fn new(range: f32, angle_degrees: f32) -> Self {
        Self {
            range,
            angle: angle_degrees.to_radians(),
            direction: Vec2::X,
        }
    }
}

#[derive(Component)]
pub struct NeurovectorCapability {
    pub range: f32,
    pub max_targets: u8,
    pub cooldown: f32,
    pub current_cooldown: f32,
    pub controlled: Vec<Entity>,
}

impl Default for NeurovectorCapability {
    fn default() -> Self {
        Self {
            range: 200.0,
            max_targets: 3,
            cooldown: 5.0,
            current_cooldown: 0.0,
            controlled: Vec::new(),
        }
    }
}

#[derive(Component)]
pub struct Patrol {
    pub points: Vec<Vec2>,
    pub current_index: usize,
}

impl Patrol {
    pub fn new(points: Vec<Vec2>) -> Self {
        Self { points, current_index: 0 }
    }
    
    pub fn current_target(&self) -> Option<Vec2> {
        self.points.get(self.current_index).copied()
    }
    
    pub fn advance(&mut self) {
        if !self.points.is_empty() {
            self.current_index = (self.current_index + 1) % self.points.len();
        }
    }
}

#[derive(Component)]
pub struct Terminal {
    pub terminal_type: TerminalType,
    pub range: f32,
    pub accessed: bool,
}

#[derive(Debug, Clone)]
pub enum TerminalType {
    Objective,
    Equipment,
    Intel,
}

#[derive(Component)]
pub struct InventoryUI;

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

// Updated the Inventory struct to support weapon configs:
#[derive(Component, Default)]
pub struct Inventory {
    pub weapons: Vec<WeaponConfig>,  // Changed from WeaponType to WeaponConfig
    pub tools: Vec<ToolType>,
    pub currency: u32,
    pub equipped_weapon: Option<WeaponConfig>,  // Changed from WeaponType
    pub equipped_tools: Vec<ToolType>,
    pub cybernetics: Vec<CyberneticType>,
    pub intel_documents: Vec<String>,
}

#[derive(Component)]
pub struct InventoryVersion(pub u32);

impl Inventory {
    pub fn add_weapon(&mut self, weapon: WeaponType) {
        let config = WeaponConfig::new(weapon);
        if self.equipped_weapon.is_none() {
            self.equipped_weapon = Some(config.clone());
        }
        self.weapons.push(config);
    }
    
    pub fn add_weapon_config(&mut self, config: WeaponConfig) {
        if self.equipped_weapon.is_none() {
            self.equipped_weapon = Some(config.clone());
        }
        self.weapons.push(config);
    }
    
    // Keep existing methods unchanged
    pub fn add_currency(&mut self, amount: u32) {
        self.currency += amount;
    }
    
    pub fn add_tool(&mut self, tool: ToolType) {
        if self.equipped_tools.len() < 2 {
            self.equipped_tools.push(tool.clone());
        }
        self.tools.push(tool);
    }
    
    pub fn add_cybernetic(&mut self, cybernetic: CyberneticType) {
        self.cybernetics.push(cybernetic);
    }
    
    pub fn add_intel(&mut self, document: String) {
        self.intel_documents.push(document);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CyberneticType {
    Neurovector,
    CombatEnhancer,
    StealthModule,
    TechInterface,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WeaponType {
    Pistol,
    Rifle,
    Minigun,
    Flamethrower,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ToolType {
    Hacker,
    Scanner,
    Lockpick,
    MedKit,
}

// === UTILITY FUNCTIONS ===
pub fn get_world_mouse_position(
    windows: &Query<&Window>,
    cameras: &Query<(&Camera, &GlobalTransform)>,
) -> Option<Vec2> {
    let window = windows.single().ok()?;
    let (camera, camera_transform) = cameras.single().ok()?;
    let cursor_pos = window.cursor_position()?;
    
    camera.viewport_to_world_2d(camera_transform, cursor_pos).ok()
}

// === GOAP FEATURES ===
#[derive(Event)]
pub struct AlertEvent {
    pub alerter: Entity,
    pub position: Vec2,
    pub alert_type: AlertType,
}

#[derive(Debug, Clone)]
pub enum AlertType {
    CallForHelp,
    GunshotHeard,
    EnemySpotted,
}

#[derive(Resource, Default)]
pub struct ManufactureState {
    pub selected_agent_idx: usize,
    pub selected_weapon_idx: usize,
    pub selected_slot: Option<AttachmentSlot>,
    pub selected_attachments: std::collections::HashMap<AttachmentSlot, String>, // NEW: Per-slot selection
}

#[derive(Clone, Serialize, Deserialize)]
pub struct AgentLoadout {
    pub weapon_configs: Vec<WeaponConfig>,
    pub equipped_weapon_idx: usize,
    pub tools: Vec<ToolType>,
    pub cybernetics: Vec<CyberneticType>,
}

impl Default for AgentLoadout {
    fn default() -> Self {
        Self {
            weapon_configs: vec![WeaponConfig::new(WeaponType::Rifle)],
            equipped_weapon_idx: 0,
            tools: vec![ToolType::Scanner],
            cybernetics: vec![],
        }
    }
}

// === MISSION STUFF ===
// Mission data structures - move from UI to core
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

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TimeOfDay {
    Day,
    Dusk,
    Night,
    Dawn,
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

// Mission state resource
#[derive(Resource, Default)]
pub struct MissionState {
    pub current_briefing: Option<MissionBriefing>,
    pub selected_objective: usize,
    pub deployment_confirmed: bool,
}

// Enhanced mission completion tracking
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

// Mission modifier system for dynamic difficulty
#[derive(Component)]
pub struct MissionModifiers {
    pub reinforcements_called: bool,
    pub alarm_triggered: bool,
    pub civilian_casualties: u8,
    pub equipment_recovered: Vec<String>,
    pub intel_gathered: Vec<String>,
}

// Integration with existing systems
impl MissionBriefing {
    // Convert briefing into actual mission parameters
    pub fn apply_to_mission_data(&self, mission_data: &mut MissionData, global_data: &GlobalData) {
        // Set time limit based on mission complexity
        let base_time = 300.0; // 5 minutes base
        let complexity_modifier = self.objectives.len() as f32 * 60.0;
        let difficulty_modifier = self.risks.mission_failure_chance * 120.0;
        
        mission_data.time_limit = base_time + complexity_modifier + difficulty_modifier;
        
        // Set total objectives
        mission_data.total_objectives = self.objectives.iter()
            .filter(|obj| obj.required)
            .count() as u32;
        
        // Adjust alert level based on region status
        mission_data.alert_level = global_data.regions[self.region_id].alert_level;
        
        info!("Mission configured: {} objectives, {:.0}s time limit", 
              mission_data.total_objectives, mission_data.time_limit);
    }
    
    // Generate scene modifications based on briefing
    pub fn modify_scene_spawn(&self, commands: &mut Commands, base_scene: &crate::systems::scenes::SceneData) {
        // Adjust enemy count based on resistance profile
        let enemy_multiplier = match self.resistance.security_level {
            1..=2 => 1.0,
            3..=4 => 1.5,
            5 => 2.0,
            _ => 1.0,
        };
        
        // Spawn additional enemies for higher threat levels
        if enemy_multiplier > 1.0 {
            let additional_enemies = ((base_scene.enemies.len() as f32) * (enemy_multiplier - 1.0)) as usize;
            for i in 0..additional_enemies {
                let patrol_points = vec![
                    [100.0 + (i as f32 * 50.0), -50.0],
                    [150.0 + (i as f32 * 50.0), -50.0],
                ];
                
                // These would be spawned using the existing scene system
                info!("Would spawn additional enemy {} with patrol {:?}", i, patrol_points);
            }
        }
        
        // Adjust civilian presence
        if self.environment.civilian_presence == 0 {
            // Remove all civilians for underground missions
            info!("Mission briefing: No civilians in underground environment");
        }
        
        // Environmental modifications
        match self.environment.time_of_day {
            TimeOfDay::Night => {
                // Reduce enemy vision range
                info!("Night mission: Reduced enemy vision");
            },
            TimeOfDay::Day => {
                // Increase detection sensitivity
                info!("Day mission: Increased detection risk");
            },
            _ => {}
        }
    }
}

// Mission completion evaluation system
pub fn evaluate_mission_completion(
    mission_data: &MissionData,
    briefing: &MissionBriefing,
    tracker: &MissionTracker,
) -> MissionPerformance {
    let mut performance = MissionPerformance::default();
    
    // Base success check
    performance.success = mission_data.objectives_completed >= mission_data.total_objectives;
    
    // Calculate performance ratings
    performance.stealth_rating = tracker.stealth_rating;
    performance.efficiency_rating = calculate_efficiency_rating(mission_data, briefing);
    performance.objective_completion = tracker.objectives.iter()
        .filter(|obj| obj.completed)
        .count() as f32 / briefing.objectives.len() as f32;
    
    // Bonus calculations
    performance.stealth_bonus = if tracker.stealth_rating > 0.8 { 500 } else { 0 };
    performance.speed_bonus = if mission_data.timer < briefing.rewards.base_credits as f32 * 0.5 { 300 } else { 0 };
    performance.civilian_penalty = tracker.efficiency_rating * -200.0; // Negative for casualties
    
    // Research progression bonuses
    if let Some(research_bonus) = calculate_research_bonus(briefing, tracker) {
        performance.research_progress = Some(research_bonus);
    }
    
    performance
}

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

fn calculate_efficiency_rating(mission_data: &MissionData, briefing: &MissionBriefing) -> f32 {
    let time_efficiency = 1.0 - (mission_data.timer / briefing.rewards.base_credits as f32);
    let combat_efficiency = 1.0 - (mission_data.enemies_killed as f32 / briefing.resistance.enemy_count as f32);
    
    (time_efficiency + combat_efficiency) / 2.0
}

fn calculate_research_bonus(briefing: &MissionBriefing, tracker: &MissionTracker) -> Option<String> {
    // Grant research progress based on mission performance
    if tracker.stealth_rating > 0.9 {
        Some("infiltration_tech".to_string())
    } else if tracker.objectives.iter().any(|obj| matches!(obj.method, Some(CompletionMethod::Hacking))) {
        Some("cyber_warfare".to_string())
    } else if briefing.resistance.enemy_types.contains(&EnemyType::Cyborg) {
        Some("cybernetic_analysis".to_string())
    } else {
        None
    }
}

// Mission difficulty scaling system
pub fn calculate_dynamic_difficulty(
    global_data: &GlobalData, 
    region_idx: usize,
    days_since_last_mission: u32
) -> f32 {
    let base_difficulty = global_data.regions[region_idx].mission_difficulty_modifier();
    
    // Scale with agent experience
    let avg_agent_level = global_data.agent_levels.iter().sum::<u8>() as f32 / 3.0;
    let experience_scaling = 1.0 + (avg_agent_level - 1.0) * 0.1;
    
    // Alert level scaling
    let alert_scaling = match global_data.regions[region_idx].alert_level {
        AlertLevel::Green => 1.0,
        AlertLevel::Yellow => 1.2,
        AlertLevel::Orange => 1.4,
        AlertLevel::Red => 1.8,
    };
    
    // Time pressure - missions get harder if delayed
    let time_pressure = 1.0 + (days_since_last_mission as f32 * 0.05);
    
    (base_difficulty * experience_scaling * alert_scaling * time_pressure).min(3.0)
}

// Equipment recommendation engine
pub fn generate_equipment_recommendations(
    briefing: &MissionBriefing,
    global_data: &GlobalData,
    research_progress: &ResearchProgress,
) -> Vec<EquipmentRecommendation> {
    let mut recommendations = Vec::new();
    
    // Weapon recommendations based on expected resistance
    match briefing.resistance.security_level {
        1..=2 => {
            recommendations.push(EquipmentRecommendation {
                item_type: RecommendationType::Weapon,
                name: "Suppressed Pistol".to_string(),
                reason: "Low security allows for stealthy approach".to_string(),
                priority: Priority::Medium,
            });
        },
        3..=4 => {
            recommendations.push(EquipmentRecommendation {
                item_type: RecommendationType::Weapon,
                name: "Assault Rifle".to_string(),
                reason: "Moderate resistance expected".to_string(),
                priority: Priority::High,
            });
        },
        5 => {
            recommendations.push(EquipmentRecommendation {
                item_type: RecommendationType::Weapon,
                name: "Heavy Weapons".to_string(),
                reason: "Extreme resistance - maximum firepower needed".to_string(),
                priority: Priority::Critical,
            });
        },
        _ => {}
    }
    
    // Environment-based recommendations
    match briefing.environment.terrain {
        TerrainType::Underground => {
            recommendations.push(EquipmentRecommendation {
                item_type: RecommendationType::Tool,
                name: "Scanner".to_string(),
                reason: "Limited visibility in underground environment".to_string(),
                priority: Priority::High,
            });
        },
        TerrainType::Corporate => {
            recommendations.push(EquipmentRecommendation {
                item_type: RecommendationType::Tool,
                name: "Hacker".to_string(),
                reason: "Corporate security systems require hacking".to_string(),
                priority: Priority::High,
            });
        },
        _ => {}
    }
    
    // Research-based recommendations
    if research_progress.completed.contains("suppression_tech") {
        recommendations.push(EquipmentRecommendation {
            item_type: RecommendationType::Attachment,
            name: "Sound Suppressor".to_string(),
            reason: "Research unlocked - reduces detection risk".to_string(),
            priority: Priority::Medium,
        });
    }
    
    recommendations
}

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

// Mission briefing caching system for performance
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




// Mission briefing generation function - move to core
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeaponBehavior {
    pub preferred_range: f32,
    pub burst_fire: bool,
    pub requires_cover: bool,
    pub area_effect: bool,
    pub reload_retreat: bool,
}

impl WeaponBehavior {
    pub fn for_weapon_type(weapon_type: &WeaponType) -> Self {
        match weapon_type {
            WeaponType::Pistol => Self {
                preferred_range: 80.0,
                burst_fire: false,
                requires_cover: false,
                area_effect: false,
                reload_retreat: false,
            },
            WeaponType::Rifle => Self {
                preferred_range: 150.0,
                burst_fire: true,
                requires_cover: true,
                area_effect: false,
                reload_retreat: true,
            },
            WeaponType::Minigun => Self {
                preferred_range: 200.0,
                burst_fire: true,
                requires_cover: false,
                area_effect: true,
                reload_retreat: false,
            },
            WeaponType::Flamethrower => Self {
                preferred_range: 60.0,
                burst_fire: false,
                requires_cover: false,
                area_effect: true,
                reload_retreat: true,
            },
        }
    }
}

// WEAPON STATE / RELOAD
#[derive(Component)]
pub struct WeaponState {
    pub current_ammo: u32,
    pub max_ammo: u32,
    pub reload_time: f32,
    pub is_reloading: bool,
    pub reload_timer: f32,
}

impl Default for WeaponState {
    fn default() -> Self {
        Self {
            current_ammo: 30,
            max_ammo: 30,
            reload_time: 2.0, // 2 seconds base reload time
            is_reloading: false,
            reload_timer: 0.0,
        }
    }
}

impl WeaponState {
    pub fn new(weapon_type: &WeaponType) -> Self {
        let (max_ammo, reload_time) = match weapon_type {
            WeaponType::Pistol => (12, 1.5),
            WeaponType::Rifle => (30, 2.0),
            WeaponType::Minigun => (100, 4.0),
            WeaponType::Flamethrower => (50, 3.0),
        };
        
        Self {
            current_ammo: max_ammo,
            max_ammo,
            reload_time,
            is_reloading: false,
            reload_timer: 0.0,
        }
    }
    
    pub fn can_fire(&self) -> bool {
        self.current_ammo > 0 && !self.is_reloading
    }
    
    pub fn needs_reload(&self) -> bool {
        self.current_ammo == 0 || (self.current_ammo < self.max_ammo / 4) // Reload when < 25%
    }
    
    pub fn start_reload(&mut self) {
        if !self.is_reloading && self.current_ammo < self.max_ammo {
            self.is_reloading = true;
            self.reload_timer = self.reload_time;
        }
    }
    
    pub fn complete_reload(&mut self) {
        self.current_ammo = self.max_ammo;
        self.is_reloading = false;
        self.reload_timer = 0.0;
    }
    
    pub fn consume_ammo(&mut self) -> bool {
        if self.can_fire() {
            self.current_ammo = self.current_ammo.saturating_sub(1);
            true
        } else {
            false
        }
    }
    
    pub fn apply_attachment_modifiers(&mut self, weapon_config: &WeaponConfig) {
        let stats = weapon_config.calculate_total_stats();
        
        // Apply reload speed modifier (negative values = faster reload)
        let reload_modifier = 1.0 + (stats.reload_speed as f32 * -0.1); // Each point = 10% faster
        self.reload_time = (self.reload_time * reload_modifier).max(0.5); // Minimum 0.5s reload
        
        // Apply ammo capacity modifier
        let base_ammo = match weapon_config.base_weapon {
            WeaponType::Pistol => 12,
            WeaponType::Rifle => 30,
            WeaponType::Minigun => 100,
            WeaponType::Flamethrower => 50,
        };
        
        self.max_ammo = (base_ammo as f32 * (1.0 + stats.ammo_capacity as f32 * 0.2)) as u32; // Each point = 20% more ammo
        
        // If current ammo exceeds new max, don't reduce it (until next reload)
        if self.current_ammo == base_ammo && self.max_ammo > base_ammo {
            self.current_ammo = self.max_ammo; // Give immediate benefit if at full ammo
        }
    }
}

// === BETTER DESPAWNING ===
/*
2025-07-20T05:35:24.557541Z  WARN bevy_ecs::error::handler: Encountered an error in command `<bevy_ecs::system::commands::entity_command::despawn::{{closure}} as bevy_ecs::error::command_handling::CommandWithEntity<core::result::Result<(), bevy_ecs::world::error::EntityMutableFetchError>>>::with_entity::{{closure}}`: The entity with ID 106v1 does not exist (enable `track_location` feature for more details)
*/
pub trait SafeDespawn {
    fn safe_despawn(&mut self, entity: Entity);
}

impl SafeDespawn for Commands<'_, '_> {
    fn safe_despawn(&mut self, entity: Entity) {
        if let Ok(mut entity_commands) = self.get_entity(entity) {
            entity_commands.despawn();
        }
    }
}

// MORALE SYSTEM
#[derive(Component)]
pub struct Morale {
    pub current: f32,
    pub max: f32,
    pub panic_threshold: f32,
    pub recovery_rate: f32,
}

impl Default for Morale {
    fn default() -> Self {
        Self {
            current: 100.0,
            max: 100.0,
            panic_threshold: 30.0,
            recovery_rate: 5.0,
        }
    }
}

impl Morale {
    pub fn new(max: f32, panic_threshold: f32) -> Self {
        Self {
            current: max,
            max,
            panic_threshold,
            recovery_rate: max * 0.05,
        }
    }
    
    pub fn is_panicked(&self) -> bool {
        self.current <= self.panic_threshold
    }
    
    pub fn reduce(&mut self, amount: f32) {
        self.current = (self.current - amount).max(0.0);
    }
    
    pub fn recover(&mut self, delta_time: f32) {
        if !self.is_panicked() {
            self.current = (self.current + self.recovery_rate * delta_time).min(self.max);
        }
    }
}

// FLEEING
#[derive(Component)]
pub struct FleeTarget {
    pub destination: Vec2,
    pub flee_speed_multiplier: f32,
}

impl Default for FleeTarget {
    fn default() -> Self {
        Self {
            destination: Vec2::ZERO,
            flee_speed_multiplier: 1.5,
        }
    }
}

// POLICE
#[derive(Component)]
pub struct Police {
    pub response_level: u8,
}

#[derive(Resource, Default)]
pub struct PoliceResponse {
    pub heat_level: f32,
    pub next_spawn_timer: f32,
    pub civilian_casualties: u32,
    pub last_incident_pos: Option<Vec2>,
}

impl PoliceResponse {
    pub fn add_incident(&mut self, pos: Vec2, severity: f32) {
        self.heat_level += severity;
        self.last_incident_pos = Some(pos);
        self.next_spawn_timer = (10.0 - self.heat_level.min(8.0)).max(2.0);
    }
    
    pub fn should_spawn_police(&self) -> bool {
        self.heat_level >= 25.0 && self.civilian_casualties > 0
    }
    
    pub fn get_spawn_count(&self) -> u8 {
        match self.heat_level as u32 {
            0..=49 => 1,
            50..=99 => 2,
            100..=149 => 3,
            _ => 4,
        }
    }
}

// FORMATIONS

#[derive(Component)]
pub struct Formation {
    pub formation_type: FormationType,
    pub leader: Entity,
    pub positions: Vec<Vec2>,
    pub members: Vec<Entity>,
    pub spacing: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FormationType {
    Line,
    Wedge,
    Column,
    Box,
}

impl Formation {
    pub fn new(formation_type: FormationType, leader: Entity) -> Self {
        Self {
            formation_type,
            leader,
            positions: Vec::new(),
            members: vec![leader],
            spacing: 40.0,
        }
    }
    
    pub fn calculate_positions(&mut self, leader_pos: Vec2) {
        self.positions.clear();
        self.positions.push(leader_pos);
        let count = self.members.len();
        
        match self.formation_type {
            FormationType::Line => {
                for i in 1..count {
                    let offset = Vec2::new((i as f32 - (count as f32 - 1.0) / 2.0) * self.spacing, 0.0);
                    self.positions.push(leader_pos + offset);
                }
            },
            FormationType::Wedge => {
                for i in 1..count {
                    let side = if i % 2 == 1 { -1.0 } else { 1.0 };
                    let rank = (i + 1) / 2;
                    let offset = Vec2::new(side * rank as f32 * 28.0, -(rank as f32 * self.spacing));
                    self.positions.push(leader_pos + offset);
                }
            },
            FormationType::Column => {
                for i in 1..count {
                    self.positions.push(leader_pos + Vec2::new(0.0, -(i as f32 * self.spacing)));
                }
            },
            FormationType::Box => {
                if count >= 4 {
                    let h = self.spacing * 0.5;
                    self.positions.push(leader_pos + Vec2::new(-h, -h));
                    self.positions.push(leader_pos + Vec2::new(h, -h));
                    if count > 4 {
                        self.positions.push(leader_pos + Vec2::new(0.0, -self.spacing));
                    }
                }
            },
        }
    }
}

#[derive(Component)]
pub struct FormationMember {
    pub formation_entity: Entity,
    pub position_index: usize,
}

#[derive(Resource, Default)]
pub struct FormationState {
    pub active_formation: Option<Entity>,
}

// vehicle components

#[derive(Component)]
pub struct Vehicle {
    pub vehicle_type: VehicleType,
    pub armor: f32,
    pub cover_value: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum VehicleType {
    CivilianCar,
    PoliceCar,
    APC,
    VTOL,
    Tank,
}

impl Vehicle {
    pub fn new(vehicle_type: VehicleType) -> Self {
        let (armor, cover_value) = match vehicle_type {
            VehicleType::CivilianCar => (50.0, 30.0),
            VehicleType::PoliceCar => (80.0, 40.0),
            VehicleType::APC => (200.0, 60.0),
            VehicleType::VTOL => (120.0, 25.0),
            VehicleType::Tank => (300.0, 80.0),
        };
        
        Self {
            vehicle_type,
            armor,
            cover_value,
        }
    }
    
    pub fn max_health(&self) -> f32 {
        match self.vehicle_type {
            VehicleType::CivilianCar => 100.0,
            VehicleType::PoliceCar => 150.0,
            VehicleType::APC => 400.0,
            VehicleType::VTOL => 200.0,
            VehicleType::Tank => 600.0,
        }
    }
    
    pub fn explosion_radius(&self) -> f32 {
        match self.vehicle_type {
            VehicleType::CivilianCar => 60.0,
            VehicleType::PoliceCar => 70.0,
            VehicleType::APC => 100.0,
            VehicleType::VTOL => 120.0,
            VehicleType::Tank => 150.0,
        }
    }
    
    pub fn explosion_damage(&self) -> f32 {
        match self.vehicle_type {
            VehicleType::CivilianCar => 40.0,
            VehicleType::PoliceCar => 50.0,
            VehicleType::APC => 80.0,
            VehicleType::VTOL => 100.0,
            VehicleType::Tank => 120.0,
        }
    }
}

#[derive(Component)]
pub struct VehicleExplosion {
    pub radius: f32,
    pub damage: f32,
    pub duration: f32,
}

#[derive(Component)]
pub struct VehicleCover {
    pub cover_positions: Vec<Vec2>,
    pub occupied: Vec<bool>,
}


// DAY NIGHT CYCLE
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


// === GAME CONFIG SYSTEM ===

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