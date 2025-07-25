// src/systems/police.rs [16621] -> [16072]
use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::core::*;
use crate::core::factions::*;
use crate::systems::*;

// === CONFIGURATION ===
#[derive(Resource, Deserialize, Serialize, Clone)]
pub struct PoliceConfig {
    pub heat_decay_rate: f32,
    pub escalation_check_delay: f32,
    pub escalation_cooldown: f32,
    pub mass_hysteria_threshold: usize,
    pub incident_heat_values: HashMap<String, f32>,
    pub escalation_levels: HashMap<String, LevelConfig>,
    pub patrol_patterns: HashMap<String, Vec<(f32, f32)>>,
    pub level_patrol_patterns: HashMap<String, String>,
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

// Load police config from file
pub fn load_police_config() -> PoliceConfig {
    let config_str = std::fs::read_to_string("data/config/police_config.ron")
        .expect("Failed to load police config");
    ron::from_str(&config_str).expect("Failed to parse police config")
}
// === ESCALATION LEVELS ===
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
    
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::None => "None",
            Self::Patrol => "Patrol",
            Self::Armed => "Armed",
            Self::Tactical => "Tactical",
            Self::Military => "Military",
            Self::Corporate => "Corporate",
        }
    }
    
    pub fn get_config<'a>(&self, config: &'a PoliceConfig) -> &'a LevelConfig {
        config.escalation_levels.get(self.as_str())
            .expect("Missing escalation level config")
    }
}

// === RESOURCES ===
#[derive(Resource)]
pub struct PoliceResponse {
    pub civilian_casualties: u32,
    pub heat_level: f32,
    pub next_spawn_timer: f32,
    pub last_incident_pos: Option<Vec2>,
}

impl Default for PoliceResponse {
    fn default() -> Self {
        Self {
            civilian_casualties: 0,
            heat_level: 0.0,
            next_spawn_timer: 0.0,
            last_incident_pos: None,
        }
    }
}

#[derive(Resource)]
pub struct PoliceEscalation {
    pub current_level: EscalationLevel,
    pub heat_level: f32,
    pub escalation_timer: f32,
    pub spawn_timer: f32,
    pub incident_count: u32,
    pub civilian_casualties: u32,
    pub agent_casualties: u32,
    pub last_incident_pos: Option<Vec2>,
    pub active_units: Vec<PoliceUnit>,
    pub escalation_cooldown: f32,
}

impl Default for PoliceEscalation {
    fn default() -> Self {
        Self {
            current_level: EscalationLevel::None,
            heat_level: 0.0,
            escalation_timer: 0.0,
            spawn_timer: 0.0,
            incident_count: 0,
            civilian_casualties: 0,
            agent_casualties: 0,
            last_incident_pos: None,
            active_units: Vec::new(),
            escalation_cooldown: 0.0,
        }
    }
}

#[derive(Clone)]
pub struct PoliceUnit {
    pub entity: Entity,
    pub unit_type: EscalationLevel,
    pub spawn_time: f32,
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
    fn as_str(&self) -> &'static str {
        match self {
            Self::Gunshot => "Gunshot",
            Self::CivilianKilled => "CivilianKilled",
            Self::PoliceKilled => "PoliceKilled",
            Self::Explosion => "Explosion",
            Self::MassHysteria => "MassHysteria",
        }
    }
    
    fn heat_value(&self, config: &PoliceConfig) -> f32 {
        config.incident_heat_values.get(self.as_str())
            .copied()
            .unwrap_or(5.0)
    }
}

impl PoliceResponse {
    pub fn add_incident(&mut self, pos: Vec2, severity: f32) {
        self.last_incident_pos = Some(pos);
        self.heat_level += severity;
    }
}

impl PoliceEscalation {
    pub fn add_incident(&mut self, pos: Vec2, severity: f32, incident_type: IncidentType, config: &PoliceConfig) {
        self.last_incident_pos = Some(pos);
        self.incident_count += 1;
        self.heat_level += incident_type.heat_value(config) * severity;
        self.escalation_timer = config.escalation_check_delay;
        
        //info!("Police incident: {:?} | Heat: {:.1} | Level: {:?}", incident_type, self.heat_level, self.current_level);
    }
    
    fn should_escalate(&self, config: &PoliceConfig) -> bool {
        self.escalation_cooldown <= 0.0 && 
        self.heat_level >= self.current_level.get_config(config).heat_threshold
    }
    
    fn escalate(&mut self, config: &PoliceConfig) {
        self.current_level = self.current_level.next();
        let level_config = self.current_level.get_config(config);
        self.spawn_timer = level_config.response_time;
        self.escalation_cooldown = config.escalation_cooldown;
        
        info!("POLICE ESCALATION: {:?} | Response in {:.1}s", 
              self.current_level, self.spawn_timer);
    }
}

// === SYSTEMS ===
pub fn police_tracking_system(
    mut police_response: ResMut<PoliceResponse>,
    mut combat_events: EventReader<CombatEvent>,
    mut audio_events: EventReader<AudioEvent>,
    civilian_query: Query<&Transform, With<Civilian>>,
    time: Res<Time>,
    game_mode: Res<GameMode>,
    config: Res<PoliceConfig>,
) {
    if game_mode.paused { return; }

    let dt = time.delta_secs();
    police_response.next_spawn_timer -= dt;
    police_response.heat_level = (police_response.heat_level - dt * config.heat_decay_rate).max(0.0);

    for event in combat_events.read() {
        if event.hit {
            if let Ok(transform) = civilian_query.get(event.target) {
                police_response.civilian_casualties += 1;
                police_response.add_incident(transform.translation.truncate(), 50.0);
            }
        }
    }

    for event in audio_events.read() {
        if matches!(event.sound, AudioType::Gunshot) {
            if let Some(pos) = police_response.last_incident_pos {
                police_response.add_incident(pos, 5.0);
            }
        }
    }
}

pub fn police_incident_tracking_system(
    mut escalation: ResMut<PoliceEscalation>,
    mut combat_events: EventReader<CombatEvent>,
    mut audio_events: EventReader<AudioEvent>,
    civilian_query: Query<&Transform, (With<Civilian>, With<Dead>)>,
    police_query: Query<&Transform, (With<Police>, With<Dead>)>,
    urban_civilian_query: Query<&UrbanCivilian, With<Civilian>>,
    time: Res<Time>,
    game_mode: Res<GameMode>,
    config: Res<PoliceConfig>,
) {
    if game_mode.paused { return; }

    let dt = time.delta_secs();
    escalation.escalation_timer -= dt;
    escalation.spawn_timer -= dt;
    escalation.escalation_cooldown -= dt;
    escalation.heat_level = (escalation.heat_level - dt * config.heat_decay_rate).max(0.0);
    
    // Process combat events
    for event in combat_events.read() {
        if !event.hit { continue; }
        
        if let Ok(transform) = civilian_query.get(event.target) {
            escalation.civilian_casualties += 1;
            escalation.add_incident(
                transform.translation.truncate(), 
                1.0,
                IncidentType::CivilianKilled,
                &config
            );
        } else if let Ok(transform) = police_query.get(event.target) {
            escalation.add_incident(
                transform.translation.truncate(), 
                1.5,
                IncidentType::PoliceKilled,
                &config
            );
        }
    }
    
    // Process audio events
    for event in audio_events.read() {
        if matches!(event.sound, AudioType::Gunshot) {
            if let Some(pos) = escalation.last_incident_pos {
                escalation.add_incident(pos, 0.5, IncidentType::Gunshot, &config);
            }
        }
    }
    
    // Check mass hysteria
    let panicked_count = urban_civilian_query.iter()
        .filter(|c| matches!(c.daily_state, DailyState::Panicked))
        .count();
    
    if panicked_count >= config.mass_hysteria_threshold && escalation.incident_count % 10 == 0 {
        if let Some(pos) = escalation.last_incident_pos {
            escalation.add_incident(pos, 1.0, IncidentType::MassHysteria, &config);
        }
    }
    
    // Check escalation
    if escalation.escalation_timer <= 0.0 && escalation.should_escalate(&config) {
        escalation.escalate(&config);
    }
}

pub fn police_spawn_system(
    mut commands: Commands,
    mut escalation: ResMut<PoliceEscalation>,
    sprites: Res<GameSprites>,
    game_mode: Res<GameMode>,
    config: Res<PoliceConfig>,
) {
    if game_mode.paused || escalation.current_level == EscalationLevel::None { return; }
    
    if escalation.spawn_timer <= 0.0 {
        let level_config = escalation.current_level.get_config(&config);
        let spawn_pos = escalation.last_incident_pos.unwrap_or(Vec2::new(400.0, 0.0));
        let current_level = escalation.current_level; // Store before mutable borrow
        
        for i in 0..level_config.count {
            let offset = Vec2::new(
                (i as f32 - level_config.count as f32 / 2.0) * 40.0,
                (i % 2) as f32 * 30.0,
            );
            
            let entity = spawn_police_unit(
                &mut commands, 
                spawn_pos + Vec2::new(400.0, 0.0) + offset, 
                current_level,
                &sprites,
                &config
            );
            
            escalation.active_units.push(PoliceUnit {
                entity,
                unit_type: current_level,
                spawn_time: 0.0,
            });
        }
        
        escalation.spawn_timer = level_config.spawn_interval;
        info!("Spawned {} {:?} units", level_config.count, current_level);
    }
}

pub fn spawn_police_unit(
    commands: &mut Commands,
    position: Vec2,
    unit_type: EscalationLevel,
    sprites: &GameSprites,
    config: &PoliceConfig,
) -> Entity {
    let level_config = unit_type.get_config(config);
    let (mut sprite, _) = crate::core::sprites::create_enemy_sprite(sprites);
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
            Vision::new(level_config.vision, 50.0),
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
        )).id()
}

fn generate_patrol_pattern(position: Vec2, unit_type: EscalationLevel, config: &PoliceConfig) -> Vec<Vec2> {
    let pattern_name = config.level_patrol_patterns.get(unit_type.as_str())
        .expect("Missing patrol pattern mapping");
    
    let pattern = config.patrol_patterns.get(pattern_name)
        .expect("Missing patrol pattern");
    
    pattern.iter()
        .map(|(x, y)| position + Vec2::new(*x, *y))
        .collect()
}

pub fn police_cleanup_system(
    mut escalation: ResMut<PoliceEscalation>,
    police_query: Query<Entity, (With<Police>, With<Dead>)>,
) {
    for dead_entity in police_query.iter() {
        escalation.active_units.retain(|unit| unit.entity != dead_entity);
    }
}

pub fn police_deescalation_system(
    mut escalation: ResMut<PoliceEscalation>,
    agent_query: Query<&Transform, (With<Agent>, Without<Dead>)>,
    time: Res<Time>,
    game_mode: Res<GameMode>,
    config: Res<PoliceConfig>,
) {
    if game_mode.paused { return; }
    
    // De-escalate conditions
    if escalation.heat_level < 10.0 && escalation.escalation_timer <= -30.0 {
        if escalation.current_level != EscalationLevel::None {
            escalation.current_level = escalation.current_level.prev();
            info!("Police de-escalation: {:?}", escalation.current_level);
            escalation.escalation_timer = 0.0;
        }
    }
    
    // Extra heat decay when no agents
    if agent_query.is_empty() {
        escalation.heat_level = (escalation.heat_level - time.delta_secs() * 5.0).max(0.0);
    }
}

// Components
#[derive(Component)]
pub struct ThreatLevelText;

pub fn render_threat_level(
    mut commands: Commands,
    police_escalation: Res<PoliceEscalation>,
    mut threat_text_query: Query<&mut Text, With<ThreatLevelText>>,
) {
    let threat_text = format!("{}", police_escalation.current_level as u32);
    
    if let Ok(mut text) = threat_text_query.single_mut() {
        if police_escalation.is_changed() {
            **text = threat_text;
        }
    } else {
        commands.spawn((
            Text::new(threat_text),
            TextFont {
                font_size: 20.0,
                ..default()
            },
            TextColor(Color::WHITE),
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(10.0),
                right: Val::Px(10.0),
                ..default()
            },
            ThreatLevelText,
        ));
    }
}

impl TryFrom<u8> for EscalationLevel {
    type Error = ();
    
    fn try_from(value: u8) -> std::result::Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::None),
            1 => Ok(Self::Patrol),
            2 => Ok(Self::Armed),
            3 => Ok(Self::Tactical),
            4 => Ok(Self::Military),
            5 => Ok(Self::Corporate),
            _ => Err(()),
        }
    }
}

// Backwards Compatability (for now)
pub fn spawn_police_unit_simple(
    commands: &mut Commands,
    position: Vec2,
    unit_type: EscalationLevel,
    sprites: &GameSprites,
) -> Entity {
    // Use default config or load it
    let config = load_police_config();
    spawn_police_unit(commands, position, unit_type, sprites, &config)
}