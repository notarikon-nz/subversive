// src/systems/combat.rs - Updated with projectile system
use bevy::prelude::*;
use leafwing_input_manager::prelude::*;
use crate::core::*;
use crate::systems::death::*;
use crate::systems::projectiles::*;

// Separate system to process attack events
pub fn process_attack_events(
    mut commands: Commands,
    mut action_events: EventReader<ActionEvent>,
    mut audio_events: EventWriter<AudioEvent>,
    agent_query: Query<(&Transform, &Inventory), With<Agent>>,
    mut agent_weapon_query: Query<&mut WeaponState, With<Agent>>,
    target_query: Query<(Entity, &Transform, &Health), Or<(With<Enemy>, With<Vehicle>)>>,
    weapon_db: Res<WeaponDatabase>,
) {
    for event in action_events.read() {
        if let Action::Attack(target) = event.action {
            execute_attack(event.entity, target, &mut commands, &agent_query, &mut agent_weapon_query, 
                         &target_query, &mut audio_events, &weapon_db);
        }
    }
}

// Main combat system - only handles direct input, no ActionEvent reading
pub fn system(
    mut commands: Commands,
    mut gizmos: Gizmos,
    input: Query<&ActionState<PlayerAction>>,
    mut audio_events: EventWriter<AudioEvent>,
    agent_query: Query<(&Transform, &Inventory), With<Agent>>,
    mut agent_weapon_query: Query<&mut WeaponState, With<Agent>>,
    target_query: Query<(Entity, &Transform, &Health), Or<(With<Enemy>, With<Vehicle>)>>,
    game_mode: Res<GameMode>,
    weapon_db: Res<WeaponDatabase>,
    windows: Query<&Window>,
    cameras: Query<(&Camera, &GlobalTransform)>,
    selection: Res<SelectionState>,
) {
    if game_mode.paused { return; }

    // Handle combat targeting mode - show UI for primary agent
    if let Some(TargetingMode::Combat { agent }) = &game_mode.targeting {
        handle_combat_targeting(*agent, &mut commands, &input, &agent_query, &mut agent_weapon_query, 
                               &target_query, &mut audio_events, &weapon_db, &mut gizmos, &windows, &cameras, &selection);
    }
}

// Update handle_combat_targeting to not use ActionEvents
fn handle_combat_targeting(
    primary_agent: Entity,
    commands: &mut Commands,
    input: &Query<&ActionState<PlayerAction>>,
    agent_query: &Query<(&Transform, &Inventory), With<Agent>>,
    agent_weapon_query: &mut Query<&mut WeaponState, With<Agent>>,
    target_query: &Query<(Entity, &Transform, &Health), Or<(With<Enemy>, With<Vehicle>)>>,
    audio_events: &mut EventWriter<AudioEvent>,
    weapon_db: &WeaponDatabase,
    gizmos: &mut Gizmos,
    windows: &Query<&Window>,
    cameras: &Query<(&Camera, &GlobalTransform)>,
    selection: &SelectionState,
) {
    let Ok(action_state) = input.single() else { return; };
    let Ok((primary_transform, primary_inventory)) = agent_query.get(primary_agent) else { return; };
    
    let primary_pos = primary_transform.translation.truncate();
    let range = get_weapon_range(primary_inventory, agent_weapon_query.get(primary_agent).ok());
    
    // Draw targeting UI for primary agent
    gizmos.circle_2d(primary_pos, range, Color::srgba(0.8, 0.2, 0.2, 0.3));
    draw_targets_in_range(gizmos, target_query, primary_pos, range);
    
    // Handle mouse click for target selection - directly execute attacks
    if action_state.just_pressed(&PlayerAction::Move) {
        if let Some(target) = find_target_at_mouse(target_query, primary_pos, range, windows, cameras) {
            // Directly execute attacks for all selected agents
            for &agent in &selection.selected {
                if agent_query.get(agent).is_ok() {
                    execute_attack(agent, target, commands, agent_query, agent_weapon_query, target_query, audio_events, weapon_db);
                }
            }
        }
    }
}


// Alternative simpler fix - just don't auto-move when out of range
fn execute_attack(
    attacker: Entity,
    target: Entity,
    commands: &mut Commands,
    agent_query: &Query<(&Transform, &Inventory), With<Agent>>,
    agent_weapon_query: &mut Query<&mut WeaponState, With<Agent>>,
    target_query: &Query<(Entity, &Transform, &Health), Or<(With<Enemy>, With<Vehicle>)>>,
    audio_events: &mut EventWriter<AudioEvent>,
    weapon_db: &WeaponDatabase,
) {
    // Get positions first
    let Ok((attacker_transform, inventory)) = agent_query.get(attacker) else { return; };
    let Ok((_, target_transform, _)) = target_query.get(target) else { return; };
    
    let attacker_pos = attacker_transform.translation.truncate();
    let target_pos = target_transform.translation.truncate();
    let distance = attacker_pos.distance(target_pos);
    
    // Get weapon range
    let range = get_weapon_range(inventory, agent_weapon_query.get(attacker).ok());
    
    // If out of range, simply don't attack (no auto-movement)
    if distance > range {
        return;
    }
    
    // Rest of the attack logic remains the same...
    if let Ok(mut weapon_state) = agent_weapon_query.get_mut(attacker) {
        if !weapon_state.can_fire() || !weapon_state.consume_ammo() {
            return;
        }
    }
    
    let weapon_type = inventory.equipped_weapon
        .as_ref()
        .map(|w| w.base_weapon.clone())
        .unwrap_or(WeaponType::Pistol);
    
    let (damage, accuracy, noise) = get_attack_stats(
        Some((attacker_transform, inventory)), 
        agent_weapon_query.get(attacker).ok(),
        weapon_db
    );
    
    let hit = rand::random::<f32>() < accuracy;
    
    if hit {
        spawn_projectile(
            commands,
            attacker,
            target,
            attacker_pos,
            target_pos,
            damage,
            weapon_type.clone(),
        );
        
        audio_events.send(AudioEvent { 
            sound: AudioType::Gunshot, 
            volume: (0.7 * noise).clamp(0.1, 1.0) 
        });
    } else {
        // Miss logic
        let miss_offset = Vec2::new(
            (rand::random::<f32>() - 0.5) * 100.0,
            (rand::random::<f32>() - 0.5) * 100.0,
        );
        let miss_target_pos = target_pos + miss_offset;
        
        let miss_target = commands.spawn((
            Transform::from_translation(miss_target_pos.extend(0.0)),
            MissTarget,
        )).id();
        
        spawn_projectile(
            commands,
            attacker,
            miss_target,
            attacker_pos,
            miss_target_pos,
            0.0,
            weapon_type,
        );
        
        audio_events.send(AudioEvent { 
            sound: AudioType::Gunshot, 
            volume: (0.5 * noise).clamp(0.1, 1.0) 
        });
    }
}

#[derive(Component)]
pub struct MissTarget;

// System to clean up miss targets after projectiles are done
pub fn cleanup_miss_targets(
    mut commands: Commands,
    miss_targets: Query<Entity, (With<MissTarget>, Without<MarkedForDespawn>)>,
    projectiles: Query<&Projectile>,
) {
    for miss_target in miss_targets.iter() {
        // Check if any projectile is targeting this miss target
        let still_targeted = projectiles.iter().any(|p| p.target == miss_target);
        
        if !still_targeted {
            commands.entity(miss_target).insert(MarkedForDespawn);
        }
    }
}

fn get_weapon_range(inventory: &Inventory, weapon_state: Option<&WeaponState>) -> f32 {
    let base_range = 150.0;
    if let Some(weapon_config) = &inventory.equipped_weapon {
        let stats = weapon_config.stats();
        (base_range * (1.0_f32 + stats.range as f32 * 0.1_f32)).max(50.0_f32)
    } else {
        base_range
    }
}

fn get_attack_stats(
    agent_data: Option<(&Transform, &Inventory)>, 
    _weapon_state: Option<&WeaponState>,
    weapon_db: &WeaponDatabase,
) -> (f32, f32, f32) {
    if let Some((_, inventory)) = agent_data {
        if let Some(weapon_config) = &inventory.equipped_weapon {
            let stats = weapon_config.calculate_total_stats();
            
            // Get base damage from weapon database
            let base_damage = weapon_db.get(&weapon_config.base_weapon)
                .map(|weapon_data| weapon_data.damage)
                .unwrap_or(35.0);
            
            let damage = base_damage * (1.0 + stats.accuracy as f32 * 0.02);
            let accuracy = (0.8 + stats.accuracy as f32 * 0.05).clamp(0.1, 0.95);
            let noise = (1.0 + stats.noise as f32 * 0.1).max(0.1);
            return (damage, accuracy, noise);
        }
    }
    (35.0, 0.8, 1.0) // Default values
}

fn draw_targets_in_range(
    gizmos: &mut Gizmos,
    target_query: &Query<(Entity, &Transform, &Health), Or<(With<Enemy>, With<Vehicle>)>>,
    agent_pos: Vec2,
    range: f32,
) {
    for (_, transform, health) in target_query.iter() {
        if health.0 > 0.0 && agent_pos.distance(transform.translation.truncate()) <= range {
            let pos = transform.translation.truncate();
            gizmos.circle_2d(pos, 25.0, Color::srgb(1.0, 0.5, 0.5));
            draw_crosshair(gizmos, pos, 15.0, Color::srgb(0.8, 0.2, 0.2));
        }
    }
}

fn find_target_at_mouse(
    target_query: &Query<(Entity, &Transform, &Health), Or<(With<Enemy>, With<Vehicle>)>>,
    agent_pos: Vec2,
    range: f32,
    windows: &Query<&Window>,
    cameras: &Query<(&Camera, &GlobalTransform)>,
) -> Option<Entity> {
    let mouse_pos = get_world_mouse_position(windows, cameras)?;
    
    target_query.iter()
        .filter(|(_, _, health)| health.0 > 0.0)
        .filter(|(_, transform, _)| agent_pos.distance(transform.translation.truncate()) <= range)
        .filter(|(_, transform, _)| mouse_pos.distance(transform.translation.truncate()) < 35.0)
        .min_by(|(_, a_transform, _), (_, b_transform, _)| {
            let a_dist = mouse_pos.distance(a_transform.translation.truncate());
            let b_dist = mouse_pos.distance(b_transform.translation.truncate());
            a_dist.partial_cmp(&b_dist).unwrap_or(std::cmp::Ordering::Equal)
        })
        .map(|(entity, _, _)| entity)
}

fn draw_crosshair(gizmos: &mut Gizmos, position: Vec2, size: f32, color: Color) {
    let h = size / 2.0;
    gizmos.line_2d(position + Vec2::new(-h, 0.0), position + Vec2::new(h, 0.0), color);
    gizmos.line_2d(position + Vec2::new(0.0, -h), position + Vec2::new(0.0, h), color);
}

/*
pub fn death_system(
    mut commands: Commands,
    mut target_query: Query<(Entity, &mut Health, &mut Sprite), (Or<(With<Enemy>, With<Vehicle>)>, Without<Dead>)>,
    enemy_query: Query<Entity, (With<Enemy>, Without<Dead>)>,
    mut mission_data: ResMut<MissionData>,
) {
    for (entity, health, mut sprite) in target_query.iter_mut() {
        if health.0 <= 0.0 {
            commands.entity(entity).insert(Dead);
            sprite.color = Color::srgb(0.3, 0.1, 0.1);
            
            if enemy_query.contains(entity) {
                mission_data.enemies_killed += 1;
            }
        }
    }
}
*/

// ENEMY SYSTEM
pub fn enemy_combat_system(
    mut commands: Commands,
    mut action_events: EventReader<ActionEvent>,
    mut audio_events: EventWriter<AudioEvent>,
    mut enemy_query: Query<(&Transform, &Inventory, &mut WeaponState), With<Enemy>>,
    agent_query: Query<(Entity, &Transform, &Health), With<Agent>>,
    weapon_db: Res<WeaponDatabase>,
    game_mode: Res<GameMode>,
) {
    if game_mode.paused { return; }

    for event in action_events.read() {
        match event.action {
            Action::Attack(target) => {
                // Check if this is an enemy attacking
                if let Ok((enemy_transform, inventory, mut weapon_state)) = enemy_query.get_mut(event.entity) {
                    // Simple validation: target should be a valid agent
                    if agent_query.get(target).is_ok() {
                        execute_enemy_attack(
                            event.entity,
                            target,
                            &mut commands,
                            enemy_transform,
                            inventory,
                            &mut weapon_state,
                            &agent_query,
                            &mut audio_events,
                            &weapon_db,
                        );
                    } else {
                        // println!("Enemy {:?} target {:?} is not a valid agent - skipping", event.entity, target);
                    }
                }
            },
            Action::Reload => {
                // Handle enemy reload - use the proper reload system
                if let Ok((_, _, mut weapon_state)) = enemy_query.get_mut(event.entity) {
                    if !weapon_state.is_reloading {
                        let old_ammo = weapon_state.current_ammo;
                        weapon_state.start_reload(); // Use start_reload instead of reload_to_full
                        // println!("Enemy {:?} started reloading: {}/{} ammo, {:.1}s reload time", event.entity, old_ammo, weapon_state.max_ammo, weapon_state.reload_time);
                    } else {
                        // println!("Enemy {:?} already reloading, ignoring reload command", event.entity);
                    }
                }
            },
            Action::UseMedKit => {
                info!("Using MedKit!");
            },
            _ => {} // Ignore other actions
        }
    }
}

fn execute_enemy_attack(
    attacker: Entity,
    target: Entity,
    commands: &mut Commands,
    attacker_transform: &Transform,
    inventory: &Inventory,
    weapon_state: &mut WeaponState,
    target_query: &Query<(Entity, &Transform, &Health), With<Agent>>,
    audio_events: &mut EventWriter<AudioEvent>,
    weapon_db: &WeaponDatabase,
) {
    // Debug output
    // println!("Enemy {:?} executing attack on agent {:?}. Ammo: {}/{}", attacker, target, weapon_state.current_ammo, weapon_state.max_ammo);
    
    // Validate and consume ammo
    if !weapon_state.can_fire() {
        // println!("Enemy {:?} cannot fire - no ammo", attacker);
        return;
    }
    
    if !weapon_state.consume_ammo() {
        // println!("Enemy {:?} failed to consume ammo", attacker);
        return;
    }
    
    // Get positions
    let Ok((_, target_transform, _)) = target_query.get(target) else { 
        // println!("Enemy {:?} target {:?} not found in agent query", attacker, target);
        return; 
    };
    
    let attacker_pos = attacker_transform.translation.truncate();
    let target_pos = target_transform.translation.truncate();
    
    // println!("Enemy {:?} firing at agent {:?}! Distance: {:.1}, Remaining ammo: {}", attacker, target, attacker_pos.distance(target_pos), weapon_state.current_ammo);
    
    // Get weapon type and calculate damage
    let weapon_type = inventory.equipped_weapon
        .as_ref()
        .map(|w| w.base_weapon.clone())
        .unwrap_or(WeaponType::Pistol);
    
    let (damage, accuracy, noise) = get_enemy_attack_stats(inventory, weapon_state, weapon_db);
    
    // Check if shot hits (accuracy check)
    let hit = rand::random::<f32>() < accuracy;
    
    // println!("Enemy attack: damage={:.1}, accuracy={:.2}, hit={}", damage, accuracy, hit);
    
    if hit {
        // Spawn projectile that will hit
        spawn_projectile(
            commands,
            attacker,
            target,
            attacker_pos,
            target_pos,
            damage,
            weapon_type.clone(),
        );
        
        // println!("Enemy {:?} HIT agent {:?} for {:.1} damage", attacker, target, damage);
        
        // Play audio
        audio_events.write(AudioEvent { 
            sound: AudioType::Gunshot, 
            volume: (0.7 * noise).clamp(0.1, 1.0) 
        });
    } else {
        // Miss - spawn projectile that goes past target
        let miss_offset = Vec2::new(
            (rand::random::<f32>() - 0.5) * 80.0,
            (rand::random::<f32>() - 0.5) * 80.0,
        );
        let miss_target_pos = target_pos + miss_offset;
        
        let miss_target = commands.spawn((
            Transform::from_translation(miss_target_pos.extend(0.0)),
            MissTarget,
        )).id();
        
        spawn_projectile(
            commands,
            attacker,
            miss_target,
            attacker_pos,
            miss_target_pos,
            0.0,
            weapon_type,
        );

        // println!("Enemy {:?} MISSED agent {:?}", attacker, target);

        // Still play audio for misses
        audio_events.write(AudioEvent { 
            sound: AudioType::Gunshot, 
            volume: (0.5 * noise).clamp(0.1, 1.0) 
        });
    }
}

fn get_enemy_attack_stats(
    inventory: &Inventory,
    _weapon_state: &WeaponState,
    weapon_db: &WeaponDatabase,
) -> (f32, f32, f32) {
    if let Some(weapon_config) = &inventory.equipped_weapon {
        let stats = weapon_config.calculate_total_stats();
        
        // Get base damage from weapon database
        let base_damage = weapon_db.get(&weapon_config.base_weapon)
            .map(|weapon_data| weapon_data.damage)
            .unwrap_or(25.0); // Slightly lower than player default
        
        let damage = base_damage * (1.0 + stats.accuracy as f32 * 0.02);
        let accuracy = (0.6 + stats.accuracy as f32 * 0.03).clamp(0.1, 0.85); // Lower than player
        let noise = (1.0 + stats.noise as f32 * 0.1).max(0.1);
        return (damage, accuracy, noise);
    }
    (25.0, 0.6, 1.0) // Default enemy values - lower than player
}


// Auto-reload system - add this to your main.rs update systems
pub fn auto_reload_system(
    mut agent_weapon_query: Query<&mut WeaponState, With<Agent>>,
    mut action_events: EventWriter<ActionEvent>,
    agent_query: Query<Entity, With<Agent>>,
    time: Res<Time>,
    game_mode: Res<GameMode>,
) {
    if game_mode.paused { return; }
    
    for agent_entity in agent_query.iter() {
        if let Ok(mut weapon_state) = agent_weapon_query.get_mut(agent_entity) {
            // Handle ongoing reload
            if weapon_state.is_reloading {
                weapon_state.reload_timer -= time.delta_secs();
                if weapon_state.reload_timer <= 0.0 {
                    weapon_state.complete_reload();
                    info!("Agent {:?} auto-reload completed: {}/{}", agent_entity, weapon_state.current_ammo, weapon_state.max_ammo);
                }
            }
            // Auto-reload when empty
            else if weapon_state.current_ammo == 0 {
                weapon_state.start_reload();
                info!("Agent {:?} starting auto-reload", agent_entity);
            }
        }
    }
}