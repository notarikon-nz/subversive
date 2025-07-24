// src/core/weapons.rs - Streamlined weapon system with external data
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::core::attachments::WeaponConfig;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum WeaponType {
    Pistol,
    Rifle,
    Minigun,
    Flamethrower,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeaponData {
    pub name: String,
    pub max_ammo: u32,
    pub reload_time: f32,
    pub damage: f32,
    pub behavior: WeaponBehavior,
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

#[derive(Resource, Default, Deserialize)]
pub struct WeaponDatabase {
    pub weapons: HashMap<WeaponType, WeaponData>,
}

impl WeaponDatabase {
    pub fn load() -> Self {
        match std::fs::read_to_string("data/weapons.json") {
            Ok(content) => {
                match serde_json::from_str::<WeaponDatabase>(&content) {
                    Ok(db) => db,
                    Err(e) => {
                        error!("Failed to parse weapons.json: {}", e);
                        Self::fallback()
                    }
                }
            },
            Err(_) => {
                warn!("weapons.json not found, using fallback data");
                Self::fallback()
            }
        }
    }
    
    fn fallback() -> Self {
        let weapons = HashMap::new();
        Self { weapons }
    }
    
    pub fn get(&self, weapon_type: &WeaponType) -> Option<&WeaponData> {
        self.weapons.get(weapon_type)
    }
}

#[derive(Component)]
pub struct WeaponState {
    pub current_ammo: u32,
    pub max_ammo: u32,
    pub reload_time: f32,
    pub is_reloading: bool,
    pub reload_timer: f32,
}

impl WeaponState {
    pub fn new(weapon_data: &WeaponData) -> Self {
        Self {
            current_ammo: weapon_data.max_ammo,
            max_ammo: weapon_data.max_ammo,
            reload_time: weapon_data.reload_time,
            is_reloading: false,
            reload_timer: 0.0,
        }
    }
    
    pub fn new_from_type(weapon_type: &WeaponType) -> Self {
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
        self.current_ammo < self.max_ammo / 4
    }
    
    pub fn start_reload(&mut self) {
        if self.current_ammo < self.max_ammo {
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
        let stats = weapon_config.stats();
        
        // Apply reload speed modifier (negative values = faster reload)
        let reload_modifier = 1.0 + (stats.reload_speed as f32 * -0.1);
        self.reload_time = (self.reload_time * reload_modifier).max(0.5);
        
        // Apply ammo capacity modifier
        let base_ammo = match weapon_config.base_weapon {
            WeaponType::Pistol => 12,
            WeaponType::Rifle => 30,
            WeaponType::Minigun => 100,
            WeaponType::Flamethrower => 50,
        };
        
        self.max_ammo = (base_ammo as f32 * (1.0 + stats.ammo_capacity as f32 * 0.2)) as u32;
        
        if self.current_ammo == base_ammo && self.max_ammo > base_ammo {
            self.current_ammo = self.max_ammo;
        }
    }
}

impl Default for WeaponState {
    fn default() -> Self {
        Self::new_from_type(&WeaponType::Pistol)
    }
}