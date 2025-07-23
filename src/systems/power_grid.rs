// src/systems/power_grid.rs - Complete power grid implementation
use bevy::prelude::*;
use crate::core::*;

// === POWER GRID SPAWNING ===
pub fn spawn_power_station(
    commands: &mut Commands,
    position: Vec2,
    network_id: String,
    power_grid: &mut ResMut<PowerGrid>,
) -> Entity {
    let entity = commands.spawn((
        Sprite {
            color: Color::srgb(0.8, 0.8, 0.2),
            custom_size: Some(Vec2::new(60.0, 40.0)),
            ..default()
        },
        Transform::from_translation(position.extend(1.0)),
        PowerStation { 
            network_id: network_id.clone(),
            max_capacity: 100,
            current_load: 0,
        },
    )).id();
    
    make_hackable_networked(commands, entity, DeviceType::PowerStation, network_id.clone(), power_grid);
    
    // Register as power source
    let network = power_grid.networks.entry(network_id.clone())
        .or_insert_with(|| PowerNetwork::new(network_id));
    network.power_sources.insert(entity);
    
    entity
}

pub fn spawn_street_light(
    commands: &mut Commands,
    position: Vec2,
    network_id: String,
    power_grid: &mut ResMut<PowerGrid>,
) -> Entity {
    let entity = commands.spawn((
        Sprite {
            color: Color::srgb(0.9, 0.9, 0.7),
            custom_size: Some(Vec2::new(8.0, 24.0)),
            ..default()
        },
        Transform::from_translation(position.extend(1.0)),
        StreetLight { brightness: 1.0 },
    )).id();
    
    make_hackable_networked(commands, entity, DeviceType::StreetLight, network_id, power_grid);
    entity
}

pub fn spawn_traffic_light(
    commands: &mut Commands,
    position: Vec2,
    network_id: String,
    power_grid: &mut ResMut<PowerGrid>,
) -> Entity {
    let entity = commands.spawn((
        Sprite {
            color: Color::srgb(0.2, 0.8, 0.2), // Green light default
            custom_size: Some(Vec2::new(12.0, 32.0)),
            ..default()
        },
        Transform::from_translation(position.extend(1.0)),
        TrafficLight { 
            state: TrafficState::Green,
            timer: 10.0,
        },
    )).id();
    
    make_hackable_networked(commands, entity, DeviceType::TrafficLight, network_id, power_grid);
    entity
}

pub fn spawn_security_camera(
    commands: &mut Commands,
    position: Vec2,
    network_id: Option<String>,
    power_grid: &mut Option<ResMut<PowerGrid>>,
) -> Entity {
    let entity = commands.spawn((
        Sprite {
            color: Color::srgb(0.3, 0.3, 0.3),
            custom_size: Some(Vec2::new(16.0, 12.0)),
            ..default()
        },
        Transform::from_translation(position.extend(1.0)),
        SecurityCamera {
            detection_range: 120.0,
            fov_angle: 60.0,
            direction: Vec2::X,
            active: true,
        },
        Vision::new(120.0, 60.0),
    )).id();
    
    if let (Some(network_id), Some(power_grid)) = (network_id, power_grid) {
        make_hackable_networked(commands, entity, DeviceType::Camera, network_id, power_grid);
    } else {
        make_hackable(commands, entity, DeviceType::Camera);
    }
    
    entity
}

pub fn spawn_automated_turret(
    commands: &mut Commands,
    position: Vec2,
    network_id: Option<String>,
    power_grid: &mut Option<ResMut<PowerGrid>>,
) -> Entity {
    let entity = commands.spawn((
        Sprite {
            color: Color::srgb(0.6, 0.2, 0.2),
            custom_size: Some(Vec2::new(20.0, 20.0)),
            ..default()
        },
        Transform::from_translation(position.extend(1.0)),
        AutomatedTurret {
            range: 150.0,
            damage: 25.0,
            fire_rate: 2.0,
            fire_timer: 0.0,
            target: None,
        },
        Vision::new(150.0, 90.0),
        WeaponState::new_from_type(&WeaponType::Rifle),
    )).id();
    
    if let (Some(network_id), Some(power_grid)) = (network_id, power_grid) {
        make_hackable_networked(commands, entity, DeviceType::Turret, network_id, power_grid);
    } else {
        setup_hackable_turret(commands, entity);
    }
    
    entity
}

pub fn spawn_security_door(
    commands: &mut Commands,
    position: Vec2,
    network_id: Option<String>,
    power_grid: &mut Option<ResMut<PowerGrid>>,
) -> Entity {
    let entity = commands.spawn((
        Sprite {
            color: Color::srgb(0.4, 0.4, 0.6),
            custom_size: Some(Vec2::new(8.0, 32.0)),
            ..default()
        },
        Transform::from_translation(position.extend(1.0)),
        SecurityDoor {
            locked: true,
            access_level: 2,
        },
        bevy_rapier2d::prelude::RigidBody::Fixed,
        bevy_rapier2d::prelude::Collider::cuboid(4.0, 16.0),
    )).id();
    
    if let (Some(network_id), Some(power_grid)) = (network_id, power_grid) {
        make_hackable_networked(commands, entity, DeviceType::Door, network_id, power_grid);
    } else {
        setup_hackable_door(commands, entity);
    }
    
    entity
}

// === DEVICE COMPONENTS ===
#[derive(Component)]
pub struct PowerStation {
    pub network_id: String,
    pub max_capacity: u32,
    pub current_load: u32,
}

#[derive(Component)]
pub struct StreetLight {
    pub brightness: f32,
}

#[derive(Component)]
pub struct TrafficLight {
    pub state: TrafficState,
    pub timer: f32,
}

#[derive(Debug, Clone)]
pub enum TrafficState {
    Red,
    Yellow,
    Green,
    Disabled,
}

#[derive(Component)]
pub struct SecurityCamera {
    pub detection_range: f32,
    pub fov_angle: f32,
    pub direction: Vec2,
    pub active: bool,
}

#[derive(Component)]
pub struct AutomatedTurret {
    pub range: f32,
    pub damage: f32,
    pub fire_rate: f32, // Shots per second
    pub fire_timer: f32,
    pub target: Option<Entity>,
}

#[derive(Component)]
pub struct SecurityDoor {
    pub locked: bool,
    pub access_level: u8,
}

#[derive(Component)]
pub struct Drone {
    pub patrol_route: Vec<Vec2>,
    pub current_waypoint: usize,
    pub speed: f32,
    pub detection_range: f32,
}

// === POWER GRID MANAGEMENT ===
pub fn power_grid_management_system(
    power_stations: Query<(Entity, &PowerStation, &DeviceState)>,
    mut power_grid: ResMut<PowerGrid>,
    mut power_events: EventWriter<PowerGridEvent>,
) {
    for (entity, station, device_state) in power_stations.iter() {
        if let Some(network) = power_grid.networks.get_mut(&station.network_id) {
            let was_powered = network.powered;
            
            // Network is powered if at least one power station is operational
            network.powered = power_stations.iter()
                .filter(|(_, s, _)| s.network_id == station.network_id)
                .any(|(_, _, state)| state.operational && state.powered);
            
            // Send event if power state changed
            if was_powered != network.powered {
                power_events.write(PowerGridEvent {
                    network_id: station.network_id.clone(),
                    powered: network.powered,
                    source: entity,
                });
            }
        }
    }
}

// === DEVICE BEHAVIOR SYSTEMS ===
pub fn street_light_system(
    mut street_lights: Query<(&mut Sprite, &DeviceState), With<StreetLight>>,
) {
    for (mut sprite, device_state) in street_lights.iter_mut() {
        if device_state.powered && device_state.operational {
            sprite.color = Color::srgb(0.9, 0.9, 0.7); // Bright
        } else {
            sprite.color = Color::srgb(0.3, 0.3, 0.3); // Dark
        }
    }
}

pub fn traffic_light_system(
    mut traffic_lights: Query<(&mut TrafficLight, &mut Sprite, &DeviceState)>,
    time: Res<Time>,
) {
    for (mut traffic_light, mut sprite, device_state) in traffic_lights.iter_mut() {
        if !device_state.powered || !device_state.operational {
            traffic_light.state = TrafficState::Disabled;
            sprite.color = Color::srgb(0.2, 0.2, 0.2);
            continue;
        }
        
        traffic_light.timer -= time.delta_secs();
        
        if traffic_light.timer <= 0.0 {
            traffic_light.state = match traffic_light.state {
                TrafficState::Green => {
                    traffic_light.timer = 3.0; // Yellow for 3 seconds
                    TrafficState::Yellow
                },
                TrafficState::Yellow => {
                    traffic_light.timer = 8.0; // Red for 8 seconds
                    TrafficState::Red
                },
                TrafficState::Red => {
                    traffic_light.timer = 10.0; // Green for 10 seconds
                    TrafficState::Green
                },
                TrafficState::Disabled => TrafficState::Green,
            };
        }
        
        sprite.color = match traffic_light.state {
            TrafficState::Green => Color::srgb(0.2, 0.8, 0.2),
            TrafficState::Yellow => Color::srgb(0.8, 0.8, 0.2),
            TrafficState::Red => Color::srgb(0.8, 0.2, 0.2),
            TrafficState::Disabled => Color::srgb(0.2, 0.2, 0.2),
        };
    }
}

pub fn security_camera_system(
    mut cameras: Query<(&mut SecurityCamera, &mut Vision, &Transform, &DeviceState)>,
    agent_query: Query<&Transform, (With<Agent>, Without<SecurityCamera>)>,
    mut alert_events: EventWriter<AlertEvent>,
    time: Res<Time>,
) {
    for (mut camera, mut vision, camera_transform, device_state) in cameras.iter_mut() {
        camera.active = device_state.powered && device_state.operational;
        
        if !camera.active {
            continue;
        }
        
        // Slowly rotate camera (simple patrol)
        let rotation_speed = 0.5; // radians per second
        let angle = time.elapsed_secs() * rotation_speed;
        camera.direction = Vec2::new(angle.cos(), angle.sin());
        vision.direction = camera.direction;
        
        // Check for agents in view
        let camera_pos = camera_transform.translation.truncate();
        
        for agent_transform in agent_query.iter() {
            let agent_pos = agent_transform.translation.truncate();
            let to_agent = agent_pos - camera_pos;
            let distance = to_agent.length();
            
            if distance <= camera.detection_range {
                let agent_direction = to_agent.normalize();
                let dot_product = camera.direction.dot(agent_direction);
                let angle_cos = (camera.fov_angle.to_radians() / 2.0).cos();
                
                if dot_product >= angle_cos {
                    // Agent spotted by camera!
                    alert_events.write(AlertEvent {
                        alerter: Entity::PLACEHOLDER, // Camera entity would go here
                        position: agent_pos,
                        alert_level: 2,
                        source: AlertSource::SpottedAgent,
                        alert_type: AlertType::EnemySpotted,
                    });
                }
            }
        }
    }
}

pub fn automated_turret_system(
    mut turrets: Query<(Entity, &mut AutomatedTurret, &Transform, &DeviceState)>,
    agent_query: Query<(Entity, &Transform), With<Agent>>,
    mut combat_events: EventWriter<CombatEvent>,
    mut audio_events: EventWriter<AudioEvent>,
    time: Res<Time>,
) {
    for (turret_entity, mut turret, turret_transform, device_state) in turrets.iter_mut() {
        if !device_state.powered || !device_state.operational {
            turret.target = None;
            continue;
        }
        
        turret.fire_timer -= time.delta_secs();
        let turret_pos = turret_transform.translation.truncate();
        
        // Find target
        if turret.target.is_none() {
            turret.target = agent_query.iter()
                .filter(|(_, agent_transform)| {
                    turret_pos.distance(agent_transform.translation.truncate()) <= turret.range
                })
                .min_by(|(_, a), (_, b)| {
                    let dist_a = turret_pos.distance(a.translation.truncate());
                    let dist_b = turret_pos.distance(b.translation.truncate());
                    dist_a.partial_cmp(&dist_b).unwrap_or(std::cmp::Ordering::Equal)
                })
                .map(|(entity, _)| entity);
        }
        
        // Fire at target
        if let Some(target) = turret.target {
            if let Ok((_, target_transform)) = agent_query.get(target) {
                let distance = turret_pos.distance(target_transform.translation.truncate());
                
                if distance <= turret.range && turret.fire_timer <= 0.0 {
                    // Fire!
                    combat_events.write(CombatEvent {
                        attacker: turret_entity,
                        target,
                        damage: turret.damage,
                        hit: rand::random::<f32>() < 0.8, // 80% accuracy
                    });
                    
                    audio_events.write(AudioEvent {
                        sound: AudioType::Gunshot,
                        volume: 0.6,
                    });
                    
                    turret.fire_timer = 1.0 / turret.fire_rate;
                } else if distance > turret.range {
                    turret.target = None; // Lost target
                }
            } else {
                turret.target = None; // Target no longer exists
            }
        }
    }
}

pub fn security_door_system(
    mut doors: Query<(&mut SecurityDoor, &DeviceState, &mut bevy_rapier2d::prelude::Collider)>,
) {
    for (mut door, device_state, mut collider) in doors.iter_mut() {
        // Doors unlock when hacked or powered down
        if !device_state.operational || !device_state.powered {
            door.locked = false;
        }
        
        // Update physics collider based on door state
        if door.locked {
            *collider = bevy_rapier2d::prelude::Collider::cuboid(4.0, 16.0); // Solid
        } else {
            *collider = bevy_rapier2d::prelude::Collider::cuboid(0.1, 0.1); // Passable
        }
    }
}

// === INTEGRATION HELPERS ===
pub fn setup_district_power_grid(
    commands: &mut Commands,
    mut power_grid: ResMut<PowerGrid>,
    center: Vec2,
) {
    let network_id = "district_main".to_string();
    
    // Main power station
    spawn_power_station(
        commands,
        center + Vec2::new(-200.0, -100.0),
        network_id.clone(),
        &mut power_grid,
    );
    
    // Street lights along main road
    for i in 0..8 {
        let x = center.x - 200.0 + (i as f32 * 50.0);
        spawn_street_light(
            commands,
            Vec2::new(x, center.y),
            network_id.clone(),
            &mut power_grid,
        );
    }
    
    // Traffic lights at intersections
    spawn_traffic_light(
        commands,
        center + Vec2::new(-50.0, 0.0),
        network_id.clone(),
        &mut power_grid,
    );
    spawn_traffic_light(
        commands,
        center + Vec2::new(100.0, 50.0),
        network_id.clone(),
        &mut power_grid,
    );
    
    // Security infrastructure
    let mut power_grid_option = Some(power_grid);
    
    spawn_security_camera(
        commands,
        center + Vec2::new(0.0, 100.0),
        Some(network_id.clone()),
        &mut power_grid_option,
    );
    
    spawn_automated_turret(
        commands,
        center + Vec2::new(150.0, -50.0),
        Some(network_id.clone()),
        &mut power_grid_option,
    );
    
    spawn_security_door(
        commands,
        center + Vec2::new(200.0, 0.0),
        Some(network_id),
        &mut power_grid_option,
    );
}

// === POWER GRID DEBUG ===
pub fn power_grid_debug_system(
    mut gizmos: Gizmos,
    power_grid: Res<PowerGrid>,
    hackable_query: Query<(&Transform, &Hackable, &DeviceState)>,
    input: Res<ButtonInput<KeyCode>>,
    mut show_power_debug: Local<bool>,
) {
    if input.just_pressed(KeyCode::KeyH) {
        *show_power_debug = !*show_power_debug;
        info!("Power grid debug: {} ({} networks)", 
              if *show_power_debug { "ON" } else { "OFF" },
              power_grid.networks.len());
    }
    
    if !*show_power_debug { return; }
    
    // Draw power connections
    for (transform, hackable, device_state) in hackable_query.iter() {
        if let Some(network_id) = &hackable.network_id {
            let pos = transform.translation.truncate();
            
            let color = if device_state.powered {
                if device_state.operational {
                    Color::srgb(0.2, 0.8, 0.2) // Green = powered and working
                } else {
                    Color::srgb(0.8, 0.8, 0.2) // Yellow = powered but hacked
                }
            } else {
                Color::srgb(0.8, 0.2, 0.2) // Red = no power
            };
            
            // Draw power indicator
            gizmos.circle_2d(pos + Vec2::new(0.0, 20.0), 4.0, color);
            
            // Draw network ID (simplified)
            let network_hash = network_id.chars().fold(0u32, |acc, c| acc.wrapping_add(c as u32));
            let network_color = match network_hash % 3 {
                0 => Color::srgb(0.8, 0.4, 0.4),
                1 => Color::srgb(0.4, 0.8, 0.4),
                _ => Color::srgb(0.4, 0.4, 0.8),
            };
            gizmos.circle_2d(pos + Vec2::new(0.0, 15.0), 2.0, network_color);
        }
    }
}
