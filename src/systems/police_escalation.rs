// src/systems/police_escalation.rs - Syndicate-style law enforcement response
use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use crate::core::*;
use crate::systems::*;
use crate::core::factions::*;

// === ESCALATION LEVELS ===
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum EscalationLevel {
    None = 0,
    Patrol = 1,      // Beat cops
    Armed = 2,       // Armed response unit
    Tactical = 3,    // SWAT team
    Military = 4,    // Army units
    Corporate = 5,   // Enemy agents
}

impl EscalationLevel {
    pub fn next(self) -> Self {
        match self {
            Self::None => Self::Patrol,
            Self::Patrol => Self::Armed,
            Self::Armed => Self::Tactical,
            Self::Tactical => Self::Military,
            Self::Military => Self::Corporate,
            Self::Corporate => Self::Corporate, // Max level
        }
    }
    
    pub fn unit_count(self) -> u8 {
        match self {
            Self::None => 0,
            Self::Patrol => 1,
            Self::Armed => 2,
            Self::Tactical => 3,
            Self::Military => 4,
            Self::Corporate => 2, // Elite units
        }
    }
    
    pub fn response_time(self) -> f32 {
        match self {
            Self::None => 0.0,
            Self::Patrol => 45.0,    // 45 seconds
            Self::Armed => 30.0,     // 30 seconds
            Self::Tactical => 25.0,  // 25 seconds
            Self::Military => 20.0,  // 20 seconds
            Self::Corporate => 15.0, // 15 seconds
        }
    }
    
    pub fn color(self) -> Color {
        match self {
            Self::None => Color::srgb(0.3, 0.3, 0.3),
            Self::Patrol => Color::srgb(0.2, 0.2, 0.8),
            Self::Armed => Color::srgb(0.4, 0.4, 0.9),
            Self::Tactical => Color::srgb(0.6, 0.6, 1.0),
            Self::Military => Color::srgb(0.5, 0.8, 0.5),
            Self::Corporate => Color::srgb(0.8, 0.2, 0.8),
        }
    }
}

// === ENHANCED POLICE RESPONSE ===
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
    pub escalation_cooldown: f32, // Prevent rapid escalation
}

#[derive(Clone)]
pub struct PoliceUnit {
    pub entity: Entity,
    pub unit_type: EscalationLevel,
    pub spawn_time: f32,
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

// Show Threat Level
// Should eventually be a sprite indicator
pub fn render_threat_level(
    mut commands: Commands,
    ui_state: Res<UIState>,
    police_escalation: Res<PoliceEscalation>,
) {
    // BAD - we don't want to keep creating (and destroying)
    let threat_text = format!("{:?}", police_escalation.current_level as u32);
    commands.spawn((
        Text::new(threat_text),
        TextFont {
            font_size: 16.0,
            ..default()
        },
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            right: Val::Px(10.0),
            ..default()
        },
    ));
}

impl PoliceEscalation {
    pub fn add_incident(&mut self, pos: Vec2, severity: f32, incident_type: IncidentType) {
        self.last_incident_pos = Some(pos);
        self.incident_count += 1;
        
        let heat_increase = match incident_type {
            IncidentType::Gunshot => 5.0,
            IncidentType::CivilianKilled => 25.0,
            IncidentType::PoliceKilled => 35.0,
            IncidentType::Explosion => 40.0,
            IncidentType::MassHysteria => 20.0,
        };
        
        self.heat_level += heat_increase * severity;
        self.escalation_timer = 5.0; // Check escalation in 5 seconds
        
        info!("Police incident: {:?} | Heat: {:.1} | Level: {:?}", 
              incident_type, self.heat_level, self.current_level);
    }
    
    pub fn should_escalate(&self) -> bool {
        if self.escalation_cooldown > 0.0 { return false; }
        
        let threshold = match self.current_level {
            EscalationLevel::None => 15.0,      // Easy to trigger patrol
            EscalationLevel::Patrol => 35.0,    // Armed response
            EscalationLevel::Armed => 60.0,     // SWAT
            EscalationLevel::Tactical => 90.0,  // Military
            EscalationLevel::Military => 120.0, // Corporate
            EscalationLevel::Corporate => f32::INFINITY, // Max level
        };
        
        self.heat_level >= threshold
    }
    
    pub fn escalate(&mut self) {
        self.current_level = self.current_level.next();
        self.spawn_timer = self.current_level.response_time();
        self.escalation_cooldown = 10.0; // 10 second cooldown
        
        info!("POLICE ESCALATION: {:?} | Response in {:.1}s", 
              self.current_level, self.spawn_timer);
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

// === INCIDENT TRACKING SYSTEM ===
pub fn police_incident_tracking_system(
    mut escalation: ResMut<PoliceEscalation>,
    mut combat_events: EventReader<CombatEvent>,
    mut audio_events: EventReader<AudioEvent>,
    civilian_query: Query<&Transform, (With<Civilian>, With<Dead>)>,
    police_query: Query<&Transform, (With<Police>, With<Dead>)>,
    urban_civilian_query: Query<&UrbanCivilian, With<Civilian>>,
    time: Res<Time>,
    game_mode: Res<GameMode>,
) {
    if game_mode.paused { return; }

    escalation.escalation_timer -= time.delta_secs();
    escalation.spawn_timer -= time.delta_secs();
    escalation.escalation_cooldown -= time.delta_secs();
    
    // Heat decay over time
    escalation.heat_level = (escalation.heat_level - time.delta_secs() * 2.0).max(0.0);
    
    // Track combat incidents
    for combat_event in combat_events.read() {
        if !combat_event.hit { continue; }
        
        // Civilian casualties
        if let Ok(civilian_transform) = civilian_query.get(combat_event.target) {
            escalation.civilian_casualties += 1;
            escalation.add_incident(
                civilian_transform.translation.truncate(), 
                1.0,
                IncidentType::CivilianKilled
            );
        }
        
        // Police casualties
        if let Ok(police_transform) = police_query.get(combat_event.target) {
            escalation.add_incident(
                police_transform.translation.truncate(), 
                1.5,
                IncidentType::PoliceKilled
            );
        }
    }
    
    // Track audio incidents (gunshots)
    for audio_event in audio_events.read() {
        if matches!(audio_event.sound, AudioType::Gunshot) {
            if let Some(pos) = escalation.last_incident_pos {
                escalation.add_incident(pos, 0.5, IncidentType::Gunshot);
            }
        }
    }
    
    // Check for mass hysteria (lots of panicked civilians)
    let panicked_count = urban_civilian_query.iter()
        .filter(|urban_civ| matches!(urban_civ.daily_state, DailyState::Panicked))
        .count();
    
    if panicked_count >= 5 && escalation.incident_count % 10 == 0 { // Check occasionally
        if let Some(pos) = escalation.last_incident_pos {
            escalation.add_incident(pos, 1.0, IncidentType::MassHysteria);
        }
    }
    
    // Check escalation
    if escalation.escalation_timer <= 0.0 && escalation.should_escalate() {
        escalation.escalate();
    }
}

// === POLICE SPAWN SYSTEM ===
pub fn police_spawn_system(
    mut commands: Commands,
    mut escalation: ResMut<PoliceEscalation>,
    sprites: Res<GameSprites>,
    time: Res<Time>,
    game_mode: Res<GameMode>,
) {
    if game_mode.paused { return; }
    
    // Spawn new units when timer expires
    if escalation.spawn_timer <= 0.0 && escalation.current_level != EscalationLevel::None {
        let spawn_count = escalation.current_level.unit_count();
        let spawn_pos = escalation.last_incident_pos.unwrap_or(Vec2::new(400.0, 0.0));
        
        for i in 0..spawn_count {
            let offset = Vec2::new(
                (i as f32 - spawn_count as f32 / 2.0) * 40.0,
                (i % 2) as f32 * 30.0,
            );
            
            let current_escalation_level = escalation.current_level;

            let entity = spawn_police_unit(
                &mut commands, 
                spawn_pos + Vec2::new(400.0, 0.0) + offset, 
                current_escalation_level,
                &sprites
            );
            
            escalation.active_units.push(PoliceUnit {
                entity,
                unit_type: current_escalation_level,
                spawn_time: 0.0,
            });
        }
        
        // Set next spawn timer (longer intervals for higher levels)
        escalation.spawn_timer = match escalation.current_level {
            EscalationLevel::Patrol => 60.0,
            EscalationLevel::Armed => 45.0,
            EscalationLevel::Tactical => 40.0,
            EscalationLevel::Military => 35.0,
            EscalationLevel::Corporate => 30.0,
            EscalationLevel::None => 0.0,
        };
        
        info!("Spawned {} {:?} units", spawn_count, escalation.current_level);
    }
}

// === ENHANCED POLICE UNIT SPAWNING ===
fn spawn_police_unit(
    commands: &mut Commands,
    position: Vec2,
    unit_type: EscalationLevel,
    sprites: &GameSprites,
) -> Entity {
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
    
    // More sophisticated patrol based on unit type
    let patrol_points = match unit_type {
        EscalationLevel::Patrol | EscalationLevel::Armed => {
            // Simple patrol around spawn area
            vec![position, position + Vec2::new(80.0, 0.0)]
        },
        EscalationLevel::Tactical => {
            // Tactical sweep pattern
            vec![
                position,
                position + Vec2::new(-100.0, 50.0),
                position + Vec2::new(-100.0, -50.0),
                position + Vec2::new(50.0, -50.0),
            ]
        },
        EscalationLevel::Military | EscalationLevel::Corporate => {
            // Aggressive search pattern
            vec![
                position,
                position + Vec2::new(-200.0, 0.0),
                position + Vec2::new(-200.0, -100.0),
                position + Vec2::new(-100.0, -100.0),
                position + Vec2::new(-100.0, 100.0),
            ]
        },
        EscalationLevel::None => vec![position],
    };
    
    let mut inventory = Inventory::default();
    inventory.equipped_weapon = Some(WeaponConfig::new(weapon.clone()));
    
    // Enhanced AI for higher tiers
    let mut ai_state = AIState::default();
    ai_state.use_goap = unit_type >= EscalationLevel::Tactical; // SWAT+ uses GOAP
    
    commands.spawn_empty()
    .insert((
        sprite,
        Transform::from_translation(position.extend(1.0)),
        Enemy,
        Police { response_level: unit_type as u8 },
        Faction::Police,
        Health(health),
        Morale::new(health * 1.5, 20.0), // Police are brave
        MovementSpeed(speed),
        Vision::new(vision_range, 50.0),
        Patrol::new(patrol_points),
    ))
    .insert((
        ai_state,
        if unit_type >= EscalationLevel::Tactical { 
            GoapAgent::default() 
        } else { 
            GoapAgent::default() // All get GOAP but legacy AI for basic units
        },
        WeaponState::new(&weapon),
        inventory,
        RigidBody::Dynamic,
        Collider::ball(9.0),
        Velocity::default(),
        Damping { linear_damping: 10.0, angular_damping: 10.0 },
    )).id()
}

// === POLICE UNIT CLEANUP ===
pub fn police_cleanup_system(
    mut escalation: ResMut<PoliceEscalation>,
    police_query: Query<Entity, (With<Police>, With<Dead>)>,
) {
    // Remove dead units from tracking
    for dead_entity in police_query.iter() {
        escalation.active_units.retain(|unit| unit.entity != dead_entity);
    }
}

// === DE-ESCALATION SYSTEM ===
pub fn police_deescalation_system(
    mut escalation: ResMut<PoliceEscalation>,
    agent_query: Query<&Transform, (With<Agent>, Without<Dead>)>,
    time: Res<Time>,
    game_mode: Res<GameMode>,
) {
    if game_mode.paused { return; }
    
    // De-escalate if no incidents for a while and low heat
    if escalation.heat_level < 10.0 && escalation.escalation_timer <= -30.0 {
        if escalation.current_level > EscalationLevel::None {
            escalation.current_level = match escalation.current_level {
                EscalationLevel::Corporate => EscalationLevel::Military,
                EscalationLevel::Military => EscalationLevel::Tactical,
                EscalationLevel::Tactical => EscalationLevel::Armed,
                EscalationLevel::Armed => EscalationLevel::Patrol,
                EscalationLevel::Patrol => EscalationLevel::None,
                EscalationLevel::None => EscalationLevel::None,
            };
            
            info!("Police de-escalation: {:?}", escalation.current_level);
            escalation.escalation_timer = 0.0;
        }
    }
    
    // If no agents alive, slowly reduce heat
    if agent_query.is_empty() {
        escalation.heat_level = (escalation.heat_level - time.delta_secs() * 5.0).max(0.0);
    }
}

// === DEBUG VISUALIZATION ===
pub fn police_debug_system(
    mut gizmos: Gizmos,
    escalation: Res<PoliceEscalation>,
    police_query: Query<(&Transform, &Police), With<Police>>,
    input: Res<ButtonInput<KeyCode>>,
    mut show_police_debug: Local<bool>,
) {
    if input.just_pressed(KeyCode::KeyP) {
        *show_police_debug = !*show_police_debug;
        info!("Police debug: {} | Level: {:?} | Heat: {:.1} | Units: {}", 
              if *show_police_debug { "ON" } else { "OFF" },
              escalation.current_level,
              escalation.heat_level,
              escalation.active_units.len());
    }
    
    if !*show_police_debug { return; }
    
    // Draw police response info in top-right
    let info_pos = Vec2::new(500.0, 300.0);
    gizmos.circle_2d(info_pos, 20.0, escalation.current_level.color());
    
    // Draw heat level bar
    let heat_ratio = (escalation.heat_level / 100.0).clamp(0.0, 1.0);
    let bar_pos = Vec2::new(450.0, 280.0);
    gizmos.line_2d(bar_pos, bar_pos + Vec2::new(100.0 * heat_ratio, 0.0), 
                   Color::srgb(heat_ratio, 1.0 - heat_ratio, 0.0));
    
    // Draw police unit indicators
    for (transform, police) in police_query.iter() {
        let pos = transform.translation.truncate();
        let unit_level = EscalationLevel::try_from(police.response_level).unwrap_or(EscalationLevel::Patrol);
        
        // Police badge indicator
        gizmos.circle_2d(pos + Vec2::new(0.0, 25.0), 8.0, unit_level.color());
        gizmos.circle_2d(pos + Vec2::new(0.0, 25.0), 6.0, Color::WHITE);
    }
    
    // Draw last incident position
    if let Some(incident_pos) = escalation.last_incident_pos {
        gizmos.circle_2d(incident_pos, 30.0, Color::srgba(1.0, 0.0, 0.0, 0.5));
    }
}

// Helper for escalation level conversion
impl TryFrom<u8> for EscalationLevel {
    type Error = ();
    
    fn try_from(value: u8) -> Result<Self, Self::Error> {
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