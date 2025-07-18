#[allow(dead_code)]

use bevy::prelude::*;
use crate::components::*;

#[derive(Event)]
pub struct AgentActionEvent {
    pub agent: Entity,
    pub action: AgentAction,
}

#[derive(Event)]
pub struct MissionEvent {
    pub event_type: MissionEventType,
}

#[derive(Event)]
pub struct AlertEvent {
    pub new_level: AlertLevel,
    pub source_position: Vec2,
    pub reason: AlertReason,
}

#[derive(Event)]
pub struct NeurovectorEvent {
    pub caster: Entity,
    pub target: Entity,
    pub success: bool,
}

#[derive(Event)]
pub struct ObjectiveEvent {
    pub objective: Entity,
    pub event_type: ObjectiveEventType,
}

#[derive(Debug, Clone)]
pub enum AgentAction {
    MoveTo(Vec2),
    Attack(Entity),
    Interact(Entity),
    UseNeurovector(Entity),
    Die,
    TakeDamage(f32),
    Heal(f32),
}

#[derive(Debug, Clone)]
pub enum MissionEventType {
    Started,
    Completed,
    Failed,
    TimeExpired,
    AllAgentsDead,
    ObjectiveCompleted(Entity),
}

#[derive(Debug, Clone)]
pub enum AlertReason {
    AgentSpotted,
    CombatNoise,
    BodyDiscovered,
    SecurityBreach,
    CivilianPanic,
}

#[derive(Debug, Clone)]
pub enum ObjectiveEventType {
    Completed,
    Failed,
    Updated,
}

#[derive(Event)]
pub struct InteractionEvent {
    pub agent: Entity,
    pub terminal: Entity,
    pub interaction_type: InteractionEventType,
}

#[derive(Event)]
pub struct InteractionCompleteEvent {
    pub agent: Entity,
    pub terminal: Entity,
    pub rewards: Vec<InteractionReward>,
}

#[derive(Event)]
pub struct DetectionEvent {
    pub detector: Entity,  // Enemy that spotted something
    pub target: Entity,    // Agent/civilian that was spotted
    pub detection_level: f32, // 0.0 to 1.0
    pub position: Vec2,    // Where the detection occurred
}

#[derive(Debug, Clone)]
pub enum InteractionEventType {
    StartInteraction,
    CancelInteraction,
    CompleteInteraction,
}


#[derive(Event)]
pub struct CombatEvent {
    pub attacker: Entity,
    pub target: Entity,
    pub damage: f32,
    pub hit: bool,
}

#[derive(Event)]
pub struct DeathEvent {
    pub entity: Entity,
    pub position: Vec2,
    pub entity_type: DeathEntityType,
}

#[derive(Debug, Clone)]
pub enum DeathEntityType {
    Agent,
    Civilian,
    Enemy,
}

