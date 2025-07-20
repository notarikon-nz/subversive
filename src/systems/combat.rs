// Update src/systems/combat.rs to use attachment stats

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
    agent_query: Query<(&Transform, &Inventory), With<Agent>>,
    mut enemy_query: Query<(Entity, &Transform, &mut Health), (With<Enemy>, Without<Dead>)>,
    game_mode: Res<GameMode>,
    windows: Query<&Window>,
    cameras: Query<(&Camera, &GlobalTransform)>,
) {
    if game_mode.paused { return; }

    // Handle combat targeting
    if let Some(TargetingMode::Combat { agent }) = &game_mode.targeting {
        let Ok(action_state) = input.get_single() else { return; };
        
        if let Ok((agent_transform, inventory)) = agent_query.get(*agent) {
            let agent_pos = agent_transform.translation.truncate();
            
            // Calculate weapon stats with attachments
            let (range, _accuracy_bonus) = calculate_weapon_stats(inventory);
            
            // Draw combat range with attachment modifiers
            gizmos.circle_2d(agent_pos, range, Color::srgba(0.8, 0.2, 0.2, 0.3));
            
            // Highlight valid targets
            for (enemy_entity, enemy_transform, enemy_health) in enemy_query.iter() {
                if enemy_health.0 <= 0.0 { continue; }
                
                let enemy_pos = enemy_transform.translation.truncate();
                let distance = agent_pos.distance(enemy_pos);
                
                if distance <= range {
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
                if let Some(target) = find_combat_target(*agent, &agent_query, &enemy_query, &windows, &cameras, range) {
                    execute_attack(*agent, target, &agent_query, &mut enemy_query, &mut combat_events, &mut audio_events);
                }
            }
        }
    }

    // Process attack actions from events
    for event in action_events.read() {
        if let Action::Attack(target) = event.action {
            execute_attack(event.entity, target, &agent_query, &mut enemy_query, &mut combat_events, &mut audio_events);
        }
    }

    // Draw health bars for damaged enemies
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

fn calculate_weapon_stats(inventory: &Inventory) -> (f32, f32) {
    let base_range = 150.0;
    let base_accuracy = 0.8;
    
    if let Some(weapon_config) = &inventory.equipped_weapon {
        let stats = weapon_config.calculate_total_stats();
        
        // Apply attachment modifiers
        let range_modifier = 1.0 + (stats.range as f32 * 0.1); // Each point = 10% range change
        let accuracy_modifier = stats.accuracy as f32 * 0.05; // Each point = 5% accuracy change
        
        let final_range = (base_range * range_modifier).max(50.0); // Minimum 50 range
        let final_accuracy = (base_accuracy + accuracy_modifier).clamp(0.1, 1.0); // 10% to 100% accuracy
        
        (final_range, final_accuracy)
    } else {
        (base_range, base_accuracy)
    }
}

fn find_combat_target(
    agent: Entity,
    agent_query: &Query<(&Transform, &Inventory), With<Agent>>,
    enemy_query: &Query<(Entity, &Transform, &mut Health), (With<Enemy>, Without<Dead>)>,
    windows: &Query<&Window>,
    cameras: &Query<(&Camera, &GlobalTransform)>,
    weapon_range: f32,
) -> Option<Entity> {
    let (agent_transform, _) = agent_query.get(agent).ok()?;
    let mouse_pos = get_world_mouse_position(windows, cameras)?;
    
    let mut closest_target = None;
    let mut closest_distance = f32::INFINITY;

    for (entity, transform, health) in enemy_query.iter() {
        if health.0 <= 0.0 { continue; }
        
        let enemy_pos = transform.translation.truncate();
        let agent_distance = agent_transform.translation.truncate().distance(enemy_pos);
        let mouse_distance = mouse_pos.distance(enemy_pos);

        if agent_distance <= weapon_range && mouse_distance < 30.0 && mouse_distance < closest_distance {
            closest_distance = mouse_distance;
            closest_target = Some(entity);
        }
    }

    closest_target
}

fn execute_attack(
    attacker: Entity,
    target: Entity,
    agent_query: &Query<(&Transform, &Inventory), With<Agent>>,
    enemy_query: &mut Query<(Entity, &Transform, &mut Health), (With<Enemy>, Without<Dead>)>,
    combat_events: &mut EventWriter<CombatEvent>,
    audio_events: &mut EventWriter<AudioEvent>,
) {
    if let Ok((_, _, mut health)) = enemy_query.get_mut(target) {
        // Calculate damage and accuracy with attachments
        let (damage, hit_chance, noise_level) = if let Ok((_, inventory)) = agent_query.get(attacker) {
            calculate_attack_stats(inventory)
        } else {
            (35.0, 0.8, 1.0) // Default stats
        };
        
        let hit = rand::random::<f32>() < hit_chance;
        
        if hit {
            health.0 -= damage;
            
            // Play gunshot with noise modifier
            let volume = (0.7 * noise_level).clamp(0.1, 1.0);
            audio_events.write(AudioEvent {
                sound: AudioType::Gunshot,
                volume,
            });
            
            if health.0 <= 0.0 {
                health.0 = 0.0;
                info!("Enemy defeated!");
            }
        }
        
        combat_events.write(CombatEvent {
            attacker,
            target,
            damage: if hit { damage } else { 0.0 },
            hit,
        });
        
        if hit {
            info!("Attack hit for {} damage. Enemy health: {}", damage, health.0);
        } else {
            info!("Attack missed!");
        }
    }
}

fn calculate_attack_stats(inventory: &Inventory) -> (f32, f32, f32) {
    let base_damage = 35.0;
    let base_accuracy = 0.8;
    let base_noise = 1.0;
    
    if let Some(weapon_config) = &inventory.equipped_weapon {
        let stats = weapon_config.calculate_total_stats();
        
        // Apply attachment modifiers
        let damage_modifier = 1.0 + (stats.accuracy as f32 * 0.02); // Accuracy slightly increases damage
        let accuracy_modifier = stats.accuracy as f32 * 0.05; // Each point = 5% accuracy
        let noise_modifier = 1.0 + (stats.noise as f32 * 0.1); // Each point = 10% noise change
        
        let final_damage = base_damage * damage_modifier;
        let final_accuracy = (base_accuracy + accuracy_modifier).clamp(0.1, 0.95);
        let final_noise = (base_noise * noise_modifier).max(0.1); // Minimum 10% noise
        
        (final_damage, final_accuracy, final_noise)
    } else {
        (base_damage, base_accuracy, base_noise)
    }
}

fn draw_health_bar(gizmos: &mut Gizmos, position: Vec2, current: f32, max: f32) {
    let bar_pos = position + Vec2::new(0.0, 30.0);
    let bar_width = 30.0;
    let bar_height = 4.0;
    let health_ratio = current / max;
    
    // Background
    gizmos.rect_2d(
        bar_pos, 
        Vec2::new(bar_width, bar_height), 
        Color::srgb(0.3, 0.3, 0.3)
    );
    
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
        Vec2::new(fill_width, bar_height),
        health_color,
    );
}