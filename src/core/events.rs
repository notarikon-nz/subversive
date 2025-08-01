use bevy::prelude::*;
use crate::core::*;

#[derive(Event)]
pub struct ActionEvent {
    pub entity: Entity,
    pub action: Action,
}

#[derive(Debug, Clone)]
pub enum Action {
    MoveTo(Vec2),
    Attack(Entity),
    TakeDamage(f32),
    NeurovectorControl { target: Entity },
    InteractWith(Entity),
    Reload,
    // NEW: Advanced actions
    UseMedKit,
    ThrowGrenade { target_pos: Vec2 },
    ActivateAlarm { panel_pos: Vec2 },
    PickupWeapon,
    MaintainDistance,
    AreaDenial { weapon_type: WeaponType },
    SuppressionFire { weapon_type: WeaponType },    
    SelectAgent(usize),
    CenterCameraOnAgent(usize),

    // 0.2.12
    InteractWithScientist(Entity),
    RecruitScientist(Entity), 
    KidnapScientist(Entity),
    AssignScientistToProject { scientist: Entity, project: String },
    ChangeResearchPriority { project: String, priority: ResearchPriority },
    CancelResearch(String),
    UsePrototype(String), 
}

#[derive(Event)]
pub struct CombatEvent {
    pub attacker: Entity,
    pub target: Entity,
    pub damage: f32,
    pub hit: bool,
}

#[derive(Event)]
pub struct AlertEvent {
    pub alerter: Entity,  // FIXED: Add alerter field that GOAP system expects
    pub position: Vec2,
    pub alert_level: u8,
    pub source: AlertSource,
    pub alert_type: AlertType, // FIXED: Add alert_type field for compatibility
}

#[derive(Debug, Clone)]
pub enum AlertSource {
    Gunshot,
    SpottedAgent,
    MissingPatrol,
    Alarm,
    Grenade, // NEW
    CivilianReport,
}

#[derive(Debug, Clone)]
pub enum AlertType {
    CallForHelp,
    GunshotHeard,
    EnemySpotted,
    TrafficIncident,
    VehicularAssault,
    ConvoyAttacked,    
}

// NEW: Specific events for advanced actions
#[derive(Event)]
pub struct HealEvent {
    pub entity: Entity,
    pub amount: f32,
}

#[derive(Event)]
pub struct GrenadeEvent {
    pub thrower: Entity,
    pub target_pos: Vec2,
    pub explosion_radius: f32,
    pub damage: f32,
}

#[derive(Event)]
pub struct AlarmActivatedEvent {
    pub activator: Entity,
    pub panel_pos: Vec2,
}

#[derive(Event)]
pub struct DamageTextEvent {
    pub position: Vec2,
    pub damage: f32,
}


// 0.2.12

#[derive(Event)]
pub struct ScientistRecruitmentEvent {
    pub scientist: Entity,
    pub recruitment_type: RecruitmentType,
}

#[derive(Debug)]
pub enum RecruitmentType {
    Negotiated,
    Bribed,
    Kidnapped,
}

#[derive(Event)]
pub struct ResearchCompletedEvent {
    pub project_id: String,
    pub assigned_scientist: Option<Entity>,
    pub completion_time: f32,
}

#[derive(Event)]
pub struct ResearchSabotageEvent {
    pub target_project: String,
    pub sabotage_type: SabotageType,
}

#[derive(Debug)]
pub enum SabotageType {
    Delayed,
    DataCorrupted,
    DataStolen,
}