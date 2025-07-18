use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use std::collections::HashMap;

#[derive(Component)]
pub struct Agent {
    pub health: f32,
    pub max_health: f32,
    pub movement_speed: f32,
    pub cybernetics: Vec<CyberneticType>,
    pub skills: SkillMatrix,
    pub equipment: Vec<Equipment>,
    pub recovery_time: Option<f32>,
    pub experience: u32,
}

impl Default for Agent {
    fn default() -> Self {
        Self {
            health: 100.0,
            max_health: 100.0,
            movement_speed: 150.0,
            cybernetics: vec![CyberneticType::Neurovector],
            skills: SkillMatrix::default(),
            equipment: vec![],
            recovery_time: None,
            experience: 0,
        }
    }
}

#[derive(Component)]
pub struct InventoryUI {
    pub is_open: bool,
    pub selected_category: InventoryCategory,
    pub selected_item: Option<usize>,
}

impl Default for InventoryUI {
    fn default() -> Self {
        Self {
            is_open: false,
            selected_category: InventoryCategory::Weapons,
            selected_item: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InventoryCategory {
    Weapons,
    Tools,
    Cybernetics,
    AccessCards,
    SkillMatrices,
    Intel,
}

#[derive(Component)]
pub struct Civilian {
    pub health: f32,
    pub occupation: OccupationType,
    pub security_clearance: SecurityLevel,
    pub neurovector_target: bool,
    pub controlled_by: Option<Entity>,
    pub awareness_level: f32,
}

#[derive(Component)]
pub struct Enemy {
    pub health: f32,
    pub patrol_route: Vec<Vec2>,
    pub current_patrol_index: usize,
    pub alert_level: AlertLevel,
    pub detection_range: f32,
    pub last_known_target: Option<Vec2>,
}

#[derive(Component)]
pub struct MissionObjective {
    pub objective_type: ObjectiveType,
    pub is_primary: bool,
    pub completed: bool,
    pub target_entity: Option<Entity>,
    pub target_position: Option<Vec2>,
}

#[derive(Component)]
pub struct Selectable {
    pub selected: bool,
    pub selection_radius: f32,
}

#[derive(Component)]
pub struct Movement {
    pub target_position: Option<Vec2>,
    pub path: Vec<Vec2>,
    pub current_path_index: usize,
}

#[derive(Component)]
pub struct AgentVision {
    pub range: f32,
    pub angle: f32,
    pub direction: Vec2,
    pub can_see: Vec<Entity>,
    pub detection_buildup: f32,
}

#[derive(Component)]
pub struct NeurovectorCapability {
    pub range: f32,
    pub max_targets: u8,
    pub line_of_sight_required: bool,
    pub control_duration: f32,
    pub cooldown: f32,
    pub current_cooldown: f32,
    pub controlled_entities: Vec<Entity>,
}

impl Default for NeurovectorCapability {
    fn default() -> Self {
        Self {
            range: 200.0,
            max_targets: 3,
            line_of_sight_required: true,
            control_duration: 10.0,
            cooldown: 5.0,
            current_cooldown: 0.0,
            controlled_entities: vec![],
        }
    }
}

// Enums and supporting types
#[derive(Debug, Clone)]
pub enum CyberneticType {
    Neurovector,
    CombatEnhancer,
    StealthModule,
    TechInterface,
}

#[derive(Debug, Clone)]
pub enum SkillType {
    Combat(CombatSkill),
    Stealth(StealthSkill),
    Technical(TechSkill),
    Social(SocialSkill),
}

#[derive(Debug, Clone)]
pub enum CombatSkill {
    Firearms,
    Melee,
    Explosives,
}

#[derive(Debug, Clone)]
pub enum StealthSkill {
    Infiltration,
    Lockpicking,
    Hacking,
}

#[derive(Debug, Clone)]
pub enum TechSkill {
    Electronics,
    Cybernetics,
    VehicleOperation,
}

#[derive(Debug, Clone)]
pub enum SocialSkill {
    Persuasion,
    Intimidation,
    Deception,
}

#[derive(Debug, Clone)]
pub struct SkillMatrix {
    pub installed_skills: HashMap<SkillType, u8>, // skill level 0-100
    pub available_slots: u8,
    pub installation_time: f32,
}

impl Default for SkillMatrix {
    fn default() -> Self {
        Self {
            installed_skills: HashMap::new(),
            available_slots: 4,
            installation_time: 0.0,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Equipment {
    Weapon(WeaponType),
    Tool(ToolType),
    Armor(ArmorType),
}

#[derive(Debug, Clone)]
pub enum WeaponType {
    Pistol,
    Rifle,
    Minigun,
    Flamethrower,
}

#[derive(Debug, Clone)]
pub enum ToolType {
    Lockpick,
    Hacker,
    Scanner,
    MedKit,
}

#[derive(Debug, Clone)]
pub enum ArmorType {
    Light,
    Medium,
    Heavy,
}

#[derive(Debug, Clone)]
pub enum OccupationType {
    Scientist,
    Security,
    Executive,
    Technician,
    Civilian,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SecurityLevel {
    None,
    Low,
    Medium,
    High,
    Maximum,
}

#[derive(Debug, Clone, Copy)]
pub enum AlertLevel {
    Green,    // Normal operations
    Yellow,   // Suspicious activity
    Orange,   // Confirmed threat
    Red,      // Full alert
}

#[derive(Debug, Clone)]
pub enum ObjectiveType {
    Assassinate(Entity),
    Retrieve(Entity),
    Escort(Entity),
    Infiltrate(Vec2),
    Sabotage(Entity),
    Survive(f32), // duration in seconds
    AccessTerminal(Entity), // New objective type for terminal access
}

#[derive(Component)]
pub struct InteractableTerminal {
    pub terminal_type: TerminalType,
    pub priority_color: PriorityColor,
    pub access_requirements: Vec<AccessRequirement>,
    pub loot_table: Vec<InteractionReward>,
    pub interaction_range: f32,
    pub is_accessed: bool,
    pub access_time: f32, // Time required to interact
}

impl Default for InteractableTerminal {
    fn default() -> Self {
        Self {
            terminal_type: TerminalType::DataTerminal,
            priority_color: PriorityColor::Optional,
            access_requirements: vec![],
            loot_table: vec![InteractionReward::Currency(100)],
            interaction_range: 30.0,
            is_accessed: false,
            access_time: 2.0,
        }
    }
}

#[derive(Component)]
pub struct InteractionPrompt {
    pub target_terminal: Entity,
    pub interacting_agent: Entity,
    pub progress: f32,
    pub total_time: f32,
}

#[derive(Component)]
pub struct EquipmentInventory {
    pub weapons: Vec<WeaponType>,
    pub tools: Vec<ToolType>,
    pub cybernetics: Vec<CyberneticType>,
    pub access_cards: Vec<SecurityLevel>,
    pub currency: u32,
    pub skill_matrices: Vec<SkillType>,
    pub intel_documents: Vec<String>,
}

impl Default for EquipmentInventory {
    fn default() -> Self {
        Self {
            weapons: vec![],
            tools: vec![],
            cybernetics: vec![CyberneticType::Neurovector], // Start with neurovector
            access_cards: vec![SecurityLevel::None],
            currency: 100, // Starting money
            skill_matrices: vec![],
            intel_documents: vec![],
        }
    }
}

#[derive(Debug, Clone)]
pub enum TerminalType {
    DataTerminal,     // Basic info/lore
    SecurityTerminal, // Access cards, system control
    SupplyCache,      // Equipment and tools
    CyberneticNode,   // Skill matrices and cybernetics
    ObjectiveTerminal, // Mission-critical access
}

#[derive(Debug, Clone, Copy)]
pub enum PriorityColor {
    Critical,    // Red - mission objectives
    Secondary,   // Blue - valuable bonuses
    Optional,    // Green - lore/flavor
}

#[derive(Debug, Clone)]
pub enum AccessRequirement {
    SecurityClearance(SecurityLevel),
    Tool(ToolType),
    Cybernetic(CyberneticType),
    NoGuards, // Must not be spotted
}

#[derive(Debug, Clone)]
pub enum InteractionReward {
    Equipment(Equipment),
    SkillMatrix(SkillType),
    Currency(u32),
    Intel(String), // Lore document content
    AccessCard(SecurityLevel),
    ObjectiveProgress, // Advances mission objectives
}

