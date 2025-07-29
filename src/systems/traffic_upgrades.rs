// src/systems/traffic_upgrades.rs - Additional extensions to the traffic system
use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use bevy::utils::*;
use std::time::{SystemTime};
use crate::core::*;
use crate::systems::*;
use crate::systems::traffic::{point_to_line_distance};


pub fn mark_road_blocked(
    traffic_system: &mut ResMut<TrafficSystem>,
    position: Vec2,
    radius: f32,
) {
    let road_network = &mut traffic_system.road_network;
    
    let roads = &mut road_network.roads;
    let flow_field = &mut road_network.flow_field;

    for road in roads.iter_mut() {
        let distance_to_road = point_to_line_distance(position, road.start, road.end);
        if distance_to_road <= radius {
            road.blocked = true;
            info!("Road blocked at {:?}", position);
        }
    }
    
    flow_field.update_flow(roads);
}

pub fn clear_road_blocks(traffic_system: &mut ResMut<TrafficSystem>) {
    let road_network = &mut traffic_system.road_network;

    // SAFELY split the mutable borrow of `road_network` into separate parts
    let roads = &mut road_network.roads;
    let flow_field = &mut road_network.flow_field;

    for road in roads.iter_mut() {
        road.blocked = false;
    }

    flow_field.update_flow(roads);
}

// === PEDESTRIAN TRAFFIC INTERACTION ===

// system to handle civilian reactions to traffic:
pub fn civilian_traffic_interaction_system(
    mut civilian_query: Query<(Entity, &Transform, &mut Morale), (With<Civilian>, Without<TrafficVehicle>)>,
    traffic_query: Query<(&Transform, &TrafficVehicle, &Velocity), Without<Civilian>>,
    mut alert_events: EventWriter<AlertEvent>,
    mut action_events: EventWriter<ActionEvent>,
) {
    for (civilian_entity, civilian_transform, mut morale) in civilian_query.iter_mut() {
        let civilian_pos = civilian_transform.translation.truncate();
        
        for (vehicle_transform, vehicle, velocity) in traffic_query.iter() {
            let vehicle_pos = vehicle_transform.translation.truncate();
            let distance = civilian_pos.distance(vehicle_pos);
            
            // Near miss detection
            if distance < 40.0 && velocity.linvel.length() > 80.0 {
                // Civilian panics from near miss
                morale.reduce(25.0);
                
                // Move away from vehicle
                let flee_direction = (civilian_pos - vehicle_pos).normalize_or_zero();
                let flee_target = civilian_pos + flee_direction * 100.0;
                
                action_events.write(ActionEvent {
                    entity: civilian_entity,
                    action: Action::MoveTo(flee_target),
                });
                
            if rand::random::<f32>() < 0.3 {
                alert_events.write(AlertEvent {
                    alerter: civilian_entity,
                    position: civilian_pos,
                    alert_level: 2,
                    source: AlertSource::CivilianReport,
                    alert_type: AlertType::TrafficIncident,
                });
            }
            }
        }
    }
}

// === TRAFFIC LIGHT INTEGRATION ===

// Extend your existing traffic light system to work with vehicles:
pub fn traffic_light_vehicle_system(
    traffic_lights: Query<(&Transform, &TrafficLight), With<TrafficLight>>,
    mut vehicle_query: Query<(&Transform, &mut TrafficVehicle, &mut Velocity)>,
) {
    for (light_transform, traffic_light) in traffic_lights.iter() {
        let light_pos = light_transform.translation.truncate();
        
        for (vehicle_transform, mut vehicle, mut velocity) in vehicle_query.iter_mut() {
            let vehicle_pos = vehicle_transform.translation.truncate();
            let distance = vehicle_pos.distance(light_pos);
            
            // Stop at red lights (within 60 units)
            if distance < 60.0 && matches!(traffic_light.state, TrafficState::Red) {
                let to_light = (light_pos - vehicle_pos).normalize_or_zero();
                let vehicle_direction = velocity.linvel.normalize_or_zero();
                
                // If heading towards the light
                if vehicle_direction.dot(to_light) > 0.5 {
                    // Apply brakes
                    let brake_force = vehicle.brake_force * 2.0; // Strong braking for lights
                    velocity.linvel *= 0.95; // Gradual stop
                    vehicle.brake_lights = true;
                    
                    // Full stop very close to light
                    if distance < 25.0 {
                        velocity.linvel *= 0.1;
                    }
                }
            }
        }
    }
}

// === HACKABLE TRAFFIC INTEGRATION ===

// Extend traffic lights to be hackable:
pub fn setup_hackable_traffic_lights(
    mut commands: Commands,
    traffic_lights: Query<Entity, (With<TrafficLight>, Without<Hackable>)>,
) {
    for entity in traffic_lights.iter() {
        commands.entity(entity).insert((
            Hackable {
                security_level: 2,
                hack_time: 3.0,
                network_id: Some("traffic_network".to_string()),
                device_type: DeviceType::TrafficLight,
                disabled_duration: 60.0,
                hack_effects: Vec::new(),
                is_hacked: true,
                requires_tool: Some(HackTool::BasicHacker),
            },
            DeviceState {
                operational: true,
                powered: true,
                current_function: DeviceFunction::Defense,
                hack_timer:0.0,
                original_function: DeviceFunction::TrafficControl,
            },
        ));
    }
}

// Handle hacked traffic lights:
pub fn hacked_traffic_light_system(
    mut traffic_lights: Query<(&mut TrafficLight, &DeviceState)>,
) {
    for (mut light, device_state) in traffic_lights.iter_mut() {
        if !device_state.operational {
            // Hacked lights cause chaos - rapid state changes
            light.timer = 0.5; // Very fast cycling
            
            // Random state when hacked
            if rand::random::<f32>() < 0.1 {
                light.state = match rand::random::<u8>() % 3 {
                    0 => TrafficState::Red,
                    1 => TrafficState::Yellow,
                    _ => TrafficState::Green,
                };
            }
        }
    }
}

// === EMERGENCY VEHICLE SIRENS ===

// Add siren effects:
pub fn emergency_siren_system(
    mut emergency_vehicles: Query<(&Transform, &mut EmergencyVehicle, &TrafficVehicle)>,
    mut other_vehicles: Query<(&Transform, &mut TrafficVehicle, &mut Velocity), Without<EmergencyVehicle>>,
    mut audio_events: EventWriter<AudioEvent>,
    time: Res<Time>,
) {
    for (emergency_transform, mut emergency, emergency_vehicle) in emergency_vehicles.iter_mut() {
        let emergency_pos = emergency_transform.translation.truncate();
        
        // Activate siren when moving fast
        emergency.siren_active = emergency_vehicle.current_speed > 60.0;
        
        if emergency.siren_active {
            // Play siren sound occasionally
            if rand::random::<f32>() < 0.02 { // 2% chance per frame
                audio_events.write(AudioEvent {
                    sound: AudioType::Alert, // Reuse alert sound for siren
                    volume: 0.8,
                });
            }
            
            // Make other vehicles move aside
            for (vehicle_transform, mut vehicle, mut velocity) in other_vehicles.iter_mut() {
                let vehicle_pos = vehicle_transform.translation.truncate();
                let distance = emergency_pos.distance(vehicle_pos);
                
                if distance < 100.0 {
                    // Push vehicle aside
                    let push_direction = (vehicle_pos - emergency_pos).normalize_or_zero();
                    let push_force = (100.0 - distance) / 100.0 * 50.0;
                    
                    velocity.linvel += push_direction * push_force;
                    vehicle.panic_level = (vehicle.panic_level + 0.5).min(1.0);
                }
            }
        }
    }
}

// === CONVOY FORMATION ===

// Keep convoy vehicles in formation:
pub fn convoy_formation_system(
    mut convoy_query: Query<(Entity, &mut MilitaryConvoy, &Transform)>,
    mut vehicle_query: Query<(&mut TrafficFlow, &mut Transform), (With<TrafficVehicle>, Without<MilitaryConvoy>)>,
) {
    for (leader_entity, convoy, leader_transform) in convoy_query.iter_mut() {
        if convoy.formation_members.is_empty() { continue; }
        
        let leader_pos = leader_transform.translation.truncate();
        
        for (i, &member_entity) in convoy.formation_members.iter().enumerate() {
            if let Ok((mut flow, mut member_transform)) = vehicle_query.get_mut(member_entity) {
                // Calculate formation position
                let offset = Vec2::new(-50.0 * (i as f32 + 1.0), 0.0);
                let formation_pos = leader_pos + offset;
                
                // Update member's path to maintain formation
                flow.path = vec![formation_pos];
                flow.path_index = 0;
            }
        }
    }
}

// === INTEGRATION WITH EXISTING SYSTEMS ===

// Add to your existing explosion system to block roads:
pub fn explosion_road_blocking_system(
    mut explosion_events: EventReader<GrenadeEvent>,
    mut traffic_system: ResMut<TrafficSystem>,
) {
    for explosion in explosion_events.read() {
        // Block roads near explosions
        mark_road_blocked(&mut traffic_system, explosion.target_pos, explosion.explosion_radius);
        
        // Roads clear after some time (you'd need a timer system for this)
    }
}

