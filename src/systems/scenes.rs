// src/systems/scenes.rs - Optimized and cleaned up
use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use serde::{Deserialize, Serialize};
use crate::core::*;
use crate::systems::panic_spread::*;
use crate::systems::ai::*;
use crate::systems::vehicles::spawn_vehicle;

// === SCENE DATA STRUCTURES ===
#[derive(Serialize, Deserialize)]
pub struct SceneData {
    pub agents: Vec<AgentSpawn>,
    pub civilians: Vec<CivilianSpawn>,
    pub enemies: Vec<EnemySpawn>,
    pub terminals: Vec<TerminalSpawn>,
    pub vehicles: Vec<VehicleSpawn>,
}

#[derive(Serialize, Deserialize)]
pub struct AgentSpawn {
    pub position: [f32; 2],
    pub level: u8,
}

#[derive(Serialize, Deserialize)]
pub struct CivilianSpawn {
    pub position: [f32; 2],
}

#[derive(Serialize, Deserialize)]
pub struct EnemySpawn {
    pub position: [f32; 2],
    pub patrol_points: Vec<[f32; 2]>,
}

#[derive(Serialize, Deserialize)]
pub struct TerminalSpawn {
    pub position: [f32; 2],
    pub terminal_type: String,
}

#[derive(Serialize, Deserialize)]
pub struct VehicleSpawn {
    pub position: [f32; 2],
    pub vehicle_type: String,
}

// === CORE FUNCTIONS ===
pub fn ensure_scenes_directory() {
    if std::fs::create_dir_all("scenes").is_err() {
        error!("Could not create scenes directory");
    }
}

pub fn load_scene(name: &str) -> Option<SceneData> {
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
    // Spawn all entity types
    spawn_agents(commands, &scene.agents, global_data, sprites);
    spawn_civilians(commands, &scene.civilians, sprites);
    spawn_enemies(commands, &scene.enemies, global_data, sprites);
    spawn_terminals(commands, &scene.terminals, sprites);
    spawn_vehicles_from_scene(commands, &scene.vehicles, sprites);
    spawn_cover_points(commands);
    
    info!("Scene spawned: {} agents, {} enemies, {} civilians, {} terminals, {} vehicles", 
          scene.agents.len(), scene.enemies.len(), scene.civilians.len(), 
          scene.terminals.len(), scene.vehicles.len());
}

pub fn spawn_fallback_mission(
    commands: &mut Commands,
    global_data: &GlobalData,
    sprites: &GameSprites,
) {
    warn!("Using fallback mission");
    // Minimal viable mission - 3 agents in formation
    let positions = [
        Vec2::new(-200.0, 0.0), 
        Vec2::new(-170.0, 0.0), 
        Vec2::new(-140.0, 0.0)
    ];
    for (i, &pos) in positions.iter().enumerate() {
        spawn_agent(commands, pos, global_data.agent_levels[i], i, global_data, sprites);
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
    
    commands.spawn((
        sprite,
        Transform::from_translation(position.extend(1.0)),
        Agent { experience: 0, level },
        Health(100.0),
        MovementSpeed(150.0),
        Controllable,
        Selectable { radius: 15.0 },
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
    
    let base_weapon = match rand::random::<f32>() {
        x if x < 0.6 => WeaponType::Rifle,
        x if x < 0.8 => WeaponType::Pistol,
        x if x < 0.9 => WeaponType::Minigun,
        _ => WeaponType::Flamethrower,
    };
    
    let mut inventory = Inventory::default();
    inventory.equipped_weapon = Some(WeaponConfig::new(base_weapon.clone()));
    
    let enemy = commands.spawn_empty()
        .insert((
        sprite,
        Transform::from_translation(position.extend(1.0)),
        Enemy,
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

