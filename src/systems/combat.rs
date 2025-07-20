// src/systems/combat.rs - Optimized and streamlined

use bevy::prelude::*;
use bevy::ecs::system::ParamSet;
use leafwing_input_manager::prelude::*;
use crate::core::*;

pub fn system(
    mut gizmos: Gizmos,
    input: Query<&ActionState<PlayerAction>>,
    mut action_events: EventReader<ActionEvent>,
    mut combat_events: EventWriter<CombatEvent>,
    mut audio_events: EventWriter<AudioEvent>,
    agent_query: Query<(&Transform, &Inventory), With<Agent>>,
    mut agent_weapon_query: Query<&mut WeaponState, With<Agent>>,
    mut health_queries: ParamSet<(
        Query<(Entity, &Transform, &mut Health), (With<Enemy>, Without<Dead>)>,
        Query<(Entity, &Transform, &mut Health), (With<Vehicle>, Without<Dead>)>,
    )>,
    game_mode: Res<GameMode>,
    windows: Query<&Window>,
    cameras: Query<(&Camera, &GlobalTransform)>,
) {
    if game_mode.paused { return; }

    // Handle combat targeting mode
    if let Some(TargetingMode::Combat { agent }) = &game_mode.targeting {
        handle_combat_targeting(*agent, &input, &agent_query, &mut agent_weapon_query, 
                               &mut health_queries, &mut combat_events, &mut audio_events,
                               &mut gizmos, &windows, &cameras);
    }

    // Process attack events
    for event in action_events.read() {
        if let Action::Attack(target) = event.action {
            execute_attack(event.entity, target, &agent_query, &mut agent_weapon_query, 
                         &mut health_queries, &mut combat_events, &mut audio_events);
        }
    }

    // Draw health bars for damaged enemies
    draw_health_bars(&mut gizmos, &mut health_queries);
}

fn handle_combat_targeting(
    agent: Entity,
    input: &Query<&ActionState<PlayerAction>>,
    agent_query: &Query<(&Transform, &Inventory), With<Agent>>,
    agent_weapon_query: &mut Query<&mut WeaponState, With<Agent>>,
    health_queries: &mut ParamSet<(
        Query<(Entity, &Transform, &mut Health), (With<Enemy>, Without<Dead>)>,
        Query<(Entity, &Transform, &mut Health), (With<Vehicle>, Without<Dead>)>,
    )>,
    combat_events: &mut EventWriter<CombatEvent>,
    audio_events: &mut EventWriter<AudioEvent>,
    gizmos: &mut Gizmos,
    windows: &Query<&Window>,
    cameras: &Query<(&Camera, &GlobalTransform)>,
) {
    let Ok(action_state) = input.single() else { return; };
    let Ok((agent_transform, inventory)) = agent_query.get(agent) else { return; };
    
    let agent_pos = agent_transform.translation.truncate();
    let range = get_weapon_range(inventory, agent_weapon_query.get(agent).ok());
    
    // Draw targeting UI
    gizmos.circle_2d(agent_pos, range, Color::srgba(0.8, 0.2, 0.2, 0.3));
    draw_targets_in_range(gizmos, health_queries, agent_pos, range);
    
    // Handle mouse click for target selection
    if action_state.just_pressed(&PlayerAction::Select) {
        if let Some(target) = find_target_at_mouse(health_queries, agent_pos, range, windows, cameras) {
            execute_attack(agent, target, agent_query, agent_weapon_query, health_queries, combat_events, audio_events);
        }
    }
}

fn get_weapon_range(inventory: &Inventory, weapon_state: Option<&WeaponState>) -> f32 {
    let base_range = 150.0;
    if let Some(weapon_config) = &inventory.equipped_weapon {
        let stats = weapon_config.calculate_total_stats();
        (base_range * (1.0 + stats.range as f32 * 0.1)).max(50.0)
    } else {
        base_range
    }
}

fn draw_targets_in_range(
    gizmos: &mut Gizmos,
    health_queries: &mut ParamSet<(
        Query<(Entity, &Transform, &mut Health), (With<Enemy>, Without<Dead>)>,
        Query<(Entity, &Transform, &mut Health), (With<Vehicle>, Without<Dead>)>,
    )>,
    agent_pos: Vec2,
    range: f32,
) {
    // Draw enemy targets
    {
        let enemy_query = health_queries.p0();
        for (_, transform, health) in enemy_query.iter() {
            if health.0 > 0.0 && agent_pos.distance(transform.translation.truncate()) <= range {
                let pos = transform.translation.truncate();
                gizmos.circle_2d(pos, 25.0, Color::srgb(1.0, 0.5, 0.5));
                draw_crosshair(gizmos, pos, 15.0, Color::srgb(0.8, 0.2, 0.2));
            }
        }
    }
    
    // Draw vehicle targets
    {
        let vehicle_query = health_queries.p1();
        for (_, transform, health) in vehicle_query.iter() {
            if health.0 > 0.0 && agent_pos.distance(transform.translation.truncate()) <= range {
                let pos = transform.translation.truncate();
                gizmos.circle_2d(pos, 35.0, Color::srgb(0.8, 0.8, 0.2));
                draw_crosshair(gizmos, pos, 20.0, Color::srgb(0.8, 0.6, 0.2));
            }
        }
    }
}

fn find_target_at_mouse(
    health_queries: &mut ParamSet<(
        Query<(Entity, &Transform, &mut Health), (With<Enemy>, Without<Dead>)>,
        Query<(Entity, &Transform, &mut Health), (With<Vehicle>, Without<Dead>)>,
    )>,
    agent_pos: Vec2,
    range: f32,
    windows: &Query<&Window>,
    cameras: &Query<(&Camera, &GlobalTransform)>,
) -> Option<Entity> {
    let mouse_pos = get_world_mouse_position(windows, cameras)?;
    let mut closest = (None, f32::INFINITY);
    
    // Check enemies (smaller click radius)
    {
        let enemy_query = health_queries.p0();
        for (entity, transform, health) in enemy_query.iter() {
            if health.0 <= 0.0 { continue; }
            
            let pos = transform.translation.truncate();
            if agent_pos.distance(pos) <= range {
                let mouse_dist = mouse_pos.distance(pos);
                if mouse_dist < 30.0 && mouse_dist < closest.1 {
                    closest = (Some(entity), mouse_dist);
                }
            }
        }
    }
    
    // Check vehicles if no enemy found (larger click radius)
    if closest.0.is_none() {
        let vehicle_query = health_queries.p1();
        for (entity, transform, health) in vehicle_query.iter() {
            if health.0 <= 0.0 { continue; }
            
            let pos = transform.translation.truncate();
            if agent_pos.distance(pos) <= range {
                let mouse_dist = mouse_pos.distance(pos);
                if mouse_dist < 40.0 && mouse_dist < closest.1 {
                    closest = (Some(entity), mouse_dist);
                }
            }
        }
    }
    
    closest.0
}

fn execute_attack(
    attacker: Entity,
    target: Entity,
    agent_query: &Query<(&Transform, &Inventory), With<Agent>>,
    agent_weapon_query: &mut Query<&mut WeaponState, With<Agent>>,
    health_queries: &mut ParamSet<(
        Query<(Entity, &Transform, &mut Health), (With<Enemy>, Without<Dead>)>,
        Query<(Entity, &Transform, &mut Health), (With<Vehicle>, Without<Dead>)>,
    )>,
    combat_events: &mut EventWriter<CombatEvent>,
    audio_events: &mut EventWriter<AudioEvent>,
) {
    // Validate and consume ammo
    if let Ok(mut weapon_state) = agent_weapon_query.get_mut(attacker) {
        if !weapon_state.can_fire() || !weapon_state.consume_ammo() {
            return;
        }
    }
    
    // Calculate attack parameters
    let (damage, accuracy, noise) = get_attack_stats(agent_query.get(attacker).ok(), 
                                                   agent_weapon_query.get(attacker).ok());
    let hit = rand::random::<f32>() < accuracy;
    
    // Apply damage to target (try enemy first, then vehicle)
    let final_damage = if apply_damage_to_enemy(&mut health_queries.p0(), target, damage, hit) {
        damage
    } else if apply_damage_to_vehicle(&mut health_queries.p1(), target, damage, hit) {
        damage * 0.5 // Vehicles take reduced damage
    } else {
        0.0 // Target not found
    };
    
    // Send events
    if hit && final_damage > 0.0 {
        audio_events.write(AudioEvent { 
            sound: AudioType::Gunshot, 
            volume: (0.7 * noise).clamp(0.1, 1.0) 
        });
    }
    
    combat_events.write(CombatEvent { attacker, target, damage: final_damage, hit });
}

fn get_attack_stats(
    agent_data: Option<(&Transform, &Inventory)>, 
    _weapon_state: Option<&WeaponState>
) -> (f32, f32, f32) {
    if let Some((_, inventory)) = agent_data {
        if let Some(weapon_config) = &inventory.equipped_weapon {
            let stats = weapon_config.calculate_total_stats();
            let damage = 35.0 * (1.0 + stats.accuracy as f32 * 0.02);
            let accuracy = (0.8 + stats.accuracy as f32 * 0.05).clamp(0.1, 0.95);
            let noise = (1.0 + stats.noise as f32 * 0.1).max(0.1);
            return (damage, accuracy, noise);
        }
    }
    (35.0, 0.8, 1.0) // Default values
}

fn apply_damage_to_enemy(
    enemy_query: &mut Query<(Entity, &Transform, &mut Health), (With<Enemy>, Without<Dead>)>,
    target: Entity,
    damage: f32,
    hit: bool,
) -> bool {
    if let Ok((_, _, mut health)) = enemy_query.get_mut(target) {
        if hit {
            health.0 = (health.0 - damage).max(0.0);
        }
        true
    } else {
        false
    }
}

fn apply_damage_to_vehicle(
    vehicle_query: &mut Query<(Entity, &Transform, &mut Health), (With<Vehicle>, Without<Dead>)>,
    target: Entity,
    damage: f32,
    hit: bool,
) -> bool {
    if let Ok((_, _, mut health)) = vehicle_query.get_mut(target) {
        if hit {
            health.0 = (health.0 - damage * 0.5).max(0.0); // Vehicles take 50% damage
        }
        true
    } else {
        false
    }
}

fn draw_health_bars(
    gizmos: &mut Gizmos,
    health_queries: &mut ParamSet<(
        Query<(Entity, &Transform, &mut Health), (With<Enemy>, Without<Dead>)>,
        Query<(Entity, &Transform, &mut Health), (With<Vehicle>, Without<Dead>)>,
    )>,
) {
    let enemy_query = health_queries.p0();
    for (_, transform, health) in enemy_query.iter() {
        if health.0 <= 100.0 && health.0 > 0.0 {
            draw_health_bar(gizmos, transform.translation.truncate(), health.0, 100.0);
        }
    }
}

fn draw_crosshair(gizmos: &mut Gizmos, position: Vec2, size: f32, color: Color) {
    let h = size / 2.0;
    gizmos.line_2d(position + Vec2::new(-h, 0.0), position + Vec2::new(h, 0.0), color);
    gizmos.line_2d(position + Vec2::new(0.0, -h), position + Vec2::new(0.0, h), color);
}

fn draw_health_bar(gizmos: &mut Gizmos, position: Vec2, current: f32, max: f32) {
    let bar_pos = position + Vec2::new(0.0, 30.0);
    let bar_size = Vec2::new(30.0, 4.0);
    let ratio = current / max;
    
    // Background
    gizmos.rect_2d(bar_pos, bar_size, Color::srgb(0.3, 0.3, 0.3));
    
    // Health fill
    let color = match ratio {
        r if r > 0.6 => Color::srgb(0.2, 0.8, 0.2),
        r if r > 0.3 => Color::srgb(0.8, 0.8, 0.2),
        _ => Color::srgb(0.8, 0.2, 0.2),
    };
    
    let fill_size = Vec2::new(bar_size.x * ratio, bar_size.y);
    let fill_pos = bar_pos - Vec2::new((bar_size.x - fill_size.x) / 2.0, 0.0);
    gizmos.rect_2d(fill_pos, fill_size, color);
}

pub fn death_system(
    mut commands: Commands,
    mut enemy_query: Query<(Entity, &mut Health, &mut Sprite), (With<Enemy>, Without<Dead>)>,
    mut mission_data: ResMut<MissionData>,
) {
    for (entity, health, mut sprite) in enemy_query.iter_mut() {
        if health.0 <= 0.0 {
            commands.entity(entity).insert(Dead);
            sprite.color = Color::srgb(0.3, 0.1, 0.1);
            mission_data.enemies_killed += 1;
        }
    }
}