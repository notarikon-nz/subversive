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

pub fn spawn_from_scene(commands: &mut Commands, scene: &SceneData, global_data: &GlobalData) {
    for (i, agent_data) in scene.agents.iter().enumerate() {
        let level = if i < 3 { global_data.agent_levels[i] } else { agent_data.level };
        spawn_agent(commands, Vec2::from(agent_data.position), level);
    }
    
    for civilian_data in &scene.civilians {
        spawn_civilian(commands, Vec2::from(civilian_data.position));
    }
    
    for enemy_data in &scene.enemies {
        let patrol_points = enemy_data.patrol_points.iter().map(|&p| Vec2::from(p)).collect();
        spawn_enemy_with_patrol(commands, Vec2::from(enemy_data.position), patrol_points, global_data);
    }
    
    for terminal_data in &scene.terminals {
        let terminal_type = match terminal_data.terminal_type.as_str() {
            "objective" => TerminalType::Objective,
            "equipment" => TerminalType::Equipment,
            "intel" => TerminalType::Intel,
            _ => TerminalType::Objective,
        };
        spawn_terminal(commands, Vec2::from(terminal_data.position), terminal_type);
    }
}

fn spawn_agent(commands: &mut Commands, position: Vec2, level: u8) {
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::srgb(0.2, 0.8, 0.2),
                custom_size: Some(Vec2::new(20.0, 20.0)),
                ..default()
            },
            transform: Transform::from_translation(position.extend(1.0)),
            ..default()
        },
        Agent { experience: 0, level },
        Health(100.0),
        MovementSpeed(150.0),
        Controllable,
        Selectable { radius: 15.0 },
        Vision::new(150.0, 60.0),
        NeurovectorCapability::default(),
        Inventory::default(),
        RigidBody::Dynamic,
        Collider::ball(10.0),
        Velocity::default(),
        Damping { linear_damping: 10.0, angular_damping: 10.0 },
    ));
}

fn spawn_civilian(commands: &mut Commands, position: Vec2) {
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::srgb(0.8, 0.8, 0.2),
                custom_size: Some(Vec2::new(15.0, 15.0)),
                ..default()
            },
            transform: Transform::from_translation(position.extend(1.0)),
            ..default()
        },
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

fn spawn_enemy_with_patrol(commands: &mut Commands, position: Vec2, patrol_points: Vec<Vec2>, global_data: &GlobalData) {
    let region = &global_data.regions[global_data.selected_region];
    let difficulty = region.mission_difficulty_modifier();
    
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: alert_color(region.alert_level),
                custom_size: Some(Vec2::new(18.0, 18.0)),
                ..default()
            },
            transform: Transform::from_translation(position.extend(1.0)),
            ..default()
        },
        Enemy,
        Health(100.0 * difficulty),
        MovementSpeed(120.0 * difficulty),
        Vision::new(120.0 * difficulty, 45.0),
        Patrol::new(patrol_points),
        RigidBody::Dynamic,
        Collider::ball(9.0),
        Velocity::default(),
        Damping { linear_damping: 10.0, angular_damping: 10.0 },
    ));
}

fn spawn_terminal(commands: &mut Commands, position: Vec2, terminal_type: TerminalType) {
    let color = match terminal_type {
        TerminalType::Objective => Color::srgb(0.9, 0.2, 0.2),
        TerminalType::Equipment => Color::srgb(0.2, 0.5, 0.9),
        TerminalType::Intel => Color::srgb(0.2, 0.8, 0.3),
    };
    
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color,
                custom_size: Some(Vec2::new(20.0, 20.0)),
                ..default()
            },
            transform: Transform::from_translation(position.extend(1.0)),
            ..default()
        },
        Terminal { terminal_type, range: 30.0, accessed: false },
        Selectable { radius: 15.0 },
    ));
}

fn alert_color(alert_level: AlertLevel) -> Color {
    match alert_level {
        AlertLevel::Green => Color::srgb(0.8, 0.2, 0.2),
        AlertLevel::Yellow => Color::srgb(0.8, 0.5, 0.2),
        AlertLevel::Orange => Color::srgb(0.8, 0.8, 0.2),
        AlertLevel::Red => Color::srgb(1.0, 0.2, 0.2),
    }
}