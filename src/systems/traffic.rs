// src/systems/traffic.rs - Efficient traffic simulation for cyberpunk urban environment
use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use crate::core::*;
use crate::systems::*;

// === TRAFFIC COMPONENTS ===

#[derive(Component)]
pub struct TrafficVehicle {
    pub vehicle_type: TrafficVehicleType,
    pub max_speed: f32,
    pub current_speed: f32,
    pub acceleration: f32,
    pub brake_force: f32,
    pub lane_position: f32, // -1.0 to 1.0 within lane
    pub destination: Option<Vec2>,
    pub panic_level: f32,
    pub brake_lights: bool,
}

#[derive(Debug, Clone)]
pub enum TrafficVehicleType {
    CivilianCar,
    Bus,
    Truck,
    EmergencyAmbulance,
    PoliceCar,
    MilitaryConvoy,
    MotorCycle,
}

#[derive(Component)]
pub struct RoadTile {
    pub direction: RoadDirection,
    pub lane_count: u8,
    pub speed_limit: f32,
    pub tile_type: RoadType,
}

#[derive(Debug, Clone)]
pub enum RoadDirection {
    North,
    South, 
    East,
    West,
    NorthSouth, // Two-way
    EastWest,   // Two-way
}

#[derive(Debug, Clone)]
pub enum RoadType {
    Street,
    Highway,
    Intersection,
    ParkingLot,
}

#[derive(Component)]
pub struct TrafficFlow {
    pub current_lane: u8,
    pub target_lane: u8,
    pub following_distance: f32,
    pub lane_change_cooldown: f32,
    pub path: Vec<Vec2>,
    pub path_index: usize,
}

#[derive(Component)]
pub struct EmergencyVehicle {
    pub siren_active: bool,
    pub priority_level: u8, // 1=highest, 3=lowest
    pub response_target: Option<Vec2>,
}

#[derive(Component)]
pub struct MilitaryConvoy {
    pub formation_leader: Option<Entity>,
    pub formation_members: Vec<Entity>,
    pub alert_status: ConvoyAlertStatus,
    pub troops_inside: u8,
}

#[derive(Debug, Clone)]
pub enum ConvoyAlertStatus {
    Patrol,
    Investigating,
    UnderAttack,
    Deploying,
}

// === TRAFFIC FLOW RESOURCE ===

#[derive(Resource)]
pub struct TrafficSystem {
    pub road_network: RoadNetwork,
    pub spawn_timer: f32,
    pub max_vehicles: usize,
    pub emergency_response_timer: f32,
}

pub struct RoadNetwork {
    pub roads: Vec<RoadSegment>,
    pub intersections: Vec<Intersection>,
    pub spawn_points: Vec<Vec2>,
    pub flow_field: FlowField,
}

pub struct RoadSegment {
    pub start: Vec2,
    pub end: Vec2,
    pub direction: RoadDirection,
    pub lanes: u8,
    pub blocked: bool,
}

pub struct Intersection {
    pub center: Vec2,
    pub traffic_light: Option<Entity>,
    pub yield_rules: Vec<RoadDirection>,
}

pub struct FlowField {
    pub grid_size: f32,
    pub width: usize,
    pub height: usize,
    pub flow_vectors: Vec<Vec2>,
    pub costs: Vec<f32>,
}

impl Default for TrafficSystem {
    fn default() -> Self {
        Self {
            road_network: RoadNetwork::default(),
            spawn_timer: 0.0,
            max_vehicles: 20,
            emergency_response_timer: 0.0,
        }
    }
}

impl Default for RoadNetwork {
    fn default() -> Self {
        Self {
            roads: create_default_road_network(),
            intersections: create_default_intersections(),
            spawn_points: vec![
                Vec2::new(-400.0, 0.0),
                Vec2::new(400.0, 0.0), 
                Vec2::new(0.0, -400.0),
                Vec2::new(0.0, 400.0),
            ],
            flow_field: FlowField::new(32.0, 50, 50),
        }
    }
}

impl FlowField {
    pub fn new(grid_size: f32, width: usize, height: usize) -> Self {
        Self {
            grid_size,
            width,
            height,
            flow_vectors: vec![Vec2::ZERO; width * height],
            costs: vec![1.0; width * height],
        }
    }
    
    pub fn update_flow(&mut self, roads: &[RoadSegment]) {
        // Simple flow field calculation
        for (i, flow_vec) in self.flow_vectors.iter_mut().enumerate() {
            let x = i % self.width;
            let y = i / self.width;
            let world_pos = Vec2::new(
                (x as f32 - self.width as f32 * 0.5) * self.grid_size,
                (y as f32 - self.height as f32 * 0.5) * self.grid_size,
            );
            
            // Find nearest road and set flow direction
            if let Some(road) = find_nearest_road(world_pos, roads) {
                *flow_vec = (road.end - road.start).normalize_or_zero();
            }
        }
    }
}

fn find_nearest_road(pos: Vec2, roads: &[RoadSegment]) -> Option<&RoadSegment> {
    roads.iter()
        .min_by(|a, b| {
            let dist_a = point_to_line_distance(pos, a.start, a.end);
            let dist_b = point_to_line_distance(pos, b.start, b.end);
            dist_a.partial_cmp(&dist_b).unwrap_or(std::cmp::Ordering::Equal)
        })
}

pub fn point_to_line_distance(point: Vec2, line_start: Vec2, line_end: Vec2) -> f32 {
    let line_vec = line_end - line_start;
    let point_vec = point - line_start;
    let line_len = line_vec.length();
    
    if line_len < 0.01 {
        return point_vec.length();
    }
    
    let t = (point_vec.dot(line_vec) / (line_len * line_len)).clamp(0.0, 1.0);
    let projection = line_start + line_vec * t;
    point.distance(projection)
}

fn create_default_road_network() -> Vec<RoadSegment> {
    vec![
        // Main horizontal road
        RoadSegment {
            start: Vec2::new(-400.0, 0.0),
            end: Vec2::new(400.0, 0.0),
            direction: RoadDirection::EastWest,
            lanes: 2,
            blocked: false,
        },
        // Main vertical road
        RoadSegment {
            start: Vec2::new(0.0, -400.0),
            end: Vec2::new(0.0, 400.0),
            direction: RoadDirection::NorthSouth,
            lanes: 2,
            blocked: false,
        },
        // Side streets
        RoadSegment {
            start: Vec2::new(-200.0, -200.0),
            end: Vec2::new(-200.0, 200.0),
            direction: RoadDirection::NorthSouth,
            lanes: 1,
            blocked: false,
        },
        RoadSegment {
            start: Vec2::new(200.0, -200.0),
            end: Vec2::new(200.0, 200.0),
            direction: RoadDirection::NorthSouth,
            lanes: 1,
            blocked: false,
        },
    ]
}

fn create_default_intersections() -> Vec<Intersection> {
    vec![
        Intersection {
            center: Vec2::new(0.0, 0.0),
            traffic_light: None,
            yield_rules: vec![RoadDirection::North, RoadDirection::South],
        },
        Intersection {
            center: Vec2::new(-200.0, 0.0),
            traffic_light: None,
            yield_rules: vec![RoadDirection::East],
        },
        Intersection {
            center: Vec2::new(200.0, 0.0),
            traffic_light: None,
            yield_rules: vec![RoadDirection::West],
        },
    ]
}

// === TRAFFIC SPAWNING ===

pub fn traffic_spawn_system(
    mut commands: Commands,
    mut traffic_system: ResMut<TrafficSystem>,
    vehicle_query: Query<Entity, With<TrafficVehicle>>,
    time: Res<Time>,
    game_mode: Res<GameMode>,
    sprites: Res<GameSprites>,
) {
    if game_mode.paused { return; }
    
    traffic_system.spawn_timer -= time.delta_secs();
    let current_count = vehicle_query.iter().count();
    
    if traffic_system.spawn_timer <= 0.0 && current_count < traffic_system.max_vehicles {
        if let Some(&spawn_pos) = traffic_system.road_network.spawn_points.get(rand::random::<usize>() % traffic_system.road_network.spawn_points.len()) {
            let vehicle_type = choose_vehicle_type();
            spawn_traffic_vehicle(&mut commands, spawn_pos, vehicle_type, &sprites);
            traffic_system.spawn_timer = 3.0 + rand::random::<f32>() * 4.0;
        }
    }
}

fn choose_vehicle_type() -> TrafficVehicleType {
    match rand::random::<f32>() {
        x if x < 0.7 => TrafficVehicleType::CivilianCar,
        x if x < 0.8 => TrafficVehicleType::Bus,
        x if x < 0.9 => TrafficVehicleType::Truck,
        x if x < 0.95 => TrafficVehicleType::MotorCycle,
        _ => TrafficVehicleType::PoliceCar,
    }
}

pub fn spawn_traffic_vehicle(
    commands: &mut Commands,
    position: Vec2,
    vehicle_type: TrafficVehicleType,
    sprites: &GameSprites,
) {
    let (max_speed, size, color, health) = match vehicle_type {
        TrafficVehicleType::CivilianCar => (120.0, Vec2::new(32.0, 16.0), Color::srgb(0.6, 0.6, 0.8), 60.0),
        TrafficVehicleType::Bus => (80.0, Vec2::new(48.0, 20.0), Color::srgb(0.8, 0.8, 0.2), 150.0),
        TrafficVehicleType::Truck => (100.0, Vec2::new(40.0, 18.0), Color::srgb(0.5, 0.3, 0.2), 120.0),
        TrafficVehicleType::EmergencyAmbulance => (150.0, Vec2::new(36.0, 18.0), Color::srgb(1.0, 1.0, 1.0), 100.0),
        TrafficVehicleType::PoliceCar => (140.0, Vec2::new(34.0, 16.0), Color::srgb(0.2, 0.2, 0.8), 100.0),
        TrafficVehicleType::MilitaryConvoy => (110.0, Vec2::new(44.0, 20.0), Color::srgb(0.3, 0.5, 0.3), 200.0),
        TrafficVehicleType::MotorCycle => (160.0, Vec2::new(12.0, 8.0), Color::srgb(0.8, 0.2, 0.2), 40.0),
    };
    
    let mut entity_commands = commands.spawn((
        Sprite {
            color,
            custom_size: Some(size),
            ..default()
        },
        Transform::from_translation(position.extend(0.9)),
        TrafficVehicle {
            vehicle_type: vehicle_type.clone(),
            max_speed,
            current_speed: 0.0,
            acceleration: 200.0,
            brake_force: 400.0,
            lane_position: 0.0,
            destination: None,
            panic_level: 0.0,
            brake_lights: false,
        },
        TrafficFlow {
            current_lane: 0,
            target_lane: 0,
            following_distance: 40.0,
            lane_change_cooldown: 0.0,
            path: Vec::new(),
            path_index: 0,
        },
        Health(health),
        Vehicle::new(match vehicle_type {
            TrafficVehicleType::CivilianCar => VehicleType::CivilianCar,
            TrafficVehicleType::Bus => VehicleType::Truck,
            TrafficVehicleType::Truck => VehicleType::Truck,
            TrafficVehicleType::PoliceCar => VehicleType::PoliceCar,
            TrafficVehicleType::MilitaryConvoy => VehicleType::APC,
            _ => VehicleType::CivilianCar,
        }),
        RigidBody::Dynamic,
        Collider::cuboid(size.x * 0.5, size.y * 0.5),
        Velocity::default(),
        Damping { linear_damping: 5.0, angular_damping: 10.0 },
        CollisionGroups::new(VEHICLE_GROUP, Group::ALL),
        GravityScale(0.0),
        Scannable,
    ));
    
    // Add special components
    match vehicle_type {
        TrafficVehicleType::EmergencyAmbulance | TrafficVehicleType::PoliceCar => {
            entity_commands.insert(EmergencyVehicle {
                siren_active: false,
                priority_level: if matches!(vehicle_type, TrafficVehicleType::EmergencyAmbulance) { 1 } else { 2 },
                response_target: None,
            });
        },
        TrafficVehicleType::MilitaryConvoy => {
            entity_commands.insert(MilitaryConvoy {
                formation_leader: None,
                formation_members: Vec::new(),
                alert_status: ConvoyAlertStatus::Patrol,
                troops_inside: 4,
            });
        },
        _ => {},
    }
}

// === TRAFFIC MOVEMENT ===

pub fn traffic_movement_system(
    mut traffic_query: Query<(
        Entity,
        &mut Transform,
        &mut TrafficVehicle,
        &mut TrafficFlow,
        &mut Velocity,
        Option<&EmergencyVehicle>,
    )>,
    obstacle_query: Query<&Transform, (Or<(With<Agent>, With<Civilian>, With<Enemy>)>, Without<TrafficVehicle>)>,
    traffic_system: Res<TrafficSystem>,
    time: Res<Time>,
    game_mode: Res<GameMode>,
) {
    if game_mode.paused { return; }
    
    let delta = time.delta_secs();
    
    for (entity, mut transform, mut vehicle, mut flow, mut velocity, emergency) in traffic_query.iter_mut() {
        let current_pos = transform.translation.truncate();
        
        // Update flow field path if needed
        if flow.path.is_empty() || flow.path_index >= flow.path.len() {
            update_vehicle_path(&mut flow, current_pos, &traffic_system);
        }
        
        // Calculate desired velocity
        let mut desired_velocity = Vec2::ZERO;
        let mut target_speed = vehicle.max_speed;
        
        if let Some(target) = get_current_target(&flow) {
            let to_target = target - current_pos;
            let distance = to_target.length();
            
            if distance > 5.0 {
                desired_velocity = to_target.normalize() * target_speed;
            } else {
                flow.path_index += 1;
            }
        }
        
        // Obstacle avoidance
        let mut brake_factor = 1.0;
        let mut should_brake = false;
        
        for obstacle_transform in obstacle_query.iter() {
            let obstacle_pos = obstacle_transform.translation.truncate();
            let to_obstacle = obstacle_pos - current_pos;
            let distance = to_obstacle.length();
            
            // Check if obstacle is in our path
            if distance < 50.0 {
                let velocity_dir = velocity.linvel.normalize_or_zero();
                let obstacle_dir = to_obstacle.normalize_or_zero();
                
                if velocity_dir.dot(obstacle_dir) > 0.7 { // Obstacle ahead
                    brake_factor = (distance / 50.0).clamp(0.1, 1.0);
                    should_brake = true;
                    
                    // Panic if too close
                    if distance < 20.0 {
                        vehicle.panic_level = (vehicle.panic_level + delta * 2.0).min(1.0);
                    }
                }
            }
        }
        
        // Emergency vehicle behavior
        if let Some(emergency) = emergency {
            if emergency.siren_active {
                target_speed *= 1.5; // Emergency vehicles go faster
                // Push other vehicles aside (simplified)
                brake_factor = brake_factor.max(0.8);
            }
        }
        
        // Apply movement
        let target_velocity = desired_velocity * brake_factor;
        vehicle.current_speed = target_velocity.length();
        
        // Smooth acceleration/deceleration
        let current_vel = velocity.linvel;
        let vel_diff = target_velocity - current_vel;
        let max_change = if should_brake { 
            vehicle.brake_force * delta 
        } else { 
            vehicle.acceleration * delta 
        };
        
        let vel_change = vel_diff.normalize_or_zero() * max_change.min(vel_diff.length());
        velocity.linvel += vel_change;
        
        // Update brake lights
        vehicle.brake_lights = should_brake || vehicle.current_speed < 20.0;
        
        // Reduce panic over time
        vehicle.panic_level = (vehicle.panic_level - delta * 0.5).max(0.0);
    }
}

fn update_vehicle_path(flow: &mut TrafficFlow, current_pos: Vec2, traffic_system: &TrafficSystem) {
    // Simple pathfinding using road network
    flow.path.clear();
    
    // Find nearest road
    if let Some(road) = find_nearest_road(current_pos, &traffic_system.road_network.roads) {
        // Follow road direction
        let road_direction = (road.end - road.start).normalize_or_zero();
        let ahead_distance = 200.0;
        
        for i in 1..=4 {
            let waypoint = current_pos + road_direction * (i as f32 * ahead_distance * 0.25);
            flow.path.push(waypoint);
        }
        
        flow.path_index = 0;
    }
}

fn get_current_target(flow: &TrafficFlow) -> Option<Vec2> {
    flow.path.get(flow.path_index).copied()
}

// === EMERGENCY RESPONSE ===

pub fn emergency_response_system(
    mut commands: Commands,
    mut traffic_system: ResMut<TrafficSystem>,
    mut alert_events: EventReader<AlertEvent>,
    sprites: Res<GameSprites>,
) {
    for alert in alert_events.read() {
        if alert.alert_level >= 3 { // High alert
            traffic_system.emergency_response_timer = 5.0; // Delay before response
            
            // Spawn emergency vehicles
            if let Some(&spawn_pos) = traffic_system.road_network.spawn_points.get(rand::random::<usize>() % traffic_system.road_network.spawn_points.len()) {                
                match rand::random::<f32>() {
                    x if x < 0.6 => {
                        spawn_emergency_vehicle(&mut commands, spawn_pos, TrafficVehicleType::PoliceCar, alert.position, &sprites);
                    },
                    _ => {
                        spawn_emergency_vehicle(&mut commands, spawn_pos, TrafficVehicleType::EmergencyAmbulance, alert.position, &sprites);
                    },
                }
            }
        }
    }
}

fn spawn_emergency_vehicle(
    commands: &mut Commands,
    spawn_pos: Vec2,
    vehicle_type: TrafficVehicleType,
    target: Vec2,
    sprites: &GameSprites,
) {
    spawn_traffic_vehicle(commands, spawn_pos, vehicle_type, sprites);
    
    // Would need to get the entity ID to set response target, but this is simplified
    info!("Emergency vehicle dispatched to {:?}", target);
}

// === VISUAL EFFECTS ===

pub fn traffic_visual_effects_system(
    mut traffic_query: Query<(&mut Sprite, &TrafficVehicle)>,
) {
    for (mut sprite, vehicle) in traffic_query.iter_mut() {
        // Brake lights
        if vehicle.brake_lights {
            let red_tint = Color::srgb(1.2, 0.8, 0.8);
            sprite.color = Color::srgb(
                sprite.color.to_srgba().red * red_tint.to_srgba().red,
                sprite.color.to_srgba().green * red_tint.to_srgba().green,
                sprite.color.to_srgba().blue * red_tint.to_srgba().blue
            );
        }
        
        // Panic effects
        if vehicle.panic_level > 0.5 {
            let flicker = (std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs_f32() * 8.0).sin() * 0.1;
            let flicker_color = Color::srgb(1.0 + flicker, 1.0, 1.0);
            sprite.color = Color::srgb(
                sprite.color.to_srgba().red * flicker_color.to_srgba().red,
                sprite.color.to_srgba().green * flicker_color.to_srgba().green,
                sprite.color.to_srgba().blue * flicker_color.to_srgba().blue
            );
        }
    }
}

// === TRAFFIC COLLISIONS ===

pub fn traffic_collision_system(
    mut collision_events: EventReader<CollisionEvent>,
    mut commands: Commands,
    traffic_query: Query<&TrafficVehicle>,
    agent_query: Query<Entity, With<Agent>>,
    mut combat_events: EventWriter<CombatEvent>,
    decal_settings: Res<DecalSettings>,
) {
    for collision_event in collision_events.read() {
        if let CollisionEvent::Started(e1, e2, _) = collision_event {
            // Vehicle hitting agent
            if let (Ok(vehicle), Ok(agent_entity)) = (traffic_query.get(*e1), agent_query.get(*e2)) {
                handle_vehicle_agent_collision(*e1, agent_entity, vehicle, &mut combat_events);
            } else if let (Ok(agent_entity), Ok(vehicle)) = (agent_query.get(*e1), traffic_query.get(*e2)) {
                handle_vehicle_agent_collision(*e2, agent_entity, vehicle, &mut combat_events);
            }
            
            // Vehicle-vehicle collisions create skid marks
            if traffic_query.get(*e1).is_ok() && traffic_query.get(*e2).is_ok() {
                // Spawn skid mark decals at collision point
                // Position would need to be calculated from transforms
                spawn_decal(
                    &mut commands,
                    Vec2::ZERO, // Would calculate actual position
                    DecalType::Tire,
                    15.0,
                    &decal_settings,
                );
            }
        }
    }
}

fn handle_vehicle_agent_collision(
    vehicle_entity: Entity,
    agent_entity: Entity,
    vehicle: &TrafficVehicle,
    combat_events: &mut EventWriter<CombatEvent>,
) {
    let damage = match vehicle.vehicle_type {
        TrafficVehicleType::Bus | TrafficVehicleType::Truck => 60.0,
        TrafficVehicleType::MilitaryConvoy => 80.0,
        TrafficVehicleType::MotorCycle => 25.0,
        _ => 40.0,
    } * (vehicle.current_speed / vehicle.max_speed).clamp(0.3, 1.0);
    
    combat_events.write(CombatEvent {
        attacker: vehicle_entity,
        target: agent_entity,
        damage,
        hit: true, // Vehicle collisions always hit
    });
}

// === MILITARY CONVOY BEHAVIOR ===

pub fn military_convoy_system(
    mut convoy_query: Query<(Entity, &mut MilitaryConvoy, &Transform, &TrafficVehicle)>,
    agent_query: Query<&Transform, With<Agent>>,
    alert_events: EventReader<AlertEvent>,
    mut combat_events: EventReader<CombatEvent>,
    mut commands: Commands,
    sprites: Res<GameSprites>,
) {
    // Check for attacks on convoys
    for combat_event in combat_events.read() {
        for (convoy_entity, mut convoy, transform, _) in convoy_query.iter_mut() {
            if combat_event.target == convoy_entity {
                convoy.alert_status = ConvoyAlertStatus::UnderAttack;
                
                // Deploy troops
                if convoy.troops_inside > 0 && matches!(convoy.alert_status, ConvoyAlertStatus::UnderAttack) {
                    let deploy_pos = transform.translation.truncate() + Vec2::new(30.0, 0.0);
                    deploy_convoy_troops(&mut commands, deploy_pos, &sprites);
                    convoy.troops_inside = convoy.troops_inside.saturating_sub(2);
                    convoy.alert_status = ConvoyAlertStatus::Deploying;
                }
            }
        }
    }
}

fn deploy_convoy_troops(commands: &mut Commands, position: Vec2, sprites: &GameSprites) {
    // Spawn 2 military enemies
    for i in 0..2 {
        let spawn_pos = position + Vec2::new(i as f32 * 20.0, (i % 2) as f32 * 15.0);
        
        let (sprite, _) = crate::core::sprites::create_enemy_sprite(sprites);

        let convoy_troop = commands.spawn_empty()
            .insert((
                sprite,
                Transform::from_translation(spawn_pos.extend(1.0)),
                Enemy,
                crate::core::factions::Faction::Military,
                Health(120.0),
                MovementSpeed(110.0),
                Morale::new(150.0, 20.0),
                Vision::new(140.0, 70.0),
                AIState::default(),
                GoapAgent::default(),
            ))
            .insert((
                WeaponState::new_from_type(&WeaponType::Rifle),
                {
                    let mut inventory = Inventory::default();
                    inventory.equipped_weapon = Some(WeaponConfig::new(WeaponType::Rifle));
                    inventory
                },
                RigidBody::Dynamic,
                Collider::ball(9.0),
                Velocity::default(),
                Damping { linear_damping: 15.0, angular_damping: 15.0 },
                CollisionGroups::new(ENEMY_GROUP, Group::ALL),
                GravityScale(0.0),
            ));
    }
    
    info!("Military convoy deployed troops at {:?}", position);
}

// === CLEANUP ===

pub fn traffic_cleanup_system(
    mut commands: Commands,
    traffic_query: Query<(Entity, &Transform, &Health), (With<TrafficVehicle>, Without<MarkedForDespawn>)>,
    camera_query: Query<&Transform, (With<Camera>, Without<TrafficVehicle>)>,
) {
    let Ok(camera_transform) = camera_query.single() else { return; };
    let camera_pos = camera_transform.translation.truncate();
    
    for (entity, transform, health) in traffic_query.iter() {
        let distance = camera_pos.distance(transform.translation.truncate());
        
        // Remove vehicles that are too far or destroyed
        if distance > 800.0 || health.0 <= 0.0 {
            commands.entity(entity).insert(MarkedForDespawn);
        }
    }
}

// === INTEGRATION SETUP ===

pub fn setup_traffic_system(mut commands: Commands) {
    commands.insert_resource(TrafficSystem::default());
    info!("Traffic system initialized with {} road segments", 
          TrafficSystem::default().road_network.roads.len());
}

