// src/core/entities.rs - Advanced entity components and systems
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

// === MORALE SYSTEM ===
#[derive(Component)]
pub struct Morale {
    pub current: f32,
    pub max: f32,
    pub panic_threshold: f32,
    pub recovery_rate: f32,
}

impl Default for Morale {
    fn default() -> Self {
        Self {
            current: 100.0,
            max: 100.0,
            panic_threshold: 30.0,
            recovery_rate: 5.0,
        }
    }
}

impl Morale {
    pub fn new(max: f32, panic_threshold: f32) -> Self {
        Self {
            current: max,
            max,
            panic_threshold,
            recovery_rate: max * 0.05,
        }
    }
    
    pub fn is_panicked(&self) -> bool {
        self.current <= self.panic_threshold
    }
    
    pub fn reduce(&mut self, amount: f32) {
        self.current = (self.current - amount).max(0.0);
    }
    
    pub fn recover(&mut self, delta_time: f32) {
        if !self.is_panicked() {
            self.current = (self.current + self.recovery_rate * delta_time).min(self.max);
        }
    }
}

// === FLEEING ===
#[derive(Component)]
pub struct FleeTarget {
    pub destination: Vec2,
    pub flee_speed_multiplier: f32,
}

impl Default for FleeTarget {
    fn default() -> Self {
        Self {
            destination: Vec2::ZERO,
            flee_speed_multiplier: 1.5,
        }
    }
}

// === POLICE ===
#[derive(Component)]
pub struct Police {
    pub response_level: u8,
}

#[derive(Resource, Default)]
pub struct PoliceResponse {
    pub heat_level: f32,
    pub next_spawn_timer: f32,
    pub civilian_casualties: u32,
    pub last_incident_pos: Option<Vec2>,
}

impl PoliceResponse {
    pub fn add_incident(&mut self, pos: Vec2, severity: f32) {
        self.heat_level += severity;
        self.last_incident_pos = Some(pos);
        self.next_spawn_timer = (10.0 - self.heat_level.min(8.0)).max(2.0);
    }
    
    pub fn should_spawn_police(&self) -> bool {
        self.heat_level >= 25.0 && self.civilian_casualties > 0
    }
    
    pub fn get_spawn_count(&self) -> u8 {
        match self.heat_level as u32 {
            0..=49 => 1,
            50..=99 => 2,
            100..=149 => 3,
            _ => 4,
        }
    }
}

// === FORMATIONS ===
#[derive(Component)]
pub struct Formation {
    pub formation_type: FormationType,
    pub leader: Entity,
    pub positions: Vec<Vec2>,
    pub members: Vec<Entity>,
    pub spacing: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FormationType {
    Line,
    Wedge,
    Column,
    Box,
}

impl Formation {
    pub fn new(formation_type: FormationType, leader: Entity) -> Self {
        Self {
            formation_type,
            leader,
            positions: Vec::new(),
            members: vec![leader],
            spacing: 40.0,
        }
    }
    
    pub fn calculate_positions(&mut self, leader_pos: Vec2) {
        self.positions.clear();
        self.positions.push(leader_pos);
        let count = self.members.len();
        
        match self.formation_type {
            FormationType::Line => {
                for i in 1..count {
                    let offset = Vec2::new((i as f32 - (count as f32 - 1.0) / 2.0) * self.spacing, 0.0);
                    self.positions.push(leader_pos + offset);
                }
            },
            FormationType::Wedge => {
                for i in 1..count {
                    let side = if i % 2 == 1 { -1.0 } else { 1.0 };
                    let rank = (i + 1) / 2;
                    let offset = Vec2::new(side * rank as f32 * 28.0, -(rank as f32 * self.spacing));
                    self.positions.push(leader_pos + offset);
                }
            },
            FormationType::Column => {
                for i in 1..count {
                    self.positions.push(leader_pos + Vec2::new(0.0, -(i as f32 * self.spacing)));
                }
            },
            FormationType::Box => {
                if count >= 4 {
                    let h = self.spacing * 0.5;
                    self.positions.push(leader_pos + Vec2::new(-h, -h));
                    self.positions.push(leader_pos + Vec2::new(h, -h));
                    if count > 4 {
                        self.positions.push(leader_pos + Vec2::new(0.0, -self.spacing));
                    }
                }
            },
        }
    }
}

#[derive(Component)]
pub struct FormationMember {
    pub formation_entity: Entity,
    pub position_index: usize,
}

#[derive(Resource, Default)]
pub struct FormationState {
    pub active_formation: Option<Entity>,
}

// === VEHICLES ===
#[derive(Component)]
pub struct Vehicle {
    pub vehicle_type: VehicleType,
    pub armor: f32,
    pub cover_value: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum VehicleType {
    CivilianCar,
    PoliceCar,
    APC,
    VTOL,
    Tank,
}

impl Vehicle {
    pub fn new(vehicle_type: VehicleType) -> Self {
        let (armor, cover_value) = match vehicle_type {
            VehicleType::CivilianCar => (50.0, 30.0),
            VehicleType::PoliceCar => (80.0, 40.0),
            VehicleType::APC => (200.0, 60.0),
            VehicleType::VTOL => (120.0, 25.0),
            VehicleType::Tank => (300.0, 80.0),
        };
        
        Self {
            vehicle_type,
            armor,
            cover_value,
        }
    }
    
    pub fn max_health(&self) -> f32 {
        match self.vehicle_type {
            VehicleType::CivilianCar => 100.0,
            VehicleType::PoliceCar => 150.0,
            VehicleType::APC => 400.0,
            VehicleType::VTOL => 200.0,
            VehicleType::Tank => 600.0,
        }
    }
    
    pub fn explosion_radius(&self) -> f32 {
        match self.vehicle_type {
            VehicleType::CivilianCar => 60.0,
            VehicleType::PoliceCar => 70.0,
            VehicleType::APC => 100.0,
            VehicleType::VTOL => 120.0,
            VehicleType::Tank => 150.0,
        }
    }
    
    pub fn explosion_damage(&self) -> f32 {
        match self.vehicle_type {
            VehicleType::CivilianCar => 40.0,
            VehicleType::PoliceCar => 50.0,
            VehicleType::APC => 80.0,
            VehicleType::VTOL => 100.0,
            VehicleType::Tank => 120.0,
        }
    }
}

#[derive(Component)]
pub struct VehicleExplosion {
    pub radius: f32,
    pub damage: f32,
    pub duration: f32,
}

#[derive(Component)]
pub struct VehicleCover {
    pub cover_positions: Vec<Vec2>,
    pub occupied: Vec<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum CyberneticType {
    Neurovector,
    CombatEnhancer,
    StealthModule,
    TechInterface,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ToolType {
    Hacker,
    Scanner,
    Lockpick,
    MedKit,
}