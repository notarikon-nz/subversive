// src/systems/cursor_enhancements.rs
use bevy::prelude::*;
use crate::core::*;
use crate::systems::cursor::*;
use crate::systems::cursor::{determine_cursor_type};
use crate::systems::input::{get_weapon_range_simple};

#[derive(Resource)]
pub struct CursorSettings {
    pub show_range_indicator: bool,
    pub cursor_scale: f32,
    pub animation_speed: f32,
    pub sound_enabled: bool,
}

impl Default for CursorSettings {
    fn default() -> Self {
        Self {
            show_range_indicator: true,
            cursor_scale: 1.0,
            animation_speed: 2.0,
            sound_enabled: true,
        }
    }
}

#[derive(Component)]
pub struct CursorAnimation {
    pub start_time: f32,
    pub duration: f32,
    pub animation_type: CursorAnimationType,
}

#[derive(Component)]
pub struct RangeIndicator {
    pub range: f32,
    pub fade_timer: f32,
}

pub enum CursorAnimationType {
    FadeIn,
    Pulse,
    Spin,
}

// Enhanced cursor system with animations and range indicators
// Core cursor detection system
pub fn cursor_detection_system(
    mut last_state: ResMut<LastCursorState>,
    windows: Query<&Window>,
    cameras: Query<(&Camera, &GlobalTransform)>,
    selection: Res<SelectionState>,
    game_mode: Res<GameMode>,
    
    enemy_query: Query<(Entity, &Transform, &Health), (With<Enemy>, Without<MarkedForDespawn>)>,
    vehicle_query: Query<(Entity, &Transform, &Health), (With<Vehicle>, Without<MarkedForDespawn>)>,
    terminal_query: Query<(Entity, &Transform, &Terminal, Option<&LoreSource>)>,
    hackable_query: Query<(Entity, &Transform, &Hackable, &DeviceState)>,
    agent_query: Query<(&Transform, &Inventory), With<Agent>>,
) {
    if game_mode.paused { return; }

    let mouse_pos = if let Some(pos) = get_world_mouse_position(&windows, &cameras) {
        pos
    } else {
        return;
    };

    let cursor_type = determine_cursor_type(
        mouse_pos,
        &selection,
        &game_mode,
        &enemy_query,
        &vehicle_query,
        &terminal_query,
        &hackable_query,
        &agent_query,
    );

    last_state.cursor_type = cursor_type;
    last_state.position = mouse_pos;
}

// Cursor sprite update system
pub fn cursor_sprite_system(
    mut commands: Commands,
    cursor_sprites: Res<CursorSprites>,
    cursor_settings: Res<CursorSettings>,
    last_state: Res<LastCursorState>,
    time: Res<Time>,
    mut cursor_query: Query<(Entity, &mut Transform, Option<&mut CursorAnimation>), (With<CursorEntity>, Without<Camera>)>,
) {
    update_cursor_with_animation(
        &mut commands,
        &cursor_sprites,
        &cursor_settings,
        last_state.cursor_type,
        last_state.position,
        time.elapsed_secs(),
        cursor_query,
    );
}

// Audio feedback system
pub fn cursor_audio_system(
    last_state: Res<LastCursorState>,
    cursor_settings: Res<CursorSettings>,
    mut audio_events: EventWriter<AudioEvent>,
    mut local_last_type: Local<CursorType>,
) {
    if *local_last_type != last_state.cursor_type && cursor_settings.sound_enabled {
        let sound_type = match last_state.cursor_type {
            CursorType::Crosshair => Some(AudioType::CursorTarget),
            CursorType::Hand => Some(AudioType::CursorInteract),
            CursorType::Hacker => Some(AudioType::CursorHack),
            _ => None,
        };
        
        if let Some(sound) = sound_type {
            audio_events.write(AudioEvent {
                sound,
                volume: 0.3,
            });
        }
        
        *local_last_type = last_state.cursor_type;
    }
}

fn update_cursor_with_animation(
    commands: &mut Commands,
    cursor_sprites: &CursorSprites,
    settings: &CursorSettings,
    cursor_type: CursorType,
    position: Vec2,
    current_time: f32,
    mut cursor_query: Query<(Entity, &mut Transform, Option<&mut CursorAnimation>), (With<CursorEntity>, Without<Camera>)>,
) {
    let sprite_handle = get_cursor_sprite(cursor_sprites, cursor_type);
    
    if let Ok((entity, mut transform, mut animation)) = cursor_query.single_mut() {
        // Update position
        transform.translation = position.extend(1000.0);
        
        // Apply animation
        if let Some(ref mut anim) = animation {
            apply_cursor_animation(&mut transform, anim, current_time, settings);
        } else {
            // Add default animation for certain cursor types
            let animation_type = match cursor_type {
                CursorType::Crosshair => Some(CursorAnimationType::Pulse),
                CursorType::Hacker => Some(CursorAnimationType::Spin),
                _ => None,
            };
            
            if let Some(anim_type) = animation_type {
                commands.entity(entity).insert(CursorAnimation {
                    start_time: current_time,
                    duration: 2.0,
                    animation_type: anim_type,
                });
            }
        }
        
        // Update sprite
        commands.entity(entity).insert(Sprite {
            image: sprite_handle.clone(),
            ..default()
        });
    } else {
        // Spawn new cursor with fade-in animation
        let cursor_entity = commands.spawn((
            CursorEntity,
            Sprite {
                image: sprite_handle.clone(),
                ..default()
            },
            Transform::from_translation(position.extend(1000.0)),
            GlobalTransform::default(),
            CursorAnimation {
                start_time: current_time,
                duration: 0.3,
                animation_type: CursorAnimationType::FadeIn,
            },
        )).id();
    }
}

fn apply_cursor_animation(
    transform: &mut Transform,
    animation: &mut CursorAnimation,
    current_time: f32,
    settings: &CursorSettings,
) {
    let progress = ((current_time - animation.start_time) / animation.duration).clamp(0.0, 1.0);
    
    match animation.animation_type {
        CursorAnimationType::FadeIn => {
            let alpha = progress;
            transform.scale = Vec3::splat(settings.cursor_scale * alpha);
        },
        CursorAnimationType::Pulse => {
            let pulse = (current_time * settings.animation_speed).sin() * 0.1 + 1.0;
            transform.scale = Vec3::splat(settings.cursor_scale * pulse);
        },
        CursorAnimationType::Spin => {
            transform.rotation = Quat::from_rotation_z(current_time * settings.animation_speed);
            transform.scale = Vec3::splat(settings.cursor_scale);
        },
    }
}

fn update_range_indicator(
    commands: &mut Commands,
    position: Vec2,
    cursor_type: CursorType,
    selection: &SelectionState,
    agent_query: &Query<(&Transform, &Inventory), With<Agent>>,
    settings: &CursorSettings,
    existing_indicators: &Query<Entity, (With<RangeIndicator>,Without<MarkedForDespawn>)>,
) {
    // Clean up existing indicators
    for entity in existing_indicators.iter() {
        commands.entity(entity).insert(MarkedForDespawn);
    }
    
    if !settings.show_range_indicator || cursor_type != CursorType::Crosshair {
        return;
    }
    
    // Show range indicator for attack cursor
    if let Some(&agent) = selection.selected.first() {
        if let Ok((agent_transform, inventory)) = agent_query.get(agent) {
            let agent_pos = agent_transform.translation.truncate();
            let range = get_weapon_range_simple(inventory);
            
            // Spawn range circle
            commands.spawn((
                RangeIndicator { range, fade_timer: 2.0 },
                Sprite {
                    color: Color::srgba(1.0, 0.0, 0.0, 0.2),
                    custom_size: Some(Vec2::new(range * 2.0, range * 2.0)),
                    ..default()
                },
                Transform::from_translation(agent_pos.extend(50.0)),
                GlobalTransform::default(),
            ));
        }
    }
}

fn get_cursor_sprite(sprites: &CursorSprites, cursor_type: CursorType) -> Handle<Image> {
    match cursor_type {
        CursorType::Arrow => sprites.arrow.clone(),
        CursorType::Crosshair => sprites.crosshair.clone(),
        CursorType::Hand => sprites.hand.clone(),
        CursorType::Hacker => sprites.hacker.clone(),
        CursorType::Examine => sprites.examine.clone(),
        CursorType::Move => sprites.move_cursor.clone(),
    }
}

// Weapon-specific cursors
pub fn weapon_specific_cursor_system(
    selection: Res<SelectionState>,
    agent_query: Query<&Inventory, With<Agent>>,
    mut last_state: ResMut<LastCursorState>,
    mut cursor_query: Query<&mut Sprite, With<CursorEntity>>,
) {
    if let Some(&agent) = selection.selected.first() {
        if let Ok(inventory) = agent_query.get(agent) {
            if let Some(weapon) = &inventory.equipped_weapon {
                let tint_color = match weapon.base_weapon {
                    WeaponType::Pistol => Color::srgb(0.8, 0.8, 1.0),    // Light blue
                    WeaponType::Rifle => Color::srgb(1.0, 0.9, 0.7),     // Light orange
                    WeaponType::Shotgun => Color::srgb(1.0, 0.7, 0.7),   // Light red
                    _ => Color::WHITE,
                };
                
                if let Ok(mut sprite) = cursor_query.single_mut() {
                    sprite.color = tint_color;
                }
            }
        }
    }
}

// Range indicator fade system
pub fn range_indicator_system(
    mut commands: Commands,
    time: Res<Time>,
    mut indicator_query: Query<(Entity, &mut RangeIndicator, &mut Sprite), Without<MarkedForDespawn>>,
) {
    for (entity, mut indicator, mut sprite) in indicator_query.iter_mut() {
        indicator.fade_timer -= time.delta_secs();
        
        if indicator.fade_timer <= 0.0 {
            commands.entity(entity).insert(MarkedForDespawn);
        } else {
            // Fade out over time
            let alpha = (indicator.fade_timer / 2.0).clamp(0.0, 0.3);
            sprite.color.set_alpha(alpha);
        }
    }
}

