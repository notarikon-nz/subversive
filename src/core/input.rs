// src/core/input.rs - Input definitions and targeting
use bevy::prelude::*;
use leafwing_input_manager::prelude::*;

// === INPUT ACTIONS ===
#[derive(Actionlike, PartialEq, Eq, Hash, Clone, Copy, Debug, Reflect)]
#[reflect(Hash, PartialEq)]
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

// === TARGETING SYSTEM ===
#[derive(Debug)]
pub enum TargetingMode {
    Neurovector { agent: Entity },
    Combat { agent: Entity },
}

// === SELECTION ===
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

#[derive(Resource, Default)]
pub struct SelectionState {
    pub selected: Vec<Entity>,
}