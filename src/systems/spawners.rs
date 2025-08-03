use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use bevy_light_2d::prelude::*;
use bevy::{color::palettes::css::{BLUE, YELLOW}};

use crate::core::*;
use crate::core::factions::Faction;
use crate::systems::scenes::*;
use crate::systems::pathfinding::{PathfindingObstacle};
use crate::systems::urban_simulation::*;
use crate::systems::traffic::*;
use crate::systems::scanner::{Scannable};
use crate::systems::ai::*;
use crate::systems::police::*;
use crate::systems::selection::*;
use crate::systems::power_grid::*;
use crate::systems::hacking_financial::*;
use crate::systems::explosions::*;
use crate::systems::civilian_spawn::*;
use crate::systems::world_scan::{WorldScanner};
use crate::systems::panic_spread::{PanicSpreader};

// === SHARED BUILDERS ===

fn build_sprite_transform_bundle(sprite: Sprite, position: Vec2, z: f32) -> (Sprite, Transform) {
    (sprite, Transform::from_translation(position.extend(z)))
}

fn build_basic_physics(radius: f32, group: Group, body_type: RigidBody, damping: Damping) -> impl Bundle {
    (
        body_type,
        Collider::ball(radius),
        Velocity::default(),
        damping,
        CollisionGroups::new(group, Group::ALL),
        Friction::coefficient(0.8),
        Restitution::coefficient(0.1),
        LockedAxes::ROTATION_LOCKED,
        GravityScale(0.0),
    )
}

fn build_unit_physics(radius: f32, group: Group) -> impl Bundle {
    build_basic_physics(
        radius, 
        group, 
        RigidBody::Dynamic, 
        Damping { linear_damping: 15.0, angular_damping: 15.0 }
    )
}

fn build_hackable_device(
    commands: &mut Commands,
    entity: Entity,
    device_type: DeviceType,
    network_id: Option<String>,
    power_grid: &mut Option<ResMut<PowerGrid>>,
    security_level: u8,
    hack_time: f32,
    required_tool: Option<HackTool>,
) {
    if let (Some(network_id), Some(power_grid)) = (network_id, power_grid) {
        make_hackable_networked(commands, entity, device_type, network_id, power_grid);
    } else {
        let mut hackable = Hackable::new(device_type);
        hackable.security_level = security_level;
        hackable.hack_time = hack_time;
        hackable.requires_tool = required_tool;
        commands.entity(entity).insert((hackable, DeviceState::new(device_type)));
    }
}

// === CORE ENTITY SPAWNERS ===

pub fn spawn_agent(commands: &mut Commands, pos: Vec2, level: u8, idx: usize, global_data: &GlobalData, sprites: &GameSprites) {
    info!("spawn_agent");
    spawn_agent_internal(commands, pos, level, Some(idx), global_data, sprites, false);
}

fn spawn_agent_internal(
    commands: &mut Commands, 
    pos: Vec2, 
    level: u8, 
    idx: Option<usize>, 
    global_data: &GlobalData, 
    sprites: &GameSprites,
    with_scanner: bool
) {
    let (sprite, _) = create_agent_sprite(sprites);
    let loadout = global_data.get_agent_loadout(idx.unwrap_or(0));
    let mut inventory = create_inventory_from_loadout(&loadout);
    inventory.add_currency(100 * level as u32);

    let weapon_state = create_weapon_state_from_loadout(&loadout);
    
    let mut entity_commands = commands.spawn((
        build_sprite_transform_bundle(sprite, pos, 1.0),
        Agent { experience: 0, level },
        Faction::Player,
        create_base_unit_bundle(100.0, 150.0),
        Controllable,
        Selectable { radius: 15.0 },
        Vision::new(150.0, 60.0),
        NeurovectorCapability::default(),
        inventory,
        weapon_state,
        build_unit_physics(16.0, AGENT_GROUP),
    ));

    if let Some(idx) = idx {
        entity_commands.insert(AgentIndex(idx));
    }

    if with_scanner || level >= 5 {
        let scan_level = level.min(3);
        entity_commands.insert(WorldScanner {
            scan_level,
            range: 150.0 + (scan_level as f32 * 50.0),
            energy: 100.0,
            max_energy: 100.0,
            scan_cost: 15.0 + (scan_level as f32 * 5.0),
            recharge_rate: 8.0 + (scan_level as f32 * 2.0),
            active: false,
        });
    }
}

pub fn spawn_enemy(commands: &mut Commands, pos: Vec2, patrol: Vec<Vec2>, global_data: &GlobalData, sprites: &GameSprites) {
    let (sprite, _) = create_enemy_sprite(sprites);
    let difficulty = global_data.regions[global_data.selected_region].mission_difficulty_modifier();
    let faction = random_enemy_faction();
    let weapon = select_weapon_for_faction(&faction);

    let mut inventory = Inventory::default();
    inventory.equipped_weapon = Some(WeaponConfig::new(weapon.clone()));
    let mut weapon_state = WeaponState::new_from_type(&weapon);
    weapon_state.complete_reload();

    commands.spawn((
        build_sprite_transform_bundle(sprite, pos, 1.0),
        Enemy,
        faction,
        create_base_unit_bundle(100.0 * difficulty, 100.0),
        Morale::new(100.0 * difficulty, 25.0),
        Vision::new(120.0 * difficulty, 60.0),
        Patrol::new(patrol),
        AIState::default(),
        GoapAgent::default(),
        weapon_state,
        inventory,
        build_unit_physics(9.0, ENEMY_GROUP),
        Scannable,
    ));
}

pub fn spawn_police_unit_simple(
    commands: &mut Commands,
    position: Vec2,
    unit_type: EscalationLevel,
    sprites: &GameSprites,
) -> Entity {
    let config = crate::systems::police::load_police_config();
    spawn_police_unit(commands, position, unit_type, sprites, &config)
}

pub fn spawn_police_unit(
    commands: &mut Commands,
    position: Vec2,
    unit_type: EscalationLevel,
    sprites: &GameSprites,
    config: &crate::systems::police::PoliceConfig,
) -> Entity {
    let level_config = unit_type.get_config(config);
    let (mut sprite, _) = crate::core::sprites::create_police_sprite(sprites);
    sprite.color = Color::srgba(level_config.color.0, level_config.color.1, level_config.color.2, level_config.color.3);

    let patrol_points = generate_patrol_pattern(position, unit_type, config);
    let weapon_type = match level_config.weapon.as_str() {
        "Pistol" => WeaponType::Pistol,
        "Rifle" => WeaponType::Rifle,
        "Minigun" => WeaponType::Minigun,
        "Flamethrower" => WeaponType::Flamethrower,
        _ => WeaponType::Pistol,
    };

    let mut inventory = Inventory::default();
    inventory.equipped_weapon = Some(WeaponConfig::new(weapon_type.clone()));
    let mut ai_state = AIState::default();
    ai_state.use_goap = true;

    commands.spawn((
        build_sprite_transform_bundle(sprite, position, 1.0),
        Enemy,
        Police { response_level: unit_type as u8 },
        Faction::Police,
        Health(level_config.health),
        Morale::new(level_config.health * 1.5, 20.0),
        MovementSpeed(level_config.speed),
        Vision::new(level_config.vision, 60.0),
        Patrol::new(patrol_points),
        ai_state,
        GoapAgent::default(),
        WeaponState::new_from_type(&weapon_type),
        inventory,
        build_basic_physics(9.0, CIVILIAN_GROUP, RigidBody::Dynamic, Damping { linear_damping: 10.0, angular_damping: 10.0 }),
        Scannable,
    )).id()
}

// === CIVILIAN SPAWNERS ===

fn spawn_civilian_base<'a>(
    commands: &'a mut Commands,
    position: Vec2,
    sprites: &GameSprites,
    health: f32,
    morale_base: f32,
    panic_threshold: f32,
    speed: f32,
) -> EntityCommands<'a> {
    let (sprite, _) = create_civilian_sprite(sprites);
    
    commands.spawn((
        build_sprite_transform_bundle(sprite, position, 1.0),
        Civilian,
        Faction::Civilian,
        Health(health),
        Morale::new(morale_base, panic_threshold),
        PanicSpreader::default(),
        MovementSpeed(speed),
        Controllable,
        NeurovectorTarget,
        build_unit_physics(7.5, CIVILIAN_GROUP),
        Scannable,
    ))
}

pub fn spawn_civilian(commands: &mut Commands, position: Vec2, sprites: &GameSprites) {
    spawn_civilian_base(commands, position, sprites, 50.0, 80.0, 40.0, 140.0)
        .insert(CivilianWander::new(position));
}

pub fn spawn_civilian_with_config(commands: &mut Commands, pos: Vec2, sprites: &GameSprites, config: &GameConfig) {
    info!("spawn_civilian_with_config");
    spawn_civilian_base(commands, pos, sprites, 50.0, config.civilians.base_morale, config.civilians.panic_threshold, config.civilians.movement_speed);
}

pub fn spawn_urban_civilian(
    commands: &mut Commands,
    position: Vec2,
    sprites: &GameSprites,
    urban_areas: &UrbanAreas,
) {
    let crowd_influence = 0.3 + rand::random::<f32>() * 0.4;
    let panic_threshold = 20.0 + rand::random::<f32>() * 40.0;
    let daily_state = match rand::random::<f32>() {
        x if x < 0.4 => DailyState::GoingToWork,
        x if x < 0.6 => DailyState::Shopping,
        x if x < 0.8 => DailyState::GoingHome,
        _ => DailyState::Idle,
    };

    spawn_civilian_base(commands, position, sprites, 50.0, 80.0, panic_threshold, 80.0)
        .insert(UrbanCivilian {
            daily_state,
            state_timer: rand::random::<f32>() * 10.0,
            next_destination: pick_destination_for_state(daily_state, urban_areas),
            crowd_influence,
            panic_threshold,
            movement_urgency: 0.0,
        });
}

pub fn spawn_original_urban_civilian(commands: &mut Commands, pos: Vec2, sprites: &GameSprites) {
    info!("spawn_urban_civilian");
    let crowd_influence = 0.2 + rand::random::<f32>() * 0.6;
    let panic_threshold = 15.0 + rand::random::<f32>() * 50.0;

    spawn_civilian_base(commands, pos, sprites, 50.0, 80.0, panic_threshold, 80.0 + rand::random::<f32>() * 40.0)
        .insert(UrbanCivilian {
            daily_state: random_daily_state(),
            state_timer: rand::random::<f32>() * 15.0,
            next_destination: None,
            crowd_influence,
            panic_threshold,
            movement_urgency: 0.0,
        });
}

// === VEHICLE SPAWNERS ===

struct VehicleConfig {
    color: Color,
    size: Vec2,
    health: f32,
    max_speed: Option<f32>,
    acceleration: Option<f32>,
    brake_force: Option<f32>,
}

impl VehicleConfig {
    fn for_vehicle_type(vehicle_type: &VehicleType) -> Self {
        match vehicle_type {
            VehicleType::CivilianCar => Self { color: Color::srgb(0.6, 0.6, 0.8), size: Vec2::new(40.0, 20.0), health: 60.0, max_speed: None, acceleration: None, brake_force: None },
            VehicleType::PoliceCar => Self { color: Color::srgb(0.2, 0.2, 0.8), size: Vec2::new(40.0, 20.0), health: 100.0, max_speed: None, acceleration: None, brake_force: None },
            VehicleType::ElectricCar => Self { color: Color::srgb(0.6, 0.6, 0.9), size: Vec2::new(40.0, 20.0), health: 60.0, max_speed: None, acceleration: None, brake_force: None },
            VehicleType::APC => Self { color: Color::srgb(0.4, 0.6, 0.4), size: Vec2::new(50.0, 30.0), health: 200.0, max_speed: None, acceleration: None, brake_force: None },
            VehicleType::VTOL => Self { color: Color::srgb(0.3, 0.3, 0.3), size: Vec2::new(60.0, 40.0), health: 150.0, max_speed: None, acceleration: None, brake_force: None },
            VehicleType::Tank => Self { color: Color::srgb(0.5, 0.5, 0.2), size: Vec2::new(60.0, 35.0), health: 300.0, max_speed: None, acceleration: None, brake_force: None },
            VehicleType::Truck | VehicleType::FuelTruck => Self { color: Color::srgb(0.4, 0.6, 0.4), size: Vec2::new(50.0, 30.0), health: 120.0, max_speed: None, acceleration: None, brake_force: None },
        }
    }

    fn for_traffic_type(traffic_type: &TrafficVehicleType) -> Self {
        match traffic_type {
            TrafficVehicleType::CivilianCar => Self { color: Color::srgb(0.6, 0.6, 0.8), size: Vec2::new(32.0, 16.0), health: 60.0, max_speed: Some(120.0), acceleration: Some(200.0), brake_force: Some(400.0) },
            TrafficVehicleType::Bus => Self { color: Color::srgb(0.8, 0.8, 0.2), size: Vec2::new(48.0, 20.0), health: 150.0, max_speed: Some(80.0), acceleration: Some(200.0), brake_force: Some(400.0) },
            TrafficVehicleType::Truck => Self { color: Color::srgb(0.5, 0.3, 0.2), size: Vec2::new(40.0, 18.0), health: 120.0, max_speed: Some(100.0), acceleration: Some(200.0), brake_force: Some(400.0) },
            TrafficVehicleType::EmergencyAmbulance => Self { color: Color::srgb(1.0, 1.0, 1.0), size: Vec2::new(36.0, 18.0), health: 100.0, max_speed: Some(150.0), acceleration: Some(200.0), brake_force: Some(400.0) },
            TrafficVehicleType::PoliceCar => Self { color: Color::srgb(0.2, 0.2, 0.8), size: Vec2::new(34.0, 16.0), health: 100.0, max_speed: Some(140.0), acceleration: Some(200.0), brake_force: Some(400.0) },
            TrafficVehicleType::MilitaryConvoy => Self { color: Color::srgb(0.3, 0.5, 0.3), size: Vec2::new(44.0, 20.0), health: 200.0, max_speed: Some(110.0), acceleration: Some(200.0), brake_force: Some(400.0) },
            TrafficVehicleType::MotorCycle => Self { color: Color::srgb(0.8, 0.2, 0.2), size: Vec2::new(12.0, 8.0), health: 40.0, max_speed: Some(160.0), acceleration: Some(200.0), brake_force: Some(400.0) },
        }
    }
}

pub fn spawn_vehicle(
    commands: &mut Commands,
    position: Vec2,
    vehicle_type: VehicleType,
    sprites: &GameSprites,
) {
    let config = VehicleConfig::for_vehicle_type(&vehicle_type);
    let sprite = Sprite {
        color: config.color,
        custom_size: Some(config.size),
        ..default()
    };

    commands.spawn((
        build_sprite_transform_bundle(sprite, position, 0.8),
        Vehicle::new(vehicle_type),
        Health(config.health),
        RigidBody::Fixed,
        Collider::cuboid(config.size.x / 2.0, config.size.y / 2.0),
        Scannable,
    ));
}

pub fn spawn_traffic_vehicle(
    commands: &mut Commands,
    position: Vec2,
    vehicle_type: TrafficVehicleType,
    sprites: &GameSprites,
) {
    let config = VehicleConfig::for_traffic_type(&vehicle_type);
    let sprite = Sprite {
        color: config.color,
        custom_size: Some(config.size),
        ..default()
    };

    let mut entity_commands = commands.spawn((
        build_sprite_transform_bundle(sprite, position, 0.9),
        TrafficVehicle {
            vehicle_type: vehicle_type.clone(),
            max_speed: config.max_speed.unwrap_or(100.0),
            current_speed: 0.0,
            acceleration: config.acceleration.unwrap_or(200.0),
            brake_force: config.brake_force.unwrap_or(400.0),
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
        Health(config.health),
        Vehicle::new(match vehicle_type {
            TrafficVehicleType::CivilianCar => VehicleType::CivilianCar,
            TrafficVehicleType::Bus | TrafficVehicleType::Truck => VehicleType::Truck,
            TrafficVehicleType::PoliceCar => VehicleType::PoliceCar,
            TrafficVehicleType::MilitaryConvoy => VehicleType::APC,
            _ => VehicleType::CivilianCar,
        }),
        build_basic_physics(config.size.x * 0.25, VEHICLE_GROUP, RigidBody::Dynamic, Damping { linear_damping: 5.0, angular_damping: 10.0 }),
        Scannable,
    ));

    // Add special components based on type
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

// === TERMINAL & DEVICE SPAWNERS ===

pub fn spawn_terminal(commands: &mut Commands, pos: Vec2, terminal_type: &str, sprites: &GameSprites) {
    let (sprite, _) = create_terminal_sprite(sprites, &parse_terminal_type(terminal_type));

    commands.spawn((
        build_sprite_transform_bundle(sprite, pos, 1.0),
        Terminal {
            terminal_type: parse_terminal_type(terminal_type),
            range: 30.0,
            accessed: false
        },
        Selectable { radius: 15.0 },
        RigidBody::Fixed,
        Collider::ball(12.0),
        CollisionGroups::new(TERMINAL_GROUP, Group::ALL),
        Scannable,
        PathfindingObstacle {
            radius: 12.0,
            blocks_movement: true,
        },
    ));
}

pub fn spawn_atm(
    commands: &mut Commands,
    position: Vec2,
    bank_id: String,
    network_id: Option<String>,
    power_grid: &mut Option<ResMut<PowerGrid>>,
) -> Entity {
    let sprite = Sprite {
        color: Color::srgb(0.3, 0.5, 0.7),
        custom_size: Some(Vec2::new(16.0, 24.0)),
        ..default()
    };

    let entity = commands.spawn((
        build_sprite_transform_bundle(sprite, position, 1.0),
        ATM {
            bank_id: bank_id.clone(),
            max_withdrawal: 5000,
            current_balance: 50000,
            requires_account_data: true,
        },
        RigidBody::Fixed,
        Collider::cuboid(8.0, 12.0),
        CollisionGroups::new(TERMINAL_GROUP, Group::ALL),
        Selectable { radius: 20.0 },
        Scannable,
        PathfindingObstacle {
            radius: 12.0,
            blocks_movement: true,
        },
    )).id();

    build_hackable_device(commands, entity, DeviceType::Terminal, network_id, power_grid, 3, 6.0, Some(HackTool::AdvancedHacker));
    entity
}

pub fn spawn_billboard(
    commands: &mut Commands,
    position: Vec2,
    network_id: Option<String>,
    power_grid: &mut Option<ResMut<PowerGrid>>,
) -> Entity {
    let sprite = Sprite {
        color: Color::srgb(0.8, 0.6, 0.2),
        custom_size: Some(Vec2::new(40.0, 20.0)),
        ..default()
    };

    let entity = commands.spawn((
        build_sprite_transform_bundle(sprite, position, 1.0),
        Billboard {
            influence_radius: 100.0,
            persuasion_bonus: 0.3,
            active: true,
        },
        RigidBody::Fixed,
        Collider::cuboid(20.0, 10.0),
        CollisionGroups::new(TERMINAL_GROUP, Group::ALL),
        Selectable { radius: 25.0 },
        Scannable,
        PathfindingObstacle {
            radius: 22.0,
            blocks_movement: true,
        },
    )).id();

    build_hackable_device(commands, entity, DeviceType::Terminal, network_id, power_grid, 2, 3.0, Some(HackTool::BasicHacker));
    entity
}

// === POWER GRID SPAWNING ===

pub fn spawn_power_station(
    commands: &mut Commands,
    position: Vec2,
    network_id: String,
    power_grid: &mut ResMut<PowerGrid>,
) -> Entity {
    let sprite = Sprite {
        color: Color::srgb(0.8, 0.8, 0.2),
        custom_size: Some(Vec2::new(60.0, 40.0)),
        ..default()
    };

    let entity = commands.spawn((
        build_sprite_transform_bundle(sprite, position, 1.0),
        PowerStation {
            network_id: network_id.clone(),
            max_capacity: 100,
            current_load: 0,
        },
    )).id();

    make_hackable_networked(commands, entity, DeviceType::PowerStation, network_id.clone(), power_grid);

    let network = power_grid.networks.entry(network_id.clone())
        .or_insert_with(|| PowerNetwork::new(network_id));
    network.power_sources.insert(entity);

    entity
}

#[derive(Component)]
struct YellowLight;

#[derive(Component)]
struct BlueLight;

pub fn spawn_street_light(
    commands: &mut Commands,
    position: Vec2,
    network_id: String,
    power_grid: &mut ResMut<PowerGrid>,
) -> Entity {
    let sprite = Sprite {
        color: Color::srgb(0.9, 0.9, 0.7),
        custom_size: Some(Vec2::new(8.0, 24.0)),
        ..default()
    };

    let entity = commands.spawn((
        build_sprite_transform_bundle(sprite, position, 1.0),
        StreetLight { brightness: 1.0 },
        PointLight2d {
            intensity: 3.0,
            radius: 100.0,
            falloff: 1.0,
            cast_shadows: true,
            color: Color::Srgba(YELLOW),
        },

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
    let sprite = Sprite {
        color: Color::srgb(0.2, 0.8, 0.2),
        custom_size: Some(Vec2::new(12.0, 32.0)),
        ..default()
    };

    let entity = commands.spawn((
        build_sprite_transform_bundle(sprite, position, 1.0),
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
    let sprite = Sprite {
        color: Color::srgb(0.3, 0.3, 0.3),
        custom_size: Some(Vec2::new(16.0, 12.0)),
        ..default()
    };

    let entity = commands.spawn((
        build_sprite_transform_bundle(sprite, position, 1.0),
        SecurityCamera {
            detection_range: 120.0,
            fov_angle: 60.0,
            direction: Vec2::X,
            active: true,
        },
        Vision::new(120.0, 60.0),
    )).id();

    build_hackable_device(commands, entity, DeviceType::Camera, network_id, power_grid, 2, 4.0, Some(HackTool::BasicHacker));
    entity
}

pub fn spawn_automated_turret(
    commands: &mut Commands,
    position: Vec2,
    network_id: Option<String>,
    power_grid: &mut Option<ResMut<PowerGrid>>,
) -> Entity {
    let sprite = Sprite {
        color: Color::srgb(0.6, 0.2, 0.2),
        custom_size: Some(Vec2::new(20.0, 20.0)),
        ..default()
    };

    let entity = commands.spawn((
        build_sprite_transform_bundle(sprite, position, 1.0),
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
    let sprite = Sprite {
        color: Color::srgb(0.4, 0.4, 0.6),
        custom_size: Some(Vec2::new(8.0, 32.0)),
        ..default()
    };

    let entity = commands.spawn((
        build_sprite_transform_bundle(sprite, position, 1.0),
        SecurityDoor {
            locked: true,
            access_level: 2,
        },
        RigidBody::Fixed,
        Collider::cuboid(4.0, 16.0),
    )).id();

    build_hackable_device(commands, entity, DeviceType::Door, network_id, power_grid, 2, 5.0, Some(HackTool::BasicHacker));
    entity
}

// === EXPLOSIVE & MISC SPAWNERS ===

pub fn spawn_time_bomb(
    commands: &mut Commands,
    position: Vec2,
    timer: f32,
    damage: f32,
    radius: f32,
) -> Entity {
    let sprite = Sprite {
        color: Color::srgb(0.8, 0.2, 0.2),
        custom_size: Some(Vec2::new(12.0, 8.0)),
        ..default()
    };

    commands.spawn((
        build_sprite_transform_bundle(sprite, position, 1.0),
        TimeBomb {
            timer,
            damage,
            radius,
            armed: true,
        },
    )).id()
}

// === RESEARCH & MISSION SPAWNERS ===

pub fn spawn_scientists_in_mission(
    mut commands: Commands,
    sprites: Res<GameSprites>,
    global_data: Res<GlobalData>,
    scientist_query: Query<Entity, With<Scientist>>,
) {
    let existing_count = scientist_query.iter().count();
    if existing_count >= 3 {
        return;
    }

    let spawn_count = if rand::random::<f32>() < 0.6 { 1 } else { 2 };

    for _ in 0..spawn_count {
        let position = Vec2::new(
            (rand::random::<f32>() - 0.5) * 400.0,
            (rand::random::<f32>() - 0.5) * 400.0,
        );

        let specialization = match rand::random::<f32>() {
            x if x < 0.25 => ResearchCategory::Weapons,
            x if x < 0.5 => ResearchCategory::Equipment,
            x if x < 0.75 => ResearchCategory::Cybernetics,
            _ => ResearchCategory::Intelligence,
        };

        spawn_scientist_npc(&mut commands, position, specialization, &sprites);
    }
}

pub fn spawn_random_research_content(
    commands: &mut Commands,
    sprites: &GameSprites,
    global_data: &GlobalData,
) {
    for i in 0..2 {
        let position = Vec2::new(
            200.0 + i as f32 * 300.0,
            (rand::random::<f32>() - 0.5) * 200.0,
        );

        let faction = match rand::random::<f32>() {
            x if x < 0.4 => Faction::Corporate,
            x if x < 0.8 => Faction::Syndicate,
            _ => Faction::Police,
        };

        spawn_research_facility(
            commands,
            position,
            faction,
            2 + rand::random::<u32>() % 4,
            vec![
                "advanced_magazines".to_string(),
                "tech_interface".to_string(),
                "combat_enhancers".to_string(),
            ],
        );
    }
}

struct MissionResearchConfig {
    facility_pos: Vec2,
    faction: Faction,
    security_level: u32,
    projects: Vec<String>,
    scientists: Vec<(Vec2, ResearchCategory)>,
}

pub fn spawn_research_content_in_scene(
    commands: &mut Commands,
    sprites: &GameSprites,
    scene_name: &str,
) {
    let config = match scene_name {
        "mission_corporate" => MissionResearchConfig {
            facility_pos: Vec2::new(300.0, 150.0),
            faction: Faction::Corporate,
            security_level: 4,
            projects: vec!["tech_interface".to_string(), "quantum_encryption".to_string()],
            scientists: vec![
                (Vec2::new(250.0, 100.0), ResearchCategory::Intelligence),
                (Vec2::new(280.0, 100.0), ResearchCategory::Intelligence),
                (Vec2::new(310.0, 100.0), ResearchCategory::Intelligence),
            ],
        },
        "mission_syndicate" => MissionResearchConfig {
            facility_pos: Vec2::new(-200.0, -100.0),
            faction: Faction::Syndicate,
            security_level: 3,
            projects: vec!["heavy_weapons".to_string(), "plasma_weapons".to_string()],
            scientists: vec![
                (Vec2::new(-150.0, -80.0), ResearchCategory::Weapons),
                (Vec2::new(-180.0, -120.0), ResearchCategory::Cybernetics),
            ],
        },
        "mission_underground" => MissionResearchConfig {
            facility_pos: Vec2::new(100.0, -200.0),
            faction: Faction::Underground,
            security_level: 2,
            projects: vec!["infiltration_kit".to_string(), "neural_interface".to_string()],
            scientists: vec![
                (Vec2::new(120.0, -180.0), ResearchCategory::Equipment),
            ],
        },
        _ => MissionResearchConfig {
            facility_pos: Vec2::new(200.0, 100.0),
            faction: Faction::Corporate,
            security_level: 2,
            projects: vec!["basic_research".to_string()],
            scientists: vec![
                (Vec2::new(200.0, 100.0), ResearchCategory::Equipment),
            ],
        },
    };

    spawn_research_facility(
        commands,
        config.facility_pos,
        config.faction,
        config.security_level,
        config.projects,
    );

    for (pos, category) in config.scientists {
        spawn_scientist_npc(commands, pos, category, sprites);
    }
}

// === HELPER FUNCTIONS ===

pub fn create_base_unit_bundle(health: f32, speed: f32) -> impl Bundle {
    (Health(health), MovementSpeed(speed))
}

pub fn create_physics_bundle(radius: f32, group: Group) -> impl Bundle {
    build_unit_physics(radius, group)
}

pub fn create_inventory_from_loadout(loadout: &AgentLoadout) -> Inventory {
    let mut inventory = Inventory::default();
    for weapon_config in &loadout.weapon_configs {
        inventory.add_weapon_config(weapon_config.clone());
    }
    if let Some(weapon_config) = loadout.weapon_configs.get(loadout.equipped_weapon_idx) {
        inventory.equipped_weapon = Some(weapon_config.clone());
    }
    for tool in &loadout.tools {
        inventory.add_tool(tool.clone());
    }
    for cybernetic in &loadout.cybernetics {
        inventory.add_cybernetic(cybernetic.clone());
    }
    inventory
}

pub fn create_weapon_state_from_loadout(loadout: &AgentLoadout) -> WeaponState {
    if let Some(weapon_config) = loadout.weapon_configs.get(loadout.equipped_weapon_idx) {
        let mut state = WeaponState::new_from_type(&weapon_config.base_weapon);
        state.apply_attachment_modifiers(weapon_config);
        state
    } else {
        WeaponState::default()
    }
}

// Backwards compatibility functions
fn spawn_agent_with_index(commands: &mut Commands, pos: Vec2, level: u8, idx: usize, global_data: &GlobalData, sprites: &GameSprites) {
    info!("spawn_agent_with_index");
    spawn_agent_internal(commands, pos, level, Some(idx), global_data, sprites, true);
}