// src/systems/isometric_camera.rs - Camera system for isometric view
use bevy::prelude::*;
use crate::core::*;

// === ISOMETRIC CAMERA COMPONENT ===
#[derive(Component)]
pub struct IsometricCamera {
    pub zoom: f32,
    pub min_zoom: f32,
    pub max_zoom: f32,
    pub follow_target: Option<Entity>,
    pub follow_smoothing: f32,
    pub bounds: Option<CameraBounds>,
}

#[derive(Clone)]
pub struct CameraBounds {
    pub min_x: f32,
    pub max_x: f32,
    pub min_y: f32,
    pub max_y: f32,
}

impl Default for IsometricCamera {
    fn default() -> Self {
        Self {
            zoom: 1.0,
            min_zoom: 0.3,
            max_zoom: 3.0,
            follow_target: None,
            follow_smoothing: 5.0,
            bounds: None,
        }
    }
}

// === CAMERA SETUP ===
pub fn setup_isometric_camera(mut commands: Commands) {
    // Create isometric camera with appropriate projection
    commands.spawn((
        Camera2d,
        Camera {
            clear_color: ClearColorConfig::Custom(Color::srgb(0.1, 0.2, 0.1)), // Darker green background
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, 1000.0),
        IsometricCamera {
            zoom: 0.8, // Start slightly zoomed out
            bounds: Some(CameraBounds {
                min_x: -1000.0,
                max_x: 1000.0,
                min_y: -800.0,
                max_y: 800.0,
            }),
            ..default()
        },
        // Enable smooth movement
        bevy_rapier2d::prelude::Velocity::default(),
    ));
    
    info!("Isometric camera initialized");
}

// === CAMERA MOVEMENT SYSTEM ===
pub fn isometric_camera_movement(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut mouse_wheel: EventReader<bevy::input::mouse::MouseWheel>,
    mut camera_query: Query<(&mut Transform, &mut IsometricCamera), With<Camera2d>>,
    agent_query: Query<&Transform, (With<Agent>, Without<Camera2d>)>,
    selection: Res<SelectionState>,
    time: Res<Time>,
    game_mode: Res<GameMode>,
) {
    if game_mode.paused { return; }
    
    let Ok((mut camera_transform, mut iso_camera)) = camera_query.single_mut() else { return; };
    
    let mut movement = Vec2::ZERO;
    let camera_speed = 400.0 / iso_camera.zoom; // Faster when zoomed out
    
    // WASD movement
    if keyboard.pressed(KeyCode::KeyW) || keyboard.pressed(KeyCode::ArrowUp) {
        movement.y += 1.0;
    }
    if keyboard.pressed(KeyCode::KeyS) || keyboard.pressed(KeyCode::ArrowDown) {
        movement.y -= 1.0;
    }
    if keyboard.pressed(KeyCode::KeyA) || keyboard.pressed(KeyCode::ArrowLeft) {
        movement.x -= 1.0;
    }
    if keyboard.pressed(KeyCode::KeyD) || keyboard.pressed(KeyCode::ArrowRight) {
        movement.x += 1.0;
    }
    
    // Apply movement
    if movement.length() > 0.0 {
        movement = movement.normalize() * camera_speed * time.delta_secs();
        camera_transform.translation += movement.extend(0.0);
        
        // Clear follow target when manually moving
        iso_camera.follow_target = None;
    }
    
    // Mouse wheel zoom
    for wheel_event in mouse_wheel.read() {
        let zoom_delta = wheel_event.y * 0.1;
        iso_camera.zoom = (iso_camera.zoom + zoom_delta).clamp(iso_camera.min_zoom, iso_camera.max_zoom);
    }
    
    // Follow selected agent
    if keyboard.just_pressed(KeyCode::KeyC) {
        if let Some(&selected_entity) = selection.selected.first() {
            iso_camera.follow_target = Some(selected_entity);
        }
    }
    
    // Handle follow target
    if let Some(target_entity) = iso_camera.follow_target {
        if let Ok(target_transform) = agent_query.get(target_entity) {
            let target_pos = target_transform.translation.truncate();
            let current_pos = camera_transform.translation.truncate();
            let lerp_factor = iso_camera.follow_smoothing * time.delta_secs();
            
            let new_pos = current_pos.lerp(target_pos, lerp_factor);
            camera_transform.translation = new_pos.extend(camera_transform.translation.z);
        } else {
            // Target no longer exists
            iso_camera.follow_target = None;
        }
    }
    
    // Apply camera bounds
    if let Some(bounds) = &iso_camera.bounds {
        camera_transform.translation.x = camera_transform.translation.x.clamp(bounds.min_x, bounds.max_x);
        camera_transform.translation.y = camera_transform.translation.y.clamp(bounds.min_y, bounds.max_y);
    }
    
    // Apply zoom to camera scale (for 2D camera)
    camera_transform.scale = Vec3::splat(1.0 / iso_camera.zoom);
}

// === MOUSE COORDINATE CONVERSION ===
pub fn get_isometric_mouse_position(
    windows: &Query<&Window>,
    cameras: &Query<(&Camera, &GlobalTransform, &IsometricCamera)>,
) -> Option<Vec2> {
    let window = windows.single().ok()?;
    let (camera, camera_transform, _) = cameras.single().ok()?;
    let cursor_pos = window.cursor_position()?;
    
    // Convert cursor position to world coordinates
    camera.viewport_to_world_2d(camera_transform, cursor_pos).ok()
}

// === CAMERA BOUNDS SYSTEM ===
pub fn update_camera_bounds(
    mut camera_query: Query<&mut IsometricCamera>,
    tilemap_settings: Option<Res<crate::systems::tilemap::IsometricSettings>>,
) {
    if let Some(settings) = tilemap_settings {
        if settings.is_changed() {
            for mut iso_camera in camera_query.iter_mut() {
                // Update bounds based on tilemap size
                let world_width = settings.map_width as f32 * settings.tile_width * 0.5;
                let world_height = settings.map_height as f32 * settings.tile_height * 0.5;
                
                iso_camera.bounds = Some(CameraBounds {
                    min_x: -world_width,
                    max_x: world_width,
                    min_y: -world_height,
                    max_y: world_height,
                });
            }
        }
    }
}

// === AGENT FOLLOWING SYSTEM ===
pub fn camera_follow_selected_agent(
    mut camera_query: Query<&mut IsometricCamera>,
    agent_query: Query<Entity, With<Agent>>,
    selection: Res<SelectionState>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    // Auto-follow first selected agent
    if selection.is_changed() && !selection.selected.is_empty() {
        let selected_entity = selection.selected[0];
        
        // Verify it's still a valid agent
        if agent_query.get(selected_entity).is_ok() {
            for mut iso_camera in camera_query.iter_mut() {
                iso_camera.follow_target = Some(selected_entity);
            }
        }
    }
    
    // Stop following with ESC
    if keyboard.just_pressed(KeyCode::Escape) {
        for mut iso_camera in camera_query.iter_mut() {
            iso_camera.follow_target = None;
        }
    }
}

// === CAMERA SHAKE SYSTEM ===
#[derive(Component)]
pub struct CameraShake {
    pub intensity: f32,
    pub duration: f32,
    pub decay: f32,
}

pub fn camera_shake_system(
    mut camera_query: Query<(Entity, &mut Transform, &mut CameraShake), With<IsometricCamera>>,
    time: Res<Time>,
    mut commands: Commands,
) {
    for (entity, mut transform, mut shake) in camera_query.iter_mut() {
        if shake.duration <= 0.0 {
            continue;
        }
        
        // Apply random offset
        let offset = Vec2::new(
            (fastrand::f32() - 0.5) * shake.intensity,
            (fastrand::f32() - 0.5) * shake.intensity,
        );
        
        transform.translation += offset.extend(0.0);
        
        // Decay shake
        shake.duration -= time.delta_secs();
        shake.intensity *= shake.decay;
        
        // Remove shake component when done
        if shake.duration <= 0.0 {
            commands.entity(entity).remove::<CameraShake>();
        }
    }
}

// === UTILITY FUNCTIONS ===
pub fn add_camera_shake(
    commands: &mut Commands,
    camera_entity: Entity,
    intensity: f32,
    duration: f32,
) {
    commands.entity(camera_entity).insert(CameraShake {
        intensity,
        duration,
        decay: 0.95, // Gradual decay
    });
}

pub fn center_camera_on_position(
    camera_query: &mut Query<(&mut Transform, &mut IsometricCamera), With<Camera2d>>,
    position: Vec2,
) {
    for (mut transform, mut iso_camera) in camera_query.iter_mut() {
        transform.translation = position.extend(transform.translation.z);
        iso_camera.follow_target = None; // Stop following when manually centered
    }
}

// === ZOOM LEVELS ===
#[derive(Resource)]
pub struct CameraZoomLevels {
    pub tactical: f32,    // Close-up for detailed actions
    pub normal: f32,      // Standard gameplay view
    pub strategic: f32,   // Wide view for planning
}

impl Default for CameraZoomLevels {
    fn default() -> Self {
        Self {
            tactical: 1.5,
            normal: 1.0,
            strategic: 0.5,
        }
    }
}

pub fn camera_zoom_presets(
    mut camera_query: Query<&mut IsometricCamera>,
    keyboard: Res<ButtonInput<KeyCode>>,
    zoom_levels: Res<CameraZoomLevels>,
) {
    let target_zoom = if keyboard.just_pressed(KeyCode::Digit1) {
        Some(zoom_levels.tactical)
    } else if keyboard.just_pressed(KeyCode::Digit2) {
        Some(zoom_levels.normal)
    } else if keyboard.just_pressed(KeyCode::Digit3) {
        Some(zoom_levels.strategic)
    } else {
        None
    };
    
    if let Some(zoom) = target_zoom {
        for mut iso_camera in camera_query.iter_mut() {
            iso_camera.zoom = zoom;
        }
    }
}

// === EDGE SCROLLING ===
pub fn camera_edge_scrolling(
    windows: Query<&Window>,
    mut camera_query: Query<(&mut Transform, &IsometricCamera), With<Camera2d>>,
    time: Res<Time>,
    game_mode: Res<GameMode>,
) {
    if game_mode.paused { return; }
    
    let Ok(window) = windows.single() else { return; };
    let Some(cursor_pos) = window.cursor_position() else { return; };
    
    let edge_threshold = 50.0; // Pixels from edge
    let scroll_speed = 300.0;
    
    let mut scroll_direction = Vec2::ZERO;
    
    // Check edges
    if cursor_pos.x < edge_threshold {
        scroll_direction.x -= 1.0;
    } else if cursor_pos.x > window.width() - edge_threshold {
        scroll_direction.x += 1.0;
    }
    
    if cursor_pos.y < edge_threshold {
        scroll_direction.y += 1.0; // Y is flipped in screen coordinates
    } else if cursor_pos.y > window.height() - edge_threshold {
        scroll_direction.y -= 1.0;
    }
    
    if scroll_direction.length() > 0.0 {
        scroll_direction = scroll_direction.normalize();
        
        for (mut transform, iso_camera) in camera_query.iter_mut() {
            let movement = scroll_direction * scroll_speed * time.delta_secs() / iso_camera.zoom;
            transform.translation += movement.extend(0.0);
            
            // Apply bounds if they exist
            if let Some(bounds) = &iso_camera.bounds {
                transform.translation.x = transform.translation.x.clamp(bounds.min_x, bounds.max_x);
                transform.translation.y = transform.translation.y.clamp(bounds.min_y, bounds.max_y);
            }
        }
    }
}