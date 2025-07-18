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
}

#[derive(Event)]
pub struct CombatEvent {
    pub attacker: Entity,
    pub target: Entity,
    pub damage: f32,
    pub hit: bool,
}