// src/systems/scenes.rs - Optimized and cleaned up
use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use serde::{Deserialize, Serialize};

use crate::core::*;
use crate::systems::panic_spread::*;
use crate::systems::ai::*;
use crate::systems::vehicles::spawn_vehicle;
use crate::core::factions::Faction;
use crate::systems::*;
use crate::systems::police_escalation::*;

// === SCENE DATA STRUCTURES ===
#[derive(Clone, Serialize, Deserialize)]
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
    pub unit_type: String, // "patrol", "armed", "tactical", "military", "corporate"
}

// === CORE FUNCTIONS ===
pub fn ensure_scenes_directory() {
    if std::fs::create_dir_all("scenes").is_err() {
        error!("Could not create scenes directory");
    }
}

// Keep the old function for backward compatibility:
pub fn load_scene(name: &str) -> Option<SceneData> {
    warn!("Using deprecated load_scene, consider using SceneCache");
    let path = format!("scenes/{}.json", name);
    let content = std::fs::read_to_string(&path).ok()?;
    serde_json::from_str(&content)
        .map_err(|e| error!("Failed to parse scene {}: {}", name, e))
        .ok()
}

pub fn load_scene_cached(scene_cache: &mut SceneCache, name: &str) -> Option<SceneData> {
    scene_cache.get_scene(name).cloned()
}

/// Legacy function for compatibility - will be deprecated
pub fn load_scene_direct(name: &str) -> Option<SceneData> {
    warn!("Using deprecated load_scene_direct, consider using SceneCache");
    let path = format!("scenes/{}.json", name);
    let content = std::fs::read_to_string(&path).ok()?;
    serde_json::from_str(&content)
        .map_err(|e| error!("Failed to parse scene {}: {}", name, e))
        .ok()
}

pub fn spawn_from_scene(
    commands: &mut Commands, 
    scene: &SceneData, 
    global_data: &GlobalData, 
    sprites: &GameSprites
) {
    // Set up urban areas first
    setup_mission_urban_areas(commands, scene, global_data.selected_region);

    // Spawn all entity types
    spawn_agents(commands, &scene.agents, global_data, sprites);
    spawn_urban_civilians(commands, &scene.civilians, sprites);
    spawn_enemies(commands, &scene.enemies, global_data, sprites);
    spawn_terminals(commands, &scene.terminals, sprites);
    spawn_vehicles_from_scene(commands, &scene.vehicles, sprites);
    if let Some(police_units) = &scene.police {
        spawn_police_units(commands, police_units, sprites);
    }    
    spawn_cover_points(commands);
    
    info!("Urban mission spawned: {} agents, {} enemies, {} civilians, {} terminals, {} vehicles", 
          scene.agents.len(), scene.enemies.len(), scene.civilians.len(), 
          scene.terminals.len(), scene.vehicles.len());
}

// NEW: Setup urban areas based on mission type
fn setup_mission_urban_areas(commands: &mut Commands, scene: &SceneData, region_idx: usize) {
    let urban_areas = if let Some(scene_urban) = &scene.urban_areas {
        // Use scene-defined urban areas
        UrbanAreas {
            work_zones: scene_urban.work_zones.iter().map(|z| UrbanZone {
                center: Vec2::from(z.center),
                radius: z.radius,
                capacity: z.capacity,
                current_occupancy: 0,
            }).collect(),
            shopping_zones: scene_urban.shopping_zones.iter().map(|z| UrbanZone {
                center: Vec2::from(z.center),
                radius: z.radius,
                capacity: z.capacity,
                current_occupancy: 0,
            }).collect(),
            residential_zones: scene_urban.residential_zones.iter().map(|z| UrbanZone {
                center: Vec2::from(z.center),
                radius: z.radius,
                capacity: z.capacity,
                current_occupancy: 0,
            }).collect(),
            transit_routes: scene_urban.transit_routes.iter().map(|r| TransitRoute {
                points: r.points.iter().map(|&p| Vec2::from(p)).collect(),
                foot_traffic_density: r.foot_traffic_density,
            }).collect(),
        }
    } else {
        // Use mission-specific defaults based on region
        create_mission_urban_areas(region_idx)
    };
    
    commands.insert_resource(urban_areas);
}

// NEW: Create mission-specific urban areas
fn create_mission_urban_areas(region_idx: usize) -> UrbanAreas {
    match region_idx {
        0 => create_urban_district_areas(),     // Mission 1: Urban commercial
        1 => create_corporate_district_areas(), // Mission 2: Corporate complex
        2 => create_industrial_areas(),         // Mission 3: Industrial/underground
        _ => UrbanAreas::default(),
    }
}


pub fn spawn_fallback_mission(
    commands: &mut Commands,
    global_data: &GlobalData,
    sprites: &GameSprites,
) {
    warn!("Using fallback mission with urban simulation");
    
    // Set up basic urban areas for fallback
    commands.insert_resource(UrbanAreas::default());
    
    // Spawn agents
    let positions = [
        Vec2::new(-200.0, 0.0), 
        Vec2::new(-170.0, 0.0), 
        Vec2::new(-140.0, 0.0)
    ];
    for (i, &pos) in positions.iter().enumerate() {
        spawn_agent(commands, pos, global_data.agent_levels[i], i, global_data, sprites);
    }
    
    // Spawn some urban civilians
    let civilian_positions = [
        Vec2::new(100.0, 100.0),
        Vec2::new(150.0, 80.0),
        Vec2::new(80.0, 150.0),
    ];
    for &pos in &civilian_positions {
        spawn_single_urban_civilian(commands, pos, sprites);
    }
    
    spawn_terminal_direct(commands, Vec2::new(200.0, 0.0), TerminalType::Objective, sprites);
    spawn_cover_points(commands);
}


// === ENTITY SPAWNING FUNCTIONS ===
fn spawn_agents(commands: &mut Commands, agents: &[AgentSpawn], global_data: &GlobalData, sprites: &GameSprites) {
    for (i, agent_data) in agents.iter().enumerate() {
        let level = if i < 3 { global_data.agent_levels[i] } else { agent_data.level };
        spawn_agent(commands, Vec2::from(agent_data.position), level, i, global_data, sprites);
    }
}

fn spawn_civilians(commands: &mut Commands, civilians: &[CivilianSpawn], sprites: &GameSprites) {
    for civilian_data in civilians {
        spawn_civilian(commands, Vec2::from(civilian_data.position), sprites);
    }
}

fn spawn_enemies(commands: &mut Commands, enemies: &[EnemySpawn], global_data: &GlobalData, sprites: &GameSprites) {
    for enemy_data in enemies {
        let patrol_points = enemy_data.patrol_points.iter().map(|&p| Vec2::from(p)).collect();
        spawn_enemy_with_patrol(commands, Vec2::from(enemy_data.position), patrol_points, global_data, sprites);
    }
}

fn spawn_terminals(commands: &mut Commands, terminals: &[TerminalSpawn], sprites: &GameSprites) {
    for terminal_data in terminals {
        let terminal_type = parse_terminal_type(&terminal_data.terminal_type);
        spawn_terminal_direct(commands, Vec2::from(terminal_data.position), terminal_type, sprites);
    }
}

fn spawn_vehicles_from_scene(commands: &mut Commands, vehicles: &[VehicleSpawn], sprites: &GameSprites) {
    for vehicle_data in vehicles {
        let vehicle_type = parse_vehicle_type(&vehicle_data.vehicle_type);
        spawn_vehicle(commands, Vec2::from(vehicle_data.position), vehicle_type, sprites);
    }
}

// === INDIVIDUAL ENTITY SPAWNERS ===
fn spawn_agent(
    commands: &mut Commands, 
    position: Vec2, 
    level: u8, 
    agent_idx: usize, 
    global_data: &GlobalData, 
    sprites: &GameSprites
) {
    let (sprite, transform) = create_agent_sprite(sprites);
    let loadout = global_data.get_agent_loadout(agent_idx);
    
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
    
    inventory.add_currency(100 * level as u32);

    let weapon_state = if let Some(weapon_config) = loadout.weapon_configs.get(loadout.equipped_weapon_idx) {
        let mut state = WeaponState::new(&weapon_config.base_weapon);
        state.apply_attachment_modifiers(weapon_config);
        state
    } else {
        WeaponState::default()
    };    
    
    let agent = commands.spawn_empty()
    .insert((
        sprite,
        Transform::from_translation(position.extend(1.0)),
        Agent { experience: 0, level },
        Faction::Player,
        Health(100.0),
        MovementSpeed(150.0),
        Controllable,
        Selectable { radius: 15.0 },
    ))
    .insert((    
        Vision::new(150.0, 60.0),
        NeurovectorCapability::default(),
        inventory,
        weapon_state,
        RigidBody::Dynamic,
        Collider::ball(10.0),
        Velocity::default(),
        Damping { linear_damping: 10.0, angular_damping: 10.0 },
    ));
}

// This replaces spawn_civilian in scenes.rs for mission loading
pub fn spawn_urban_civilian_from_scene(
    commands: &mut Commands, 
    position: Vec2, 
    sprites: &GameSprites,
    urban_areas: &UrbanAreas,
) {
    urban_simulation::spawn_urban_civilian(commands, position, sprites, urban_areas);
}

fn spawn_civilian(commands: &mut Commands, position: Vec2, sprites: &GameSprites) {
    let (sprite, _) = create_civilian_sprite(sprites);
    
    commands.spawn((
        sprite,
        Transform::from_translation(position.extend(1.0)),
        Civilian,
        Health(50.0),
        Morale::new(80.0, 40.0),
        PanicSpreader::default(),
        MovementSpeed(100.0),
        Controllable,
        NeurovectorTarget,
        RigidBody::Dynamic,
        Collider::ball(7.5),
        Velocity::default(),
        Damping { linear_damping: 10.0, angular_damping: 10.0 },
    ));
}

pub fn spawn_civilian_with_config(
    commands: &mut Commands, 
    position: Vec2, 
    sprites: &GameSprites,
    config: &GameConfig,
) {
    let (sprite, _) = create_civilian_sprite(sprites);
    
    commands.spawn((
        sprite,
        Transform::from_translation(position.extend(1.0)),
        Civilian,
        Health(50.0),
        Morale::new(config.civilians.base_morale, config.civilians.panic_threshold),
        PanicSpreader::default(),
        MovementSpeed(config.civilians.movement_speed),
        Controllable,
        NeurovectorTarget,
        RigidBody::Dynamic,
        Collider::ball(7.5),
        Velocity::default(),
        Damping { linear_damping: 10.0, angular_damping: 10.0 },
    ));
}

fn spawn_enemy_with_patrol(
    commands: &mut Commands, 
    position: Vec2, 
    patrol_points: Vec<Vec2>, 
    global_data: &GlobalData, 
    sprites: &GameSprites
) {
    let (sprite, _) = create_enemy_sprite(sprites);
    let difficulty = global_data.regions[global_data.selected_region].mission_difficulty_modifier();
    
    // TESTING
    // Randomly assign faction for testing
    let faction = match rand::random::<f32>() {
        x if x < 0.4 => Faction::Corporate,
        x if x < 0.8 => Faction::Syndicate,
        _ => Faction::Police,
    };    

    // Vary weapon by faction
    let base_weapon = match faction {
        Faction::Corporate => match rand::random::<f32>() {
            x if x < 0.7 => WeaponType::Rifle,
            _ => WeaponType::Pistol,
        },
        Faction::Syndicate => match rand::random::<f32>() {
            x if x < 0.5 => WeaponType::Minigun,
            x if x < 0.8 => WeaponType::Flamethrower,
            _ => WeaponType::Rifle,
        },
        Faction::Police => WeaponType::Pistol,
        _ => WeaponType::Rifle,
    };
    
    let mut inventory = Inventory::default();
    inventory.equipped_weapon = Some(WeaponConfig::new(base_weapon.clone()));
    
    let enemy = commands.spawn_empty()
        .insert((
        sprite,
        Transform::from_translation(position.extend(1.0)),
        Enemy,
        faction,
        Health(100.0 * difficulty),
        Morale::new(100.0 * difficulty, 25.0),
        MovementSpeed(120.0 * difficulty),
        Vision::new(120.0 * difficulty, 45.0),
        Patrol::new(patrol_points),
        )) // DO NOT REMOVE/JOIN, BEVY HAS A MAX INCLUDE LIMIT AS ONCE
        .insert((
        AIState::default(),
        GoapAgent::default(),
        WeaponState::new(&base_weapon),
        inventory,
        RigidBody::Dynamic,
        Collider::ball(9.0),
        Velocity::default(),
        Damping { linear_damping: 10.0, angular_damping: 10.0 },
    ));
}

fn spawn_terminal_direct(commands: &mut Commands, position: Vec2, terminal_type: TerminalType, sprites: &GameSprites) {
    let (sprite, _) = create_terminal_sprite(sprites, &terminal_type);
    
    commands.spawn((
        sprite,
        Transform::from_translation(position.extend(1.0)),
        Terminal { terminal_type, range: 30.0, accessed: false },
        Selectable { radius: 15.0 },
    ));
}

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
        ));
    }
}

// === UTILITY FUNCTIONS ===
fn parse_terminal_type(type_str: &str) -> TerminalType {
    match type_str {
        "objective" => TerminalType::Objective,
        "equipment" => TerminalType::Equipment,
        "intel" => TerminalType::Intel,
        _ => {
            warn!("Unknown terminal type: {}, using Objective", type_str);
            TerminalType::Objective
        }
    }
}

fn parse_vehicle_type(type_str: &str) -> VehicleType {
    match type_str {
        "civilian_car" => VehicleType::CivilianCar,
        "police_car" => VehicleType::PoliceCar,
        "apc" => VehicleType::APC,
        "vtol" => VehicleType::VTOL,
        "tank" => VehicleType::Tank,
        _ => {
            warn!("Unknown vehicle type: {}, using CivilianCar", type_str);
            VehicleType::CivilianCar
        }
    }
}





// Mission 1: Neo-Tokyo Central - Urban commercial district
fn create_urban_district_areas() -> UrbanAreas {
    UrbanAreas {
        work_zones: vec![
            UrbanZone { center: Vec2::new(150.0, -80.0), radius: 70.0, capacity: 12, current_occupancy: 0 }, // Near equipment terminal
            UrbanZone { center: Vec2::new(50.0, 120.0), radius: 60.0, capacity: 8, current_occupancy: 0 },   // Near intel terminal
        ],
        shopping_zones: vec![
            UrbanZone { center: Vec2::new(200.0, 100.0), radius: 80.0, capacity: 15, current_occupancy: 0 }, // Main shopping area
            UrbanZone { center: Vec2::new(100.0, 60.0), radius: 50.0, capacity: 8, current_occupancy: 0 },   // Small shops
        ],
        residential_zones: vec![
            UrbanZone { center: Vec2::new(300.0, 180.0), radius: 90.0, capacity: 20, current_occupancy: 0 }, // Residential block
            UrbanZone { center: Vec2::new(80.0, 200.0), radius: 70.0, capacity: 12, current_occupancy: 0 },  // Apartments
        ],
        transit_routes: vec![
            // Main street (horizontal)
            TransitRoute { 
                points: vec![Vec2::new(-100.0, 0.0), Vec2::new(100.0, 0.0), Vec2::new(300.0, 0.0)], 
                foot_traffic_density: 0.8 
            },
            // Shopping district route
            TransitRoute { 
                points: vec![Vec2::new(150.0, -50.0), Vec2::new(200.0, 50.0), Vec2::new(250.0, 150.0)], 
                foot_traffic_density: 0.6 
            },
            // Residential connector
            TransitRoute { 
                points: vec![Vec2::new(100.0, 100.0), Vec2::new(200.0, 120.0), Vec2::new(300.0, 180.0)], 
                foot_traffic_density: 0.4 
            },
        ],
    }
}

// Mission 2: Corporate District
fn create_corporate_district_areas() -> UrbanAreas {
    UrbanAreas {
        work_zones: vec![
            UrbanZone { center: Vec2::new(400.0, -20.0), radius: 100.0, capacity: 25, current_occupancy: 0 }, // Main corporate tower
            UrbanZone { center: Vec2::new(100.0, -150.0), radius: 80.0, capacity: 15, current_occupancy: 0 },  // Secondary office
        ],
        shopping_zones: vec![
            UrbanZone { center: Vec2::new(200.0, 200.0), radius: 60.0, capacity: 10, current_occupancy: 0 }, // Corporate plaza shops
        ],
        residential_zones: vec![
            UrbanZone { center: Vec2::new(50.0, 100.0), radius: 80.0, capacity: 18, current_occupancy: 0 },  // Executive housing
            UrbanZone { center: Vec2::new(150.0, 50.0), radius: 70.0, capacity: 12, current_occupancy: 0 },  // Mid-level housing
        ],
        transit_routes: vec![
            // Corporate corridor
            TransitRoute { 
                points: vec![Vec2::new(0.0, -150.0), Vec2::new(200.0, -100.0), Vec2::new(400.0, -20.0)], 
                foot_traffic_density: 0.9 
            },
            // Residential to corporate
            TransitRoute { 
                points: vec![Vec2::new(50.0, 100.0), Vec2::new(150.0, 50.0), Vec2::new(300.0, 0.0)], 
                foot_traffic_density: 0.7 
            },
            // Plaza access
            TransitRoute { 
                points: vec![Vec2::new(150.0, 150.0), Vec2::new(200.0, 200.0), Vec2::new(250.0, 180.0)], 
                foot_traffic_density: 0.5 
            },
        ],
    }
}

// Mission 3: Industrial/Underground - Minimal civilian presence
fn create_industrial_areas() -> UrbanAreas {
    UrbanAreas {
        work_zones: vec![
            UrbanZone { center: Vec2::new(200.0, -100.0), radius: 60.0, capacity: 8, current_occupancy: 0 }, // Factory area
            UrbanZone { center: Vec2::new(350.0, -50.0), radius: 50.0, capacity: 6, current_occupancy: 0 },  // Industrial complex
        ],
        shopping_zones: vec![
            // Minimal - just a small supply depot
            UrbanZone { center: Vec2::new(50.0, -200.0), radius: 40.0, capacity: 4, current_occupancy: 0 },
        ],
        residential_zones: vec![
            UrbanZone { center: Vec2::new(-150.0, 200.0), radius: 70.0, capacity: 10, current_occupancy: 0 }, // Worker housing
        ],
        transit_routes: vec![
            // Main industrial route
            TransitRoute { 
                points: vec![Vec2::new(-200.0, 0.0), Vec2::new(0.0, -50.0), Vec2::new(200.0, -100.0), Vec2::new(400.0, -50.0)], 
                foot_traffic_density: 0.3 // Low foot traffic in industrial area
            },
            // Worker commute
            TransitRoute { 
                points: vec![Vec2::new(-150.0, 200.0), Vec2::new(0.0, 100.0), Vec2::new(150.0, 0.0)], 
                foot_traffic_density: 0.4 
            },
        ],
    }
}

// UPDATED: Replace spawn_civilians with spawn_urban_civilians
fn spawn_urban_civilians(commands: &mut Commands, civilians: &[CivilianSpawn], sprites: &GameSprites) {
    for civilian_data in civilians {
        spawn_single_urban_civilian(commands, Vec2::from(civilian_data.position), sprites);
    }
}

// NEW: Spawn individual urban civilian (for scene-placed civilians)
fn spawn_single_urban_civilian(commands: &mut Commands, position: Vec2, sprites: &GameSprites) {
    let (sprite, _) = create_civilian_sprite(sprites);
    
    // Give scene-placed civilians varied personalities
    let crowd_influence = 0.2 + rand::random::<f32>() * 0.6; // 0.2-0.8
    let panic_threshold = 15.0 + rand::random::<f32>() * 50.0; // 15-65
    
    // Scene civilians start in random appropriate states
    let daily_state = match rand::random::<f32>() {
        x if x < 0.3 => DailyState::Working,
        x if x < 0.5 => DailyState::Shopping,  
        x if x < 0.7 => DailyState::GoingHome,
        _ => DailyState::Idle,
    };
    
    commands.spawn((
        sprite,
        Transform::from_translation(position.extend(1.0)),
        Civilian,
        Health(50.0),
        Morale::new(80.0, panic_threshold),
        MovementSpeed(80.0 + rand::random::<f32>() * 40.0),
        Controllable,
        NeurovectorTarget,
        UrbanCivilian {
            daily_state,
            state_timer: rand::random::<f32>() * 15.0, // Stagger initial state changes
            next_destination: None, // Will be assigned by daily routine system
            crowd_influence,
            panic_threshold,
            movement_urgency: 0.0,
        },
        bevy_rapier2d::prelude::RigidBody::Dynamic,
        bevy_rapier2d::prelude::Collider::ball(7.5),
        bevy_rapier2d::prelude::Velocity::default(),
        bevy_rapier2d::prelude::Damping { linear_damping: 10.0, angular_damping: 10.0 },
    ));
}


// NEW: Spawn police units from scene data
fn spawn_police_units(commands: &mut Commands, police_data: &[PoliceSpawn], sprites: &GameSprites) {
    for police_spawn in police_data {
        let position = Vec2::from(police_spawn.position);
        let patrol_points: Vec<Vec2> = police_spawn.patrol_points.iter()
            .map(|&p| Vec2::from(p))
            .collect();
        
        let unit_type = parse_police_unit_type(&police_spawn.unit_type);
        spawn_scene_police_unit(commands, position, patrol_points, unit_type, sprites);
    }
}

fn parse_police_unit_type(type_str: &str) -> EscalationLevel {
    match type_str.to_lowercase().as_str() {
        "patrol" => EscalationLevel::Patrol,
        "armed" => EscalationLevel::Armed,
        "tactical" | "swat" => EscalationLevel::Tactical,
        "military" | "army" => EscalationLevel::Military,
        "corporate" | "elite" => EscalationLevel::Corporate,
        _ => {
            warn!("Unknown police unit type: {}, using Patrol", type_str);
            EscalationLevel::Patrol
        }
    }
}

// NEW: Spawn individual police unit for scenes
fn spawn_scene_police_unit(
    commands: &mut Commands,
    position: Vec2,
    patrol_points: Vec<Vec2>,
    unit_type: EscalationLevel,
    sprites: &GameSprites,
) {
    let (mut sprite, _) = crate::core::sprites::create_enemy_sprite(sprites);
    sprite.color = unit_type.color();
    
    let (health, weapon, speed, vision_range) = match unit_type {
        EscalationLevel::Patrol => (80.0, WeaponType::Pistol, 100.0, 100.0),
        EscalationLevel::Armed => (120.0, WeaponType::Rifle, 120.0, 120.0),
        EscalationLevel::Tactical => (150.0, WeaponType::Rifle, 140.0, 140.0),
        EscalationLevel::Military => (180.0, WeaponType::Minigun, 130.0, 160.0),
        EscalationLevel::Corporate => (200.0, WeaponType::Flamethrower, 150.0, 180.0),
        EscalationLevel::None => (100.0, WeaponType::Pistol, 100.0, 100.0),
    };
    
    // Use provided patrol points or create simple patrol
    let patrol = if patrol_points.is_empty() {
        Patrol::new(vec![position, position + Vec2::new(80.0, 0.0)])
    } else {
        Patrol::new(patrol_points)
    };
    
    let mut inventory = Inventory::default();
    inventory.equipped_weapon = Some(WeaponConfig::new(weapon.clone()));
    
    // Enhanced AI for higher tiers
    let mut ai_state = AIState::default();
    ai_state.use_goap = unit_type >= EscalationLevel::Tactical;
    
    commands.spawn_empty()
    .insert((
        sprite,
        Transform::from_translation(position.extend(1.0)),
        Enemy,
        Police { response_level: unit_type as u8 },
        Faction::Police,
        Health(health),
        Morale::new(health * 1.5, 20.0),
        MovementSpeed(speed),
        Vision::new(vision_range, 50.0),
        patrol,
    ))
    .insert((
        ai_state,
        GoapAgent::default(),
        WeaponState::new(&weapon),
        inventory,
        bevy_rapier2d::prelude::RigidBody::Dynamic,
        bevy_rapier2d::prelude::Collider::ball(9.0),
        bevy_rapier2d::prelude::Velocity::default(),
        bevy_rapier2d::prelude::Damping { linear_damping: 10.0, angular_damping: 10.0 },
    ));
}