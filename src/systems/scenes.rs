use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use serde::{Deserialize, Serialize};
use crate::core::*;
use crate::systems::*;

// First, let's create the scenes directory structure
pub fn ensure_scenes_directory() {
    if let Err(_) = std::fs::create_dir_all("scenes") {
        warn!("Could not create scenes directory");
    }
    
    // Create default mission files if they don't exist
    let missions = [
        ("mission1", SceneData::mission1()),
        ("mission2", SceneData::mission2()),
        ("mission3", SceneData::mission3()),
    ];
    
    for (name, scene) in missions {
        let path = format!("scenes/{}.json", name);
        if !std::path::Path::new(&path).exists() {
            if let Ok(json) = serde_json::to_string_pretty(&scene) {
                if let Err(e) = std::fs::write(&path, json) {
                    warn!("Failed to create {}: {}", path, e);
                } else {
                    info!("Created default scene file: {}", path);
                }
            }
        }
    }
}

 
#[derive(Serialize, Deserialize)]
pub struct SceneData {
    pub agents: Vec<AgentSpawn>,
    pub civilians: Vec<CivilianSpawn>,
    pub enemies: Vec<EnemySpawn>,
    pub terminals: Vec<TerminalSpawn>,
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

impl SceneData {
    // Mission 1: Tutorial/Easy
    pub fn mission1() -> Self {
        Self {
            agents: vec![
                AgentSpawn { position: [-200.0, 0.0], level: 1 },
                AgentSpawn { position: [-170.0, 0.0], level: 1 },
                AgentSpawn { position: [-140.0, 0.0], level: 1 },
            ],
            civilians: vec![
                CivilianSpawn { position: [100.0, 100.0] },
                CivilianSpawn { position: [160.0, 120.0] },
                CivilianSpawn { position: [220.0, 80.0] },
            ],
            enemies: vec![
                EnemySpawn {
                    position: [200.0, -100.0],
                    patrol_points: vec![
                        [200.0, -100.0],
                        [300.0, -100.0],
                        [300.0, 50.0],
                        [200.0, 50.0],
                    ],
                },
            ],
            terminals: vec![
                TerminalSpawn { position: [320.0, -50.0], terminal_type: "objective".to_string() },
                TerminalSpawn { position: [150.0, -80.0], terminal_type: "equipment".to_string() },
                TerminalSpawn { position: [50.0, 120.0], terminal_type: "intel".to_string() },
            ],
        }
    }

    // Mission 2: Medium difficulty
    pub fn mission2() -> Self {
        Self {
            agents: vec![
                AgentSpawn { position: [-250.0, -50.0], level: 1 },
                AgentSpawn { position: [-220.0, -50.0], level: 1 },
                AgentSpawn { position: [-190.0, -50.0], level: 1 },
            ],
            civilians: vec![
                CivilianSpawn { position: [80.0, 150.0] },
                CivilianSpawn { position: [120.0, 180.0] },
            ],
            enemies: vec![
                EnemySpawn {
                    position: [150.0, -120.0],
                    patrol_points: vec![
                        [150.0, -120.0],
                        [250.0, -120.0],
                    ],
                },
                EnemySpawn {
                    position: [350.0, 50.0],
                    patrol_points: vec![
                        [350.0, 50.0],
                        [350.0, 150.0],
                        [250.0, 150.0],
                    ],
                },
            ],
            terminals: vec![
                TerminalSpawn { position: [400.0, -20.0], terminal_type: "objective".to_string() },
                TerminalSpawn { position: [200.0, 200.0], terminal_type: "objective".to_string() },
                TerminalSpawn { position: [100.0, -150.0], terminal_type: "equipment".to_string() },
            ],
        }
    }

    // Mission 3: Hard
    pub fn mission3() -> Self {
        Self {
            agents: vec![
                AgentSpawn { position: [-300.0, 0.0], level: 1 },
                AgentSpawn { position: [-270.0, 0.0], level: 1 },
                AgentSpawn { position: [-240.0, 0.0], level: 1 },
            ],
            civilians: vec![], // No civilians in underground
            enemies: vec![
                EnemySpawn {
                    position: [100.0, -150.0],
                    patrol_points: vec![
                        [100.0, -150.0],
                        [200.0, -150.0],
                        [200.0, -50.0],
                        [100.0, -50.0],
                    ],
                },
                EnemySpawn {
                    position: [300.0, 100.0],
                    patrol_points: vec![
                        [300.0, 100.0],
                        [400.0, 100.0],
                    ],
                },
                EnemySpawn {
                    position: [150.0, 200.0],
                    patrol_points: vec![
                        [150.0, 200.0],
                        [250.0, 200.0],
                        [250.0, 300.0],
                    ],
                },
            ],
            terminals: vec![
                TerminalSpawn { position: [450.0, 50.0], terminal_type: "objective".to_string() },
                TerminalSpawn { position: [200.0, 350.0], terminal_type: "objective".to_string() },
                TerminalSpawn { position: [50.0, -200.0], terminal_type: "equipment".to_string() },
                TerminalSpawn { position: [350.0, -100.0], terminal_type: "intel".to_string() },
            ],
        }
    }
}

impl Default for SceneData {
    fn default() -> Self {
        Self::mission1()
    }
}

pub fn load_scene(name: &str) -> SceneData {
    let path = format!("scenes/{}.json", name);
    
    // First try to load from file
    if let Ok(content) = std::fs::read_to_string(&path) {
        if let Ok(scene) = serde_json::from_str(&content) {
            return scene;
        }
    }
    
    // Fall back to built-in scenes
    let scene = match name {
        "mission1" => SceneData::mission1(),
        "mission2" => SceneData::mission2(),
        "mission3" => SceneData::mission3(),
        _ => {
            warn!("Unknown scene '{}', using mission1", name);
            SceneData::mission1()
        }
    };
    
    scene
}

pub fn spawn_from_scene(
    commands: &mut Commands, 
    scene: &SceneData, 
    global_data: &GlobalData, 
    sprites: &GameSprites
) {
    for (i, agent_data) in scene.agents.iter().enumerate() {
        let level = if i < 3 { global_data.agent_levels[i] } else { agent_data.level };
        spawn_agent(commands, Vec2::from(agent_data.position), level, i, global_data, sprites);
    }
    
    for civilian_data in &scene.civilians {
        spawn_civilian(commands, Vec2::from(civilian_data.position), sprites);
    }
    
    for enemy_data in &scene.enemies {
        let patrol_points = enemy_data.patrol_points.iter().map(|&p| Vec2::from(p)).collect();
        spawn_enemy_with_patrol(commands, Vec2::from(enemy_data.position), patrol_points, global_data, sprites);
    }
    
    for terminal_data in &scene.terminals {
        let terminal_type = match terminal_data.terminal_type.as_str() {
            "objective" => TerminalType::Objective,
            "equipment" => TerminalType::Equipment,
            "intel" => TerminalType::Intel,
            _ => TerminalType::Objective,
        };
        spawn_terminal(commands, Vec2::from(terminal_data.position), terminal_type, sprites);
    }

    spawn_cover_points(commands);    
}

fn spawn_agent(
    commands: &mut Commands, 
    position: Vec2, 
    level: u8, 
    agent_idx: usize, 
    global_data: &GlobalData, 
    sprites: &GameSprites
) {
    let (sprite, mut transform) = crate::core::sprites::create_agent_sprite(sprites);
    transform.translation = position.extend(1.0);
    
    // Load agent's saved configuration
    let loadout = global_data.get_agent_loadout(agent_idx);
    
    let mut inventory = Inventory::default();
    
    // Apply saved weapon configurations
    for weapon_config in &loadout.weapon_configs {
        inventory.add_weapon_config(weapon_config.clone());
    }
    
    // Set equipped weapon
    if let Some(weapon_config) = loadout.weapon_configs.get(loadout.equipped_weapon_idx) {
        inventory.equipped_weapon = Some(weapon_config.clone());
    }
    
    // Apply saved tools and cybernetics
    for tool in &loadout.tools {
        inventory.add_tool(tool.clone());
    }
    
    for cybernetic in &loadout.cybernetics {
        inventory.add_cybernetic(cybernetic.clone());
    }
    
    // Starting credits based on level
    inventory.add_currency(100 * level as u32);

    // Create weapon state based on equipped weapon
    let weapon_state = if let Some(weapon_config) = loadout.weapon_configs.get(loadout.equipped_weapon_idx) {
        let mut state = WeaponState::new(&weapon_config.base_weapon);
        state.apply_attachment_modifiers(weapon_config);
        state
    } else {
        WeaponState::default()
    };    
    
    let entity = commands.spawn((
        sprite,
        transform,
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
    )).id();
    
}

fn spawn_civilian(commands: &mut Commands, position: Vec2, sprites: &GameSprites) {
    let (sprite, mut transform) = crate::core::sprites::create_civilian_sprite(sprites);
    transform.translation = position.extend(1.0);
    
    commands.spawn((
        sprite,
        transform,
        Civilian,
        Health(50.0),
        Morale::new(80.0, 40.0),
        PanicSpreader::default(),  // ADD THIS LINE
        MovementSpeed(100.0),
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
    let region = &global_data.regions[global_data.selected_region];
    let difficulty = region.mission_difficulty_modifier();
    
    let (sprite, mut transform) = crate::core::sprites::create_enemy_sprite(sprites);
    transform.translation = position.extend(1.0);
    
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
        transform,
        Enemy,
        Health(100.0 * difficulty),
        Morale::new(100.0 * difficulty, 25.0),
        MovementSpeed(120.0 * difficulty),
        Vision::new(120.0 * difficulty, 45.0),
        Patrol::new(patrol_points.clone())
    ))
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

fn spawn_terminal(commands: &mut Commands, position: Vec2, terminal_type: TerminalType, sprites: &GameSprites) {
    let (sprite, mut transform) = crate::core::sprites::create_terminal_sprite(sprites, &terminal_type);
    transform.translation = position.extend(1.0);
    
    let entity = commands.spawn((
        sprite,
        transform,
        Terminal { terminal_type: terminal_type.clone(), range: 30.0, accessed: false },
        Selectable { radius: 15.0 },
    )).id();
    
    
}

pub fn spawn_cover_points(commands: &mut Commands) {
    let cover_positions = [
        Vec2::new(50.0, -50.0),
        Vec2::new(250.0, -150.0),
        Vec2::new(-50.0, 100.0),
        Vec2::new(300.0, 50.0),
        Vec2::new(150.0, 150.0),
    ];
    
    for &pos in &cover_positions {
        let entity = commands.spawn((
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
        )).id();
    }
}

fn alert_color(alert_level: AlertLevel) -> Color {
    match alert_level {
        AlertLevel::Green => Color::srgb(0.8, 0.2, 0.2),
        AlertLevel::Yellow => Color::srgb(0.8, 0.5, 0.2),
        AlertLevel::Orange => Color::srgb(0.8, 0.8, 0.2),
        AlertLevel::Red => Color::srgb(1.0, 0.2, 0.2),
    }
}