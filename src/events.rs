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