// src/core/components.rs - Core entity components
use bevy::prelude::*;
use crate::core::{WeaponConfig};
use serde::{Deserialize, Serialize};
use crate::systems::access_control::{CardType};

// === BASIC ENTITY COMPONENTS ===
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

impl Agent {
    pub fn with_scanner(mut self, scan_level: u8) -> Self {
        // This would be added to your agent creation
        self
    }
}


/// Component to mark projectile impacts for decal creation
#[derive(Component)]
pub struct ProjectileImpact;

// BETTER (?) DESPAWN HANDLING
#[derive(Component)]
pub struct MarkedForDespawn;

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

// === TERMINAL SYSTEM ===
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

// === INVENTORY SYSTEM ===
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OriginalInventoryItem {
    AccessCard { level: u8, card_type: CardType },
    Keycard { access_level: u8, facility_id: String },
}

#[derive(Component, Default)]
pub struct Inventory {
    pub weapons: Vec<WeaponConfig>,
    pub tools: Vec<crate::core::ToolType>,
    pub currency: u32,
    pub equipped_weapon: Option<WeaponConfig>,
    pub equipped_tools: Vec<crate::core::ToolType>,
    pub cybernetics: Vec<crate::core::CyberneticType>,
    pub intel_documents: Vec<String>,
    pub items: Vec<OriginalInventoryItem>,
}

#[derive(Component)]
pub struct InventoryVersion(pub u32);

impl Inventory {
    pub fn add_weapon(&mut self, weapon: crate::core::WeaponType) {
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

    pub fn add_currency(&mut self, amount: u32) {
        self.currency += amount;
    }

    pub fn add_tool(&mut self, tool: crate::core::ToolType) {
        if self.equipped_tools.len() < 2 {
            self.equipped_tools.push(tool.clone());
        }
        self.tools.push(tool);
    }

    pub fn add_cybernetic(&mut self, cybernetic: crate::core::CyberneticType) {
        self.cybernetics.push(cybernetic);
    }

    pub fn add_intel(&mut self, document: String) {
        self.intel_documents.push(document);
    }

    pub fn add_access_card(&mut self, level: u8, card_type: CardType) {
        self.items.push(OriginalInventoryItem::AccessCard { level, card_type });
        info!("Added {:?} access card (level {})", card_type, level);
    }

    pub fn has_access_card(&self, required_level: u8) -> bool {
        self.items.iter().any(|item| {
            match item {
                OriginalInventoryItem::AccessCard { level, .. } => *level >= required_level,
                OriginalInventoryItem::Keycard { access_level, .. } => *access_level >= required_level,
            }
        })
    }

    pub fn get_highest_access_level(&self) -> u8 {
        self.items.iter().fold(0, |max_level, item| {
            match item {
                OriginalInventoryItem::AccessCard { level, .. } => max_level.max(*level),
                OriginalInventoryItem::Keycard { access_level, .. } => max_level.max(*access_level),
            }
        })
    }

    pub fn remove_access_card(&mut self, required_level: u8) -> bool {
        if let Some(pos) = self.items.iter().position(|item| {
            match item {
                OriginalInventoryItem::AccessCard { level, .. } => *level >= required_level,
                OriginalInventoryItem::Keycard { access_level, .. } => *access_level >= required_level,
            }
        }) {
            self.items.remove(pos);
            true
        } else {
            false
        }
    }
}


// Remove duplicate enum definitions - these are defined elsewhere
#[derive(Component)]
pub struct Objective;