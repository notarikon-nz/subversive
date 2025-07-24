// src/systems/npc_barks.rs - NPC barks and chat bubbles
// 7687 chars -> 
// src/systems/npc_barks.rs - NPC barks and chat bubbles
use bevy::prelude::*;
use crate::core::*;

// === COMPONENTS ===
#[derive(Component)]
pub struct ChatBubble {
    pub lifetime: f32,
}

#[derive(Component)]
pub struct BarkCooldown(f32);

// === EVENTS ===
#[derive(Event)]
pub struct BarkEvent {
    pub entity: Entity,
    pub bark_type: BarkType,
}

#[derive(Clone, Copy, Debug)]
pub enum BarkType {
    // Combat
    SpottedEnemy,
    TakingFire,
    Hit,
    Dying,
    // Investigation  
    Investigating,
    // Status
    OnPatrol,
    CallForHelp,
    Retreating,
}

impl BarkType {
    pub fn get_text(&self) -> &'static str {
        match self {
            Self::SpottedEnemy => "Enemy spotted!",
            Self::TakingFire => "Taking fire!",
            Self::Investigating => "Checking it out...",
            Self::OnPatrol => "Sector secure.",
            Self::CallForHelp => "Need backup!",
            Self::Retreating => "Fall back!",
            Self::Hit => "Argh!",
            Self::Dying => "I'm hit!",
        }
    }
}

// === SYSTEMS ===
// Monitor GOAP state changes and trigger barks
pub fn goap_bark_system(
    mut bark_events: EventWriter<BarkEvent>,
    goap_query: Query<(Entity, &GoapAgent), (With<Enemy>, Without<Dead>, Without<BarkCooldown>)>,
    mut commands: Commands,
    mut last_goals: Local<std::collections::HashMap<Entity, String>>,
) {
    for (entity, goap_agent) in goap_query.iter() {
        if let Some(goal) = &goap_agent.current_goal {
            let entry = last_goals.entry(entity).or_default();
            
            // Skip if same goal
            if entry == &goal.name {
                continue;
            }
            
            // Update and get bark type
            *entry = goal.name.to_string();
            
            let bark = match goal.name.as_ref() {
                "eliminate_threat" => BarkType::SpottedEnemy,
                "investigate_disturbance" => BarkType::Investigating,
                "coordinate_defense" => BarkType::CallForHelp,
                "panic_survival" | "survival" => BarkType::Retreating,
                "patrol_area" => BarkType::OnPatrol,
                "taking_fire" => BarkType::TakingFire,
                _ => continue,
            };
            
            bark_events.write(BarkEvent { entity, bark_type: bark });
            commands.entity(entity).insert(BarkCooldown(3.0));
        } else {
            last_goals.remove(&entity);
        }
    }
}

// Handle combat barks  
pub fn combat_bark_system(
    mut bark_events: EventWriter<BarkEvent>,
    mut combat_events: EventReader<CombatEvent>,
    mut commands: Commands,
    enemy_query: Query<(), (With<Enemy>, Without<Dead>, Without<BarkCooldown>)>,
) {
    for event in combat_events.read() {
        if event.hit && enemy_query.contains(event.target) {
            bark_events.write(BarkEvent {
                entity: event.target,
                bark_type: if event.damage >= 50.0 { BarkType::Dying } else { BarkType::Hit },
            });
            commands.entity(event.target).insert(BarkCooldown(2.0));
        }
    }
}

// Spawn chat bubbles from bark events
pub fn bark_handler_system(
    mut bark_events: EventReader<BarkEvent>,
    mut commands: Commands,
    mut audio_events: EventWriter<AudioEvent>,
    entity_query: Query<&Transform>,
) {
    for event in bark_events.read() {
        if let Ok(transform) = entity_query.get(event.entity) {
            let text = event.bark_type.get_text();
            let pos = transform.translation.truncate() + Vec2::new(0.0, 40.0);
            
            // Single entity with both sprite and text
            commands.spawn((
                Sprite {
                    color: Color::srgba(0.0, 0.0, 0.0, 0.8),
                    custom_size: Some(Vec2::new(text.len() as f32 * 8.0 + 16.0, 24.0)),
                    ..default()
                },
                Transform::from_translation(pos.extend(50.0)),
                ChatBubble { lifetime: 2.0 },
            )).with_children(|parent| {
                parent.spawn((
                    Text2d::new(text),
                    TextFont { font_size: 12.0, ..default() },
                    TextColor(Color::WHITE),
                    Transform::from_xyz(0.0, 0.0, 0.1),
                ));
            });
            
            // Play audio based on bark type
            let audio_type = match event.bark_type {
                BarkType::SpottedEnemy | BarkType::CallForHelp => AudioType::Alert,
                BarkType::TakingFire | BarkType::Hit | BarkType::Dying => AudioType::Gunshot,
                _ => AudioType::Footstep,
            };
            
            audio_events.write(AudioEvent {
                sound: audio_type,
                volume: 0.3,
            });
        }
    }
}

// Update bubbles and cooldowns in one system
pub fn update_bubble_system(
    mut commands: Commands,
    mut bubble_query: Query<(Entity, &mut ChatBubble, &mut Transform, &Children)>,
    mut sprite_query: Query<&mut Sprite>,
    mut text_query: Query<&mut TextColor>,
    mut cooldown_query: Query<(Entity, &mut BarkCooldown)>,
    time: Res<Time>,
    game_mode: Res<GameMode>,
) {
    if game_mode.paused { return; }
    
    let dt = time.delta_secs();
    
    // Update bubbles
    for (entity, mut bubble, mut transform, children) in bubble_query.iter_mut() {
        bubble.lifetime -= dt;
        
        if bubble.lifetime <= 0.0 {
            commands.entity(entity).despawn();
        } else {
            transform.translation.y += 20.0 * dt;
            
            // Fade if under 0.5s
            if bubble.lifetime < 0.5 {
                let alpha = bubble.lifetime * 2.0;
                
                if let Ok(mut sprite) = sprite_query.get_mut(entity) {
                    sprite.color.set_alpha(alpha * 0.8);
                }
                
                for &child in children {
                    if let Ok(mut text) = text_query.get_mut(child) {
                        text.0.set_alpha(alpha);
                    }
                }
            }
        }
    }
    
    // Update cooldowns
    for (entity, mut cooldown) in cooldown_query.iter_mut() {
        cooldown.0 -= dt;
        if cooldown.0 <= 0.0 {
            commands.entity(entity).remove::<BarkCooldown>();
        }
    }
}