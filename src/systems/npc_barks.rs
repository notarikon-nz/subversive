// src/systems/npc_barks.rs - NPC barks and chat bubbles
use bevy::prelude::*;
use crate::core::*;

// === COMPONENTS ===
#[derive(Component)]
pub struct ChatBubble {
    pub lifetime: f32,
    pub fade_time: f32,
}

#[derive(Component)]
pub struct BarkCooldown {
    pub timer: f32,
}

// === EVENTS ===
#[derive(Event)]
pub struct BarkEvent {
    pub entity: Entity,
    pub bark_type: BarkType,
}

#[derive(Clone, Debug)]
pub enum BarkType {
    // Combat
    SpottedEnemy,
    TakingFire,
    Reloading,
    Grenade,
    
    // Investigation
    HeardSomething,
    Investigating,
    NothingHere,
    
    // Status
    OnPatrol,
    CallForHelp,
    Retreating,
    
    // Pain/Death
    Hit,
    Dying,
}

// === BARK CONTENT ===
impl BarkType {
    pub fn get_text(&self) -> &'static str {
        match self {
            BarkType::SpottedEnemy => "Enemy spotted!",
            BarkType::TakingFire => "Taking fire!",
            BarkType::Reloading => "Reloading!",
            BarkType::Grenade => "Grenade out!",
            BarkType::HeardSomething => "What was that?",
            BarkType::Investigating => "Checking it out...",
            BarkType::NothingHere => "All clear.",
            BarkType::OnPatrol => "Sector secure.",
            BarkType::CallForHelp => "Need backup!",
            BarkType::Retreating => "Fall back!",
            BarkType::Hit => "Argh!",
            BarkType::Dying => "I'm hit!",
        }
    }
    
    pub fn get_audio_file(&self) -> &'static str {
        match self {
            BarkType::SpottedEnemy => "audio/bark_spotted.ogg",
            BarkType::TakingFire => "audio/bark_fire.ogg",
            BarkType::Reloading => "audio/bark_reload.ogg",
            BarkType::Grenade => "audio/bark_grenade.ogg",
            BarkType::HeardSomething => "audio/bark_heard.ogg",
            BarkType::Investigating => "audio/bark_investigate.ogg",
            BarkType::NothingHere => "audio/bark_clear.ogg",
            BarkType::OnPatrol => "audio/bark_patrol.ogg",
            BarkType::CallForHelp => "audio/bark_help.ogg",
            BarkType::Retreating => "audio/bark_retreat.ogg",
            BarkType::Hit => "audio/bark_hit.ogg",
            BarkType::Dying => "audio/bark_dying.ogg",
        }
    }
}

// === SYSTEMS ===
// Monitor GOAP state changes and trigger barks
pub fn goap_bark_system(
    mut bark_events: EventWriter<BarkEvent>,
    goap_query: Query<(Entity, &mut GoapAgent, Option<&mut BarkCooldown>), (With<Enemy>, Without<Dead>)>,
    mut commands: Commands,
    last_goals: Local<std::collections::HashMap<Entity, String>>,
) {
    for (entity, goap_agent, bark_cooldown) in goap_query.iter() {

        // Skip if on cooldown
        if let Some(cooldown) = &bark_cooldown {
            if cooldown.timer > 0.0 {
                continue;
            }
        }

        // Check for goal changes (new plans indicate state changes)
        // if !goap_agent.current_plan.is_empty() {
        if let Some(current_goal) = &goap_agent.current_goal {
            let goal_name = current_goal.name;

            // info!("Entity {} has plan: {}", entity.index(), goal_name);

            // Check if goal changed
            let goal_changed = match last_goals.get(&entity) {
                Some(last_goal) => last_goal != &goal_name,
                None => true, // First time seeing this entity
            };

            let action_name = goal_name.as_ref();
            
            let bark_type = match action_name {
                "eliminate_threat" => Some(BarkType::SpottedEnemy),
                "investigate_disturbance" => Some(BarkType::Investigating),
                "coordinate_defense" => Some(BarkType::CallForHelp),
                "panic_survival" => Some(BarkType::Retreating),
                "patrol_area" => Some(BarkType::OnPatrol),
                "taking_fire" => Some(BarkType::TakingFire),
                "survival" => Some(BarkType::Retreating),

                _ => {
                        info!("Unknown goal: {}", goal_name);
                        None
                    }
            };
            
            if let Some(bark) = bark_type {
                bark_events.write(BarkEvent { entity, bark_type: bark });
                commands.entity(entity).insert(BarkCooldown { timer: 3.0 });
            }
        }
    }
}

// Handle combat-specific barks
pub fn combat_bark_system(
    mut bark_events: EventWriter<BarkEvent>,
    mut combat_events: EventReader<CombatEvent>,
    commands: Commands,
    enemy_query: Query<Entity, (With<Enemy>, Without<Dead>)>,
) {
    for event in combat_events.read() {
        // Bark when taking damage
        if event.hit && enemy_query.contains(event.target) {
            bark_events.write(BarkEvent {
                entity: event.target,
                bark_type: if event.damage >= 50.0 { BarkType::Dying } else { BarkType::Hit },
            });
        }
        
        // Bark when attacking
        if enemy_query.contains(event.attacker) {
            bark_events.write(BarkEvent {
                entity: event.attacker,
                bark_type: BarkType::TakingFire,
            });
        }
    }
}

// Handle bark events - MINIMAL VERSION FOR TESTING
pub fn bark_handler_system(
    mut bark_events: EventReader<BarkEvent>,
    mut commands: Commands,
    mut audio_events: EventWriter<AudioEvent>,
    entity_query: Query<&Transform>,
) {
    for event in bark_events.read() {
        if let Ok(transform) = entity_query.get(event.entity) {
            // Spawn simple world-space chat bubble
            spawn_simple_chat_bubble(&mut commands, transform.translation.truncate(), &event.bark_type);
            
            // Play audio
            let audio_type = match event.bark_type {
                BarkType::SpottedEnemy | BarkType::CallForHelp => AudioType::Alert,
                BarkType::TakingFire | BarkType::Hit | BarkType::Dying => AudioType::Gunshot,
                BarkType::Reloading => AudioType::Reload,
                _ => AudioType::Footstep,
            };
            
            audio_events.write(AudioEvent {
                sound: audio_type,
                volume: 0.3,
            });
        }
    }
}

// Update chat bubbles - SIMPLIFIED
pub fn chat_bubble_system(
    mut bubble_query: Query<(Entity, &mut ChatBubble, &mut Transform)>,
    mut sprite_query: Query<&mut Sprite, With<ChatBubble>>,
    mut text_query: Query<&mut TextColor, With<ChatBubble>>,
    mut commands: Commands,
    time: Res<Time>,
    game_mode: Res<GameMode>,
) {
    if game_mode.paused { return; }

    for (entity, mut bubble, mut transform) in bubble_query.iter_mut() {
        bubble.lifetime -= time.delta_secs();
        
        if bubble.lifetime <= 0.0 {
            commands.entity(entity).despawn();
        } else {
            // Float upward slowly
            transform.translation.y += 20.0 * time.delta_secs();
            
            // Fade out
            if bubble.lifetime <= bubble.fade_time {
                let alpha = bubble.lifetime / bubble.fade_time;
                
                if let Ok(mut sprite) = sprite_query.get_mut(entity) {
                    sprite.color.set_alpha(alpha);
                }
                
                if let Ok(mut text_color) = text_query.get_mut(entity) {
                    text_color.0.set_alpha(alpha);
                }
            }
        }
    }
}

// Update bark cooldowns
pub fn bark_cooldown_system(
    mut cooldown_query: Query<(Entity, &mut BarkCooldown)>,
    mut commands: Commands,
    time: Res<Time>,
    game_mode: Res<GameMode>,
) {
    if game_mode.paused { return; }

    for (entity, mut cooldown) in cooldown_query.iter_mut() {
        cooldown.timer -= time.delta_secs();
        
        if cooldown.timer <= 0.0 {
            commands.entity(entity).remove::<BarkCooldown>();
        }
    }
}

// === HELPER FUNCTIONS ===
fn spawn_simple_chat_bubble(commands: &mut Commands, position: Vec2, bark_type: &BarkType) {
    let text = bark_type.get_text();
    let bubble_pos = position + Vec2::new(0.0, 40.0);
    
    // Background bubble (pure world-space sprite)
    commands.spawn((
        Sprite {
            color: Color::srgba(0.0, 0.0, 0.0, 0.8),
            custom_size: Some(Vec2::new(text.len() as f32 * 8.0 + 16.0, 24.0)),
            ..default()
        },
        Transform::from_translation(bubble_pos.extend(50.0)),
        ChatBubble {
            lifetime: 2.0,
            fade_time: 0.5,
        },
    ));
    
    // World-space text (separate entity)
    commands.spawn((
        Text2d::new(text),
        TextFont {
            font_size: 12.0,
            ..default()
        },
        TextColor(Color::WHITE),
        Transform::from_translation(bubble_pos.extend(51.0)),
        ChatBubble {
            lifetime: 2.0,
            fade_time: 0.5,
        },
    ));
}