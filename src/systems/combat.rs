use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use leafwing_input_manager::prelude::*;
use crate::core::*;

pub fn system(
    mut gizmos: Gizmos,
    input: Query<&ActionState<PlayerAction>>,
    mut action_events: EventReader<ActionEvent>,
    mut combat_events: EventWriter<CombatEvent>,
    mut audio_events: EventWriter<AudioEvent>,
    selection: Res<SelectionState>,
    agent_query: Query<&Transform, With<Agent>>,
    mut enemy_query: Query<(Entity, &Transform, &mut Health), (With<Enemy>, Without<Dead>)>,
    game_mode: Res<GameMode>,
    windows: Query<&Window>,
    cameras: Query<(&Camera, &GlobalTransform)>,
) {
    if game_mode.paused { return; }

    // Handle combat targeting
    if let Some(TargetingMode::Combat { agent }) = &game_mode.targeting {
        let Ok(action_state) = input.get_single() else { return; };
        
        if let Ok(agent_transform) = agent_query.get(*agent) {
            let agent_pos = agent_transform.translation.truncate();
            
            // Draw combat range
            gizmos.circle_2d(agent_pos, 150.0, Color::srgba(0.8, 0.2, 0.2, 0.3));
            
            // Highlight valid targets (only living enemies)
            for (enemy_entity, enemy_transform, enemy_health) in enemy_query.iter() {
                if enemy_health.0 <= 0.0 { continue; }
                
                let enemy_pos = enemy_transform.translation.truncate();
                let distance = agent_pos.distance(enemy_pos);
                
                if distance <= 150.0 {
                    gizmos.circle_2d(enemy_pos, 25.0, Color::srgb(1.0, 0.5, 0.5));
                    
                    // Draw crosshairs
                    let size = 15.0;
                    gizmos.line_2d(
                        enemy_pos + Vec2::new(-size, 0.0),
                        enemy_pos + Vec2::new(size, 0.0),
                        Color::srgb(0.8, 0.2, 0.2),
                    );
                    gizmos.line_2d(
                        enemy_pos + Vec2::new(0.0, -size),
                        enemy_pos + Vec2::new(0.0, size),
                        Color::srgb(0.8, 0.2, 0.2),
                    );
                }
            }
            
            // Handle target selection
            if action_state.just_pressed(&PlayerAction::Select) {
                if let Some(target) = find_combat_target(*agent, &agent_query, &enemy_query, &windows, &cameras) {
                    // Directly execute attack instead of sending event
                    execute_attack(*agent, target, &mut enemy_query, &mut combat_events, &mut audio_events);
                }
            }
        }
    }

    // Process attack actions from events
    for event in action_events.read() {
        if let Action::Attack(target) = event.action {
            execute_attack(event.entity, target, &mut enemy_query, &mut combat_events, &mut audio_events);
        }
    }

    // Draw health bars for damaged enemies (only living ones)
    for (_, transform, health) in enemy_query.iter() {
        if health.0 < 100.0 && health.0 > 0.0 {
            draw_health_bar(&mut gizmos, transform.translation.truncate(), health.0, 100.0);
        }
    }
}

pub fn death_system(
    mut commands: Commands,
    mut enemy_query: Query<(Entity, &mut Health, &mut Sprite), (With<Enemy>, Without<Dead>)>,
    mut mission_data: ResMut<MissionData>,
) {
    for (entity, mut health, mut sprite) in enemy_query.iter_mut() {
        if health.0 <= 0.0 {
            commands.entity(entity).insert(Dead);
            sprite.color = Color::srgb(0.3, 0.1, 0.1);
            commands.entity(entity).remove::<Velocity>();
            
            mission_data.enemies_killed += 1;
            info!("Enemy {} defeated. Total kills: {}", entity.index(), mission_data.enemies_killed);
        }
    }
}

fn find_combat_target(
    agent: Entity,
    agent_query: &Query<&Transform, With<Agent>>,
    enemy_query: &Query<(Entity, &Transform, &mut Health), (With<Enemy>, Without<Dead>)>,
    windows: &Query<&Window>,
    cameras: &Query<(&Camera, &GlobalTransform)>,
) -> Option<Entity> {
    let agent_transform = agent_query.get(agent).ok()?;
    let mouse_pos = get_world_mouse_position(windows, cameras)?;
    
    let mut closest_target = None;
    let mut closest_distance = f32::INFINITY;

    for (entity, transform, health) in enemy_query.iter() {
        if health.0 <= 0.0 { continue; }
        
        let enemy_pos = transform.translation.truncate();
        let agent_distance = agent_transform.translation.truncate().distance(enemy_pos);
        let mouse_distance = mouse_pos.distance(enemy_pos);

        if agent_distance <= 150.0 && mouse_distance < 30.0 && mouse_distance < closest_distance {
            closest_distance = mouse_distance;
            closest_target = Some(entity);
        }
    }

    closest_target
}

fn execute_attack(
    attacker: Entity,
    target: Entity,
    enemy_query: &mut Query<(Entity, &Transform, &mut Health), (With<Enemy>, Without<Dead>)>,
    combat_events: &mut EventWriter<CombatEvent>,
    audio_events: &mut EventWriter<AudioEvent>,
) {
    if let Ok((_, _, mut health)) = enemy_query.get_mut(target) {
        let damage = 35.0;
        let hit = rand::random::<f32>() < 0.8; // 80% hit chance
        
        if hit {
            health.0 -= damage;
            audio_events.send(AudioEvent {
                sound: AudioType::Gunshot,
                volume: 0.7,
            });            
            if health.0 <= 0.0 {
                health.0 = 0.0;
                info!("Enemy defeated!");
            }
        }
        
        combat_events.send(CombatEvent {
            attacker,
            target,
            damage: if hit { damage } else { 0.0 },
            hit,
        });
        
        if hit {
            info!("Attack hit for {} damage. Enemy health: {}", damage, health.0);
            if health.0 <= 0.0 {
                info!("Enemy defeated!");
            }
        } else {
            info!("Attack missed!");
        }
    }
}

fn draw_health_bar(gizmos: &mut Gizmos, position: Vec2, current: f32, max: f32) {
    let bar_pos = position + Vec2::new(0.0, 30.0);
    let bar_width = 30.0;
    let bar_height = 4.0;
    let health_ratio = current / max;
    
    // Background
    gizmos.rect_2d(bar_pos, 0.0, Vec2::new(bar_width, bar_height), Color::srgb(0.3, 0.3, 0.3));
    
    // Health fill
    let health_color = if health_ratio > 0.6 {
        Color::srgb(0.2, 0.8, 0.2)
    } else if health_ratio > 0.3 {
        Color::srgb(0.8, 0.8, 0.2)
    } else {
        Color::srgb(0.8, 0.2, 0.2)
    };
    
    let fill_width = bar_width * health_ratio;
    gizmos.rect_2d(
        bar_pos - Vec2::new((bar_width - fill_width) / 2.0, 0.0),
        0.0,
        Vec2::new(fill_width, bar_height),
        health_color,
    );
}