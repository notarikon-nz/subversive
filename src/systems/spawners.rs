use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::core::*;
use crate::core::factions::Faction;

use crate::systems::scenes::*; // components/structs
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
// use crate::systems::research_gameplay::{};
use crate::systems::world_scan::{WorldScanner};
use crate::systems::panic_spread::{PanicSpreader};

// === DECAL SPAWNERS ===

// interactive_decals.rs

// === ENTITY SPAWNERS ===
pub fn spawn_agent(commands: &mut Commands, pos: Vec2, level: u8, idx: usize, global_data: &GlobalData, sprites: &GameSprites) {
    info!("spawn_agent");
    let (sprite, _) = create_agent_sprite(sprites);
    let loadout = global_data.get_agent_loadout(idx);
    let mut inventory = create_inventory_from_loadout(&loadout);
    inventory.add_currency(100 * level as u32);

    let weapon_state = create_weapon_state_from_loadout(&loadout);

    commands.spawn((
        sprite,
        Transform::from_translation(pos.extend(1.0)),
        Agent { experience: 0, level },
        Faction::Player,
        create_base_unit_bundle(100.0, 150.0),
        Controllable,
        Selectable { radius: 15.0 },
        Vision::new(150.0, 60.0),
        NeurovectorCapability::default(),
        inventory,
        weapon_state,
        create_physics_bundle(16.0, AGENT_GROUP),
    ));
}

pub fn spawn_enemy(commands: &mut Commands, pos: Vec2, patrol: Vec<Vec2>, global_data: &GlobalData, sprites: &GameSprites) {
    let (sprite, _) = create_enemy_sprite(sprites);
    let difficulty = global_data.regions[global_data.selected_region].mission_difficulty_modifier();

    let faction = random_enemy_faction();
    let weapon = select_weapon_for_faction(&faction);

    // Create inventory with weapon
    let mut inventory = Inventory::default();
    inventory.equipped_weapon = Some(WeaponConfig::new(weapon.clone()));

    // Create weapon state with proper ammo - THIS WAS THE ISSUE
    let mut weapon_state = WeaponState::new_from_type(&weapon);
    weapon_state.complete_reload(); // Ensure enemies start with full ammo

    commands.spawn((
        sprite,
        Transform::from_translation(pos.extend(1.0)),
        Enemy,
        faction,
        create_base_unit_bundle(100.0 * difficulty, 100.0),
        Morale::new(100.0 * difficulty, 25.0),
        Vision::new(120.0 * difficulty, 60.0),
        Patrol::new(patrol),
        AIState::default(),
        GoapAgent::default(),
        weapon_state,  // Now properly initialized
        inventory,
        create_physics_bundle(9.0, ENEMY_GROUP),
        Scannable,
    ));
}

// Backwards Compatability (for now)
pub fn spawn_police_unit_simple(
    commands: &mut Commands,
    position: Vec2,
    unit_type: EscalationLevel,
    sprites: &GameSprites,
) -> Entity {
    // Use default config or load it
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

    commands.spawn_empty()
        .insert((
            sprite,
            Transform::from_translation(position.extend(1.0)),
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
        ))
        .insert((
            WeaponState::new_from_type(&weapon_type),
            inventory,
            RigidBody::Dynamic,
            Collider::ball(9.0),
            Velocity::default(),
            Damping { linear_damping: 10.0, angular_damping: 10.0 },
            CollisionGroups::new(CIVILIAN_GROUP, Group::ALL),
            Friction::coefficient(0.8),
            Restitution::coefficient(0.1),
            LockedAxes::ROTATION_LOCKED,
            GravityScale(0.0),
            Scannable,
        )).id()
}



// 0.2.5.3 - added Pathfinding Obstacle component
pub fn spawn_terminal(commands: &mut Commands, pos: Vec2, terminal_type: &str, sprites: &GameSprites) {
    let (sprite, _) = create_terminal_sprite(sprites, &parse_terminal_type(terminal_type));

    commands.spawn((
        sprite,
        Transform::from_translation(pos.extend(1.0)),
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
            blocks_movement: true, // Terminals block movement
        },
    ));
}

fn spawn_agent_with_index(commands: &mut Commands, pos: Vec2, level: u8, idx: usize, global_data: &GlobalData, sprites: &GameSprites) {
    info!("spawn_agent_with_index");
    let (sprite, _) = create_agent_sprite(sprites);
    let loadout = global_data.get_agent_loadout(idx);
    let mut inventory = create_inventory_from_loadout(&loadout);
    inventory.add_currency(100 * level as u32);

    let weapon_state = create_weapon_state_from_loadout(&loadout);
    let scan_level = level.min(3);
    let agent_entity = commands.spawn_empty()
    .insert((
        sprite,
        Transform::from_translation(pos.extend(1.0)),
        Agent { experience: 0, level },
        AgentIndex(idx),
        Faction::Player,
        create_base_unit_bundle(100.0, 150.0),
        Controllable,
        Selectable { radius: 15.0 },
        Vision::new(150.0, 60.0),
        NeurovectorCapability::default(),
        inventory,
        weapon_state,
        create_physics_bundle(16.0, AGENT_GROUP),
        // Add scanner to support agents
        WorldScanner {
            scan_level,
            range: 150.0 + (scan_level as f32 * 50.0), // Higher level = longer range
            energy: 100.0,
            max_energy: 100.0,
            scan_cost: 15.0 + (scan_level as f32 * 5.0), // Higher level = more expensive
            recharge_rate: 8.0 + (scan_level as f32 * 2.0), // Higher level = faster recharge
            active: false,
        },
    ));
    if level >= 5 { // high-level agent
        // add_scanner_to_agent(commands, agent_entity, level.min(3));
    }
}

pub fn spawn_civilian(commands: &mut Commands, position: Vec2, sprites: &GameSprites) {
    let (sprite, mut transform) = crate::core::sprites::create_civilian_sprite(sprites);
    transform.translation = position.extend(1.0);

    commands.spawn((
        sprite,
        transform,
        Civilian,
        Health(50.0),
        Morale::new(80.0, 40.0),
        PanicSpreader::default(),
        MovementSpeed(100.0 + rand::random::<f32>() * 40.0),
        Controllable,
        NeurovectorTarget,
        CivilianWander::new(position),
        RigidBody::Dynamic,
        Collider::ball(7.5),
        Velocity::default(),
        Damping { linear_damping: 10.0, angular_damping: 10.0 },
    ));
}

pub fn spawn_civilian_with_config(commands: &mut Commands, pos: Vec2, sprites: &GameSprites, config: &GameConfig) {
    info!("spawn_civilian_with_config");
    let (sprite, _) = create_civilian_sprite(sprites);

    commands.spawn((
        sprite,
        Transform::from_translation(pos.extend(1.0)),
        Civilian,
        Faction::Civilian,
        Health(50.0),
        Morale::new(config.civilians.base_morale, config.civilians.panic_threshold),
        PanicSpreader::default(),
        MovementSpeed(config.civilians.movement_speed),
        Controllable,
        NeurovectorTarget,
        create_physics_bundle(7.5, CIVILIAN_GROUP),
        Scannable,
    ));
}



pub fn spawn_vehicle(
    commands: &mut Commands,
    position: Vec2,
    vehicle_type: VehicleType,
    sprites: &GameSprites,
) {
    let vehicle = Vehicle::new(vehicle_type.clone());
    let max_health = vehicle.max_health();

    let (color, size) = match vehicle_type {
        VehicleType::CivilianCar => (Color::srgb(0.6, 0.6, 0.8), Vec2::new(40.0, 20.0)),
        VehicleType::PoliceCar => (Color::srgb(0.2, 0.2, 0.8), Vec2::new(40.0, 20.0)),
        VehicleType::ElectricCar => (Color::srgb(0.6, 0.6, 0.9), Vec2::new(40.0, 20.0)),
        VehicleType::APC => (Color::srgb(0.4, 0.6, 0.4), Vec2::new(50.0, 30.0)),
        VehicleType::VTOL => (Color::srgb(0.3, 0.3, 0.3), Vec2::new(60.0, 40.0)),
        VehicleType::Tank => (Color::srgb(0.5, 0.5, 0.2), Vec2::new(60.0, 35.0)),
        VehicleType::Truck => (Color::srgb(0.4, 0.6, 0.4), Vec2::new(50.0, 30.0)), // change
        VehicleType::FuelTruck => (Color::srgb(0.4, 0.6, 0.4), Vec2::new(50.0, 30.0)), // change
    };

    commands.spawn((
        Sprite {
            color,
            custom_size: Some(size),
            ..default()
        },
        Transform::from_translation(position.extend(0.8)),
        vehicle,
        Health(max_health),
        RigidBody::Fixed,
        Collider::cuboid(size.x / 2.0, size.y / 2.0),
        Scannable,
    ));
}

pub fn spawn_urban_civilian(
    commands: &mut Commands,
    position: Vec2,
    sprites: &GameSprites,
    urban_areas: &UrbanAreas,
) {
    let (sprite, _) = crate::core::sprites::create_civilian_sprite(sprites);

    // Randomize civilian personality
    let crowd_influence = 0.3 + rand::random::<f32>() * 0.4; // 0.3-0.7
    let panic_threshold = 20.0 + rand::random::<f32>() * 40.0; // 20-60

    // Pick initial daily state based on "time of day" simulation
    let daily_state = match rand::random::<f32>() {
        x if x < 0.4 => DailyState::GoingToWork,
        x if x < 0.6 => DailyState::Shopping,
        x if x < 0.8 => DailyState::GoingHome,
        _ => DailyState::Idle,
    };

    let next_destination = pick_destination_for_state(daily_state, urban_areas);

    commands.spawn((
        sprite,
        Transform::from_translation(position.extend(1.0)),
        Civilian,
        Health(50.0),
        Morale::new(80.0, panic_threshold),
        MovementSpeed(80.0),
        Controllable,
        NeurovectorTarget,
        UrbanCivilian {
            daily_state,
            state_timer: rand::random::<f32>() * 10.0, // Stagger state changes
            next_destination,
            crowd_influence,
            panic_threshold,
            movement_urgency: 0.0,
        },
        bevy_rapier2d::prelude::RigidBody::Dynamic,
        bevy_rapier2d::prelude::Collider::ball(7.5),
        bevy_rapier2d::prelude::Velocity::default(),
        bevy_rapier2d::prelude::Damping { linear_damping: 10.0, angular_damping: 10.0 },
        bevy_rapier2d::prelude::GravityScale(0.0),
    ));
}

pub fn spawn_original_urban_civilian(commands: &mut Commands, pos: Vec2, sprites: &GameSprites) {
    info!("spawn_urban_civilian");
    let (sprite, _) = create_civilian_sprite(sprites);
    let crowd_influence = 0.2 + rand::random::<f32>() * 0.6;
    let panic_threshold = 15.0 + rand::random::<f32>() * 50.0;
    let daily_state = random_daily_state();

    commands.spawn((
        sprite,
        Transform::from_translation(pos.extend(1.0)),
        Civilian,

        create_base_unit_bundle(50.0, 80.0 + rand::random::<f32>() * 40.0),
        Morale::new(80.0, panic_threshold),
        Controllable,
        NeurovectorTarget,
        UrbanCivilian {
            daily_state,
            state_timer: rand::random::<f32>() * 15.0,
            next_destination: None,
            crowd_influence,
            panic_threshold,
            movement_urgency: 0.0,
        },
        create_physics_bundle(7.5, CIVILIAN_GROUP),
        Scannable,
    ));
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

// === SCIENTIST SPAWNING SYSTEM ===
pub fn spawn_scientists_in_mission(
    mut commands: Commands,
    sprites: Res<GameSprites>,
    global_data: Res<GlobalData>,
    scientist_query: Query<Entity, With<Scientist>>,
) {
    // Only spawn if we don't have many scientists already
    let existing_count = scientist_query.iter().count();
    if existing_count >= 3 {
        return;
    }

    // Spawn 1-2 scientists per mission randomly
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

// === STATIC CONTENT ===
pub fn spawn_time_bomb(
    commands: &mut Commands,
    position: Vec2,
    timer: f32,
    damage: f32,
    radius: f32,
) -> Entity {
    commands.spawn((
        Sprite {
            color: Color::srgb(0.8, 0.2, 0.2),
            custom_size: Some(Vec2::new(12.0, 8.0)),
            ..default()
        },
        Transform::from_translation(position.extend(1.0)),
        TimeBomb {
            timer,
            damage,
            radius,
            armed: true,
        },
    )).id()
}

pub fn spawn_atm(
    commands: &mut Commands,
    position: Vec2,
    bank_id: String,
    network_id: Option<String>,
    power_grid: &mut Option<ResMut<PowerGrid>>,
) -> Entity {
    let entity = commands.spawn((
        Sprite {
            color: Color::srgb(0.3, 0.5, 0.7),
            custom_size: Some(Vec2::new(16.0, 24.0)),
            ..default()
        },
        Transform::from_translation(position.extend(1.0)),
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

    // Make hackable - ATMs are like advanced terminals
    if let (Some(network_id), Some(power_grid)) = (network_id, power_grid) {
        make_hackable_networked(commands, entity, DeviceType::Terminal, network_id, power_grid);
    } else {
        let mut hackable = Hackable::new(DeviceType::Terminal);
        hackable.security_level = 3; // ATMs are more secure
        hackable.hack_time = 6.0;
        hackable.requires_tool = Some(HackTool::AdvancedHacker);
        commands.entity(entity).insert((hackable, DeviceState::new(DeviceType::Terminal)));
    }

    entity
}

pub fn spawn_billboard(
    commands: &mut Commands,
    position: Vec2,
    network_id: Option<String>,
    power_grid: &mut Option<ResMut<PowerGrid>>,
) -> Entity {
    let entity = commands.spawn((
        Sprite {
            color: Color::srgb(0.8, 0.6, 0.2),
            custom_size: Some(Vec2::new(40.0, 20.0)),
            ..default()
        },
        Transform::from_translation(position.extend(1.0)),
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

    // Billboards are easier to hack
    if let (Some(network_id), Some(power_grid)) = (network_id, power_grid) {
        make_hackable_networked(commands, entity, DeviceType::Terminal, network_id, power_grid);
    } else {
        let mut hackable = Hackable::new(DeviceType::Terminal);
        hackable.security_level = 2;
        hackable.hack_time = 3.0;
        hackable.requires_tool = Some(HackTool::BasicHacker);
        commands.entity(entity).insert((hackable, DeviceState::new(DeviceType::Terminal)));
    }

    entity
}

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


pub fn spawn_random_research_content(
    commands: &mut Commands,
    sprites: &GameSprites,
    global_data: &GlobalData,
) {
    // Spawn 1-2 research facilities per mission
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

        let available_projects = vec![
            "advanced_magazines".to_string(),
            "tech_interface".to_string(),
            "combat_enhancers".to_string(),
        ];

        spawn_research_facility(
            commands,
            position,
            faction,
            2 + rand::random::<u32>() % 4, // Security level 2-5
            available_projects,
        );
    }

    // Spawn scientists (handled by spawn_scientists_in_mission)
}


// 0.2.12
pub fn spawn_research_content_in_scene(
    commands: &mut Commands,
    sprites: &GameSprites,
    scene_name: &str,
) {
    match scene_name {
        "mission_corporate" => {
            // Corporate missions have high-tech research facilities
            spawn_research_facility(
                commands,
                Vec2::new(300.0, 150.0),
                Faction::Corporate,
                4, // High security
                vec!["tech_interface".to_string(), "quantum_encryption".to_string()],
            );

            // Spawn 2-3 scientists
            for i in 0..3 {
                let pos = Vec2::new(250.0 + i as f32 * 30.0, 100.0);
                spawn_scientist_npc(commands, pos, ResearchCategory::Intelligence, sprites);
            }
        },
        "mission_syndicate" => {
            // Syndicate missions have weapons research
            spawn_research_facility(
                commands,
                Vec2::new(-200.0, -100.0),
                Faction::Syndicate,
                3,
                vec!["heavy_weapons".to_string(), "plasma_weapons".to_string()],
            );

            spawn_scientist_npc(commands, Vec2::new(-150.0, -80.0), ResearchCategory::Weapons, sprites);
            spawn_scientist_npc(commands, Vec2::new(-180.0, -120.0), ResearchCategory::Cybernetics, sprites);
        },
        "mission_underground" => {
            // Underground has equipment and cybernetics
            spawn_research_facility(
                commands,
                Vec2::new(100.0, -200.0),
                Faction::Underground,
                2,
                vec!["infiltration_kit".to_string(), "neural_interface".to_string()],
            );

            spawn_scientist_npc(commands, Vec2::new(120.0, -180.0), ResearchCategory::Equipment, sprites);
        },
        _ => {
            // Default mission gets basic research content
            spawn_scientist_npc(commands, Vec2::new(200.0, 100.0), ResearchCategory::Equipment, sprites);
        }
    }
}


// === HELPER FUNCTIONS ===
pub fn create_base_unit_bundle(health: f32, speed: f32) -> impl Bundle {
    (Health(health), MovementSpeed(speed))
}

pub fn create_physics_bundle(radius: f32, group: Group) -> impl Bundle {
    (
        RigidBody::Dynamic,  // Changed from KinematicPositionBased for better collision
        Collider::ball(radius),
        Velocity::default(),
        Damping { linear_damping: 15.0, angular_damping: 15.0 }, // Higher damping for stability
        CollisionGroups::new(group, Group::ALL), // This entity belongs to 'group', collides with all
        Friction::coefficient(0.8), // Prevent sliding
        Restitution::coefficient(0.1), // Low bounce
        LockedAxes::ROTATION_LOCKED, // Prevent spinning
        GravityScale(0.0),
    )
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
