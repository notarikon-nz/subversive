// src/core/weapons.rs - Streamlined weapon system with external data
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::core::attachments::WeaponConfig;
use crate::core::components::*;
use crate::core::resources::*;

#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum WeaponType {
    Pistol,
    Rifle,
    Minigun,
    Flamethrower,
    GrenadeLauncher,
    RocketLauncher,
    LaserRifle,
    PlasmaGun,
    Shotgun,
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
    pub area_damage: Option<f32>,
    pub penetration: bool,
    pub energy_cost: Option<f32>, // For energy weapons
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
                area_damage: None,
                energy_cost: None,
                penetration: false,
            },
            WeaponType::Rifle => Self {
                preferred_range: 150.0,
                burst_fire: true,
                requires_cover: true,
                area_effect: false,
                reload_retreat: true,
                area_damage: None,
                energy_cost: None,
                penetration: true,
            },
            WeaponType::Minigun => Self {
                preferred_range: 200.0,
                burst_fire: true,
                requires_cover: false,
                area_effect: true,
                reload_retreat: false,
                area_damage: None,
                energy_cost: None,
                penetration: false,
            },
            WeaponType::Flamethrower => Self {
                preferred_range: 60.0,
                burst_fire: false,
                requires_cover: false,
                area_effect: true,
                reload_retreat: true,
                area_damage: Some(20.0), // Damage over time in area
                energy_cost: None,
                penetration: false,
            },
            WeaponType::GrenadeLauncher => Self {
                preferred_range: 200.0,
                burst_fire: false,
                requires_cover: true,
                area_effect: true,
                reload_retreat: true,
                area_damage: Some(80.0),
                penetration: false,
                energy_cost: None,
            },
            WeaponType::RocketLauncher => Self {
                preferred_range: 300.0,
                burst_fire: false,
                requires_cover: false,
                area_effect: true,
                area_damage: Some(120.0),
                reload_retreat: true,
                energy_cost: None,
                penetration: false,
            },
            WeaponType::LaserRifle => Self {
                preferred_range: 250.0,
                burst_fire: false,
                requires_cover: false,
                penetration: true,
                energy_cost: Some(10.0),
                reload_retreat: true,
                area_effect: false,
                area_damage: None,
            },
            WeaponType::PlasmaGun => Self {
                preferred_range: 500.0,
                burst_fire: false,
                requires_cover: false,
                penetration: true,
                reload_retreat: true,
                area_effect: true,
                area_damage: Some(60.0),
                energy_cost: Some(40.0),
            },
            WeaponType::Shotgun => Self {
                preferred_range: 75.0,
                burst_fire: false,
                requires_cover: false,
                penetration: true,
                reload_retreat: false,
                area_effect: true,
                area_damage: Some(10.0),
                energy_cost: None,
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
            WeaponType::Shotgun => (2, 2.0),
            WeaponType::Rifle => (30, 2.0),
            WeaponType::Minigun => (100, 4.0),
            WeaponType::Flamethrower => (50, 3.0),
            WeaponType::GrenadeLauncher => (1, 7.5),
            WeaponType::RocketLauncher => (1, 10.0),
            WeaponType::LaserRifle => (10, 5.0),
            WeaponType::PlasmaGun => (5, 5.0),
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
    
    pub fn reload_to_full(&mut self) {
        info!("Reloading weapon: {}/{} -> {}/{}", 
                 self.current_ammo, self.max_ammo, self.max_ammo, self.max_ammo);
        self.current_ammo = self.max_ammo;
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
            WeaponType::Shotgun => 2,
            WeaponType::Rifle => 30,
            WeaponType::Minigun => 100,
            WeaponType::Flamethrower => 50,
            WeaponType::GrenadeLauncher => 1,
            WeaponType::RocketLauncher => 1,
            WeaponType::LaserRifle => 10,
            WeaponType::PlasmaGun => 5,
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

pub fn enemy_weapon_update_system(
    mut enemy_query: Query<&mut WeaponState, With<Enemy>>,
    time: Res<Time>,
    game_mode: Res<GameMode>,
) {
    if game_mode.paused { return; }
    
    for mut weapon_state in enemy_query.iter_mut() {
        if weapon_state.is_reloading {
            weapon_state.reload_timer -= time.delta_secs();
            
            if weapon_state.reload_timer <= 0.0 {
                weapon_state.complete_reload();
                // println!("Enemy weapon reload completed: {}/{} ammo", weapon_state.current_ammo, weapon_state.max_ammo);
            }
        }
    }
}