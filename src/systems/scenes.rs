// src/systems/scenes.rs - Optimized and cleaned up
// 25869 -> 20313 21% smaller

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
    pub unit_type: String,
}

// === CORE FUNCTIONS ===
pub fn ensure_scenes_directory() {
    if std::fs::create_dir_all("scenes").is_err() {
        error!("Could not create scenes directory");
    }
}

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

pub fn load_scene_direct(name: &str) -> Option<SceneData> {
    warn!("Using deprecated load_scene_direct, consider using SceneCache");
    let path = format!("scenes/{}.json", name);
    let content = std::fs::read_to_string(&path).ok()?;
    serde_json::from_str(&content)
        .map_err(|e| error!("Failed to parse scene {}: {}", name, e))
        .ok()
}

pub fn spawn_from_scene(commands: &mut Commands, scene: &SceneData, global_data: &GlobalData, sprites: &GameSprites) {
    setup_urban_areas(commands, scene, global_data.selected_region);

    for (i, agent) in scene.agents.iter().enumerate() {
        let level = if i < 3 { global_data.agent_levels[i] } else { agent.level };
        spawn_agent(commands, Vec2::from(agent.position), level, i, global_data, sprites);
    }

    for civilian in &scene.civilians {
        spawn_urban_civilian(commands, Vec2::from(civilian.position), sprites);
    }

    for enemy in &scene.enemies {
        let patrol = enemy.patrol_points.iter().map(|&p| Vec2::from(p)).collect();
        spawn_enemy(commands, Vec2::from(enemy.position), patrol, global_data, sprites);
    }

    for terminal in &scene.terminals {
        let t_type = parse_terminal_type(&terminal.terminal_type);
        spawn_terminal(commands, Vec2::from(terminal.position), t_type, sprites);
    }

    for vehicle in &scene.vehicles {
        let v_type = parse_vehicle_type(&vehicle.vehicle_type);
        spawn_vehicle(commands, Vec2::from(vehicle.position), v_type, sprites);
    }

    if let Some(police) = &scene.police {
        for unit in police {
            let patrol = unit.patrol_points.iter().map(|&p| Vec2::from(p)).collect();
            let unit_type = parse_police_unit_type(&unit.unit_type);
            spawn_police(commands, Vec2::from(unit.position), patrol, unit_type, sprites);
        }
    }

    spawn_cover_points(commands);
    
    info!("Mission spawned: {} agents, {} enemies, {} civilians, {} terminals, {} vehicles", 
          scene.agents.len(), scene.enemies.len(), scene.civilians.len(), 
          scene.terminals.len(), scene.vehicles.len());
}

pub fn spawn_fallback_mission(commands: &mut Commands, global_data: &GlobalData, sprites: &GameSprites) {
    warn!("Using fallback mission with urban simulation");
    
    commands.insert_resource(UrbanAreas::default());
    
    let positions = [Vec2::new(-200.0, 0.0), Vec2::new(-170.0, 0.0), Vec2::new(-140.0, 0.0)];
    for (i, &pos) in positions.iter().enumerate() {
        spawn_agent(commands, pos, global_data.agent_levels[i], i, global_data, sprites);
    }
    
    let civilian_positions = [Vec2::new(100.0, 100.0), Vec2::new(150.0, 80.0), Vec2::new(80.0, 150.0)];
    for &pos in &civilian_positions {
        spawn_urban_civilian(commands, pos, sprites);
    }
    
    spawn_terminal(commands, Vec2::new(200.0, 0.0), TerminalType::Objective, sprites);
    spawn_cover_points(commands);
}

// === ENTITY SPAWNERS ===
fn spawn_agent(commands: &mut Commands, pos: Vec2, level: u8, idx: usize, global_data: &GlobalData, sprites: &GameSprites) {
    let (sprite, _) = create_agent_sprite(sprites);
    let loadout = global_data.get_agent_loadout(idx);
    
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

    let e = commands.spawn_empty()
    .insert((
        sprite,
        Transform::from_translation(pos.extend(1.0)),
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

fn spawn_urban_civilian(commands: &mut Commands, pos: Vec2, sprites: &GameSprites) {
    let (sprite, _) = create_civilian_sprite(sprites);
    let crowd_influence = 0.2 + rand::random::<f32>() * 0.6;
    let panic_threshold = 15.0 + rand::random::<f32>() * 50.0;
    let daily_state = match rand::random::<f32>() {
        x if x < 0.3 => DailyState::Working,
        x if x < 0.5 => DailyState::Shopping,  
        x if x < 0.7 => DailyState::GoingHome,
        _ => DailyState::Idle,
    };

    commands.spawn((
        sprite,
        Transform::from_translation(pos.extend(1.0)),
        Civilian,
        Health(50.0),
        Morale::new(80.0, panic_threshold),
        MovementSpeed(80.0 + rand::random::<f32>() * 40.0),
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
        RigidBody::Dynamic,
        Collider::ball(7.5),
        Velocity::default(),
        Damping { linear_damping: 10.0, angular_damping: 10.0 },
    ));
}

pub fn spawn_civilian_with_config(commands: &mut Commands, pos: Vec2, sprites: &GameSprites, config: &GameConfig) {
    let (sprite, _) = create_civilian_sprite(sprites);
    
    commands.spawn((
        sprite,
        Transform::from_translation(pos.extend(1.0)),
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

fn spawn_enemy(commands: &mut Commands, pos: Vec2, patrol: Vec<Vec2>, global_data: &GlobalData, sprites: &GameSprites) {
    let (sprite, _) = create_enemy_sprite(sprites);
    let difficulty = global_data.regions[global_data.selected_region].mission_difficulty_modifier();
    
    let faction = match rand::random::<f32>() {
        x if x < 0.4 => Faction::Corporate,
        x if x < 0.8 => Faction::Syndicate,
        _ => Faction::Police,
    };

    let weapon = match faction {
        Faction::Corporate => if rand::random::<f32>() < 0.7 { WeaponType::Rifle } else { WeaponType::Pistol },
        Faction::Syndicate => match rand::random::<f32>() {
            x if x < 0.5 => WeaponType::Minigun,
            x if x < 0.8 => WeaponType::Flamethrower,
            _ => WeaponType::Rifle,
        },
        _ => WeaponType::Pistol,
    };
    
    let mut inventory = Inventory::default();
    inventory.equipped_weapon = Some(WeaponConfig::new(weapon.clone()));

    let e = commands.spawn_empty()
    .insert ((
        sprite,
        Transform::from_translation(pos.extend(1.0)),
        Enemy,
        faction,
        Health(100.0 * difficulty),
        Morale::new(100.0 * difficulty, 25.0),
        MovementSpeed(120.0 * difficulty),
        Vision::new(120.0 * difficulty, 45.0),
    ))
    .insert ((
        Patrol::new(patrol),
        AIState::default(),
        GoapAgent::default(),
        WeaponState::new(&weapon),
        inventory,
        RigidBody::Dynamic,
        Collider::ball(9.0),
        Velocity::default(),
        Damping { linear_damping: 10.0, angular_damping: 10.0 },
    ));
}

fn spawn_terminal(commands: &mut Commands, pos: Vec2, terminal_type: TerminalType, sprites: &GameSprites) {
    let (sprite, _) = create_terminal_sprite(sprites, &terminal_type);
    
    commands.spawn((
        sprite,
        Transform::from_translation(pos.extend(1.0)),
        Terminal { terminal_type, range: 30.0, accessed: false },
        Selectable { radius: 15.0 },
    ));
}

fn spawn_police(commands: &mut Commands, pos: Vec2, patrol: Vec<Vec2>, unit_type: EscalationLevel, sprites: &GameSprites) {
    let (mut sprite, _) = create_enemy_sprite(sprites);
    sprite.color = unit_type.color();
    
    let (health, weapon, speed, vision_range) = match unit_type {
        EscalationLevel::Patrol => (80.0, WeaponType::Pistol, 100.0, 100.0),
        EscalationLevel::Armed => (120.0, WeaponType::Rifle, 120.0, 120.0),
        EscalationLevel::Tactical => (150.0, WeaponType::Rifle, 140.0, 140.0),
        EscalationLevel::Military => (180.0, WeaponType::Minigun, 130.0, 160.0),
        EscalationLevel::Corporate => (200.0, WeaponType::Flamethrower, 150.0, 180.0),
        EscalationLevel::None => (100.0, WeaponType::Pistol, 100.0, 100.0),
    };
    
    let patrol = if patrol.is_empty() {
        Patrol::new(vec![pos, pos + Vec2::new(80.0, 0.0)])
    } else {
        Patrol::new(patrol)
    };
    
    let mut inventory = Inventory::default();
    inventory.equipped_weapon = Some(WeaponConfig::new(weapon.clone()));
    
    let mut ai_state = AIState::default();
    ai_state.use_goap = unit_type >= EscalationLevel::Tactical;

    let e = commands.spawn_empty()
    .insert((
        sprite,
        Transform::from_translation(pos.extend(1.0)),
        Enemy,
        Police { response_level: unit_type as u8 },
        Faction::Police,
        Health(health),
        Morale::new(health * 1.5, 20.0),
        MovementSpeed(speed),
    ))
    .insert((
        Vision::new(vision_range, 50.0),
        patrol,
        ai_state,
        GoapAgent::default(),
        WeaponState::new(&weapon),
        inventory,
        RigidBody::Dynamic,
        Collider::ball(9.0),
        Velocity::default(),
        Damping { linear_damping: 10.0, angular_damping: 10.0 },
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

// === URBAN AREAS SETUP ===
fn setup_urban_areas(commands: &mut Commands, scene: &SceneData, region_idx: usize) {
    let urban_areas = if let Some(scene_urban) = &scene.urban_areas {
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
        match region_idx {
            0 => create_urban_district_areas(),
            1 => create_corporate_district_areas(),
            2 => create_industrial_areas(),
            _ => UrbanAreas::default(),
        }
    };
    
    commands.insert_resource(urban_areas);
}

fn create_urban_district_areas() -> UrbanAreas {
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