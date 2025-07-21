// src/systems/urban_simulation.rs - Lean crowd dynamics for Syndicate-like city feel
use bevy::prelude::*;
use crate::core::*;
use crate::systems::*;

// === CORE URBAN COMPONENTS ===
#[derive(Component)]
pub struct UrbanCivilian {
    pub daily_state: DailyState,
    pub state_timer: f32,
    pub next_destination: Option<Vec2>,
    pub crowd_influence: f32,        // How much they're affected by crowds
    pub panic_threshold: f32,        // When they start panicking
    pub movement_urgency: f32,       // 0.0 = casual, 1.0 = running
}

#[derive(Debug, Clone, Copy)]
pub enum DailyState {
    GoingToWork,
    Working,
    Shopping,
    GoingHome,
    Idle,
    Panicked,
    Following,    // Following crowd
}

#[derive(Component)]
pub struct CrowdNode {
    pub crowd_size: usize,
    pub panic_level: f32,
    pub movement_direction: Vec2,
    pub influence_radius: f32,
}

// === URBAN AREAS ===
#[derive(Resource)]
pub struct UrbanAreas {
    pub work_zones: Vec<UrbanZone>,
    pub shopping_zones: Vec<UrbanZone>,
    pub residential_zones: Vec<UrbanZone>,
    pub transit_routes: Vec<TransitRoute>,
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

impl Default for UrbanAreas {
    fn default() -> Self {
        Self {
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
                TransitRoute { 
                    points: vec![Vec2::new(0.0, -200.0), Vec2::new(0.0, 0.0), Vec2::new(0.0, 200.0)], 
                    foot_traffic_density: 0.6 
                },
            ],
        }
    }
}

// === ENHANCED CIVILIAN SPAWNING ===
pub fn urban_civilian_spawn_system(
    mut commands: Commands,
    mut spawner: ResMut<CivilianSpawner>,
    urban_areas: ResMut<UrbanAreas>,
    civilian_query: Query<Entity, With<Civilian>>,
    sprites: Res<GameSprites>,
    time: Res<Time>,
    game_mode: Res<GameMode>,
) {
    if game_mode.paused { return; }

    spawner.spawn_timer -= time.delta_secs();
    let current_count = civilian_query.iter().count();

    if spawner.spawn_timer <= 0.0 && current_count < spawner.max_civilians as usize {
        // Spawn near transit routes or residential areas for realism
        if let Some(spawn_pos) = find_realistic_spawn_position(&urban_areas) {
            spawn_urban_civilian(&mut commands, spawn_pos, &sprites, &urban_areas);
            spawner.spawn_timer = 3.0 + rand::random::<f32>() * 6.0; // More frequent spawning
        }
    }
}

fn find_realistic_spawn_position(urban_areas: &UrbanAreas) -> Option<Vec2> {
    // 60% spawn near transit routes, 40% near residential
    if rand::random::<f32>() < 0.6 && !urban_areas.transit_routes.is_empty() {
        let route = &urban_areas.transit_routes[rand::random::<usize>() % urban_areas.transit_routes.len()];
        if !route.points.is_empty() {
            let point = route.points[rand::random::<usize>() % route.points.len()];
            let offset = Vec2::new(
                (rand::random::<f32>() - 0.5) * 40.0,
                (rand::random::<f32>() - 0.5) * 40.0,
            );
            return Some(point + offset);
        }
    }
    
    // Fallback to residential areas
    if !urban_areas.residential_zones.is_empty() {
        let zone = &urban_areas.residential_zones[rand::random::<usize>() % urban_areas.residential_zones.len()];
        let angle = rand::random::<f32>() * std::f32::consts::TAU;
        let distance = rand::random::<f32>() * zone.radius;
        let offset = Vec2::new(angle.cos(), angle.sin()) * distance;
        return Some(zone.center + offset);
    }
    
    None
}

pub fn spawn_urban_civilian(
    commands: &mut Commands, 
    position: Vec2, 
    sprites: &GameSprites,
    urban_areas: &UrbanAreas,
) {
    let (sprite, _) = crate::core::sprites::create_civilian_sprite(sprites);
    
    // Randomize civilian personality
    let crowd_influence = 0.3 + rand::random::<f32>() * 0.4; // 0.3-0.7
    let panic_threshold = 20.0 + rand::random::<f32>() * 40.0; // 20-60
    
    // Pick initial daily state based on "time of day" simulation
    let daily_state = match rand::random::<f32>() {
        x if x < 0.4 => DailyState::GoingToWork,
        x if x < 0.6 => DailyState::Shopping,
        x if x < 0.8 => DailyState::GoingHome,
        _ => DailyState::Idle,
    };
    
    let next_destination = pick_destination_for_state(daily_state, urban_areas);
    
    commands.spawn((
        sprite,
        Transform::from_translation(position.extend(1.0)),
        Civilian,
        Health(50.0),
        Morale::new(80.0, panic_threshold),
        MovementSpeed(80.0 + rand::random::<f32>() * 40.0), // Varied walking speeds
        Controllable,
        NeurovectorTarget,
        UrbanCivilian {
            daily_state,
            state_timer: rand::random::<f32>() * 10.0, // Stagger state changes
            next_destination,
            crowd_influence,
            panic_threshold,
            movement_urgency: 0.0,
        },
        bevy_rapier2d::prelude::RigidBody::Dynamic,
        bevy_rapier2d::prelude::Collider::ball(7.5),
        bevy_rapier2d::prelude::Velocity::default(),
        bevy_rapier2d::prelude::Damping { linear_damping: 10.0, angular_damping: 10.0 },
    ));
}

fn pick_destination_for_state(state: DailyState, urban_areas: &UrbanAreas) -> Option<Vec2> {
    match state {
        DailyState::GoingToWork => pick_random_zone_center(&urban_areas.work_zones),
        DailyState::Shopping => pick_random_zone_center(&urban_areas.shopping_zones),
        DailyState::GoingHome => pick_random_zone_center(&urban_areas.residential_zones),
        _ => None,
    }
}

fn pick_random_zone_center(zones: &[UrbanZone]) -> Option<Vec2> {
    if zones.is_empty() { return None; }
    let zone = &zones[rand::random::<usize>() % zones.len()];
    
    // Add some randomness within the zone
    let angle = rand::random::<f32>() * std::f32::consts::TAU;
    let distance = rand::random::<f32>() * zone.radius * 0.8;
    let offset = Vec2::new(angle.cos(), angle.sin()) * distance;
    
    Some(zone.center + offset)
}

// === CROWD DYNAMICS SYSTEM ===
pub fn crowd_dynamics_system(
    commands: Commands,
    mut civilian_query: Query<(Entity, &Transform, &mut UrbanCivilian, &mut MovementSpeed, &Morale), With<Civilian>>,
    crowd_query: Query<(&Transform, &CrowdNode), Without<Civilian>>,
    mut action_events: EventWriter<ActionEvent>,
    time: Res<Time>,
    game_mode: Res<GameMode>,
) {
    if game_mode.paused { return; }

    // Create dynamic crowd nodes from civilian clusters
    let mut crowd_nodes = Vec::new();
    create_dynamic_crowd_nodes(&civilian_query, &mut crowd_nodes);
    
    // Apply crowd influence to each civilian
    for (entity, transform, mut urban_civ, mut movement_speed, morale) in civilian_query.iter_mut() {
        let pos = transform.translation.truncate();
        
        // Find nearby crowd influence
        let mut crowd_effect = 0.0;
        let mut crowd_direction = Vec2::ZERO;
        let mut panic_influence = 0.0;
        
        for crowd_node in &crowd_nodes {
            let distance = pos.distance(crowd_node.position);
            if distance <= crowd_node.influence_radius {
                let influence_strength = 1.0 - (distance / crowd_node.influence_radius);
                crowd_effect += influence_strength * urban_civ.crowd_influence;
                crowd_direction += crowd_node.movement_direction * influence_strength;
                panic_influence += crowd_node.panic_level * influence_strength;
            }
        }
        
        // Update civilian state based on crowd and panic
        if panic_influence > urban_civ.panic_threshold || morale.is_panicked() {
            if !matches!(urban_civ.daily_state, DailyState::Panicked) {
                urban_civ.daily_state = DailyState::Panicked;
                urban_civ.movement_urgency = 1.0;
                urban_civ.next_destination = Some(find_escape_destination(pos));
            }
        } else if crowd_effect > 0.5 {
            // Follow the crowd
            urban_civ.daily_state = DailyState::Following;
            urban_civ.movement_urgency = 0.3 + crowd_effect * 0.4;
            
            if crowd_direction.length() > 0.1 {
                urban_civ.next_destination = Some(pos + crowd_direction.normalize() * 150.0);
            }
        }
        
        // Apply movement urgency to speed
        movement_speed.0 = (80.0 + rand::random::<f32>() * 40.0) * (1.0 + urban_civ.movement_urgency);
        
        // Send movement command if we have a destination
        if let Some(destination) = urban_civ.next_destination {
            if pos.distance(destination) > 20.0 {
                action_events.write(ActionEvent {
                    entity,
                    action: Action::MoveTo(destination),
                });
            } else {
                // Reached destination - pick new state
                transition_daily_state(&mut urban_civ);
            }
        }
    }
}

#[derive(Clone)]
struct DynamicCrowdNode {
    position: Vec2,
    crowd_size: usize,
    panic_level: f32,
    movement_direction: Vec2,
    influence_radius: f32,
}

fn create_dynamic_crowd_nodes(
    civilian_query: &Query<(Entity, &Transform, &mut UrbanCivilian, &mut MovementSpeed, &Morale), With<Civilian>>,
    crowd_nodes: &mut Vec<DynamicCrowdNode>,
) {
    let positions: Vec<(Vec2, f32, Vec2)> = civilian_query.iter()
        .map(|(_, transform, urban_civ, movement_speed, morale)| {
            let panic_level = if morale.is_panicked() { 1.0 } else { 0.0 };
            let movement_dir = match urban_civ.next_destination {
                Some(dest) => (dest - transform.translation.truncate()).normalize_or_zero(),
                None => Vec2::ZERO,
            };
            (transform.translation.truncate(), panic_level, movement_dir)
        })
        .collect();
    
    // Simple clustering: find groups of 3+ civilians within 50 units
    let cluster_radius = 50.0;
    let mut processed = vec![false; positions.len()];
    
    for i in 0..positions.len() {
        if processed[i] { continue; }
        
        let mut cluster = vec![i];
        let mut avg_pos = positions[i].0;
        let mut avg_panic = positions[i].1;
        let mut avg_movement = positions[i].2;
        
        // Find nearby civilians
        for j in (i + 1)..positions.len() {
            if processed[j] { continue; }
            
            if positions[i].0.distance(positions[j].0) <= cluster_radius {
                cluster.push(j);
                avg_pos += positions[j].0;
                avg_panic += positions[j].1;
                avg_movement += positions[j].2;
                processed[j] = true;
            }
        }
        
        // Create crowd node if cluster is big enough
        if cluster.len() >= 3 {
            avg_pos /= cluster.len() as f32;
            avg_panic /= cluster.len() as f32;
            avg_movement = avg_movement.normalize_or_zero();
            
            crowd_nodes.push(DynamicCrowdNode {
                position: avg_pos,
                crowd_size: cluster.len(),
                panic_level: avg_panic,
                movement_direction: avg_movement,
                influence_radius: 40.0 + (cluster.len() as f32 * 10.0).min(80.0),
            });
        }
        
        processed[i] = true;
    }
}

// === DAILY ROUTINE SYSTEM ===
pub fn daily_routine_system(
    mut civilian_query: Query<(Entity, &Transform, &mut UrbanCivilian), With<Civilian>>,
    urban_areas: Res<UrbanAreas>,
    time: Res<Time>,
    game_mode: Res<GameMode>,
) {
    if game_mode.paused { return; }

    for (entity, transform, mut urban_civ) in civilian_query.iter_mut() {
        // Skip if panicked or following crowd
        if matches!(urban_civ.daily_state, DailyState::Panicked | DailyState::Following) {
            continue;
        }
        
        urban_civ.state_timer -= time.delta_secs();
        
        if urban_civ.state_timer <= 0.0 {
            transition_daily_state(&mut urban_civ);
            urban_civ.next_destination = pick_destination_for_state(urban_civ.daily_state, &urban_areas);
            urban_civ.state_timer = 15.0 + rand::random::<f32>() * 20.0; // 15-35 seconds per state
        }
    }
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
        DailyState::Panicked => DailyState::Idle, // Calm down eventually
        DailyState::Following => DailyState::Idle, // Stop following crowd
    };
    
    // Reset urgency when transitioning out of panic/following
    if matches!(urban_civ.daily_state, DailyState::Idle | DailyState::GoingToWork | DailyState::Shopping) {
        urban_civ.movement_urgency = 0.0;
    }
}

fn find_escape_destination(current_pos: Vec2) -> Vec2 {
    // Simple escape: run towards map edges
    let to_edge = if current_pos.x.abs() > current_pos.y.abs() {
        Vec2::new(current_pos.x.signum(), 0.0)
    } else {
        Vec2::new(0.0, current_pos.y.signum())
    };
    
    current_pos + to_edge * 400.0
}

// === CLEANUP AND OPTIMIZATION ===
pub fn urban_cleanup_system(
    mut commands: Commands,
    civilian_query: Query<(Entity, &Transform, &UrbanCivilian), With<Civilian>>,
    agent_query: Query<&Transform, With<Agent>>,
) {
    // Remove civilians that are too far from action
    if agent_query.is_empty() { return; }
    
    let agent_positions: Vec<Vec2> = agent_query.iter()
        .map(|t| t.translation.truncate())
        .collect();
    
    for (entity, transform, urban_civ) in civilian_query.iter() {
        let pos = transform.translation.truncate();
        let min_distance = agent_positions.iter()
            .map(|&agent_pos| pos.distance(agent_pos))
            .fold(f32::INFINITY, f32::min);
        
        // Despawn if too far and not in an interesting state
        if min_distance > 800.0 && matches!(urban_civ.daily_state, DailyState::Idle | DailyState::GoingHome) {
            commands.entity(entity).despawn();
        }
    }
}

// === DEBUG VISUALIZATION ===
pub fn urban_debug_system(
    mut gizmos: Gizmos,
    civilian_query: Query<(&Transform, &UrbanCivilian), With<Civilian>>,
    urban_areas: Res<UrbanAreas>,
    input: Res<ButtonInput<KeyCode>>,
    mut show_urban_debug: Local<bool>,
) {
    if input.just_pressed(KeyCode::KeyL) {
        *show_urban_debug = !*show_urban_debug;
        info!("Urban debug: {}", if *show_urban_debug { "ON" } else { "OFF" });
    }
    
    if !*show_urban_debug { return; }
    
    // Draw urban zones
    for zone in &urban_areas.work_zones {
        gizmos.circle_2d(zone.center, zone.radius, Color::srgba(0.8, 0.8, 0.2, 0.3));
    }
    for zone in &urban_areas.shopping_zones {
        gizmos.circle_2d(zone.center, zone.radius, Color::srgba(0.2, 0.8, 0.8, 0.3));
    }
    for zone in &urban_areas.residential_zones {
        gizmos.circle_2d(zone.center, zone.radius, Color::srgba(0.2, 0.8, 0.2, 0.3));
    }
    
    // Draw transit routes
    for route in &urban_areas.transit_routes {
        for i in 0..route.points.len().saturating_sub(1) {
            gizmos.line_2d(route.points[i], route.points[i + 1], Color::srgb(0.6, 0.6, 0.6));
        }
    }
    
    // Draw civilian state indicators
    for (transform, urban_civ) in civilian_query.iter() {
        let pos = transform.translation.truncate();
        let state_color = match urban_civ.daily_state {
            DailyState::GoingToWork => Color::srgb(0.8, 0.8, 0.2),
            DailyState::Working => Color::srgb(0.6, 0.6, 0.2),
            DailyState::Shopping => Color::srgb(0.2, 0.8, 0.8),
            DailyState::GoingHome => Color::srgb(0.2, 0.8, 0.2),
            DailyState::Idle => Color::srgb(0.6, 0.6, 0.6),
            DailyState::Panicked => Color::srgb(0.8, 0.2, 0.2),
            DailyState::Following => Color::srgb(0.8, 0.2, 0.8),
        };
        
        gizmos.circle_2d(pos + Vec2::new(0.0, 15.0), 4.0, state_color);
        
        // Draw destination line
        if let Some(dest) = urban_civ.next_destination {
            gizmos.line_2d(pos, dest, Color::srgba(1.0, 1.0, 1.0, 0.3));
        }
    }
}