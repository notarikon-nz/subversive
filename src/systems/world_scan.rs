// src/systems/world_scan.rs - Complete World Scan System inspired by Satellite Reign
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

use crate::core::*;
use crate::systems::power_grid::*;
use crate::systems::hacking_financial::*;
use crate::core::factions::{Faction};

// === CORE COMPONENTS ===
#[derive(Component)]
pub struct WorldScanner {
    pub range: f32,
    pub energy: f32,
    pub max_energy: f32,
    pub scan_cost: f32,
    pub recharge_rate: f32,
    pub active: bool,
    pub scan_level: u8, // Higher levels reveal more info
}

impl Default for WorldScanner {
    fn default() -> Self {
        Self {
            range: 200.0,
            energy: 100.0,
            max_energy: 100.0,
            scan_cost: 20.0,
            recharge_rate: 10.0,
            active: false,
            scan_level: 1,
        }
    }
}

#[derive(Component, Serialize, Deserialize)]
pub struct ScannableEntity {
    pub entity_type: ScannableType,
    pub threat_level: ThreatLevel,
    pub network_connections: Vec<String>,
    pub security_rating: u8,
    pub intel_value: IntelValue,
    pub discovered: bool,
    pub analysis_complete: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ScannableType {
    // Infrastructure
    PowerStation { capacity: u32, network_id: String },
    SecurityCamera { fov: f32, detection_range: f32 },
    AutomatedTurret { damage: f32, range: f32 },
    AccessPoint { security_level: u8 },
    
    // Financial
    ATM { bank_id: String, funds: u32 },
    Terminal { data_type: String, access_level: u8 },
    
    // Personnel
    Enemy { faction: String, weapon: String, patrol_route: Option<Vec<Vec2>> },
    Civilian { controllable: bool, morale: f32 },
    Scientist { specialization: String, projects: Vec<String> },
    
    // Objects
    Vehicle { vehicle_type: String, explosive: bool },
    Loot { item_type: String, value: u32 },
    ResearchData { category: String, progress: f32 },
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ThreatLevel {
    None,      // Civilians, basic infrastructure
    Low,       // Cameras, basic enemies
    Medium,    // Armed guards, secure systems
    High,      // Turrets, elite enemies
    Critical,  // Military, high-security assets
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IntelValue {
    None,
    Financial(u32),
    Tactical(String),
    Research(String),
    Network(Vec<String>),
}

// === NETWORK ANALYSIS ===
#[derive(Component)]
pub struct NetworkNode {
    pub node_id: String,
    pub node_type: NetworkNodeType,
    pub connections: HashSet<String>,
    pub security_level: u8,
    pub operational: bool,
}

#[derive(Debug, Clone)]
pub enum NetworkNodeType {
    PowerSource,
    PowerConsumer,
    SecurityHub,
    DataNode,
    AccessController,
}

// === SCAN VISUALIZATION ===
#[derive(Component)]
pub struct ScanOverlay {
    pub target_entity: Entity,
    pub overlay_type: OverlayType,
    pub fade_timer: f32,
    pub max_fade_time: f32,
}

#[derive(Debug, Clone)]
pub enum OverlayType {
    PowerConnection(Color),
    SecurityCoverage(f32), // radius
    PatrolRoute(Vec<Vec2>),
    NetworkLink(Entity),
    ThreatIndicator(ThreatLevel),
    IntelHighlight(IntelValue),
}

// === RESOURCES ===
#[derive(Resource, Default)]
pub struct WorldScanState {
    pub active_scanner: Option<Entity>,
    pub scan_mode: ScanMode,
    pub discovered_entities: HashMap<Entity, ScannableEntity>,
    pub network_topology: HashMap<String, Vec<Entity>>,
    pub show_overlays: bool,
    pub scan_history: Vec<ScanRecord>,
}

#[derive(Debug, Clone, Copy, Default)]
pub enum ScanMode {
    #[default]
    Infrastructure,
    Security,
    Financial,
    Personnel,
    All,
}

#[derive(Debug, Clone)]
pub struct ScanRecord {
    pub timestamp: f32,
    pub scanner_pos: Vec2,
    pub entities_found: usize,
    pub intel_gathered: Vec<String>,
}

// === EVENTS ===
#[derive(Event)]
pub struct WorldScanEvent {
    pub scanner: Entity,
    pub scan_center: Vec2,
    pub scan_range: f32,
    pub scan_mode: ScanMode,
}

#[derive(Event)]
pub struct EntityScannedEvent {
    pub scanner: Entity,
    pub target: Entity,
    pub intel_discovered: Vec<String>,
    pub network_connections: Vec<String>,
}

// === MAIN SYSTEMS ===
pub fn world_scan_input_system(
    mut scan_events: EventWriter<WorldScanEvent>,
    mut scan_state: ResMut<WorldScanState>,
    input: Res<ButtonInput<KeyCode>>,
    scanner_query: Query<(Entity, &Transform, &WorldScanner), With<Agent>>,
    selection: Res<SelectionState>,
) {
    // Toggle scan mode with Tab
    if input.just_pressed(KeyCode::Tab) {
        scan_state.scan_mode = match scan_state.scan_mode {
            ScanMode::Infrastructure => ScanMode::Security,
            ScanMode::Security => ScanMode::Financial,
            ScanMode::Financial => ScanMode::Personnel,
            ScanMode::Personnel => ScanMode::All,
            ScanMode::All => ScanMode::Infrastructure,
        };
        info!("Scan mode: {:?}", scan_state.scan_mode);
    }

    // Toggle overlay visibility
    if input.just_pressed(KeyCode::KeyO) {
        scan_state.show_overlays = !scan_state.show_overlays;
    }

    // Perform scan with Enter
    if input.just_pressed(KeyCode::Enter) {
        for &selected_agent in &selection.selected {
            if let Ok((entity, transform, scanner)) = scanner_query.get(selected_agent) {
                if scanner.energy >= scanner.scan_cost {
                    scan_events.write(WorldScanEvent {
                        scanner: entity,
                        scan_center: transform.translation.truncate(),
                        scan_range: scanner.range,
                        scan_mode: scan_state.scan_mode,
                    });
                    scan_state.active_scanner = Some(entity);
                }
            }
        }
    }
}

pub fn world_scan_execution_system(
    mut commands: Commands,
    mut scan_events: EventReader<WorldScanEvent>,
    mut entity_scanned_events: EventWriter<EntityScannedEvent>,
    mut scan_state: ResMut<WorldScanState>,
    mut scanner_query: Query<&mut WorldScanner>,
    
    // Scannable entities
    power_stations: Query<(Entity, &Transform, &PowerStation, &DeviceState)>,
    cameras: Query<(Entity, &Transform, &SecurityCamera, &Vision)>,
    turrets: Query<(Entity, &Transform, &AutomatedTurret)>,
    atms: Query<(Entity, &Transform, &ATM)>,
    terminals: Query<(Entity, &Transform, &Terminal)>,
    enemies: Query<(Entity, &Transform, &Faction, &Inventory, Option<&Patrol>), With<Enemy>>,
    civilians: Query<(Entity, &Transform, &Morale, Has<Controllable>), With<Civilian>>,
    vehicles: Query<(Entity, &Transform, &Vehicle)>,
    
    hackable_query: Query<&Hackable>,
    time: Res<Time>,
) {
    for event in scan_events.read() {
        if let Ok(mut scanner) = scanner_query.get_mut(event.scanner) {
            if scanner.energy < scanner.scan_cost {
                continue;
            }

            scanner.energy -= scanner.scan_cost;
            let scan_center = event.scan_center;
            let scan_range = event.scan_range;
            let mut entities_found = 0;
            let mut intel_gathered = Vec::new();

            // Scan based on mode
            match event.scan_mode {
                ScanMode::Infrastructure | ScanMode::All => {
                    entities_found += scan_infrastructure(
                        &mut commands, &mut scan_state, &power_stations, &cameras, 
                        &turrets, &hackable_query, scan_center, scan_range, &mut intel_gathered
                    );
                }
                ScanMode::Financial | ScanMode::All => {
                    entities_found += scan_financial(
                        &mut commands, &mut scan_state, &atms, &terminals, 
                        scan_center, scan_range, &mut intel_gathered
                    );
                }
                ScanMode::Personnel | ScanMode::All => {
                    entities_found += scan_personnel(
                        &mut commands, &mut scan_state, &enemies, &civilians,
                        scan_center, scan_range, &mut intel_gathered
                    );
                }
                ScanMode::Security | ScanMode::All => {
                    entities_found += scan_security_systems(
                        &mut commands, &mut scan_state, &cameras, &turrets,
                        scan_center, scan_range, &mut intel_gathered
                    );
                }
            }

            // Scan vehicles if in range
            if matches!(event.scan_mode, ScanMode::All | ScanMode::Personnel) {
                entities_found += scan_vehicles(
                    &mut commands, &mut scan_state, &vehicles,
                    scan_center, scan_range, &mut intel_gathered
                );
            }

            // Record scan
            scan_state.scan_history.push(ScanRecord {
                timestamp: time.elapsed_secs(),
                scanner_pos: scan_center,
                entities_found,
                intel_gathered: intel_gathered.clone(),
            });

            if entities_found > 0 {
                entity_scanned_events.write(EntityScannedEvent {
                    scanner: event.scanner,
                    target: Entity::PLACEHOLDER, // Multiple targets
                    intel_discovered: intel_gathered,
                    network_connections: Vec::new(),
                });

                info!("Scan complete: {} entities discovered", entities_found);
            }
        }
    }
}

// === SCANNING FUNCTIONS ===
fn scan_infrastructure(
    commands: &mut Commands,
    scan_state: &mut WorldScanState,
    power_stations: &Query<(Entity, &Transform, &PowerStation, &DeviceState)>,
    cameras: &Query<(Entity, &Transform, &SecurityCamera, &Vision)>,
    turrets: &Query<(Entity, &Transform, &AutomatedTurret)>,
    hackable_query: &Query<&Hackable>,
    scan_center: Vec2,
    scan_range: f32,
    intel_gathered: &mut Vec<String>,
) -> usize {
    let mut count = 0;

    // Power stations
    for (entity, transform, station, device_state) in power_stations.iter() {
        let pos = transform.translation.truncate();
        if pos.distance(scan_center) <= scan_range {
            let scannable = ScannableEntity {
                entity_type: ScannableType::PowerStation {
                    capacity: station.max_capacity,
                    network_id: station.network_id.clone(),
                },
                threat_level: ThreatLevel::Medium,
                network_connections: vec![station.network_id.clone()],
                security_rating: if let Ok(hackable) = hackable_query.get(entity) {
                    hackable.security_level
                } else { 0 },
                intel_value: IntelValue::Network(vec![station.network_id.clone()]),
                discovered: true,
                analysis_complete: true,
            };

            scan_state.discovered_entities.insert(entity, scannable);
            scan_state.network_topology
                .entry(station.network_id.clone())
                .or_default()
                .push(entity);

            intel_gathered.push(format!("Power Station: {} MW capacity", station.max_capacity));
            if !device_state.operational {
                intel_gathered.push("⚠ SYSTEM OFFLINE".to_string());
            }
            count += 1;

            spawn_scan_overlay(commands, entity, OverlayType::PowerConnection(Color::srgb(0.8, 0.8, 0.2)));
        }
    }

    count
}

fn scan_financial(
    commands: &mut Commands,
    scan_state: &mut WorldScanState,
    atms: &Query<(Entity, &Transform, &ATM)>,
    terminals: &Query<(Entity, &Transform, &Terminal)>,
    scan_center: Vec2,
    scan_range: f32,
    intel_gathered: &mut Vec<String>,
) -> usize {
    let mut count = 0;

    // ATMs
    for (entity, transform, atm) in atms.iter() {
        let pos = transform.translation.truncate();
        if pos.distance(scan_center) <= scan_range {
            let scannable = ScannableEntity {
                entity_type: ScannableType::ATM {
                    bank_id: atm.bank_id.clone(),
                    funds: atm.current_balance,
                },
                threat_level: ThreatLevel::Low,
                network_connections: vec![atm.bank_id.clone()],
                security_rating: 3,
                intel_value: IntelValue::Financial(atm.current_balance),
                discovered: true,
                analysis_complete: true,
            };

            scan_state.discovered_entities.insert(entity, scannable);
            intel_gathered.push(format!("ATM: ${} available", atm.current_balance));
            intel_gathered.push(format!("Bank: {}", atm.bank_id));
            count += 1;

            spawn_scan_overlay(commands, entity, OverlayType::IntelHighlight(IntelValue::Financial(atm.current_balance)));
        }
    }

    count
}

fn scan_personnel(
    commands: &mut Commands,
    scan_state: &mut WorldScanState,
    enemies: &Query<(Entity, &Transform, &Faction, &Inventory, Option<&Patrol>), With<Enemy>>,
    civilians: &Query<(Entity, &Transform, &Morale, Has<Controllable>), With<Civilian>>,
    scan_center: Vec2,
    scan_range: f32,
    intel_gathered: &mut Vec<String>,
) -> usize {
    let mut count = 0;

    // Enemies
    for (entity, transform, faction, inventory, patrol) in enemies.iter() {
        let pos = transform.translation.truncate();
        if pos.distance(scan_center) <= scan_range {
            let patrol_route = patrol.map(|p| p.points.clone());
            let threat = match faction {
                Faction::Police => ThreatLevel::Medium,
                Faction::Corporate => ThreatLevel::High,
                Faction::Syndicate => ThreatLevel::High,
                _ => ThreatLevel::Low,
            };

            // Get weapon type from inventory
            let weapon_name = if let Some(equipped_weapon) = &inventory.equipped_weapon {
                format!("{:?}", equipped_weapon.base_weapon)
            } else {
                "Unarmed".to_string()
            };

            let scannable = ScannableEntity {
                entity_type: ScannableType::Enemy {
                    faction: format!("{:?}", faction),
                    weapon: weapon_name.clone(),
                    patrol_route,
                },
                threat_level: threat,
                network_connections: Vec::new(),
                security_rating: 2,
                intel_value: IntelValue::Tactical(format!("{:?} with {}", faction, weapon_name)),
                discovered: true,
                analysis_complete: true,
            };

            scan_state.discovered_entities.insert(entity, scannable);
            intel_gathered.push(format!("Hostile: {:?} - {}", faction, weapon_name));
            
            if let Some(patrol) = patrol {
                intel_gathered.push(format!("Patrol route: {} waypoints", patrol.points.len()));
                spawn_scan_overlay(commands, entity, OverlayType::PatrolRoute(patrol.points.clone()));
            }
            
            spawn_scan_overlay(commands, entity, OverlayType::ThreatIndicator(threat));
            count += 1;
        }
    }

    // Civilians
    for (entity, transform, morale, controllable) in civilians.iter() {
        let pos = transform.translation.truncate();
        if pos.distance(scan_center) <= scan_range {
            let scannable = ScannableEntity {
                entity_type: ScannableType::Civilian {
                    controllable,
                    morale: morale.current,
                },
                threat_level: ThreatLevel::None,
                network_connections: Vec::new(),
                security_rating: 0,
                intel_value: if controllable {
                    IntelValue::Tactical("Controllable target".to_string())
                } else {
                    IntelValue::None
                },
                discovered: true,
                analysis_complete: true,
            };

            scan_state.discovered_entities.insert(entity, scannable);
            
            if controllable {
                intel_gathered.push("Civilian: Neurovector compatible".to_string());
                spawn_scan_overlay(commands, entity, OverlayType::IntelHighlight(IntelValue::Tactical("Controllable".to_string())));
            }
            count += 1;
        }
    }

    count
}

fn scan_security_systems(
    commands: &mut Commands,
    scan_state: &mut WorldScanState,
    cameras: &Query<(Entity, &Transform, &SecurityCamera, &Vision)>,
    turrets: &Query<(Entity, &Transform, &AutomatedTurret)>,
    scan_center: Vec2,
    scan_range: f32,
    intel_gathered: &mut Vec<String>,
) -> usize {
    let mut count = 0;

    // Security cameras
    for (entity, transform, camera, vision) in cameras.iter() {
        let pos = transform.translation.truncate();
        if pos.distance(scan_center) <= scan_range {
            let scannable = ScannableEntity {
                entity_type: ScannableType::SecurityCamera {
                    fov: vision.angle,
                    detection_range: camera.detection_range,
                },
                threat_level: ThreatLevel::Low,
                network_connections: Vec::new(),
                security_rating: 2,
                intel_value: IntelValue::Tactical("Surveillance coverage".to_string()),
                discovered: true,
                analysis_complete: true,
            };

            scan_state.discovered_entities.insert(entity, scannable);
            intel_gathered.push(format!("Camera: {:.0}° FOV, {:.0}m range", 
                                      vision.angle.to_degrees(), camera.detection_range));
            
            spawn_scan_overlay(commands, entity, OverlayType::SecurityCoverage(camera.detection_range));
            count += 1;
        }
    }

    // Turrets
    for (entity, transform, turret) in turrets.iter() {
        let pos = transform.translation.truncate();
        if pos.distance(scan_center) <= scan_range {
            let scannable = ScannableEntity {
                entity_type: ScannableType::AutomatedTurret {
                    damage: turret.damage,
                    range: turret.range,
                },
                threat_level: ThreatLevel::Critical,
                network_connections: Vec::new(),
                security_rating: 4,
                intel_value: IntelValue::Tactical(format!("Automated defense: {:.0} damage", turret.damage)),
                discovered: true,
                analysis_complete: true,
            };

            scan_state.discovered_entities.insert(entity, scannable);
            intel_gathered.push(format!("Turret: {:.0} damage, {:.0}m range", turret.damage, turret.range));
            
            spawn_scan_overlay(commands, entity, OverlayType::ThreatIndicator(ThreatLevel::Critical));
            spawn_scan_overlay(commands, entity, OverlayType::SecurityCoverage(turret.range));
            count += 1;
        }
    }

    count
}

fn scan_vehicles(
    commands: &mut Commands,
    scan_state: &mut WorldScanState,
    vehicles: &Query<(Entity, &Transform, &Vehicle)>,
    scan_center: Vec2,
    scan_range: f32,
    intel_gathered: &mut Vec<String>,
) -> usize {
    let mut count = 0;

    for (entity, transform, vehicle) in vehicles.iter() {
        let pos = transform.translation.truncate();
        if pos.distance(scan_center) <= scan_range {
            let explosive = vehicle.explosion_damage() > 0.0;
            let threat = if explosive { ThreatLevel::Medium } else { ThreatLevel::None };

            let scannable = ScannableEntity {
                entity_type: ScannableType::Vehicle {
                    vehicle_type: format!("{:?}", vehicle.vehicle_type),
                    explosive,
                },
                threat_level: threat,
                network_connections: Vec::new(),
                security_rating: 1,
                intel_value: if explosive {
                    IntelValue::Tactical("Explosive vehicle".to_string())
                } else {
                    IntelValue::None
                },
                discovered: true,
                analysis_complete: true,
            };

            scan_state.discovered_entities.insert(entity, scannable);
            intel_gathered.push(format!("Vehicle: {:?}", vehicle.vehicle_type));
            
            if explosive {
                intel_gathered.push("⚠ EXPLOSIVE".to_string());
                spawn_scan_overlay(commands, entity, OverlayType::ThreatIndicator(ThreatLevel::Medium));
            }
            count += 1;
        }
    }

    count
}

// === VISUALIZATION SYSTEM ===
pub fn world_scan_visualization_system(
    mut commands: Commands,
    mut gizmos: Gizmos,
    scan_state: Res<WorldScanState>,
    overlay_query: Query<(Entity, &ScanOverlay, &Transform)>,
    scanner_query: Query<(&Transform, &WorldScanner), With<Agent>>,
    time: Res<Time>,
) {
    if !scan_state.show_overlays {
        return;
    }

    // Draw scan range for active scanner
    if let Some(scanner_entity) = scan_state.active_scanner {
        if let Ok((transform, scanner)) = scanner_query.get(scanner_entity) {
            let pos = transform.translation.truncate();
            gizmos.circle_2d(pos, scanner.range, Color::srgba(0.2, 0.8, 0.8, 0.3));
        }
    }

    // Draw network connections
    draw_network_topology(&mut gizmos, &scan_state);

    // Update scan overlays
    for (entity, overlay, transform) in overlay_query.iter() {
        let pos = transform.translation.truncate();
        let alpha = (overlay.fade_timer / overlay.max_fade_time).clamp(0.0, 1.0);

        match &overlay.overlay_type {
            OverlayType::PowerConnection(base_color) => {
                let color = Color::srgba(base_color.to_srgba().red, base_color.to_srgba().green, base_color.to_srgba().blue, alpha * 0.7);
                gizmos.circle_2d(pos, 20.0, color);
            }
            OverlayType::SecurityCoverage(radius) => {
                let color = Color::srgba(0.8, 0.2, 0.2, alpha * 0.2);
                gizmos.circle_2d(pos, *radius, color);
            }
            OverlayType::PatrolRoute(points) => {
                let color = Color::srgba(0.8, 0.8, 0.2, alpha * 0.5);
                for i in 0..points.len() {
                    let next_i = (i + 1) % points.len();
                    gizmos.line_2d(points[i], points[next_i], color);
                }
            }
            OverlayType::ThreatIndicator(threat) => {
                let color = match threat {
                    ThreatLevel::Critical => Color::srgba(1.0, 0.0, 0.0, alpha),
                    ThreatLevel::High => Color::srgba(0.8, 0.2, 0.0, alpha),
                    ThreatLevel::Medium => Color::srgba(0.8, 0.6, 0.0, alpha),
                    ThreatLevel::Low => Color::srgba(0.6, 0.8, 0.2, alpha),
                    ThreatLevel::None => Color::srgba(0.2, 0.8, 0.2, alpha),
                };
                gizmos.circle_2d(pos + Vec2::new(0.0, 25.0), 8.0, color);
            }
            OverlayType::IntelHighlight(_) => {
                let color = Color::srgba(0.2, 0.8, 0.8, alpha * 0.8);
                gizmos.circle_2d(pos, 15.0, color);
            }
            _ => {}
        }
    }
}

fn draw_network_topology(gizmos: &mut Gizmos, scan_state: &WorldScanState) {
    // Draw connections between network nodes
    for (network_id, entities) in &scan_state.network_topology {
        if entities.len() < 2 { continue; }
        
        let color = match network_id.as_str() {
            id if id.contains("power") => Color::srgba(0.8, 0.8, 0.2, 0.4),
            id if id.contains("security") => Color::srgba(0.8, 0.2, 0.2, 0.4),
            id if id.contains("financial") => Color::srgba(0.2, 0.8, 0.2, 0.4),
            _ => Color::srgba(0.6, 0.6, 0.6, 0.4),
        };

        // Draw connections between all entities in the network
        for i in 0..entities.len() {
            for j in (i + 1)..entities.len() {
                // This would need entity position lookup - simplified for now
                // In real implementation, you'd query Transform components
            }
        }
    }
}

// === UTILITY SYSTEMS ===
pub fn scanner_energy_system(
    mut scanner_query: Query<&mut WorldScanner>,
    time: Res<Time>,
) {
    for mut scanner in scanner_query.iter_mut() {
        if scanner.energy < scanner.max_energy {
            scanner.energy = (scanner.energy + scanner.recharge_rate * time.delta_secs())
                .min(scanner.max_energy);
        }
    }
}

pub fn scan_overlay_fade_system(
    mut commands: Commands,
    mut overlay_query: Query<(Entity, &mut ScanOverlay)>,
    time: Res<Time>,
) {
    for (entity, mut overlay) in overlay_query.iter_mut() {
        overlay.fade_timer -= time.delta_secs();
        
        if overlay.fade_timer <= 0.0 {
            commands.entity(entity).insert(MarkedForDespawn);
        }
    }
}

pub fn cleanup_scan_overlays(
    mut commands: Commands,
    overlay_query: Query<Entity, With<ScanOverlay>>,
    game_state: Res<State<GameState>>,
) {
    if game_state.is_changed() && !matches!(*game_state.get(), GameState::Mission) {
        for entity in overlay_query.iter() {
            commands.entity(entity).insert(MarkedForDespawn);
        }
    }
}

// === HELPER FUNCTIONS ===
fn spawn_scan_overlay(commands: &mut Commands, target_entity: Entity, overlay_type: OverlayType) {
    commands.spawn(ScanOverlay {
        target_entity,
        overlay_type,
        fade_timer: 10.0, // 10 seconds visibility
        max_fade_time: 10.0,
    });
}

// === INTEGRATION FUNCTIONS ===
pub fn add_scanner_to_agent(commands: &mut Commands, agent_entity: Entity, scan_level: u8) {
    let scanner = WorldScanner {
        scan_level,
        range: 150.0 + (scan_level as f32 * 50.0), // Higher level = longer range
        energy: 100.0,
        max_energy: 100.0,
        scan_cost: 15.0 + (scan_level as f32 * 5.0), // Higher level = more expensive
        recharge_rate: 8.0 + (scan_level as f32 * 2.0), // Higher level = faster recharge
        active: false,
    };
    
    commands.entity(agent_entity).insert(scanner);
}

pub fn setup_world_scan_system(app: &mut App) {
    app.init_resource::<WorldScanState>()
       .add_event::<WorldScanEvent>()
       .add_event::<EntityScannedEvent>()
       .add_systems(Update, (
           world_scan_input_system,
           world_scan_execution_system,
           world_scan_visualization_system,
           scanner_energy_system,
           scan_overlay_fade_system,
       ).run_if(in_state(GameState::Mission)))
       .add_systems(OnExit(GameState::Mission), cleanup_scan_overlays);
}