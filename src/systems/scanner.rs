// src/systems/scanner.rs
use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use crate::core::*;
use crate::core::factions::*;
use crate::systems::npc_barks::*;

#[derive(Resource, Default)]
pub struct ScannerState {
    pub active: bool,
    pub target: Option<Entity>,
    pub window_pos: Vec2,
}

#[derive(Component)]
pub struct Scannable;

// Marker components for cleanup
#[derive(Component)]
pub struct ScannerOverlay;

#[derive(Component)]
pub struct ScannerWindow;

// Function to be called from your main input system
pub fn handle_scanner_input(
    keyboard: &Res<ButtonInput<KeyCode>>,
    mouse: &Res<ButtonInput<MouseButton>>,
    windows: &Query<&Window>,
    cameras: &Query<(&Camera, &GlobalTransform)>,
    scanner_state: &mut ResMut<ScannerState>,
    scannable_query: &Query<(Entity, &Transform), (With<Scannable>, Without<ChatBubble>, Without<MarkedForDespawn>)>,
) {
    // Toggle scanner with Tab
    if keyboard.just_pressed(KeyCode::KeyQ) {
        scanner_state.active = !scanner_state.active;
        if !scanner_state.active {
            scanner_state.target = None;
        }
    }

    // Close scanner with Escape
    if keyboard.just_pressed(KeyCode::Escape) && scanner_state.active {
        scanner_state.active = false;
        scanner_state.target = None;
    }

    // Only process mouse input if scanner is active
    if !scanner_state.active { return; }

    // Scan target on left click
    if mouse.just_pressed(MouseButton::Left) {
        if let Some(world_pos) = get_world_mouse_position(windows, cameras) {
            let mut closest_entity = None;
            let mut closest_distance = f32::INFINITY;
            
            for (entity, transform) in scannable_query.iter() {
                let entity_pos = transform.translation.truncate();
                let distance = world_pos.distance(entity_pos);
                
                if distance < 30.0 && distance < closest_distance {
                    closest_distance = distance;
                    closest_entity = Some(entity);
                }
            }
            
            if let Some(entity) = closest_entity {
                scanner_state.target = Some(entity);
                scanner_state.window_pos = world_pos;
            } else {
                scanner_state.target = None;
            }
        }
    }
}

pub fn scanner_ui_system(
    mut commands: Commands,
    scanner_state: Res<ScannerState>,
    windows: Query<&Window>,
    cameras: Query<(&Camera, &GlobalTransform)>,
    
    vehicles: Query<(&Vehicle, &Health), (With<Vehicle>, Without<MarkedForDespawn>)>,
    enemies: Query<(&Faction, &GoapAgent, &WeaponState), (With<Enemy>, Without<MarkedForDespawn>)>,
    civilians: Query<(&Morale, Has<Controllable>), (With<Civilian>, Without<MarkedForDespawn>)>,
    terminals: Query<&Terminal, Without<MarkedForDespawn>>,
    agents: Query<(&Agent, &Health, &WeaponState), Without<MarkedForDespawn>>,
    
    health_query: Query<&Health, Without<MarkedForDespawn>>,
    names: Query<&Name, Without<MarkedForDespawn>>,
    game_mode: Res<GameMode>,
) {
    if !scanner_state.active || game_mode.paused { return; }

    let Ok(window) = windows.single() else { return; };
    let Ok((camera, camera_transform)) = cameras.single() else { return; };

    // Show scan window if target exists and is still valid
    if let Some(target) = scanner_state.target {
        if vehicles.contains(target) || enemies.contains(target) || civilians.contains(target) 
           || terminals.contains(target) || agents.contains(target) {
            let screen_pos = world_to_screen_pos(scanner_state.window_pos, camera, camera_transform, window);
            show_scan_window(&mut commands, target, screen_pos, 
                            &vehicles, &enemies, &civilians, &terminals, &agents, &health_query, &names);
        }
    }
}

fn show_scan_window(
    commands: &mut Commands,
    target: Entity,
    screen_pos: Vec2,
    vehicles: &Query<(&Vehicle, &Health), (With<Vehicle>, Without<MarkedForDespawn>)>,
    enemies: &Query<(&Faction, &GoapAgent, &WeaponState), (With<Enemy>, Without<MarkedForDespawn>)>,
    civilians: &Query<(&Morale, Has<Controllable>), (With<Civilian>, Without<MarkedForDespawn>)>,
    terminals: &Query<&Terminal, Without<MarkedForDespawn>>,
    agents: &Query<(&Agent, &Health, &WeaponState), Without<MarkedForDespawn>>,
    health_query: &Query<&Health, Without<MarkedForDespawn>>,
    names: &Query<&Name, Without<MarkedForDespawn>>,
) {
    let mut lines = Vec::new();
    let mut title = "UNKNOWN".to_string();

    // Determine entity type and gather info
    if let Ok((vehicle, health)) = vehicles.get(target) {
        title = format!("{:?}", vehicle.vehicle_type);
        lines.push(format!("Health: {:.0}/{:.0}", health.0, vehicle.max_health()));
        lines.push(format!("Type: {:?}", vehicle.vehicle_type));
        
        if vehicle.explosion_damage() > 0.0 {
            lines.push("⚠ EXPLOSIVE".to_string());
        }
    }
    else if let Ok((faction, goap_agent, weapon_state)) = enemies.get(target) {
        title = "HOSTILE".to_string();
        lines.push(format!("Faction: {:?}", faction));
        lines.push(format!("State: {:?}", get_ai_state_display(goap_agent)));
        lines.push(format!("Weapon: {:?}", get_weapon_type(weapon_state)));
        
        if let Ok(health) = health_query.get(target) {
            lines.push(format!("Health: {:.0}", health.0));
        }
    }
    else if let Ok((morale, controllable)) = civilians.get(target) {
        title = "CIVILIAN".to_string();
        lines.push(format!("Morale: {:.0}", morale.current));
        
        if controllable {
            lines.push("● CONTROLLED".to_string());
        }
    }
    else if let Ok(terminal) = terminals.get(target) {
        title = "TERMINAL".to_string();
        lines.push(format!("Type: {:?}", terminal.terminal_type));
        
        if terminal.accessed {
            lines.push("✓ ACCESSED".to_string());
        } else {
            lines.push("○ HACKABLE".to_string());
        }
    }
    else if let Ok((agent, health, weapon_state)) = agents.get(target) {
        title = "AGENT".to_string();
        lines.push(format!("Level: {}", agent.level));
        lines.push(format!("Health: {:.0}", health.0));
        lines.push(format!("Weapon: {:?}", get_weapon_type(weapon_state)));
    }

    // Add name if available
    if let Ok(name) = names.get(target) {
        title = name.to_string();
    }

    // Spawn scan window
    let window_height = (lines.len() + 1) as f32 * 20.0 + 20.0;
    let window_width = 200.0;
    
    commands.spawn((
        Sprite {
            color: Color::srgba(0.1, 0.1, 0.1, 0.95),
            custom_size: Some(Vec2::new(window_width, window_height)),
            ..default()
        },
        Transform::from_xyz(screen_pos.x, screen_pos.y, 101.0),
        ScannerWindow,
    ));
    
    // Title text
    commands.spawn((
        Text2d::new(title),
        TextFont {
            font_size: 14.0,
            ..default()
        },
        TextColor(Color::WHITE),
        Transform::from_xyz(screen_pos.x, screen_pos.y + window_height/2.0 - 15.0, 102.0),
        ScannerWindow,
    ));
    
    // Info lines
    for (i, line) in lines.iter().enumerate() {
        commands.spawn((
            Text2d::new(line.clone()),
            TextFont {
                font_size: 12.0,
                ..default()
            },
            TextColor(Color::srgb(0.8, 0.8, 0.8)),
            Transform::from_xyz(
                screen_pos.x, 
                screen_pos.y + window_height/2.0 - 35.0 - (i as f32 * 16.0), 
                102.0
            ),
            ScannerWindow,
        ));
    }
}

pub fn scanner_cleanup_system(
    mut commands: Commands,
    scanner_state: Res<ScannerState>,
    overlay_query: Query<Entity, (With<ScannerOverlay>, Without<MarkedForDespawn>)>,
    window_query: Query<Entity, (With<ScannerWindow>, Without<MarkedForDespawn>)>,
) {
    if !scanner_state.active {
        for entity in overlay_query.iter() {
            commands.entity(entity).insert(MarkedForDespawn);
        }
        for entity in window_query.iter() {
            commands.entity(entity).insert(MarkedForDespawn);
        }
    }
}

fn get_ai_state_display(goap_agent: &GoapAgent) -> &str {
    // TODO: Replace with your actual goap state checking
    "UNKNOWN"
}

fn get_weapon_type(weapon_state: &WeaponState) -> String {
    // TODO: Replace with your actual weapon type field access
    format!("{:?}", WeaponType::Pistol)
}

fn world_to_screen_pos(world_pos: Vec2, camera: &Camera, camera_transform: &GlobalTransform, window: &Window) -> Vec2 {
    if let Ok(screen_pos) = camera.world_to_viewport(camera_transform, world_pos.extend(0.0)) {
        Vec2::new(
            screen_pos.x - window.width() / 2.0,
            window.height() / 2.0 - screen_pos.y,
        )
    } else {
        Vec2::ZERO
    }
}
