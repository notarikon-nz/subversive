// src/core/hackable.rs - Infrastructure hacking system
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use crate::core::*;

// === HACKABLE COMPONENT ===
#[derive(Component, Clone, Serialize, Deserialize)]
pub struct Hackable {
    pub device_type: DeviceType,
    pub security_level: u8, // 1-5, higher = harder to hack
    pub hack_time: f32,     // Seconds to complete hack
    pub requires_tool: Option<HackTool>,
    pub is_hacked: bool,
    pub network_id: Option<String>, // For power grid connections
    pub disabled_duration: f32, // How long device stays hacked (-1 = permanent)
    pub hack_effects: Vec<HackEffect>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DeviceType {
    Camera,
    Turret,
    Drone,
    Door,
    Elevator,
    Vehicle,
    PowerStation,
    StreetLight,
    TrafficLight,
    Terminal,
    SecuritySystem,
    AlarmPanel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HackTool {
    BasicHacker,
    AdvancedHacker,
    VirusKit,
    PhysicalAccess,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HackEffect {
    Disable,
    TakeControl,
    ExtractData,
    PlantVirus,
    Overload,
    PowerCut,
}

impl Hackable {
    pub fn new(device_type: DeviceType) -> Self {
        let (security_level, hack_time, effects) = match device_type {
            DeviceType::Camera => (2, 3.0, vec![HackEffect::Disable]),
            DeviceType::Turret => (4, 8.0, vec![HackEffect::Disable, HackEffect::TakeControl]),
            DeviceType::Drone => (3, 5.0, vec![HackEffect::Disable, HackEffect::TakeControl]),
            DeviceType::Door => (1, 2.0, vec![HackEffect::Disable]),
            DeviceType::Elevator => (2, 4.0, vec![HackEffect::Disable]),
            DeviceType::Vehicle => (3, 6.0, vec![HackEffect::Disable, HackEffect::TakeControl]),
            DeviceType::PowerStation => (5, 12.0, vec![HackEffect::PowerCut, HackEffect::Overload]),
            DeviceType::StreetLight => (1, 1.0, vec![HackEffect::Disable]),
            DeviceType::TrafficLight => (1, 2.0, vec![HackEffect::Disable]),
            DeviceType::Terminal => (2, 4.0, vec![HackEffect::ExtractData]),
            DeviceType::SecuritySystem => (4, 10.0, vec![HackEffect::Disable, HackEffect::PlantVirus]),
            DeviceType::AlarmPanel => (2, 3.0, vec![HackEffect::Disable]),
        };
        
        Self {
            device_type,
            security_level,
            hack_time,
            requires_tool: if security_level >= 3 { Some(HackTool::AdvancedHacker) } else { Some(HackTool::BasicHacker) },
            is_hacked: false,
            network_id: None,
            disabled_duration: 30.0, // Default 30 seconds
            hack_effects: effects,
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

// === DEVICE STATES ===
#[derive(Component)]
pub struct DeviceState {
    pub powered: bool,
    pub operational: bool,
    pub hack_timer: f32,
    pub original_function: DeviceFunction,
    pub current_function: DeviceFunction,
}

#[derive(Debug, Clone)]
pub enum DeviceFunction {
    Surveillance,
    Defense,
    Transport,
    Lighting,
    TrafficControl,
    DataStorage,
    PowerDistribution,
    Security,
}

impl Default for DeviceState {
    fn default() -> Self {
        Self {
            powered: true,
            operational: true,
            hack_timer: 0.0,
            original_function: DeviceFunction::Surveillance,
            current_function: DeviceFunction::Surveillance,
        }
    }
}

// === POWER GRID SYSTEM ===
#[derive(Resource, Default)]
pub struct PowerGrid {
    pub networks: std::collections::HashMap<String, PowerNetwork>,
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

// === HACKING EVENTS ===
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

// === HACKING SYSTEM ===
pub fn hacking_system(
    mut hack_attempts: EventReader<HackAttemptEvent>,
    mut hack_completed: EventWriter<HackCompletedEvent>,
    mut hackable_query: Query<(&mut Hackable, &mut DeviceState)>,
    mut audio_events: EventWriter<AudioEvent>,
    agent_inventory: Query<&Inventory, With<Agent>>,
    time: Res<Time>,
) {
    for event in hack_attempts.read() {
        if let Ok((mut hackable, mut device_state)) = hackable_query.get_mut(event.target) {
            // Check if agent has required tool
            if let Ok(inventory) = agent_inventory.get(event.agent) {
                let has_tool = match &hackable.requires_tool {
                    Some(required_tool) => inventory.equipped_tools.iter().any(|tool| {
                        matches!((tool, required_tool), 
                            (ToolType::Hacker, HackTool::BasicHacker) |
                            (ToolType::Hacker, HackTool::AdvancedHacker)
                        )
                    }),
                    None => true,
                };
                
                if !has_tool {
                    info!("Hack failed: Missing required tool");
                    continue;
                }
            }
            
            if hackable.is_hacked {
                info!("Device already hacked");
                continue;
            }
            
            // Start hack process
            device_state.hack_timer = hackable.hack_time;
            hackable.is_hacked = true;
            device_state.operational = false;
            
            // Apply hack effects
            for effect in &hackable.hack_effects {
                apply_hack_effect(effect, &mut device_state, &hackable.device_type);
            }
            
            // Send completion event
            hack_completed.write(HackCompletedEvent {
                agent: event.agent,
                target: event.target,
                device_type: hackable.device_type.clone(),
                effects: hackable.hack_effects.clone(),
            });
            
            // Play hack sound
            audio_events.write(AudioEvent {
                sound: AudioType::TerminalAccess,
                volume: 0.5,
            });
            
            info!("Hacked {:?} (Security Level: {})", hackable.device_type, hackable.security_level);
        }
    }
}

fn apply_hack_effect(effect: &HackEffect, device_state: &mut DeviceState, device_type: &DeviceType) {
    match effect {
        HackEffect::Disable => {
            device_state.operational = false;
        },
        HackEffect::TakeControl => {
            device_state.current_function = match device_type {
                DeviceType::Turret => DeviceFunction::Defense, // Now under player control
                DeviceType::Drone => DeviceFunction::Surveillance, // Player surveillance
                DeviceType::Vehicle => DeviceFunction::Transport, // Player transport
                _ => device_state.original_function.clone(),
            };
        },
        HackEffect::PowerCut => {
            device_state.powered = false;
        },
        _ => {} // Other effects handled elsewhere
    }
}

// === HACK RECOVERY SYSTEM ===
pub fn hack_recovery_system(
    mut hackable_query: Query<(Entity, &mut Hackable, &mut DeviceState)>,
    mut power_events: EventWriter<PowerGridEvent>,
    time: Res<Time>,
) {
    for (entity, mut hackable, mut device_state) in hackable_query.iter_mut() {
        if !hackable.is_hacked || hackable.disabled_duration < 0.0 {
            continue; // Not hacked or permanent hack
        }
        
        device_state.hack_timer -= time.delta_secs();
        
        if device_state.hack_timer <= 0.0 {
            // Restore device
            hackable.is_hacked = false;
            device_state.operational = true;
            device_state.powered = true;
            device_state.current_function = device_state.original_function.clone();
            
            info!("Device {:?} recovered from hack", hackable.device_type);
            
            // Restore power if it was a power station
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

// === POWER GRID PROPAGATION ===
pub fn power_grid_system(
    mut power_events: EventReader<PowerGridEvent>,
    mut power_grid: ResMut<PowerGrid>,
    mut device_query: Query<(&Hackable, &mut DeviceState)>,
) {
    for event in power_events.read() {
        if let Some(network) = power_grid.networks.get_mut(&event.network_id) {
            network.powered = event.powered;
            
            // Propagate power state to all connected devices
            for &device_entity in &network.connected_devices {
                if let Ok((hackable, mut device_state)) = device_query.get_mut(device_entity) {
                    if hackable.network_id.as_ref() == Some(&event.network_id) {
                        device_state.powered = event.powered;
                        
                        // Devices without power can't function
                        if !event.powered {
                            device_state.operational = false;
                        }
                    }
                }
            }
            
            info!("Power network '{}' set to: {}", event.network_id, event.powered);
        }
    }
}

// === HELPER FUNCTIONS FOR ADDING HACKABLE COMPONENTS ===

/// Add hackable component to any entity
pub fn make_hackable(commands: &mut Commands, entity: Entity, device_type: DeviceType) {
    let hackable = Hackable::new(device_type.clone());
    let device_state = DeviceState {
        original_function: match device_type {
            DeviceType::Camera => DeviceFunction::Surveillance,
            DeviceType::Turret => DeviceFunction::Defense,
            DeviceType::Drone => DeviceFunction::Surveillance,
            DeviceType::Door | DeviceType::Elevator => DeviceFunction::Transport,
            DeviceType::Vehicle => DeviceFunction::Transport,
            DeviceType::PowerStation => DeviceFunction::PowerDistribution,
            DeviceType::StreetLight => DeviceFunction::Lighting,
            DeviceType::TrafficLight => DeviceFunction::TrafficControl,
            DeviceType::Terminal => DeviceFunction::DataStorage,
            DeviceType::SecuritySystem | DeviceType::AlarmPanel => DeviceFunction::Security,
        },
        current_function: match device_type {
            DeviceType::Camera => DeviceFunction::Surveillance,
            DeviceType::Turret => DeviceFunction::Defense,
            DeviceType::Drone => DeviceFunction::Surveillance,
            DeviceType::Door | DeviceType::Elevator => DeviceFunction::Transport,
            DeviceType::Vehicle => DeviceFunction::Transport,
            DeviceType::PowerStation => DeviceFunction::PowerDistribution,
            DeviceType::StreetLight => DeviceFunction::Lighting,
            DeviceType::TrafficLight => DeviceFunction::TrafficControl,
            DeviceType::Terminal => DeviceFunction::DataStorage,
            DeviceType::SecuritySystem | DeviceType::AlarmPanel => DeviceFunction::Security,
        },
        ..default()
    };
    
    commands.entity(entity)
        .insert(hackable)
        .insert(device_state);
}

/// Add hackable component with power grid connection
pub fn make_hackable_networked(
    commands: &mut Commands, 
    entity: Entity, 
    device_type: DeviceType, 
    network_id: String,
    power_grid: &mut ResMut<PowerGrid>
) {
    let hackable = Hackable::new(device_type.clone()).with_network(network_id.clone());
    let device_state = DeviceState::default();
    
    // Add to power network
    let network = power_grid.networks.entry(network_id.clone())
        .or_insert_with(|| PowerNetwork::new(network_id));
    network.connected_devices.insert(entity);
    
    commands.entity(entity)
        .insert(hackable)
        .insert(device_state);
}

/// Quick setup for common hackable objects
pub fn setup_hackable_camera(commands: &mut Commands, entity: Entity) {
    make_hackable(commands, entity, DeviceType::Camera);
}

pub fn setup_hackable_turret(commands: &mut Commands, entity: Entity) {
    let hackable = Hackable::new(DeviceType::Turret).high_security();
    commands.entity(entity).insert(hackable).insert(DeviceState::default());
}

pub fn setup_hackable_door(commands: &mut Commands, entity: Entity) {
    let hackable = Hackable::new(DeviceType::Door).quick_hack();
    commands.entity(entity).insert(hackable).insert(DeviceState::default());
}

// === INTEGRATION WITH EXISTING VEHICLE SYSTEM ===
pub fn add_hackable_to_vehicles(
    mut commands: Commands,
    vehicle_query: Query<Entity, (With<Vehicle>, Without<Hackable>)>,
) {
    for entity in vehicle_query.iter() {
        make_hackable(&mut commands, entity, DeviceType::Vehicle);
    }
}