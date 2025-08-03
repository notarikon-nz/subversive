// src/core/hackable.rs - Infrastructure hacking system
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use crate::core::*;

// === DEVICE CONFIG DATA ===
// Move static configuration to const data
const DEVICE_CONFIGS: &[(DeviceType, u8, f32, &[HackEffect], DeviceFunction)] = &[
    (DeviceType::Camera, 2, 3.0, &[HackEffect::Disable], DeviceFunction::Surveillance),
    (DeviceType::Turret, 4, 8.0, &[HackEffect::Disable, HackEffect::TakeControl], DeviceFunction::Defense),
    (DeviceType::Drone, 3, 5.0, &[HackEffect::Disable, HackEffect::TakeControl], DeviceFunction::Surveillance),
    (DeviceType::Door, 1, 2.0, &[HackEffect::Disable], DeviceFunction::Transport),
    (DeviceType::Elevator, 2, 4.0, &[HackEffect::Disable], DeviceFunction::Transport),
    (DeviceType::Vehicle, 3, 6.0, &[HackEffect::Disable, HackEffect::TakeControl], DeviceFunction::Transport),
    (DeviceType::PowerStation, 5, 12.0, &[HackEffect::PowerCut, HackEffect::Overload], DeviceFunction::PowerDistribution),
    (DeviceType::StreetLight, 1, 1.0, &[HackEffect::Disable], DeviceFunction::Lighting),
    (DeviceType::TrafficLight, 1, 2.0, &[HackEffect::Disable], DeviceFunction::TrafficControl),
    (DeviceType::Terminal, 2, 4.0, &[HackEffect::ExtractData], DeviceFunction::DataStorage),
    (DeviceType::SecuritySystem, 4, 10.0, &[HackEffect::Disable, HackEffect::PlantVirus], DeviceFunction::Security),
    (DeviceType::AlarmPanel, 2, 3.0, &[HackEffect::Disable], DeviceFunction::Security),
    (DeviceType::ATM, 3, 6.0, &[HackEffect::ExtractData], DeviceFunction::DataStorage),
    (DeviceType::Billboard, 2, 3.0, &[HackEffect::TakeControl], DeviceFunction::Surveillance),    
];

// === COMPONENTS ===
#[derive(Component, Clone, Serialize, Deserialize)]
pub struct Hackable {
    pub device_type: DeviceType,
    pub security_level: u8,
    pub hack_time: f32,
    pub requires_tool: Option<HackTool>,
    pub is_hacked: bool,
    pub network_id: Option<String>,
    pub disabled_duration: f32,
    pub hack_effects: Vec<HackEffect>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum DeviceType {
    Camera, Turret, Drone, Door, Elevator, Vehicle,
    PowerStation, StreetLight, TrafficLight, Terminal,
    SecuritySystem, AlarmPanel,
    ATM, Billboard, // 0.2.10
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HackTool {
    BasicHacker, AdvancedHacker, VirusKit, PhysicalAccess,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HackEffect {
    Disable, TakeControl, ExtractData, PlantVirus, Overload, PowerCut,
}

#[derive(Debug, Clone, Copy)]
pub enum DeviceFunction {
    Surveillance, Defense, Transport, Lighting,
    TrafficControl, DataStorage, PowerDistribution, Security,
}

impl Hackable {
    pub fn new(device_type: DeviceType) -> Self {
        let (_, security_level, hack_time, effects, _) = DEVICE_CONFIGS
            .iter()
            .find(|(dt, _, _, _, _)| *dt == device_type)
            .copied()
            .unwrap();

        Self {
            device_type,
            security_level,
            hack_time,
            requires_tool: if security_level >= 3 { 
                Some(HackTool::AdvancedHacker) 
            } else { 
                Some(HackTool::BasicHacker) 
            },
            is_hacked: false,
            network_id: None,
            disabled_duration: 30.0,
            hack_effects: effects.to_vec(),
        }
    }
    
    pub fn with_network(mut self, network_id: String) -> Self {
        self.network_id = Some(network_id);
        self
    }
    
    pub fn permanent_hack(mut self) -> Self {
        self.disabled_duration = -1.0;
        self
    }
    
    pub fn high_security(mut self) -> Self {
        self.security_level = 5;
        self.hack_time *= 2.0;
        self.requires_tool = Some(HackTool::AdvancedHacker);
        self
    }
    
    pub fn quick_hack(mut self) -> Self {
        self.security_level = 1;
        self.hack_time = 1.0;
        self.requires_tool = Some(HackTool::BasicHacker);
        self
    }
}

#[derive(Component)]
pub struct DeviceState {
    pub powered: bool,
    pub operational: bool,
    pub hack_timer: f32,
    pub original_function: DeviceFunction,
    pub current_function: DeviceFunction,
}

impl DeviceState {
    pub fn new(device_type: DeviceType) -> Self {
        let function = get_device_function(device_type);
        Self {
            powered: true,
            operational: true,
            hack_timer: 0.0,
            original_function: function,
            current_function: function,
        }
    }
}

fn get_device_function(device_type: DeviceType) -> DeviceFunction {
    DEVICE_CONFIGS
        .iter()
        .find(|(dt, _, _, _, _)| *dt == device_type)
        .map(|(_, _, _, _, func)| *func)
        .unwrap_or(DeviceFunction::Surveillance)
}

// === POWER GRID ===
#[derive(Resource, Default)]
pub struct PowerGrid {
    pub networks: HashMap<String, PowerNetwork>,
}

#[derive(Clone)]
pub struct PowerNetwork {
    pub id: String,
    pub powered: bool,
    pub connected_devices: HashSet<Entity>,
    pub power_sources: HashSet<Entity>,
}

impl PowerNetwork {
    pub fn new(id: String) -> Self {
        Self {
            id,
            powered: true,
            connected_devices: HashSet::new(),
            power_sources: HashSet::new(),
        }
    }
}

// === EVENTS ===
#[derive(Event)]
pub struct HackAttemptEvent {
    pub agent: Entity,
    pub target: Entity,
    pub tool_used: HackTool,
}

#[derive(Event)]
pub struct HackCompletedEvent {
    pub agent: Entity,
    pub target: Entity,
    pub device_type: DeviceType,
    pub effects: Vec<HackEffect>,
}

#[derive(Event)]
pub struct PowerGridEvent {
    pub network_id: String,
    pub powered: bool,
    pub source: Entity,
}


pub fn hack_recovery_system(
    mut hackable_query: Query<(Entity, &mut Hackable, &mut DeviceState)>,
    mut power_events: EventWriter<PowerGridEvent>,
    time: Res<Time>,
) {
    for (entity, mut hackable, mut device_state) in hackable_query.iter_mut() {
        if !hackable.is_hacked || hackable.disabled_duration < 0.0 {
            continue;
        }
        
        device_state.hack_timer -= time.delta_secs();
        
        if device_state.hack_timer <= 0.0 {
            hackable.is_hacked = false;
            device_state.operational = true;
            device_state.powered = true;
            device_state.current_function = device_state.original_function;
            
            // Restore power grid if needed
            if hackable.device_type == DeviceType::PowerStation {
                if let Some(network_id) = &hackable.network_id {
                    power_events.write(PowerGridEvent {
                        network_id: network_id.clone(),
                        powered: true,
                        source: entity,
                    });
                }
            }
        }
    }
}

pub fn power_grid_system(
    mut power_events: EventReader<PowerGridEvent>,
    mut power_grid: ResMut<PowerGrid>,
    mut device_query: Query<(&Hackable, &mut DeviceState)>,
) {
    for event in power_events.read() {
        let Some(network) = power_grid.networks.get_mut(&event.network_id) else {
            continue;
        };
        
        network.powered = event.powered;
        
        // Update all connected devices
        for &device_entity in &network.connected_devices.clone() {
            if let Ok((hackable, mut device_state)) = device_query.get_mut(device_entity) {
                if hackable.network_id.as_ref() == Some(&event.network_id) {
                    device_state.powered = event.powered;
                    if !event.powered {
                        device_state.operational = false;
                    }
                }
            }
        }
    }
}

// === HELPER FUNCTIONS ===
pub fn make_hackable(commands: &mut Commands, entity: Entity, device_type: DeviceType) {
    commands.entity(entity)
        .insert(Hackable::new(device_type))
        .insert(DeviceState::new(device_type));
}

pub fn make_hackable_networked(
    commands: &mut Commands, 
    entity: Entity, 
    device_type: DeviceType, 
    network_id: String,
    power_grid: &mut ResMut<PowerGrid>
) {
    let hackable = Hackable::new(device_type).with_network(network_id.clone());
    
    power_grid.networks.entry(network_id.clone())
        .or_insert_with(|| PowerNetwork::new(network_id))
        .connected_devices.insert(entity);
    
    commands.entity(entity)
        .insert(hackable)
        .insert(DeviceState::new(device_type));
}



pub fn setup_hackable_turret(commands: &mut Commands, entity: Entity) {
    commands.entity(entity)
        .insert(Hackable::new(DeviceType::Turret).high_security())
        .insert(DeviceState::new(DeviceType::Turret));
}

pub fn setup_hackable_door(commands: &mut Commands, entity: Entity) {
    commands.entity(entity)
        .insert(Hackable::new(DeviceType::Door).quick_hack())
        .insert(DeviceState::new(DeviceType::Door));
}

