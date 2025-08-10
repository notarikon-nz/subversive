use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use bevy_light_2d::prelude::*;
use bevy::color::palettes::css::{YELLOW, GREEN};

use crate::core::*;
use crate::core::factions::Faction;
use crate::systems::scenes::*;
use crate::systems::pathfinding::PathfindingObstacle;
use crate::systems::traffic::*;
use crate::systems::scanner::Scannable;
use crate::systems::ai::*;
use crate::systems::urban_security::*;
use crate::systems::selection::*;
use crate::systems::power_grid::*;
use crate::systems::hacking_financial::*;
use crate::systems::explosions::*;
use crate::systems::world_scan::WorldScanner;

// === CONSTANTS ===
const DEFAULT_DAMPING: Damping = Damping { linear_damping: 15.0, angular_damping: 15.0 };
const VEHICLE_DAMPING: Damping = Damping { linear_damping: 5.0, angular_damping: 10.0 };
const CIVILIAN_DAMPING: Damping = Damping { linear_damping: 10.0, angular_damping: 10.0 };

const AGENT_RADIUS: f32 = 16.0;
const ENEMY_RADIUS: f32 = 9.0;
const CIVILIAN_RADIUS: f32 = 7.5;
const TERMINAL_RADIUS: f32 = 12.0;

const LIGHT_OCCLUDER_SIZE: f32 = 32.0;
const DEFAULT_VISION_FOV: f32 = 60.0;
const DEFAULT_FRICTION: f32 = 0.8;
const DEFAULT_RESTITUTION: f32 = 0.1;

const SCANNER_BASE_RANGE: f32 = 150.0;
const SCANNER_RANGE_PER_LEVEL: f32 = 50.0;
const SCANNER_BASE_COST: f32 = 15.0;
const SCANNER_COST_PER_LEVEL: f32 = 5.0;
const SCANNER_BASE_RECHARGE: f32 = 8.0;
const SCANNER_RECHARGE_PER_LEVEL: f32 = 2.0;

// === SHARED BUILDERS ===

fn sprite_bundle(color: Color, size: Vec2, pos: Vec2, z: f32) -> (Sprite, Transform) {
    (
        Sprite { color, custom_size: Some(size), ..default() },
        Transform::from_translation(pos.extend(z))
    )
}

fn physics_bundle(radius: f32, group: Group, body: RigidBody, damping: Damping) -> impl Bundle {
    (
        body,
        Collider::ball(radius),
        Velocity::default(),
        damping,
        CollisionGroups::new(group, Group::ALL),
        Friction::coefficient(DEFAULT_FRICTION),
        Restitution::coefficient(DEFAULT_RESTITUTION),
        LockedAxes::ROTATION_LOCKED,
        GravityScale(0.0),
    )
}

fn unit_physics(radius: f32, group: Group) -> impl Bundle {
    physics_bundle(radius, group, RigidBody::Dynamic, DEFAULT_DAMPING)
}

fn hackable_device(
    commands: &mut Commands,
    entity: Entity,
    device_type: DeviceType,
    network_id: Option<String>,
    power_grid: &mut Option<ResMut<PowerGrid>>,
    security: u8,
    hack_time: f32,
    tool: Option<HackTool>,
) {
    match (network_id, power_grid) {
        (Some(id), Some(grid)) => make_hackable_networked(commands, entity, device_type, id, grid),
        _ => {
            let mut hackable = Hackable::new(device_type);
            hackable.security_level = security;
            hackable.hack_time = hack_time;
            hackable.requires_tool = tool;
            commands.entity(entity).insert((hackable, DeviceState::new(device_type)));
        }
    }
}

fn base_unit_components(health: f32, speed: f32) -> impl Bundle {
    (Health(health), MovementSpeed(speed))
}

fn light_occluder() -> LightOccluder2d {
    LightOccluder2d {
        shape: LightOccluder2dShape::Rectangle {
            half_size: Vec2::splat(LIGHT_OCCLUDER_SIZE),
        },
    }
}

// === AGENT SPAWNING ===

pub fn spawn_agent(
    commands: &mut Commands, 
    pos: Vec2, 
    level: u8, 
    idx: usize, 
    global_data: &GlobalData, 
    sprites: &GameSprites
) {
    info!("spawn_agent");
    let (sprite, _) = create_agent_sprite(sprites);
    let loadout = global_data.get_agent_loadout(idx);
    
    let mut inventory = build_inventory(&loadout);
    inventory.add_currency(100 * level as u32);
    
    let mut entity_cmd = commands.spawn((
        sprite_bundle(sprite.color, sprite.custom_size.unwrap_or(Vec2::splat(32.0)), pos, 1.0),
        Agent { experience: 0, level },
        AgentIndex(idx),
        Faction::Player,
        base_unit_components(100.0, 150.0),
        Controllable,
        Selectable { radius: 15.0 },
        Vision::new(150.0, DEFAULT_VISION_FOV),
        NeurovectorCapability::default(),
        inventory,
        build_weapon_state(&loadout),
        unit_physics(AGENT_RADIUS, AGENT_GROUP),
        light_occluder(),
    ));
    
    // Add scanner for high-level agents
    if level >= 5 {
        entity_cmd.insert(build_scanner(level.min(3)));
    }
}

fn build_scanner(level: u8) -> WorldScanner {
    WorldScanner {
        scan_level: level,
        range: SCANNER_BASE_RANGE + (level as f32 * SCANNER_RANGE_PER_LEVEL),
        energy: 100.0,
        max_energy: 100.0,
        scan_cost: SCANNER_BASE_COST + (level as f32 * SCANNER_COST_PER_LEVEL),
        recharge_rate: SCANNER_BASE_RECHARGE + (level as f32 * SCANNER_RECHARGE_PER_LEVEL),
        active: false,
    }
}

fn build_inventory(loadout: &AgentLoadout) -> Inventory {
    let mut inv = Inventory::default();
    
    for weapon in &loadout.weapon_configs {
        inv.add_weapon_config(weapon.clone());
    }
    
    if let Some(weapon) = loadout.weapon_configs.get(loadout.equipped_weapon_idx) {
        inv.equipped_weapon = Some(weapon.clone());
    }
    
    for tool in &loadout.tools {
        inv.add_tool(tool.clone());
    }
    
    for cyber in &loadout.cybernetics {
        inv.add_cybernetic(cyber.clone());
    }
    
    inv
}

fn build_weapon_state(loadout: &AgentLoadout) -> WeaponState {
    loadout.weapon_configs
        .get(loadout.equipped_weapon_idx)
        .map(|config| {
            let mut state = WeaponState::new_from_type(&config.base_weapon);
            state.apply_attachment_modifiers(config);
            state
        })
        .unwrap_or_default()
}

// === ENEMY SPAWNING ===

pub fn spawn_enemy(
    commands: &mut Commands, 
    pos: Vec2, 
    patrol: Vec<Vec2>, 
    global_data: &GlobalData, 
    sprites: &GameSprites
) {
    let (sprite, _) = create_enemy_sprite(sprites);
    let difficulty = global_data.regions[global_data.selected_region].mission_difficulty_modifier();
    let faction = random_enemy_faction();
    let weapon = select_weapon_for_faction(&faction);
    
    let mut inventory = Inventory::default();
    inventory.equipped_weapon = Some(WeaponConfig::new(weapon.clone()));
    
    let mut weapon_state = WeaponState::new_from_type(&weapon);
    weapon_state.complete_reload();
    
    commands.spawn((
        sprite_bundle(sprite.color, sprite.custom_size.unwrap_or(Vec2::splat(24.0)), pos, 1.0),
        Enemy,
        faction,
        base_unit_components(100.0 * difficulty, 100.0),
        Morale::new(100.0 * difficulty, 25.0),
        Vision::new(120.0 * difficulty, DEFAULT_VISION_FOV),
        Patrol::new(patrol),
        AIState::default(),
        GoapAgent::default(),
        weapon_state,
        inventory,
        unit_physics(ENEMY_RADIUS, ENEMY_GROUP),
        Scannable,
    ));
}

// === POLICE SPAWNING ===

pub fn spawn_police_unit_simple(
    commands: &mut Commands,
    position: Vec2,
    unit_type: EscalationLevel,
    sprites: &GameSprites,
) -> Entity {
    let config = load_urban_config();
    spawn_police_unit(commands, position, unit_type, sprites, &config)
}

pub fn spawn_police_unit(
    commands: &mut Commands,
    position: Vec2,
    unit_type: EscalationLevel,
    sprites: &GameSprites,
    config: &UrbanConfig,
) -> Entity {
    let level_config = unit_type.get_config(config);
    let (mut sprite, _) = create_police_sprite(sprites);
    sprite.color = Color::srgba(level_config.color.0, level_config.color.1, level_config.color.2, level_config.color.3);
    
    let patrol = generate_patrol_pattern(position, unit_type, config);
    let weapon_type = parse_weapon_type(&level_config.weapon);
    
    let mut inventory = Inventory::default();
    inventory.equipped_weapon = Some(WeaponConfig::new(weapon_type.clone()));
    
    let mut ai_state = AIState::default();
    ai_state.use_goap = true;
    
    commands.spawn((
        sprite_bundle(sprite.color, sprite.custom_size.unwrap_or(Vec2::splat(24.0)), position, 1.0),
        Enemy,
        Police { response_level: unit_type as u8 },
        Faction::Police,
        base_unit_components(level_config.health, level_config.speed),
        Morale::new(level_config.health * 1.5, 20.0),
        Vision::new(level_config.vision, DEFAULT_VISION_FOV),
        Patrol::new(patrol),
        ai_state,
        GoapAgent::default(),
        WeaponState::new_from_type(&weapon_type),
        inventory,
        physics_bundle(ENEMY_RADIUS, CIVILIAN_GROUP, RigidBody::Dynamic, CIVILIAN_DAMPING),
        Scannable,
    )).id()
}

fn parse_weapon_type(weapon: &str) -> WeaponType {
    match weapon {
        "pistol" => WeaponType::Pistol,
        "rifle" => WeaponType::Rifle,
        "minigun" => WeaponType::Minigun,
        "flamethrower" => WeaponType::Flamethrower,
        _ => WeaponType::Pistol,
    }
}

// === CIVILIAN SPAWNING ===

fn spawn_civilian_base<'a>(
    commands: &'a mut Commands,
    position: Vec2,
    sprites: &GameSprites,
    health: f32,
    morale: f32,
    panic: f32,
    speed: f32,
) -> EntityCommands<'a> {
    let (sprite, _) = create_civilian_sprite(sprites);
    
    commands.spawn((
        sprite_bundle(sprite.color, sprite.custom_size.unwrap_or(Vec2::splat(20.0)), position, 1.0),
        Civilian,
        Faction::Civilian,
        base_unit_components(health, speed),
        Morale::new(morale, panic),
        Controllable,
        NeurovectorTarget,
        unit_physics(CIVILIAN_RADIUS, CIVILIAN_GROUP),
        Scannable,
    ))
}

pub fn spawn_civilian(commands: &mut Commands, position: Vec2, sprites: &GameSprites) {
    let crowd_influence = 0.3 + rand::random::<f32>() * 0.4;
    let panic_threshold = 20.0 + rand::random::<f32>() * 40.0;
    let daily_state = random_daily_state();
    
    spawn_civilian_base(commands, position, sprites, 50.0, 80.0, 40.0, 140.0)
        .insert(UrbanCivilian {
            daily_state,
            state_timer: rand::random::<f32>() * 10.0,
            next_destination: None,
            crowd_influence,
            panic_threshold,
            movement_urgency: 0.0,
            home_position: position,
        });
}

pub fn spawn_civilian_with_config(
    commands: &mut Commands, 
    pos: Vec2, 
    sprites: &GameSprites, 
    config: &GameConfig
) {
    info!("spawn_civilian_with_config");
    spawn_civilian_base(
        commands, pos, sprites, 50.0, 
        config.civilians.base_morale, 
        config.civilians.panic_threshold, 
        config.civilians.movement_speed
    );
}

pub fn spawn_urban_civilian(
    commands: &mut Commands,
    position: Vec2,
    sprites: &GameSprites,
    urban_areas: &UrbanSecurity,
) {
    let crowd_influence = 0.3 + rand::random::<f32>() * 0.4;
    let panic_threshold = 20.0 + rand::random::<f32>() * 40.0;
    let daily_state = random_daily_state();
    
    spawn_civilian_base(commands, position, sprites, 50.0, 80.0, panic_threshold, 80.0)
        .insert(UrbanCivilian {
            daily_state,
            state_timer: rand::random::<f32>() * 10.0,
            next_destination: pick_destination_for_state(daily_state, urban_areas),
            crowd_influence,
            panic_threshold,
            movement_urgency: 0.0,
            home_position: position,
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
            home_position: pos,
        });
}

// === VEHICLE SPAWNING ===

#[derive(Clone)]
struct VehicleSpec {
    color: Color,
    size: Vec2,
    health: f32,
    speed: f32,
    accel: f32,
    brake: f32,
}

impl VehicleSpec {
    const CAR: Self = Self {
        color: Color::srgb(0.6, 0.6, 0.8),
        size: Vec2::new(40.0, 20.0),
        health: 60.0,
        speed: 120.0,
        accel: 200.0,
        brake: 400.0,
    };
    
    const POLICE: Self = Self {
        color: Color::srgb(0.2, 0.2, 0.8),
        size: Vec2::new(40.0, 20.0),
        health: 100.0,
        speed: 140.0,
        accel: 200.0,
        brake: 400.0,
    };
    
    const TRUCK: Self = Self {
        color: Color::srgb(0.4, 0.6, 0.4),
        size: Vec2::new(50.0, 30.0),
        health: 120.0,
        speed: 100.0,
        accel: 200.0,
        brake: 400.0,
    };
    
    fn for_vehicle(vtype: &VehicleType) -> Self {
        match vtype {
            VehicleType::CivilianCar | VehicleType::ElectricCar => Self::CAR,
            VehicleType::PoliceCar => Self::POLICE,
            VehicleType::Truck | VehicleType::FuelTruck | VehicleType::APC => Self::TRUCK,
            VehicleType::VTOL => Self { size: Vec2::new(60.0, 40.0), health: 150.0, ..Self::TRUCK },
            VehicleType::Tank => Self { size: Vec2::new(60.0, 35.0), health: 300.0, ..Self::TRUCK },
        }
    }
    
    fn for_traffic(ttype: &TrafficVehicleType) -> Self {
        match ttype {
            TrafficVehicleType::CivilianCar => Self { size: Vec2::new(32.0, 16.0), ..Self::CAR },
            TrafficVehicleType::PoliceCar => Self { size: Vec2::new(34.0, 16.0), ..Self::POLICE },
            TrafficVehicleType::Bus => Self {
                color: Color::srgb(0.8, 0.8, 0.2),
                size: Vec2::new(48.0, 20.0),
                health: 150.0,
                speed: 80.0,
                ..Self::CAR
            },
            TrafficVehicleType::Truck => Self { size: Vec2::new(40.0, 18.0), ..Self::TRUCK },
            TrafficVehicleType::EmergencyAmbulance => Self {
                color: Color::srgb(1.0, 1.0, 1.0),
                size: Vec2::new(36.0, 18.0),
                speed: 150.0,
                ..Self::CAR
            },
            TrafficVehicleType::MilitaryConvoy => Self {
                color: Color::srgb(0.3, 0.5, 0.3),
                size: Vec2::new(44.0, 20.0),
                health: 200.0,
                speed: 110.0,
                ..Self::TRUCK
            },
            TrafficVehicleType::MotorCycle => Self {
                color: Color::srgb(0.8, 0.2, 0.2),
                size: Vec2::new(12.0, 8.0),
                health: 40.0,
                speed: 160.0,
                ..Self::CAR
            },
        }
    }
}

pub fn spawn_vehicle(
    commands: &mut Commands,
    position: Vec2,
    vehicle_type: VehicleType,
    sprites: &GameSprites,
) {
    let spec = VehicleSpec::for_vehicle(&vehicle_type);
    
    commands.spawn((
        sprite_bundle(spec.color, spec.size, position, 0.8),
        Vehicle::new(vehicle_type),
        Health(spec.health),
        RigidBody::Fixed,
        Collider::cuboid(spec.size.x / 2.0, spec.size.y / 2.0),
        Scannable,
    ));
}

pub fn spawn_traffic_vehicle(
    commands: &mut Commands,
    position: Vec2,
    vehicle_type: TrafficVehicleType,
    sprites: &GameSprites,
) {
    let spec = VehicleSpec::for_traffic(&vehicle_type);
    
    let base_vehicle = match vehicle_type {
        TrafficVehicleType::CivilianCar => VehicleType::CivilianCar,
        TrafficVehicleType::Bus | TrafficVehicleType::Truck => VehicleType::Truck,
        TrafficVehicleType::PoliceCar => VehicleType::PoliceCar,
        TrafficVehicleType::MilitaryConvoy => VehicleType::APC,
        _ => VehicleType::CivilianCar,
    };
    
    let mut entity_cmd = commands.spawn((
        sprite_bundle(spec.color, spec.size, position, 0.9),
        TrafficVehicle {
            vehicle_type: vehicle_type.clone(),
            max_speed: spec.speed,
            current_speed: 0.0,
            acceleration: spec.accel,
            brake_force: spec.brake,
            lane_position: 0.0,
            destination: None,
            panic_level: 0.0,
            brake_lights: false,
        },
        TrafficFlow::default(),
        Health(spec.health),
        Vehicle::new(base_vehicle),
        physics_bundle(spec.size.x * 0.25, VEHICLE_GROUP, RigidBody::Dynamic, VEHICLE_DAMPING),
        Scannable,
    ));
    
    // Add special components
    match vehicle_type {
        TrafficVehicleType::EmergencyAmbulance => {
            entity_cmd.insert(EmergencyVehicle {
                siren_active: false,
                priority_level: 1,
                response_target: None,
            });
        }
        TrafficVehicleType::PoliceCar => {
            entity_cmd.insert(EmergencyVehicle {
                siren_active: false,
                priority_level: 2,
                response_target: None,
            });
        }
        TrafficVehicleType::MilitaryConvoy => {
            entity_cmd.insert(MilitaryConvoy {
                formation_leader: None,
                formation_members: Vec::new(),
                alert_status: ConvoyAlertStatus::Patrol,
                troops_inside: 4,
            });
        }
        _ => {}
    }
}

// === TERMINAL & DEVICE SPAWNERS ===

pub fn spawn_terminal(
    commands: &mut Commands, 
    pos: Vec2, 
    terminal_type: &str, 
    sprites: &GameSprites
) {
    let ttype = parse_terminal_type(terminal_type);
    let (sprite, _) = create_terminal_sprite(sprites, &ttype);
    
    commands.spawn((
        sprite_bundle(sprite.color, sprite.custom_size.unwrap_or(Vec2::splat(24.0)), pos, 1.0),
        Terminal { terminal_type: ttype, range: 30.0, accessed: false },
        Selectable { radius: 15.0 },
        RigidBody::Fixed,
        Collider::ball(TERMINAL_RADIUS),
        CollisionGroups::new(TERMINAL_GROUP, Group::ALL),
        Scannable,
        PathfindingObstacle { radius: TERMINAL_RADIUS, blocks_movement: true },
    ));
}

pub fn spawn_atm(
    commands: &mut Commands,
    position: Vec2,
    bank_id: String,
    network_id: Option<String>,
    power_grid: &mut Option<ResMut<PowerGrid>>,
) -> Entity {
    let entity = commands.spawn((
        sprite_bundle(Color::srgb(0.3, 0.5, 0.7), Vec2::new(16.0, 24.0), position, 1.0),
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
        PathfindingObstacle { radius: TERMINAL_RADIUS, blocks_movement: true },
    )).id();
    
    hackable_device(commands, entity, DeviceType::Terminal, network_id, power_grid, 3, 6.0, Some(HackTool::AdvancedHacker));
    entity
}

pub fn spawn_billboard(
    commands: &mut Commands,
    position: Vec2,
    network_id: Option<String>,
    power_grid: &mut Option<ResMut<PowerGrid>>,
) -> Entity {
    let entity = commands.spawn((
        sprite_bundle(Color::srgb(0.8, 0.6, 0.2), Vec2::new(40.0, 20.0), position, 1.0),
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
        PathfindingObstacle { radius: 22.0, blocks_movement: true },
    )).id();
    
    hackable_device(commands, entity, DeviceType::Terminal, network_id, power_grid, 2, 3.0, Some(HackTool::BasicHacker));
    entity
}

// === POWER GRID DEVICES ===

fn spawn_power_device<T: Component>(
    commands: &mut Commands,
    position: Vec2,
    network_id: String,
    power_grid: &mut ResMut<PowerGrid>,
    component: T,
    device_type: DeviceType,
    color: Color,
    size: Vec2,
    light_color: Option<Color>,
) -> Entity {
    let mut entity_cmd = commands.spawn((
        sprite_bundle(color, size, position, 1.0),
        component,
    ));
    
    if let Some(lcolor) = light_color {
        entity_cmd.insert(PointLight2d {
            intensity: 3.0,
            radius: 100.0,
            falloff: 1.0,
            cast_shadows: true,
            color: lcolor,
        });
    }
    
    let entity = entity_cmd.id();
    make_hackable_networked(commands, entity, device_type, network_id.clone(), power_grid);

    let cloned_network_id = network_id.clone();
    if device_type == DeviceType::PowerStation {
        let network = power_grid.networks.entry(network_id)
            .or_insert_with(|| PowerNetwork::new(cloned_network_id));
        network.power_sources.insert(entity);
    }
    
    entity
}

pub fn spawn_power_station(
    commands: &mut Commands,
    position: Vec2,
    network_id: String,
    power_grid: &mut ResMut<PowerGrid>,
) -> Entity {
    let cloned_network_id = network_id.clone();
    spawn_power_device(
        commands, position, network_id, power_grid,
        PowerStation { network_id: cloned_network_id, max_capacity: 100, current_load: 0 },
        DeviceType::PowerStation,
        Color::srgb(0.8, 0.8, 0.2),
        Vec2::new(60.0, 40.0),
        None
    )
}

pub fn spawn_street_light(
    commands: &mut Commands,
    position: Vec2,
    network_id: String,
    power_grid: &mut ResMut<PowerGrid>,
) -> Entity {
    spawn_power_device(
        commands, position, network_id, power_grid,
        StreetLight { brightness: 1.0 },
        DeviceType::StreetLight,
        Color::srgb(0.9, 0.9, 0.7),
        Vec2::new(8.0, 24.0),
        Some(Color::Srgba(YELLOW))
    )
}

pub fn spawn_traffic_light(
    commands: &mut Commands,
    position: Vec2,
    network_id: String,
    power_grid: &mut ResMut<PowerGrid>,
) -> Entity {
    spawn_power_device(
        commands, position, network_id, power_grid,
        TrafficLight { state: TrafficState::Green, timer: 10.0 },
        DeviceType::TrafficLight,
        Color::srgb(0.2, 0.8, 0.2),
        Vec2::new(12.0, 32.0),
        Some(Color::Srgba(GREEN))
    )
}

// === SECURITY DEVICES ===

pub fn spawn_security_camera(
    commands: &mut Commands,
    position: Vec2,
    network_id: Option<String>,
    power_grid: &mut Option<ResMut<PowerGrid>>,
) -> Entity {
    let entity = commands.spawn((
        sprite_bundle(Color::srgb(0.3, 0.3, 0.3), Vec2::new(16.0, 12.0), position, 1.0),
        SecurityCamera {
            detection_range: 120.0,
            fov_angle: 60.0,
            direction: Vec2::X,
            active: true,
        },
        Vision::new(120.0, 60.0),
    )).id();
    
    hackable_device(commands, entity, DeviceType::Camera, network_id, power_grid, 2, 4.0, Some(HackTool::BasicHacker));
    entity
}

pub fn spawn_automated_turret(
    commands: &mut Commands,
    position: Vec2,
    network_id: Option<String>,
    power_grid: &mut Option<ResMut<PowerGrid>>,
) -> Entity {
    let entity = commands.spawn((
        sprite_bundle(Color::srgb(0.6, 0.2, 0.2), Vec2::new(20.0, 20.0), position, 1.0),
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
    
    if network_id.is_some() && power_grid.is_some() {
        hackable_device(commands, entity, DeviceType::Turret, network_id, power_grid, 4, 8.0, Some(HackTool::AdvancedHacker));
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
        sprite_bundle(Color::srgb(0.4, 0.4, 0.6), Vec2::new(8.0, 32.0), position, 1.0),
        SecurityDoor { locked: true, access_level: 2 },
        RigidBody::Fixed,
        Collider::cuboid(4.0, 16.0),
    )).id();
    
    hackable_device(commands, entity, DeviceType::Door, network_id, power_grid, 2, 5.0, Some(HackTool::BasicHacker));
    entity
}

// === EXPLOSIVE SPAWNERS ===

pub fn spawn_time_bomb(
    commands: &mut Commands,
    position: Vec2,
    timer: f32,
    damage: f32,
    radius: f32,
) -> Entity {
    commands.spawn((
        sprite_bundle(Color::srgb(0.8, 0.2, 0.2), Vec2::new(12.0, 8.0), position, 1.0),
        TimeBomb { timer, damage, radius, armed: true },
    )).id()
}

// === RESEARCH & MISSION SPAWNERS ===

pub fn spawn_scientists_in_mission(
    mut commands: Commands,
    sprites: Res<GameSprites>,
    global_data: Res<GlobalData>,
    scientist_query: Query<Entity, With<Scientist>>,
) {
    if scientist_query.iter().count() >= 3 {
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
    const RESEARCH_PROJECTS: [&str; 3] = [
        "advanced_magazines",
        "tech_interface",
        "combat_enhancers",
    ];
    
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
            RESEARCH_PROJECTS.iter().map(|s| s.to_string()).collect(),
        );
    }
}

pub fn spawn_research_content_in_scene(
    commands: &mut Commands,
    sprites: &GameSprites,
    scene_name: &str,
) {
    struct SceneResearch {
        facility_pos: Vec2,
        faction: Faction,
        security: u32,
        projects: &'static [&'static str],
        scientists: &'static [(f32, f32, ResearchCategory)],
    }
    
    const CORPORATE: SceneResearch = SceneResearch {
        facility_pos: Vec2::new(300.0, 150.0),
        faction: Faction::Corporate,
        security: 4,
        projects: &["tech_interface", "quantum_encryption"],
        scientists: &[
            (250.0, 100.0, ResearchCategory::Intelligence),
            (280.0, 100.0, ResearchCategory::Intelligence),
            (310.0, 100.0, ResearchCategory::Intelligence),
        ],
    };
    
    const SYNDICATE: SceneResearch = SceneResearch {
        facility_pos: Vec2::new(-200.0, -100.0),
        faction: Faction::Syndicate,
        security: 3,
        projects: &["heavy_weapons", "plasma_weapons"],
        scientists: &[
            (-150.0, -80.0, ResearchCategory::Weapons),
            (-180.0, -120.0, ResearchCategory::Cybernetics),
        ],
    };
    
    const UNDERGROUND: SceneResearch = SceneResearch {
        facility_pos: Vec2::new(100.0, -200.0),
        faction: Faction::Underground,
        security: 2,
        projects: &["infiltration_kit", "neural_interface"],
        scientists: &[(120.0, -180.0, ResearchCategory::Equipment)],
    };
    
    const DEFAULT: SceneResearch = SceneResearch {
        facility_pos: Vec2::new(200.0, 100.0),
        faction: Faction::Corporate,
        security: 2,
        projects: &["basic_research"],
        scientists: &[(200.0, 100.0, ResearchCategory::Equipment)],
    };
    
    let config = match scene_name {
        "mission_corporate" => &CORPORATE,
        "mission_syndicate" => &SYNDICATE,
        "mission_underground" => &UNDERGROUND,
        _ => &DEFAULT,
    };
    
    spawn_research_facility(
        commands,
        config.facility_pos,
        config.faction,
        config.security,
        config.projects.iter().map(|s| s.to_string()).collect(),
    );
    
    for &(x, y, cat) in config.scientists {
        spawn_scientist_npc(commands, Vec2::new(x, y), cat, sprites);
    }
}

// === HELPER FUNCTIONS ===

fn random_daily_state() -> DailyState {
    match rand::random::<f32>() {
        x if x < 0.4 => DailyState::GoingToWork,
        x if x < 0.6 => DailyState::Shopping,
        x if x < 0.8 => DailyState::GoingHome,
        _ => DailyState::Idle,
    }
}

// === PUBLIC COMPATIBILITY FUNCTIONS ===

pub fn create_base_unit_bundle(health: f32, speed: f32) -> impl Bundle {
    base_unit_components(health, speed)
}

pub fn create_physics_bundle(radius: f32, group: Group) -> impl Bundle {
    unit_physics(radius, group)
}

pub fn create_inventory_from_loadout(loadout: &AgentLoadout) -> Inventory {
    build_inventory(loadout)
}

pub fn create_weapon_state_from_loadout(loadout: &AgentLoadout) -> WeaponState {
    build_weapon_state(loadout)
}