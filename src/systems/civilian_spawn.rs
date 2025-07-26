// src/systems/civilian_spawn.rs
use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use crate::core::*;
use crate::systems::*;
use crate::systems::scenes::*;

#[derive(Resource)]
pub struct CivilianSpawner {
    pub spawn_timer: f32,
    pub max_civilians: u32,
    pub spawn_zones: Vec<SpawnZone>,
}

#[derive(Clone)]
pub struct SpawnZone {
    pub center: Vec2,
    pub radius: f32,
}

impl Default for CivilianSpawner {
    fn default() -> Self {
        Self {
            spawn_timer: 0.0,
            max_civilians: 12, // Will be overridden by config
            spawn_zones: vec![
                SpawnZone { center: Vec2::new(150.0, 150.0), radius: 80.0 },
                SpawnZone { center: Vec2::new(-100.0, 100.0), radius: 60.0 },
                SpawnZone { center: Vec2::new(200.0, -50.0), radius: 70.0 },
            ],
        }
    }
}

pub fn dynamic_civilian_spawn_system_with_config(
    mut commands: Commands,
    mut spawner: ResMut<CivilianSpawner>,
    civilian_query: Query<Entity, With<Civilian>>,
    sprites: Res<GameSprites>,
    config: Res<GameConfig>,
    time: Res<Time>,
    game_mode: Res<GameMode>,
) {
    if game_mode.paused { return; }

    spawner.spawn_timer -= time.delta_secs();
    spawner.max_civilians = config.civilians.max_civilians; // Update from config

    if spawner.spawn_timer <= 0.0 && civilian_query.iter().count() < spawner.max_civilians as usize {
        if let Some(spawn_pos) = find_spawn_position(&spawner.spawn_zones) {
            spawn_civilian_with_config(&mut commands, spawn_pos, &sprites, &config);
            let interval = config.civilians.spawn_interval_min + 
                         rand::random::<f32>() * (config.civilians.spawn_interval_max - config.civilians.spawn_interval_min);
            spawner.spawn_timer = interval;
        }
    }
}

pub fn dynamic_civilian_spawn_system(
    mut commands: Commands,
    mut spawner: ResMut<CivilianSpawner>,
    civilian_query: Query<Entity, With<Civilian>>,
    sprites: Res<GameSprites>,
    time: Res<Time>,
    game_mode: Res<GameMode>,
) {
    if game_mode.paused { return; }

    spawner.spawn_timer -= time.delta_secs();

    if spawner.spawn_timer <= 0.0 && civilian_query.iter().count() < spawner.max_civilians as usize {
        if let Some(spawn_pos) = find_spawn_position(&spawner.spawn_zones) {
            spawn_civilian(&mut commands, spawn_pos, &sprites);
            spawner.spawn_timer = 8.0 + rand::random::<f32>() * 4.0;
        }
    }
}

fn find_spawn_position(spawn_zones: &[SpawnZone]) -> Option<Vec2> {
    let zone = &spawn_zones[rand::random::<usize>() % spawn_zones.len()];
    let angle = rand::random::<f32>() * std::f32::consts::TAU;
    let distance = rand::random::<f32>() * zone.radius;
    let offset = Vec2::new(angle.cos(), angle.sin()) * distance;
    Some(zone.center + offset)
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

#[derive(Component)]
pub struct CivilianWander {
    pub wander_timer: f32,
    pub home_position: Vec2,
}

impl CivilianWander {
    pub fn new(position: Vec2) -> Self {
        Self {
            wander_timer: 0.0,
            home_position: position,
        }
    }
}

pub fn civilian_wander_system(
    mut civilian_query: Query<(Entity, &Transform, &mut CivilianWander), (With<Civilian>, Without<FleeTarget>, Without<ControlledCivilian>)>,
    mut commands: Commands,
    time: Res<Time>,
    game_mode: Res<GameMode>,
) {
    if game_mode.paused { return; }

    let delta = time.delta_secs();
    if delta <= 0.0 { return; }

    for (entity, _transform, mut wander) in civilian_query.iter_mut() {
        wander.wander_timer -= delta;
        
        if wander.wander_timer <= 0.0 {
            let angle = rand::random::<f32>() * std::f32::consts::TAU;
            let distance = rand::random::<f32>() * 120.0;
            let offset = Vec2::new(angle.cos(), angle.sin()) * distance;
            let target = wander.home_position + offset;
            
            if target.is_finite() {
                // Use commands instead of events to avoid despawned entity issues
                commands.entity(entity).try_insert(MoveTarget { position: target });
            }
            
            wander.wander_timer = 5.0 + rand::random::<f32>() * 10.0;
        }
    }
}

pub fn civilian_cleanup_system(
    mut commands: Commands,
    civilian_query: Query<(Entity, &Transform), (With<Civilian>, Without<MarkedForDespawn>)>,
    agent_query: Query<&Transform, With<Agent>>,
) {
    let agent_positions: Vec<Vec2> = agent_query.iter()
        .map(|t| t.translation.truncate())
        .collect();
    
    if agent_positions.is_empty() { return; }
    
    for (entity, civilian_transform) in civilian_query.iter() {
        let civilian_pos = civilian_transform.translation.truncate();
        let min_distance = agent_positions.iter()
            .map(|&agent_pos| civilian_pos.distance(agent_pos))
            .fold(f32::INFINITY, f32::min);
        
        if min_distance > 600.0 && civilian_query.get(entity).is_ok() {
            commands.entity(entity).insert(MarkedForDespawn);
        }
    }
}

#[derive(Clone)]
pub enum CivilianType {
    Pedestrian,
    Shopper,
    Worker,
    Tourist,
}

#[derive(Clone)]
pub struct CivilianBehavior {
    pub behavior_type: CivilianType,
    pub wander_timer: f32,
    pub wander_target: Option<Vec2>,
    pub home_position: Vec2,
}

impl CivilianBehavior {
    pub fn new() -> Self {
        let behavior_types = [
            CivilianType::Pedestrian,
            CivilianType::Shopper, 
            CivilianType::Worker,
            CivilianType::Tourist,
        ];
        
        Self {
            behavior_type: behavior_types[rand::random::<usize>() % behavior_types.len()].clone(),
            wander_timer: 0.0,
            wander_target: None,
            home_position: Vec2::ZERO,
        }
    }
}
