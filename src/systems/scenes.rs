// src/systems/scenes.rs - Quick wins: removed debug/fallback code and consolidated helpers
use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use serde::{Deserialize, Serialize};

use crate::core::*;
use crate::systems::ai::*;
use crate::systems::spawners::{spawn_agent};
use crate::core::factions::Faction;
use crate::systems::*;
use crate::systems::police::*;
use crate::systems::selection::*;

// === Z-SORTING COMPONENT ===
#[derive(Component)]
pub struct IsometricDepth(pub f32);

// === SCENE DATA STRUCTURES ===
#[derive(Resource, Clone, Default, Serialize, Deserialize)]
pub struct SceneData {
    pub agents: Vec<AgentSpawn>,
    pub civilians: Vec<CivilianSpawn>,
    pub enemies: Vec<EnemySpawn>,
    pub terminals: Vec<TerminalSpawn>,
    pub vehicles: Vec<VehicleSpawn>,
    pub urban_areas: Option<UrbanAreasData>,
    pub police: Option<Vec<PoliceSpawn>>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct UrbanAreasData {
    pub work_zones: Vec<UrbanZoneData>,
    pub shopping_zones: Vec<UrbanZoneData>,
    pub residential_zones: Vec<UrbanZoneData>,
    pub transit_routes: Vec<TransitRouteData>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct UrbanZoneData {
    pub center: [f32; 2],
    pub radius: f32,
    pub capacity: usize,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct TransitRouteData {
    pub points: Vec<[f32; 2]>,
    pub foot_traffic_density: f32,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct AgentSpawn {
    pub position: [f32; 2],
    pub level: u8,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct CivilianSpawn {
    pub position: [f32; 2],
}

#[derive(Clone, Serialize, Deserialize)]
pub struct EnemySpawn {
    pub position: [f32; 2],
    pub patrol_points: Vec<[f32; 2]>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct TerminalSpawn {
    pub position: [f32; 2],
    pub terminal_type: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct VehicleSpawn {
    pub position: [f32; 2],
    pub vehicle_type: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct PoliceSpawn {
    pub position: [f32; 2],
    pub patrol_points: Vec<[f32; 2]>,
    pub unit_type: String,
}

// === CORE FUNCTIONS ===
pub fn load_scene_cached(scene_cache: &mut SceneCache, name: &str) -> Option<SceneData> {
    scene_cache.get_scene(name).cloned()
}

pub fn spawn_from_scene(commands: &mut Commands, scene: &SceneData, global_data: &GlobalData, sprites: &GameSprites) {
    setup_urban_areas(commands, scene, global_data.selected_region);

    for (i, agent) in scene.agents.iter().enumerate() {
        let level = if i < 3 { global_data.agent_levels[i] } else { agent.level };
        spawn_agent(commands, Vec2::from(agent.position), level, i, global_data, sprites);
    }

    for civilian in &scene.civilians {
        spawn_original_urban_civilian(commands, Vec2::from(civilian.position), sprites);
    }

    for enemy in &scene.enemies {
        let patrol = enemy.patrol_points.iter().map(|&p| Vec2::from(p)).collect();
        spawn_enemy(commands, Vec2::from(enemy.position), patrol, global_data, sprites);
    }

    for terminal in &scene.terminals {
        spawn_terminal(commands, Vec2::from(terminal.position), &terminal.terminal_type, sprites);
    }

    for vehicle in &scene.vehicles {
        let v_type = parse_vehicle_type(&vehicle.vehicle_type);
        spawn_vehicle(commands, Vec2::from(vehicle.position), v_type, sprites);
    }

    if let Some(police) = &scene.police {
        for unit in police {
            //let patrol = unit.patrol_points.iter().map(|&p| Vec2::from(p)).collect();
            spawn_police_unit_simple(commands, Vec2::from(unit.position), EscalationLevel::Patrol, sprites);
        }
    }

    spawn_cover_points(commands);

    info!("Mission spawned: {} agents, {} enemies, {} civilians",
          scene.agents.len(), scene.enemies.len(), scene.civilians.len());
}


// === Z-SORTING SYSTEM ===
pub fn isometric_depth_sorting(
    mut query: Query<(&mut Transform, &IsometricDepth), Changed<Transform>>,
) {
    for (mut transform, depth) in query.iter_mut() {
        // Calculate Z based on Y position (lower Y = higher Z for proper depth sorting)
        let base_z = depth.0;
        let y_offset = -transform.translation.y * 0.01; // Small factor to avoid Z conflicts
        transform.translation.z = base_z + y_offset;
    }
}

// ISOMETRIC VERSION
pub fn spawn_from_scene_isometric(
    commands: &mut Commands,
    scene: &SceneData,
    global_data: &GlobalData,
    sprites: &GameSprites,
    tilemap_settings: &Option<Res<IsometricSettings>>,
) {
    // Setup urban areas first
    setup_urban_areas_isometric(commands, scene, global_data.selected_region);

    // Spawn agents with isometric positioning
    for (i, agent) in scene.agents.iter().enumerate() {
        let level = if i < 3 { global_data.agent_levels[i] } else { agent.level };
        let world_pos = Vec2::from(agent.position);
        let adjusted_pos = adjust_position_for_isometric(world_pos, tilemap_settings);
        spawn_agent_isometric(commands, adjusted_pos, level, i, global_data, sprites);
    }

    // Spawn other entities with position adjustment
    for civilian in &scene.civilians {
        let world_pos = Vec2::from(civilian.position);
        let adjusted_pos = adjust_position_for_isometric(world_pos, tilemap_settings);
        spawn_urban_civilian_isometric(commands, adjusted_pos, sprites);
    }

    for enemy in &scene.enemies {
        let world_pos = Vec2::from(enemy.position);
        let adjusted_pos = adjust_position_for_isometric(world_pos, tilemap_settings);
        let patrol = enemy.patrol_points.iter()
            .map(|&p| adjust_position_for_isometric(Vec2::from(p), tilemap_settings))
            .collect();
        spawn_enemy_isometric(commands, adjusted_pos, patrol, global_data, sprites);
    }

    for terminal in &scene.terminals {
        let world_pos = Vec2::from(terminal.position);
        let adjusted_pos = adjust_position_for_isometric(world_pos, tilemap_settings);
        spawn_terminal_isometric(commands, adjusted_pos, &terminal.terminal_type, sprites);
    }

    for vehicle in &scene.vehicles {
        let world_pos = Vec2::from(vehicle.position);
        let adjusted_pos = adjust_position_for_isometric(world_pos, tilemap_settings);
        let v_type = parse_vehicle_type(&vehicle.vehicle_type);
        spawn_vehicle_isometric(commands, adjusted_pos, v_type, sprites);
    }
}

// === POSITION ADJUSTMENT FOR ISOMETRIC ===
fn adjust_position_for_isometric(
    world_pos: Vec2,
    tilemap_settings: &Option<Res<IsometricSettings>>,
) -> Vec2 {
    if let Some(settings) = tilemap_settings {
        // Convert old world coordinates to tile coordinates, then back to isometric world
        let tile_pos = settings.world_to_tile(world_pos);
        settings.tile_to_world(tile_pos)
    } else {
        // No tilemap, use position as-is
        world_pos
    }
}

// === ISOMETRIC ENTITY SPAWNERS ===
fn spawn_agent_isometric(
    commands: &mut Commands,
    pos: Vec2,
    level: u8,
    idx: usize,
    global_data: &GlobalData,
    sprites: &GameSprites,
) {
    let (sprite, _) = create_agent_sprite(sprites);
    let loadout = global_data.get_agent_loadout(idx);
    let mut inventory = create_inventory_from_loadout(&loadout);
    inventory.add_currency(100 * level as u32);

    let weapon_state = create_weapon_state_from_loadout(&loadout);
    let scan_level = level.min(3);

    commands.spawn((
        sprite,
        Transform::from_translation(pos.extend(10.0)), // Higher Z for proper sorting
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
        WorldScanner {
            scan_level,
            range: 150.0 + (scan_level as f32 * 50.0),
            energy: 100.0,
            max_energy: 100.0,
            scan_cost: 15.0 + (scan_level as f32 * 5.0),
            recharge_rate: 8.0 + (scan_level as f32 * 2.0),
            active: false,
        },
        IsometricDepth(10.0), // For proper z-sorting
    ));
}

fn spawn_urban_civilian_isometric(commands: &mut Commands, pos: Vec2, sprites: &GameSprites) {
    let (sprite, _) = create_civilian_sprite(sprites);
    let crowd_influence = 0.2 + rand::random::<f32>() * 0.6;
    let panic_threshold = 15.0 + rand::random::<f32>() * 50.0;
    let daily_state = random_daily_state();

    commands.spawn((
        sprite,
        Transform::from_translation(pos.extend(5.0)), // Lower Z than agents
        Civilian,
        Faction::Civilian,
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
        IsometricDepth(5.0),
    ));
}

fn spawn_enemy_isometric(
    commands: &mut Commands,
    pos: Vec2,
    patrol: Vec<Vec2>,
    global_data: &GlobalData,
    sprites: &GameSprites,
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
        sprite,
        Transform::from_translation(pos.extend(8.0)), // Mid-level Z
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
        create_physics_bundle(9.0, ENEMY_GROUP),
        Scannable,
        IsometricDepth(8.0),
    ));
}

fn spawn_terminal_isometric(commands: &mut Commands, pos: Vec2, terminal_type: &str, sprites: &GameSprites) {
    let (sprite, _) = create_terminal_sprite(sprites, &parse_terminal_type(terminal_type));

    commands.spawn((
        sprite,
        Transform::from_translation(pos.extend(2.0)), // Low Z for ground objects
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
        IsometricDepth(2.0),
    ));
}

fn spawn_vehicle_isometric(
    commands: &mut Commands,
    pos: Vec2,
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

    let entity = commands.spawn((
        Sprite {
            color,
            custom_size: Some(size),
            ..default()
        },
        Transform::from_translation(pos.extend(0.8)),
        vehicle,
        Health(max_health),
        RigidBody::Fixed,
        Collider::cuboid(size.x / 2.0, size.y / 2.0),
        Scannable,
    ))
    .insert(IsometricDepth(3.0)); // Ground level for vehicles
}

// === UTILITY FUNCTIONS ===
fn setup_urban_areas_isometric(commands: &mut Commands, scene: &SceneData, region_idx: usize) {
    // Same as original but could be enhanced to better fit isometric tiles
    let urban_areas = if let Some(scene_urban) = &scene.urban_areas {
        convert_scene_urban_data(scene_urban)
    } else {
        create_default_urban_areas(region_idx)
    };

    commands.insert_resource(urban_areas);
}

pub fn spawn_fallback_isometric_mission(
    commands: &mut Commands,
    global_data: &GlobalData,
    sprites: &GameSprites,
    tilemap_settings: &Option<Res<IsometricSettings>>,
) {
    commands.insert_resource(UrbanAreas::default());

    let positions = [Vec2::new(-200.0, 0.0), Vec2::new(-170.0, 0.0), Vec2::new(-140.0, 0.0)];
    for (i, &pos) in positions.iter().enumerate() {
        let adjusted_pos = adjust_position_for_isometric(pos, tilemap_settings);
        spawn_agent_isometric(commands, adjusted_pos, global_data.agent_levels[i], i, global_data, sprites);
    }

    let civilian_positions = [Vec2::new(100.0, 100.0), Vec2::new(150.0, 80.0), Vec2::new(80.0, 150.0)];
    for &pos in &civilian_positions {
        let adjusted_pos = adjust_position_for_isometric(pos, tilemap_settings);
        spawn_urban_civilian_isometric(commands, adjusted_pos, sprites);
    }

    let terminal_pos = adjust_position_for_isometric(Vec2::new(200.0, 0.0), tilemap_settings);
    spawn_terminal_isometric(commands, terminal_pos, "objective", sprites);
}





fn parse_vehicle_type(type_str: &str) -> VehicleType {
    match type_str {
        "civilian_car" => VehicleType::CivilianCar,
        "police_car" => VehicleType::PoliceCar,
        "apc" => VehicleType::APC,
        "vtol" => VehicleType::VTOL,
        "tank" => VehicleType::Tank,
        "truck" => VehicleType::Truck,
        "fuel_truck" => VehicleType::FuelTruck,
        _ => VehicleType::CivilianCar,
    }
}

/*
pub fn spawn_fallback_mission(commands: &mut Commands, global_data: &GlobalData, sprites: &GameSprites) {
    commands.insert_resource(UrbanAreas::default());

    let positions = [Vec2::new(-200.0, 0.0), Vec2::new(-170.0, 0.0), Vec2::new(-140.0, 0.0)];
    for (i, &pos) in positions.iter().enumerate() {
        spawn_agent_with_index(commands, pos, global_data.agent_levels[i], i, global_data, sprites);
    }

    let civilian_positions = [Vec2::new(100.0, 100.0), Vec2::new(150.0, 80.0), Vec2::new(80.0, 150.0)];
    for &pos in &civilian_positions {
        spawn_urban_civilian(commands, pos, sprites);
    }

    spawn_terminal(commands, Vec2::new(200.0, 0.0), "objective", sprites);
    spawn_cover_points(commands);
}
*/


pub fn load_scene(name: &str) -> Option<SceneData> {
    let path = format!("scenes/{}.json", name);
    let content = std::fs::read_to_string(&path).ok()?;
    serde_json::from_str(&content).ok()
}


// 0.2.5.3 - added Pathfinding Obstacle component
pub fn spawn_cover_points(commands: &mut Commands) {
    let positions = [
        Vec2::new(50.0, -50.0), Vec2::new(250.0, -150.0), Vec2::new(-50.0, 100.0),
        Vec2::new(300.0, 50.0), Vec2::new(150.0, 150.0),
    ];

    for &pos in &positions {
        commands.spawn((
            Sprite {
                color: Color::srgba(0.4, 0.2, 0.1, 0.7),
                custom_size: Some(Vec2::new(20.0, 40.0)),
                ..default()
            },
            Transform::from_translation(pos.extend(0.5)),
            CoverPoint {
                capacity: 2,
                current_users: 0,
                cover_direction: Vec2::X,
            },
            // Physics
            RigidBody::Fixed,
            Collider::cuboid(10.0, 20.0),
            CollisionGroups::new(COVER_GROUP, Group::ALL),
            // NEW: Pathfinding obstacle
            PathfindingObstacle {
                radius: 18.0,
                blocks_movement: true, // Cover provides concealment but can be moved around
            },
        ));
    }
}



pub fn random_daily_state() -> DailyState {
    match rand::random::<f32>() {
        x if x < 0.3 => DailyState::Working,
        x if x < 0.5 => DailyState::Shopping,
        x if x < 0.7 => DailyState::GoingHome,
        _ => DailyState::Idle,
    }
}

pub fn random_enemy_faction() -> Faction {
    match rand::random::<f32>() {
        x if x < 0.4 => Faction::Corporate,
        x if x < 0.8 => Faction::Syndicate,
        _ => Faction::Police,
    }
}

pub fn select_weapon_for_faction(faction: &Faction) -> WeaponType {

    match faction {
        Faction::Corporate => if rand::random::<f32>() < 0.7 { WeaponType::Rifle } else { WeaponType::Pistol },
        Faction::Syndicate => match rand::random::<f32>() {
            x if x < 0.5 => WeaponType::Pistol,
            // x if x < 0.8 => WeaponType::Flamethrower,
            _ => WeaponType::Rifle,
        },
        _ => WeaponType::Pistol,
    }
}

pub fn parse_terminal_type(type_str: &str) -> TerminalType {
    match type_str {
        "objective" => TerminalType::Objective,
        "equipment" => TerminalType::Equipment,
        "intel" => TerminalType::Intel,
        _ => TerminalType::Objective,
    }
}

// === URBAN AREAS SETUP ===
pub fn setup_urban_areas(commands: &mut Commands, scene: &SceneData, region_idx: usize) {
    let urban_areas = if let Some(scene_urban) = &scene.urban_areas {
        convert_scene_urban_data(scene_urban)
    } else {
        create_default_urban_areas(region_idx)
    };

    commands.insert_resource(urban_areas);
}

pub fn convert_scene_urban_data(scene_urban: &UrbanAreasData) -> UrbanAreas {
    UrbanAreas {
        work_zones: scene_urban.work_zones.iter().map(convert_zone_data).collect(),
        shopping_zones: scene_urban.shopping_zones.iter().map(convert_zone_data).collect(),
        residential_zones: scene_urban.residential_zones.iter().map(convert_zone_data).collect(),
        transit_routes: scene_urban.transit_routes.iter().map(convert_route_data).collect(),
    }
}

pub fn convert_zone_data(z: &UrbanZoneData) -> UrbanZone {
    UrbanZone {
        center: Vec2::from(z.center),
        radius: z.radius,
        capacity: z.capacity,
        current_occupancy: 0,
    }
}

pub fn convert_route_data(r: &TransitRouteData) -> TransitRoute {
    TransitRoute {
        points: r.points.iter().map(|&p| Vec2::from(p)).collect(),
        foot_traffic_density: r.foot_traffic_density,
    }
}

pub fn create_default_urban_areas(region_idx: usize) -> UrbanAreas {
    match region_idx {
        0 => create_urban_district_areas(),
        1 => create_corporate_district_areas(),
        2 => create_industrial_areas(),
        _ => UrbanAreas::default(),
    }
}

pub fn create_urban_district_areas() -> UrbanAreas {
    UrbanAreas {
        work_zones: vec![
            UrbanZone { center: Vec2::new(150.0, -80.0), radius: 70.0, capacity: 12, current_occupancy: 0 },
            UrbanZone { center: Vec2::new(50.0, 120.0), radius: 60.0, capacity: 8, current_occupancy: 0 },
        ],
        shopping_zones: vec![
            UrbanZone { center: Vec2::new(200.0, 100.0), radius: 80.0, capacity: 15, current_occupancy: 0 },
            UrbanZone { center: Vec2::new(100.0, 60.0), radius: 50.0, capacity: 8, current_occupancy: 0 },
        ],
        residential_zones: vec![
            UrbanZone { center: Vec2::new(300.0, 180.0), radius: 90.0, capacity: 20, current_occupancy: 0 },
            UrbanZone { center: Vec2::new(80.0, 200.0), radius: 70.0, capacity: 12, current_occupancy: 0 },
        ],
        transit_routes: vec![
            TransitRoute {
                points: vec![Vec2::new(-100.0, 0.0), Vec2::new(100.0, 0.0), Vec2::new(300.0, 0.0)],
                foot_traffic_density: 0.8
            },
            TransitRoute {
                points: vec![Vec2::new(150.0, -50.0), Vec2::new(200.0, 50.0), Vec2::new(250.0, 150.0)],
                foot_traffic_density: 0.6
            },
            TransitRoute {
                points: vec![Vec2::new(100.0, 100.0), Vec2::new(200.0, 120.0), Vec2::new(300.0, 180.0)],
                foot_traffic_density: 0.4
            },
        ],
    }
}

fn create_corporate_district_areas() -> UrbanAreas {
    UrbanAreas {
        work_zones: vec![
            UrbanZone { center: Vec2::new(400.0, -20.0), radius: 100.0, capacity: 25, current_occupancy: 0 },
            UrbanZone { center: Vec2::new(100.0, -150.0), radius: 80.0, capacity: 15, current_occupancy: 0 },
        ],
        shopping_zones: vec![
            UrbanZone { center: Vec2::new(200.0, 200.0), radius: 60.0, capacity: 10, current_occupancy: 0 },
        ],
        residential_zones: vec![
            UrbanZone { center: Vec2::new(50.0, 100.0), radius: 80.0, capacity: 18, current_occupancy: 0 },
            UrbanZone { center: Vec2::new(150.0, 50.0), radius: 70.0, capacity: 12, current_occupancy: 0 },
        ],
        transit_routes: vec![
            TransitRoute {
                points: vec![Vec2::new(0.0, -150.0), Vec2::new(200.0, -100.0), Vec2::new(400.0, -20.0)],
                foot_traffic_density: 0.9
            },
            TransitRoute {
                points: vec![Vec2::new(50.0, 100.0), Vec2::new(150.0, 50.0), Vec2::new(300.0, 0.0)],
                foot_traffic_density: 0.7
            },
            TransitRoute {
                points: vec![Vec2::new(150.0, 150.0), Vec2::new(200.0, 200.0), Vec2::new(250.0, 180.0)],
                foot_traffic_density: 0.5
            },
        ],
    }
}

fn create_industrial_areas() -> UrbanAreas {
    UrbanAreas {
        work_zones: vec![
            UrbanZone { center: Vec2::new(200.0, -100.0), radius: 60.0, capacity: 8, current_occupancy: 0 },
            UrbanZone { center: Vec2::new(350.0, -50.0), radius: 50.0, capacity: 6, current_occupancy: 0 },
        ],
        shopping_zones: vec![
            UrbanZone { center: Vec2::new(50.0, -200.0), radius: 40.0, capacity: 4, current_occupancy: 0 },
        ],
        residential_zones: vec![
            UrbanZone { center: Vec2::new(-150.0, 200.0), radius: 70.0, capacity: 10, current_occupancy: 0 },
        ],
        transit_routes: vec![
            TransitRoute {
                points: vec![Vec2::new(-200.0, 0.0), Vec2::new(0.0, -50.0), Vec2::new(200.0, -100.0), Vec2::new(400.0, -50.0)],
                foot_traffic_density: 0.3
            },
            TransitRoute {
                points: vec![Vec2::new(-150.0, 200.0), Vec2::new(0.0, 100.0), Vec2::new(150.0, 0.0)],
                foot_traffic_density: 0.4
            },
        ],
    }
}


