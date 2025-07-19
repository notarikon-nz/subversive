use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use serde::{Deserialize, Serialize};
use crate::core::*;
use crate::systems::*;

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

impl Default for SceneData {
    fn default() -> Self {
        Self {
            agents: vec![
                AgentSpawn { position: [-200.0, 0.0], level: 1 },
                AgentSpawn { position: [-150.0, 0.0], level: 1 },
                AgentSpawn { position: [-100.0, 0.0], level: 1 },
            ],
            civilians: vec![
                CivilianSpawn { position: [100.0, 100.0] },
                CivilianSpawn { position: [160.0, 120.0] },
                CivilianSpawn { position: [220.0, 80.0] },
                CivilianSpawn { position: [280.0, 140.0] },
                CivilianSpawn { position: [340.0, 60.0] },
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
}

pub fn load_scene(name: &str) -> SceneData {
    let path = format!("scenes/{}.json", name);
    std::fs::read_to_string(&path)
        .ok()
        .and_then(|content| serde_json::from_str(&content).ok())
        .unwrap_or_else(|| {
            warn!("Failed to load scene '{}', using default", name);
            SceneData::default()
        })
}

pub fn spawn_from_scene(commands: &mut Commands, scene: &SceneData, global_data: &GlobalData, sprites: &GameSprites) {
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

fn spawn_agent(commands: &mut Commands, position: Vec2, level: u8, agent_idx: usize, global_data: &GlobalData, sprites: &GameSprites) {
    let mut sprite_bundle = crate::core::sprites::create_agent_sprite(sprites);
    sprite_bundle.transform = Transform::from_translation(position.extend(1.0));
    
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
    
    info!("Spawned Agent {} with {} weapons, {} tools", 
          agent_idx + 1, 
          inventory.weapons.len(), 
          inventory.tools.len());
    
    commands.spawn((
        sprite_bundle,
        Agent { experience: 0, level },
        Health(100.0),
        MovementSpeed(150.0),
        Controllable,
        Selectable { radius: 15.0 },
        Vision::new(150.0, 60.0),
        NeurovectorCapability::default(),
        inventory,
        RigidBody::Dynamic,
        Collider::ball(10.0),
        Velocity::default(),
        Damping { linear_damping: 10.0, angular_damping: 10.0 },
    ));
}


fn spawn_civilian(commands: &mut Commands, position: Vec2, sprites: &GameSprites) {
    let mut sprite_bundle = crate::core::sprites::create_civilian_sprite(sprites);
    sprite_bundle.transform = Transform::from_translation(position.extend(1.0));
    
    commands.spawn((
        sprite_bundle,
        Civilian,
        Health(50.0),
        MovementSpeed(100.0),
        Controllable,
        NeurovectorTarget,
        RigidBody::Dynamic,
        Collider::ball(7.5),
        Velocity::default(),
        Damping { linear_damping: 10.0, angular_damping: 10.0 },
    ));
}

fn spawn_enemy_with_patrol(commands: &mut Commands, position: Vec2, patrol_points: Vec<Vec2>, global_data: &GlobalData, sprites: &GameSprites) {
    let region = &global_data.regions[global_data.selected_region];
    let difficulty = region.mission_difficulty_modifier();
    
    let mut sprite_bundle = crate::core::sprites::create_enemy_sprite(sprites);
    sprite_bundle.transform = Transform::from_translation(position.extend(1.0));
    
    // Spawn with both AI systems - GOAP is primary, legacy as fallback
    commands.spawn((
        sprite_bundle,
        Enemy,
        Health(100.0 * difficulty),
        MovementSpeed(120.0 * difficulty),
        Vision::new(120.0 * difficulty, 45.0),
        Patrol::new(patrol_points),
        AIState::default(), // Legacy AI state
        GoapAgent::default(), // GOAP AI agent
        RigidBody::Dynamic,
        Collider::ball(9.0),
        Velocity::default(),
        Damping { linear_damping: 10.0, angular_damping: 10.0 },
    ));
}


fn spawn_terminal(commands: &mut Commands, position: Vec2, terminal_type: TerminalType, sprites: &GameSprites) {
    let mut sprite_bundle = crate::core::sprites::create_terminal_sprite(sprites, &terminal_type);
    sprite_bundle.transform = Transform::from_translation(position.extend(1.0));
    
    commands.spawn((
        sprite_bundle,
        Terminal { terminal_type, range: 30.0, accessed: false },
        Selectable { radius: 15.0 },
    ));
}

pub fn spawn_cover_points(commands: &mut Commands) {
    let cover_positions = [
        Vec2::new(50.0, -50.0),   // Near terminals
        Vec2::new(250.0, -150.0), // Corner positions
        Vec2::new(-50.0, 100.0),  // Scattered around map
        Vec2::new(300.0, 50.0),
        Vec2::new(150.0, 150.0),
    ];
    
    for &pos in &cover_positions {
        commands.spawn((
            SpriteBundle {
                sprite: Sprite {
                    color: Color::srgba(0.4, 0.2, 0.1, 0.7), // Brown, semi-transparent
                    custom_size: Some(Vec2::new(20.0, 40.0)), // Rectangular cover
                    ..default()
                },
                transform: Transform::from_translation(pos.extend(0.5)), // Slightly behind other objects
                ..default()
            },
            CoverPoint {
                capacity: 2,
                current_users: 0,
                cover_direction: Vec2::X, // Covers from the right (will be calculated dynamically)
            },
        ));
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