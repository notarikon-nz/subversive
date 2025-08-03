// src/systems/urban_security.rs - Unified police, civilian and urban simulation
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::core::*;
use crate::systems::spawners::*;

// === CONFIGURATION ===
#[derive(Resource, Deserialize, Serialize, Clone)]
pub struct UrbanConfig {
    // Police configuration
    pub heat_decay_rate: f32,
    pub escalation_check_delay: f32,
    pub mass_hysteria_threshold: usize,
    pub incident_heat_values: HashMap<String, f32>,
    pub escalation_levels: HashMap<String, LevelConfig>,
    
    // Civilian configuration  
    pub max_civilians: u32,
    pub spawn_interval_min: f32,
    pub spawn_interval_max: f32,
    pub cleanup_distance: f32,
    
    // Urban simulation
    pub crowd_cluster_radius: f32,
    pub crowd_min_size: usize,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct LevelConfig {
    pub count: u8,
    pub response_time: f32,
    pub health: f32,
    pub weapon: String,
    pub speed: f32,
    pub vision: f32,
    pub color: (f32, f32, f32, f32),
    pub heat_threshold: f32,
    pub spawn_interval: f32,
}

// === ESCALATION SYSTEM ===
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum EscalationLevel {
    None = 0,
    Patrol = 1, 
    Armed = 2,
    Tactical = 3,
    Military = 4,
    Corporate = 5,
}

impl EscalationLevel {
    pub fn next(self) -> Self {
        match self {
            Self::None => Self::Patrol,
            Self::Patrol => Self::Armed,
            Self::Armed => Self::Tactical,
            Self::Tactical => Self::Military,
            Self::Military | Self::Corporate => Self::Corporate,
        }
    }

    pub fn prev(self) -> Self {
        match self {
            Self::None | Self::Patrol => Self::None,
            Self::Armed => Self::Patrol,
            Self::Tactical => Self::Armed,
            Self::Military => Self::Tactical,
            Self::Corporate => Self::Military,
        }
    }

    const fn as_str(&self) -> &'static str {
        match self {
            Self::None => "None",
            Self::Patrol => "Patrol", 
            Self::Armed => "Armed",
            Self::Tactical => "Tactical",
            Self::Military => "Military",
            Self::Corporate => "Corporate",
        }
    }

    pub fn get_config<'a>(&self, config: &'a UrbanConfig) -> &'a LevelConfig {
        config.escalation_levels.get(self.as_str())
            .expect("Missing escalation level config")
    }    
}

#[derive(Debug, Clone, Copy)]
pub enum IncidentType {
    Gunshot,
    CivilianKilled,
    PoliceKilled,
    Explosion,
    MassHysteria,
}

impl IncidentType {
    const fn as_str(&self) -> &'static str {
        match self {
            Self::Gunshot => "Gunshot",
            Self::CivilianKilled => "CivilianKilled",
            Self::PoliceKilled => "PoliceKilled",
            Self::Explosion => "Explosion",
            Self::MassHysteria => "MassHysteria",
        }
    }
}

// === UNIFIED RESOURCES ===
#[derive(Resource)]
pub struct UrbanSecurity {
    // Police state
    pub escalation_level: EscalationLevel,
    pub heat_level: f32,
    pub escalation_timer: f32,
    pub spawn_timer: f32,
    pub incident_count: u32,
    pub civilian_casualties: u32,
    pub last_incident_pos: Option<Vec2>,
    pub active_units: Vec<PoliceUnit>,
    pub escalation_cooldown: f32,
    
    // Civilian spawning
    pub civilian_spawn_timer: f32,
    pub spawn_zones: Vec<SpawnZone>,
    
    // Urban areas
    pub work_zones: Vec<UrbanZone>,
    pub shopping_zones: Vec<UrbanZone>, 
    pub residential_zones: Vec<UrbanZone>,
    pub transit_routes: Vec<TransitRoute>,
}

impl Default for UrbanSecurity {
    fn default() -> Self {
        Self {
            escalation_level: EscalationLevel::None,
            heat_level: 0.0,
            escalation_timer: 0.0,
            spawn_timer: 0.0,
            incident_count: 0,
            civilian_casualties: 0,
            last_incident_pos: None,
            active_units: Vec::new(),
            escalation_cooldown: 0.0,
            civilian_spawn_timer: 0.0,
            spawn_zones: vec![
                SpawnZone { center: Vec2::new(150.0, 150.0), radius: 80.0 },
                SpawnZone { center: Vec2::new(-100.0, 100.0), radius: 60.0 },
                SpawnZone { center: Vec2::new(200.0, -50.0), radius: 70.0 },
            ],
            work_zones: vec![
                UrbanZone { center: Vec2::new(200.0, 100.0), radius: 80.0, capacity: 15, current_occupancy: 0 },
                UrbanZone { center: Vec2::new(-150.0, 150.0), radius: 60.0, capacity: 10, current_occupancy: 0 },
            ],
            shopping_zones: vec![
                UrbanZone { center: Vec2::new(0.0, -100.0), radius: 70.0, capacity: 20, current_occupancy: 0 },
                UrbanZone { center: Vec2::new(100.0, 200.0), radius: 50.0, capacity: 8, current_occupancy: 0 },
            ],
            residential_zones: vec![
                UrbanZone { center: Vec2::new(-200.0, -50.0), radius: 90.0, capacity: 25, current_occupancy: 0 },
                UrbanZone { center: Vec2::new(300.0, -150.0), radius: 70.0, capacity: 18, current_occupancy: 0 },
            ],
            transit_routes: vec![
                TransitRoute {
                    points: vec![Vec2::new(-200.0, 0.0), Vec2::new(0.0, 0.0), Vec2::new(200.0, 0.0)],
                    foot_traffic_density: 0.8
                },
            ],
        }
    }
}

// === COMPONENTS ===
#[derive(Component)]
pub struct UrbanCivilian {
    pub daily_state: DailyState,
    pub state_timer: f32,
    pub next_destination: Option<Vec2>,
    pub crowd_influence: f32,
    pub panic_threshold: f32,
    pub movement_urgency: f32,
    pub home_position: Vec2,
}

#[derive(Debug, Clone, Copy)]
pub enum DailyState {
    GoingToWork,
    Working,
    Shopping,
    GoingHome,
    Idle,
    Panicked,
    Following,
}

#[derive(Clone)]
pub struct PoliceUnit {
    pub entity: Entity,
    pub unit_type: EscalationLevel,
    pub spawn_time: f32,
}

#[derive(Clone)]
pub struct SpawnZone {
    pub center: Vec2,
    pub radius: f32,
}

#[derive(Clone)]
pub struct UrbanZone {
    pub center: Vec2,
    pub radius: f32,
    pub capacity: usize,
    pub current_occupancy: usize,
}

#[derive(Clone)]
pub struct TransitRoute {
    pub points: Vec<Vec2>,
    pub foot_traffic_density: f32,
}

// === UNIFIED MAIN SYSTEM ===
pub fn unified_urban_security_system(
    mut commands: Commands,
    mut urban_security: ResMut<UrbanSecurity>,
    mut combat_events: EventReader<CombatEvent>,
    mut audio_events: EventReader<AudioEvent>,
    mut action_events: EventWriter<ActionEvent>,
    mut civilian_query: Query<(Entity, &Transform, Option<&mut UrbanCivilian>, Option<&Morale>), (With<Civilian>, Without<MarkedForDespawn>)>,
    dead_civilian_query: Query<&Transform, (With<Civilian>, With<Dead>, Without<MarkedForDespawn>)>,
    dead_police_query: Query<(Entity, &Transform), (With<Police>, With<Dead>, Without<MarkedForDespawn>)>,
    agent_query: Query<&Transform, With<Agent>>,
    sprites: Res<GameSprites>,
    config: Res<UrbanConfig>,
    time: Res<Time>,
    game_mode: Res<GameMode>,
) {
    if game_mode.paused { return; }

    let dt = time.delta_secs();
    
    // === HEAT DECAY ===
    urban_security.heat_level = (urban_security.heat_level - dt * config.heat_decay_rate).max(0.0);
    urban_security.escalation_timer -= dt;
    urban_security.spawn_timer -= dt;
    urban_security.escalation_cooldown -= dt;
    urban_security.civilian_spawn_timer -= dt;

    // === INCIDENT PROCESSING ===
    for event in combat_events.read() {
        if !event.hit { continue; }
        
        if let Ok(transform) = dead_civilian_query.get(event.target) {
            urban_security.civilian_casualties += 1;
            add_incident(&mut urban_security, transform.translation.truncate(), IncidentType::CivilianKilled, &config);
        } else if let Ok((_, transform)) = dead_police_query.get(event.target) {
            add_incident(&mut urban_security, transform.translation.truncate(), IncidentType::PoliceKilled, &config);
        }
    }

    for event in audio_events.read() {
        if matches!(event.sound, AudioType::Gunshot) {
            if let Some(pos) = urban_security.last_incident_pos {
                add_incident(&mut urban_security, pos, IncidentType::Gunshot, &config);
            }
        }
    }

    // === MASS HYSTERIA CHECK ===
    let panicked_count = civilian_query.iter()
        .filter(|(_, _, urban_civ, morale)| {
            urban_civ.map_or(false, |u| matches!(u.daily_state, DailyState::Panicked)) ||
            morale.map_or(false, |m| m.is_panicked())
        })
        .count();

    if panicked_count >= config.mass_hysteria_threshold && urban_security.incident_count % 10 == 0 {
        if let Some(pos) = urban_security.last_incident_pos {
            add_incident(&mut urban_security, pos, IncidentType::MassHysteria, &config);
        }
    }

    // === POLICE ESCALATION ===
    if urban_security.escalation_timer <= 0.0 && should_escalate(&urban_security, &config) {
        escalate(&mut urban_security, &config);
    }

    // === POLICE SPAWNING ===
    if urban_security.spawn_timer <= 0.0 && urban_security.escalation_level != EscalationLevel::None {
        spawn_police_response(&mut commands, &mut urban_security, &sprites, &config);
    }

    // === CIVILIAN SPAWNING ===
    let current_civilian_count = civilian_query.iter().count();
    if urban_security.civilian_spawn_timer <= 0.0 && current_civilian_count < config.max_civilians as usize {
        if let Some(spawn_pos) = find_spawn_position(&urban_security.spawn_zones, &urban_security.transit_routes) {
            spawn_urban_civilian(&mut commands, spawn_pos, &sprites, &urban_security);
            let interval = config.spawn_interval_min + rand::random::<f32>() * (config.spawn_interval_max - config.spawn_interval_min);
            urban_security.civilian_spawn_timer = interval;
        }
    }

    // === CROWD DYNAMICS ===
    // First pass: collect data for crowd clustering (immutable)
    let crowd_data: Vec<(Vec2, f32)> = civilian_query.iter()
        .map(|(_, transform, urban_civ, morale)| {
            let panic_level = if morale.map_or(false, |m| m.is_panicked()) { 1.0 } else { 0.0 };
            (transform.translation.truncate(), panic_level)
        })
        .collect();
    
    let mut crowd_nodes = Vec::new();
    create_crowd_clusters_from_data(&crowd_data, &mut crowd_nodes, &config);

    // Second pass: apply crowd influence (mutable)
    for (entity, transform, mut urban_civ_option, morale) in civilian_query.iter_mut() {
        if let Some(ref mut urban_civ) = urban_civ_option {
            apply_crowd_influence(entity, transform, urban_civ, morale, &crowd_nodes, &mut action_events, &urban_security);
        }
    }

    // === CLEANUP ===
    cleanup_distant_entities(&mut commands, &civilian_query, &agent_query, config.cleanup_distance);
    
    // Remove dead police from active units
    for (dead_entity, _) in dead_police_query.iter() {
        urban_security.active_units.retain(|unit| unit.entity != dead_entity);
    }

    // === DE-ESCALATION ===
    if urban_security.heat_level < 10.0 && urban_security.escalation_timer <= -30.0 {
        if urban_security.escalation_level != EscalationLevel::None {
            urban_security.escalation_level = urban_security.escalation_level.prev();
            urban_security.escalation_timer = 0.0;
        }
    }
}

// === HELPER FUNCTIONS ===
pub fn pick_destination_for_state(state: DailyState, urban_security: &UrbanSecurity) -> Option<Vec2> {
    match state {
        DailyState::GoingToWork => pick_random_zone_center(&urban_security.work_zones),
        DailyState::Shopping => pick_random_zone_center(&urban_security.shopping_zones),
        DailyState::GoingHome => pick_random_zone_center(&urban_security.residential_zones),
        _ => None,
    }
}

fn pick_random_zone_center(zones: &[UrbanZone]) -> Option<Vec2> {
    if zones.is_empty() { return None; }
    let zone = &zones[rand::random::<usize>() % zones.len()];

    let angle = rand::random::<f32>() * std::f32::consts::TAU;
    let distance = rand::random::<f32>() * zone.radius * 0.8;
    let offset = Vec2::new(angle.cos(), angle.sin()) * distance;

    Some(zone.center + offset)
}

fn transition_daily_state(urban_civ: &mut UrbanCivilian) {
    urban_civ.daily_state = match urban_civ.daily_state {
        DailyState::GoingToWork => DailyState::Working,
        DailyState::Working => if rand::random::<f32>() < 0.7 { DailyState::Shopping } else { DailyState::GoingHome },
        DailyState::Shopping => DailyState::GoingHome,
        DailyState::GoingHome => DailyState::Idle,
        DailyState::Idle => match rand::random::<f32>() {
            x if x < 0.4 => DailyState::GoingToWork,
            x if x < 0.7 => DailyState::Shopping,
            _ => DailyState::Idle,
        },
        DailyState::Panicked => DailyState::Idle,
        DailyState::Following => DailyState::Idle,
    };

    if matches!(urban_civ.daily_state, DailyState::Idle | DailyState::GoingToWork | DailyState::Shopping) {
        urban_civ.movement_urgency = 0.0;
    }
}

fn find_escape_destination(current_pos: Vec2) -> Vec2 {
    let to_edge = if current_pos.x.abs() > current_pos.y.abs() {
        Vec2::new(current_pos.x.signum(), 0.0)
    } else {
        Vec2::new(0.0, current_pos.y.signum())
    };
    current_pos + to_edge * 400.0
}

fn add_incident(urban_security: &mut UrbanSecurity, pos: Vec2, incident_type: IncidentType, config: &UrbanConfig) {
    urban_security.last_incident_pos = Some(pos);
    urban_security.incident_count += 1;
    
    let heat_value = config.incident_heat_values.get(incident_type.as_str()).copied().unwrap_or(5.0);
    urban_security.heat_level += heat_value;
    urban_security.escalation_timer = config.escalation_check_delay;
}

fn should_escalate(urban_security: &UrbanSecurity, config: &UrbanConfig) -> bool {
    urban_security.escalation_cooldown <= 0.0 && {
        let level_config = config.escalation_levels.get(urban_security.escalation_level.as_str()).unwrap();
        urban_security.heat_level >= level_config.heat_threshold
    }
}

fn escalate(urban_security: &mut UrbanSecurity, config: &UrbanConfig) {
    urban_security.escalation_level = urban_security.escalation_level.next();
    let level_config = config.escalation_levels.get(urban_security.escalation_level.as_str()).unwrap();
    urban_security.spawn_timer = level_config.response_time;
    urban_security.escalation_cooldown = config.escalation_check_delay;
}

fn spawn_police_response(
    commands: &mut Commands,
    urban_security: &mut UrbanSecurity,
    sprites: &GameSprites,
    config: &UrbanConfig,
) {
    let level_config = config.escalation_levels.get(urban_security.escalation_level.as_str()).unwrap();
    let spawn_pos = urban_security.last_incident_pos.unwrap_or(Vec2::new(400.0, 0.0));

    for i in 0..level_config.count {
        let offset = Vec2::new((i as f32 - level_config.count as f32 / 2.0) * 40.0, (i % 2) as f32 * 30.0);
        let entity = spawn_police_unit(commands, spawn_pos + Vec2::new(400.0, 0.0) + offset, urban_security.escalation_level, sprites, config);
        
        urban_security.active_units.push(PoliceUnit {
            entity,
            unit_type: urban_security.escalation_level,
            spawn_time: 0.0,
        });
    }

    urban_security.spawn_timer = level_config.spawn_interval;
}

fn find_spawn_position(spawn_zones: &[SpawnZone], transit_routes: &[TransitRoute]) -> Option<Vec2> {
    // 60% spawn near transit, 40% near zones
    if rand::random::<f32>() < 0.6 && !transit_routes.is_empty() {
        let route = &transit_routes[rand::random::<usize>() % transit_routes.len()];
        if !route.points.is_empty() {
            let point = route.points[rand::random::<usize>() % route.points.len()];
            let offset = Vec2::new((rand::random::<f32>() - 0.5) * 40.0, (rand::random::<f32>() - 0.5) * 40.0);
            return Some(point + offset);
        }
    }

    if !spawn_zones.is_empty() {
        let zone = &spawn_zones[rand::random::<usize>() % spawn_zones.len()];
        let angle = rand::random::<f32>() * std::f32::consts::TAU;
        let distance = rand::random::<f32>() * zone.radius;
        let offset = Vec2::new(angle.cos(), angle.sin()) * distance;
        return Some(zone.center + offset);
    }

    None
}

fn create_crowd_clusters_from_data(
    crowd_data: &[(Vec2, f32)],
    crowd_nodes: &mut Vec<CrowdNode>,
    config: &UrbanConfig,
) {
    let mut processed = vec![false; crowd_data.len()];
    
    for i in 0..crowd_data.len() {
        if processed[i] { continue; }

        let mut cluster = vec![i];
        let mut avg_pos = crowd_data[i].0;
        let mut avg_panic = crowd_data[i].1;

        for j in (i + 1)..crowd_data.len() {
            if processed[j] { continue; }
            
            if crowd_data[i].0.distance(crowd_data[j].0) <= config.crowd_cluster_radius {
                cluster.push(j);
                avg_pos += crowd_data[j].0;
                avg_panic += crowd_data[j].1;
                processed[j] = true;
            }
        }

        if cluster.len() >= config.crowd_min_size {
            avg_pos /= cluster.len() as f32;
            avg_panic /= cluster.len() as f32;

            crowd_nodes.push(CrowdNode {
                position: avg_pos,
                crowd_size: cluster.len(),
                panic_level: avg_panic,
                movement_direction: Vec2::ZERO,
                influence_radius: 40.0 + (cluster.len() as f32 * 10.0).min(80.0),
            });
        }

        processed[i] = true;
    }
}

#[derive(Clone)]
struct CrowdNode {
    position: Vec2,
    crowd_size: usize,
    panic_level: f32,
    movement_direction: Vec2,
    influence_radius: f32,
}

fn apply_crowd_influence(
    entity: Entity,
    transform: &Transform,
    urban_civ: &mut UrbanCivilian,
    morale: Option<&Morale>,
    crowd_nodes: &[CrowdNode],
    action_events: &mut EventWriter<ActionEvent>,
    urban_security: &UrbanSecurity,
) {
    let pos = transform.translation.truncate();
    let mut panic_influence = 0.0;

    for crowd_node in crowd_nodes {
        let distance = pos.distance(crowd_node.position);
        if distance <= crowd_node.influence_radius {
            let influence_strength = 1.0 - (distance / crowd_node.influence_radius);
            panic_influence += crowd_node.panic_level * influence_strength;
        }
    }

    // Update state based on panic
    if panic_influence > urban_civ.panic_threshold || morale.map_or(false, |m| m.is_panicked()) {
        if !matches!(urban_civ.daily_state, DailyState::Panicked) {
            urban_civ.daily_state = DailyState::Panicked;
            urban_civ.movement_urgency = 1.0;
            urban_civ.next_destination = Some(find_escape_destination(pos));
        }
    } else {
        // Handle daily routine state transitions
        urban_civ.state_timer -= 0.016; // Approximate delta time
        
        if urban_civ.state_timer <= 0.0 && !matches!(urban_civ.daily_state, DailyState::Panicked) {
            transition_daily_state(urban_civ);
            urban_civ.next_destination = pick_destination_for_state(urban_civ.daily_state, urban_security);
            urban_civ.state_timer = 15.0 + rand::random::<f32>() * 20.0;
        }
    }

    // Send movement command
    if let Some(destination) = urban_civ.next_destination {
        if pos.distance(destination) > 20.0 {
            action_events.write(ActionEvent {
                entity,
                action: Action::MoveTo(destination),
            });
        } else {
            urban_civ.daily_state = DailyState::Idle;
            urban_civ.next_destination = None;
        }
    }
}

fn cleanup_distant_entities(
    commands: &mut Commands,
    civilian_query: &Query<(Entity, &Transform, Option<&mut UrbanCivilian>, Option<&Morale>), (With<Civilian>, Without<MarkedForDespawn>)>,
    agent_query: &Query<&Transform, With<Agent>>,
    cleanup_distance: f32,
) {
    if agent_query.is_empty() { return; }

    let agent_positions: Vec<Vec2> = agent_query.iter().map(|t| t.translation.truncate()).collect();

    for (entity, transform, urban_civ, _) in civilian_query.iter() {
        let pos = transform.translation.truncate();
        let min_distance = agent_positions.iter()
            .map(|&agent_pos| pos.distance(agent_pos))
            .fold(f32::INFINITY, f32::min);

        let should_despawn = min_distance > cleanup_distance && 
            urban_civ.map_or(true, |u| matches!(u.daily_state, DailyState::Idle | DailyState::GoingHome));

        if should_despawn {
            commands.entity(entity).insert(MarkedForDespawn);
        }
    }
}

// === SPAWNING FUNCTIONS ===
/*
pub fn spawn_urban_civilian(
    commands: &mut Commands,
    position: Vec2,
    sprites: &GameSprites,
    urban_security: &UrbanSecurity,
) {
    let civilian_entity = spawn_civilian(commands, position, sprites);
    
    commands.entity(civilian_entity).insert(UrbanCivilian {
        daily_state: DailyState::Idle,
        state_timer: rand::random::<f32>() * 10.0,
        next_destination: None,
        crowd_influence: 0.3 + rand::random::<f32>() * 0.4,
        panic_threshold: 0.4 + rand::random::<f32>() * 0.3,
        movement_urgency: 0.0,
        home_position: position,
    });
} */

pub fn load_urban_config() -> UrbanConfig {
    if let Ok(config_str) = std::fs::read_to_string("data/config/urban_config.ron") {
        ron::from_str(&config_str).unwrap_or_default()
    } else {
        UrbanConfig::default()
    }
}

impl Default for UrbanConfig {
    fn default() -> Self {
        let mut incident_heat_values = HashMap::new();
        incident_heat_values.insert("Gunshot".to_string(), 2.0);
        incident_heat_values.insert("CivilianKilled".to_string(), 15.0);
        incident_heat_values.insert("PoliceKilled".to_string(), 25.0);
        incident_heat_values.insert("Explosion".to_string(), 20.0);
        incident_heat_values.insert("MassHysteria".to_string(), 10.0);

        let mut escalation_levels = HashMap::new();
        escalation_levels.insert("None".to_string(), LevelConfig {
            count: 0, response_time: 0.0, health: 0.0, weapon: "none".to_string(),
            speed: 0.0, vision: 0.0, color: (0.0, 0.0, 0.0, 0.0), heat_threshold: 0.0, spawn_interval: 0.0,
        });
        escalation_levels.insert("Patrol".to_string(), LevelConfig {
            count: 2, response_time: 15.0, health: 60.0, weapon: "pistol".to_string(),
            speed: 80.0, vision: 150.0, color: (0.3, 0.3, 0.8, 1.0), heat_threshold: 20.0, spawn_interval: 12.0,
        });

        Self {
            heat_decay_rate: 1.0,
            escalation_check_delay: 5.0,
            mass_hysteria_threshold: 8,
            incident_heat_values,
            escalation_levels,
            max_civilians: 12,
            spawn_interval_min: 3.0,
            spawn_interval_max: 8.0,
            cleanup_distance: 600.0,
            crowd_cluster_radius: 50.0,
            crowd_min_size: 3,
        }
    }
}

pub fn generate_patrol_pattern(position: Vec2, unit_type: EscalationLevel, config: &UrbanConfig) -> Vec<Vec2> {
    // Simple patrol pattern based on unit type
    match unit_type {
        EscalationLevel::None => vec![position],
        EscalationLevel::Patrol => {
            // Small patrol route
            vec![
                position,
                position + Vec2::new(50.0, 0.0),
                position + Vec2::new(50.0, 50.0),
                position + Vec2::new(0.0, 50.0),
            ]
        },
        EscalationLevel::Armed => {
            // Medium patrol route
            vec![
                position,
                position + Vec2::new(80.0, 0.0),
                position + Vec2::new(80.0, 80.0),
                position + Vec2::new(0.0, 80.0),
                position + Vec2::new(-40.0, 40.0),
            ]
        },
        _ => {
            // Large patrol route for tactical and above
            vec![
                position,
                position + Vec2::new(100.0, 0.0),
                position + Vec2::new(100.0, 100.0),
                position + Vec2::new(-50.0, 100.0),
                position + Vec2::new(-50.0, -50.0),
                position + Vec2::new(50.0, -50.0),
            ]
        }
    }
}

pub fn setup_urban_security_system(mut commands: Commands) {
    let config = crate::systems::urban_security::load_urban_config();
    commands.insert_resource(config);
    commands.init_resource::<crate::systems::urban_security::UrbanSecurity>();
}