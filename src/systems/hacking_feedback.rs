// src/systems/hacking_feedback.rs - Visual feedback for hacking process
use bevy::prelude::*;
use crate::core::*;
use std::time::{SystemTime};

// === HACKING PROGRESS COMPONENT ===
#[derive(Component)]
pub struct HackingProgress {
    pub hacker: Entity,
    pub target: Entity,
    pub progress: f32, // 0.0 to 1.0
    pub total_time: f32,
    pub device_type: DeviceType,
}

// === HACK STATUS INDICATORS ===
#[derive(Component)]
pub struct HackStatusIndicator {
    pub target_device: Entity,
}

// === ENHANCED HACKING SYSTEM ===
pub fn enhanced_hacking_system(
    mut commands: Commands,
    mut hack_attempts: EventReader<HackAttemptEvent>,
    mut hack_completed: EventWriter<HackCompletedEvent>,
    mut hackable_query: Query<(Entity, &mut Hackable, &mut DeviceState, &Transform)>,
    mut hacking_progress_query: Query<(Entity, &mut HackingProgress)>,
    agent_inventory: Query<&Inventory, With<Agent>>,
    mut audio_events: EventWriter<AudioEvent>,
    time: Res<Time>,
) {
    // Start new hack attempts
    for event in hack_attempts.read() {
        if let Ok((target_entity, hackable, device_state, transform)) = hackable_query.get(event.target) {
            // Check if agent has required tool
            if let Ok(inventory) = agent_inventory.get(event.agent) {
                let has_tool = check_hack_tool_available(inventory, &hackable);
                
                if !has_tool {
                    info!("Hack failed: Missing required tool");
                    continue;
                }
            }
            
            if hackable.is_hacked {
                info!("Device already hacked");
                continue;
            }
            
            // Start hacking progress
            commands.spawn(HackingProgress {
                hacker: event.agent,
                target: event.target,
                progress: 0.0,
                total_time: hackable.hack_time,
                device_type: hackable.device_type.clone(),
            });
            
            // Spawn visual indicator
            spawn_hack_status_indicator(&mut commands, transform.translation.truncate(), event.target);
            
            // Play hack start sound
            audio_events.write(AudioEvent {
                sound: AudioType::TerminalAccess,
                volume: 0.3,
            });
            
            info!("Started hacking {:?} (Security: {}, Time: {:.1}s)", 
                  hackable.device_type, hackable.security_level, hackable.hack_time);
        }
    }
    
    // Update hacking progress
    let mut completed_hacks = Vec::new();
    
    for (progress_entity, mut progress) in hacking_progress_query.iter_mut() {
        progress.progress += time.delta_secs() / progress.total_time;
        
        if progress.progress >= 1.0 {
            // Hack completed!
            if let Ok((_, mut hackable, mut device_state, _)) = hackable_query.get_mut(progress.target) {
                hackable.is_hacked = true;
                device_state.operational = false;
                device_state.hack_timer = hackable.disabled_duration;
                
                // Apply hack effects
                for effect in &hackable.hack_effects {
                    apply_hack_effect(effect, &mut device_state, &hackable.device_type);
                }
                
                // Send completion event
                hack_completed.write(HackCompletedEvent {
                    agent: progress.hacker,
                    target: progress.target,
                    device_type: progress.device_type.clone(),
                    effects: hackable.hack_effects.clone(),
                });
                
                // Play completion sound
                audio_events.write(AudioEvent {
                    sound: AudioType::Neurovector,
                    volume: 0.6,
                });
                
                info!("Successfully hacked {:?}!", progress.device_type);
            }
            
            completed_hacks.push(progress_entity);
        }
    }
    
    // Clean up completed hacks
    for entity in completed_hacks {
        commands.entity(entity).despawn();
    }
}

// === VISUAL FEEDBACK SYSTEMS ===
pub fn hack_progress_visualization(
    mut gizmos: Gizmos,
    progress_query: Query<&HackingProgress>,
    hackable_query: Query<&Transform, With<Hackable>>,
) {
    for progress in progress_query.iter() {
        if let Ok(transform) = hackable_query.get(progress.target) {
            let pos = transform.translation.truncate();
            
            // Draw progress circle
            let radius = 25.0;
            let progress_angle = progress.progress * std::f32::consts::TAU;
            
            // Background circle
            gizmos.circle_2d(pos, radius, Color::srgba(0.2, 0.2, 0.2, 0.5));
            
            // Progress arc (approximated with line segments)
            let segments = 32;
            let segment_angle = progress_angle / segments as f32;
            
            for i in 0..=(progress_angle * segments as f32 / std::f32::consts::TAU) as i32 {
                let angle = i as f32 * segment_angle;
                let start_pos = pos + Vec2::new(angle.cos(), angle.sin()) * (radius - 2.0);
                let end_pos = pos + Vec2::new(angle.cos(), angle.sin()) * (radius + 2.0);
                gizmos.line_2d(start_pos, end_pos, Color::srgb(0.2, 0.8, 0.8));
            }
            
            // Center dot
            gizmos.circle_2d(pos, 3.0, Color::srgb(0.8, 0.8, 0.2));
            
            // Progress text (simplified as colored indicator)
            let text_color = match progress.progress {
                p if p < 0.3 => Color::srgb(0.8, 0.2, 0.2),
                p if p < 0.7 => Color::srgb(0.8, 0.8, 0.2),
                _ => Color::srgb(0.2, 0.8, 0.2),
            };
            gizmos.circle_2d(pos + Vec2::new(0.0, -35.0), 4.0, text_color);
        }
    }
}

pub fn hack_status_indicator_system(
    mut commands: Commands,
    mut indicator_query: Query<(Entity, &HackStatusIndicator, &mut Transform)>,
    hackable_query: Query<(&Hackable, &DeviceState), With<Hackable>>,
    time: Res<Time>,
) {
    for (entity, indicator, mut transform) in indicator_query.iter_mut() {
        if let Ok((hackable, device_state)) = hackable_query.get(indicator.target_device) {
            if hackable.is_hacked {
                // Show hacked status
                transform.translation.y += 20.0 * time.delta_secs(); // Float upward
                
                // Remove indicator after a while
                if transform.translation.y > 100.0 {
                    commands.entity(entity).despawn();
                }
            } else {
                // Device no longer exists or hack failed
                commands.entity(entity).despawn();
            }
        } else {
            commands.entity(entity).despawn();
        }
    }
}

pub fn device_visual_feedback_system(
    mut hackable_query: Query<(&Hackable, &DeviceState, &mut Sprite), (With<Hackable>, Changed<DeviceState>)>,
) {
    for (hackable, device_state, mut sprite) in hackable_query.iter_mut() {
        // Update device appearance based on state
        sprite.color = if !device_state.powered {
            Color::srgb(0.2, 0.2, 0.2) // Dark = no power
        } else if hackable.is_hacked {
            // Hacked devices flash red
            let pulse = (SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs_f32() * 3.0).sin() * 0.5 + 0.5;
            Color::srgb(0.8, 0.2 * pulse, 0.2 * pulse)
        } else if !device_state.operational {
            Color::srgb(0.6, 0.6, 0.2) // Yellow = offline but powered
        } else {
            // Original color based on device type
            match hackable.device_type {
                DeviceType::Camera => Color::srgb(0.3, 0.3, 0.3),
                DeviceType::Turret => Color::srgb(0.6, 0.2, 0.2),
                DeviceType::Door => Color::srgb(0.4, 0.4, 0.6),
                DeviceType::StreetLight => Color::srgb(0.9, 0.9, 0.7),
                DeviceType::TrafficLight => Color::srgb(0.2, 0.8, 0.2),
                DeviceType::PowerStation => Color::srgb(0.8, 0.8, 0.2),
                _ => Color::WHITE,
            }
        };
    }
}

// === HACK INTERRUPTION SYSTEM ===
pub fn hack_interruption_system(
    mut commands: Commands,
    progress_query: Query<(Entity, &HackingProgress)>,
    agent_query: Query<&Transform, With<Agent>>,
    hackable_query: Query<&Transform, With<Hackable>>,
) {
    for (progress_entity, progress) in progress_query.iter() {
        // Check if hacker moved away from target
        if let (Ok(agent_transform), Ok(target_transform)) = (
            agent_query.get(progress.hacker),
            hackable_query.get(progress.target)
        ) {
            let distance = agent_transform.translation.truncate()
                .distance(target_transform.translation.truncate());
            
            if distance > 50.0 { // Max hack range
                commands.entity(progress_entity).despawn();
                info!("Hack interrupted: Too far from target");
            }
        }
    }
}

// === HELPER FUNCTIONS ===
fn spawn_hack_status_indicator(commands: &mut Commands, position: Vec2, target: Entity) {
    commands.spawn((
        Sprite {
            color: Color::srgba(0.2, 0.8, 0.8, 0.8),
            custom_size: Some(Vec2::new(8.0, 8.0)),
            ..default()
        },
        Transform::from_translation((position + Vec2::new(0.0, 30.0)).extend(10.0)),
        HackStatusIndicator { target_device: target },
    ));
}

fn check_hack_tool_available(inventory: &Inventory, hackable: &Hackable) -> bool {
    match &hackable.requires_tool {
        Some(required_tool) => {
            inventory.equipped_tools.iter().any(|tool| {
                matches!((tool, required_tool), 
                    (ToolType::Hacker, HackTool::BasicHacker) |
                    (ToolType::Hacker, HackTool::AdvancedHacker)
                )
            })
        },
        None => true,
    }
}

fn apply_hack_effect(effect: &HackEffect, device_state: &mut DeviceState, device_type: &DeviceType) {
    match effect {
        HackEffect::Disable => {
            device_state.operational = false;
        },
        HackEffect::TakeControl => {
            device_state.current_function = match device_type {
                DeviceType::Turret => DeviceFunction::Defense,
                DeviceType::Drone => DeviceFunction::Surveillance,
                DeviceType::Vehicle => DeviceFunction::Transport,
                _ => device_state.original_function.clone(),
            };
        },
        HackEffect::PowerCut => {
            device_state.powered = false;
        },
        _ => {}
    }
}

// === SUCCESS NOTIFICATION SYSTEM ===
#[derive(Component)]
pub struct HackNotification {
    pub lifetime: f32,
}

pub fn hack_notification_system(
    mut commands: Commands,
    mut hack_completed: EventReader<HackCompletedEvent>,
    mut notifications: Query<(Entity, &mut HackNotification, &mut Transform)>,
    time: Res<Time>,
) {
    // Spawn notifications for completed hacks
    for event in hack_completed.read() {
        let device_name = match event.device_type {
            DeviceType::Camera => "Camera",
            DeviceType::Turret => "Turret", 
            DeviceType::Door => "Door",
            DeviceType::StreetLight => "Street Light",
            DeviceType::TrafficLight => "Traffic Light",
            DeviceType::PowerStation => "Power Station",
            _ => "Device",
        };
        
        commands.spawn((
            Text::new(format!("🔓 Hacked: {}", device_name)),
            TextFont { font_size: 14.0, ..default() },
            TextColor(Color::srgb(0.2, 0.8, 0.8)),
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(150.0),
                right: Val::Px(20.0),
                padding: UiRect::all(Val::Px(8.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.8)),
            ZIndex(1000),
            HackNotification { lifetime: 2.5 },
        ));
    }
    
    // Update and cleanup notifications
    for (entity, mut notification, mut transform) in notifications.iter_mut() {
        notification.lifetime -= time.delta_secs();
        
        if notification.lifetime <= 0.0 {
            commands.entity(entity).despawn();
        } else {
            // Fade out effect
            transform.translation.y += 30.0 * time.delta_secs();
        }
    }
}
