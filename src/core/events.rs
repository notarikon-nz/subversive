use bevy::prelude::*;

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
    pub position: Vec2,
    pub alert_level: u8,
    pub source: AlertSource,
}

#[derive(Debug, Clone)]
pub enum AlertSource {
    Gunshot,
    SpottedAgent,
    MissingPatrol,
    Alarm,
}
