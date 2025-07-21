// src/systems/testing_spawn.rs - On-demand enemy spawning for GOAP testing
use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use crate::core::*;
use crate::core::factions::Faction;
use crate::systems::ai::*;

pub fn testing_spawn_system(
    mut commands: Commands,
    input: Res<ButtonInput<KeyCode>>,
    sprites: Res<GameSprites>,
    global_data: Res<GlobalData>,
    game_state: Res<State<GameState>>,
    cameras: Query<&Transform, With<Camera>>,
) {
    // Only work in mission state
    if *game_state.get() != GameState::Mission {
        return;
    }

    let camera_pos = if let Ok(camera_transform) = cameras.single() {
        camera_transform.translation.truncate()
    } else {
        Vec2::ZERO // Fallback if no camera found
    };
    
    
    // T - Spawn Corporate enemy
    if input.just_pressed(KeyCode::KeyT) {
        let spawn_pos = camera_pos + Vec2::new(100.0, 50.0);
        spawn_test_enemy(&mut commands, spawn_pos, Faction::Corporate, &sprites, &global_data);
        info!("Spawned Corporate enemy at {:?}", spawn_pos);
    }
    
    // Y - Spawn Syndicate enemy
    if input.just_pressed(KeyCode::KeyY) {
        let spawn_pos = camera_pos + Vec2::new(-100.0, 50.0);
        spawn_test_enemy(&mut commands, spawn_pos, Faction::Syndicate, &sprites, &global_data);
        info!("Spawned Syndicate enemy at {:?}", spawn_pos);
    }
    
    // U - Spawn Police
    if input.just_pressed(KeyCode::KeyU) {
        let spawn_pos = camera_pos + Vec2::new(0.0, 100.0);
        spawn_test_enemy(&mut commands, spawn_pos, Faction::Police, &sprites, &global_data);
        info!("Spawned Police at {:?}", spawn_pos);
    }
    
    // C - Spawn Cover Point
    if input.just_pressed(KeyCode::KeyC) {
        let spawn_pos = camera_pos + Vec2::new(50.0, -50.0);
        spawn_test_cover(&mut commands, spawn_pos);
        info!("Spawned Cover Point at {:?}", spawn_pos);
    }
    
    // V - Spawn Vehicle Cover
    if input.just_pressed(KeyCode::KeyV) {
        let spawn_pos = camera_pos + Vec2::new(-50.0, -50.0);
        crate::systems::vehicles::spawn_vehicle(&mut commands, spawn_pos, VehicleType::APC, &sprites);
        info!("Spawned APC cover at {:?}", spawn_pos);
    }
}

fn spawn_test_enemy(
    commands: &mut Commands,
    position: Vec2,
    faction: Faction,
    sprites: &GameSprites,
    global_data: &GlobalData,
) {
    let (sprite, _) = crate::core::sprites::create_enemy_sprite(sprites);
    
    // Create small patrol area around spawn point
    let patrol_points = vec![
        position,
        position + Vec2::new(80.0, 0.0),
        position + Vec2::new(80.0, 80.0),
        position + Vec2::new(0.0, 80.0),
    ];
    
    let difficulty = global_data.regions[global_data.selected_region].mission_difficulty_modifier();
    
    // Vary weapon types by faction
    let base_weapon = match faction {
        Faction::Corporate => WeaponType::Rifle,
        Faction::Syndicate => WeaponType::Minigun,
        Faction::Police => WeaponType::Pistol,
        _ => WeaponType::Rifle,
    };
    
    let mut inventory = Inventory::default();
    inventory.equipped_weapon = Some(WeaponConfig::new(base_weapon.clone()));
    
    let test_enemy = commands.spawn_empty()
    .insert((
        sprite,
        Transform::from_translation(position.extend(1.0)),
        Enemy,
        faction, // Add faction component
        Health(100.0 * difficulty),
        Morale::new(100.0 * difficulty, 25.0),
        MovementSpeed(120.0 * difficulty),
        Vision::new(120.0 * difficulty, 45.0),
        Patrol::new(patrol_points),
    ))
    .insert((
        AIState::default(),
        GoapAgent::default(),
        WeaponState::new(&base_weapon),
        inventory,
        RigidBody::Dynamic,
        Collider::ball(9.0),
        Velocity::default(),
        Damping { linear_damping: 10.0, angular_damping: 10.0 },
    ));
}

fn spawn_test_cover(commands: &mut Commands, position: Vec2) {
    commands.spawn((
        Sprite {
            color: Color::srgba(0.4, 0.2, 0.1, 0.7),
            custom_size: Some(Vec2::new(20.0, 40.0)),
            ..default()
        },
        Transform::from_translation(position.extend(0.5)),
        CoverPoint {
            capacity: 2,
            current_users: 0,
            cover_direction: Vec2::X,
        },
    ));
}


// Enhanced cover visualization
pub fn cover_debug_system(
    mut gizmos: Gizmos,
    cover_query: Query<(&Transform, &CoverPoint)>,
    in_cover_query: Query<(&Transform, &InCover), With<Enemy>>,
    input: Res<ButtonInput<KeyCode>>,
    mut show_cover: Local<bool>,
) {
    // Toggle cover visualization with K
    if input.just_pressed(KeyCode::KeyK) {
        *show_cover = !*show_cover;
        info!("Cover visualization: {}", if *show_cover { "ON" } else { "OFF" });
    }
    
    if !*show_cover {
        return;
    }
    
    // Draw all cover points
    for (transform, cover_point) in cover_query.iter() {
        let pos = transform.translation.truncate();
        let usage_ratio = cover_point.current_users as f32 / cover_point.capacity as f32;
        
        let color = if usage_ratio >= 1.0 {
            Color::srgb(0.8, 0.2, 0.2) // Red when full
        } else if usage_ratio > 0.0 {
            Color::srgb(0.8, 0.8, 0.2) // Yellow when partially used
        } else {
            Color::srgb(0.2, 0.8, 0.2) // Green when available
        };
        
        // Draw cover outline
        gizmos.rect_2d(pos, Vec2::new(22.0, 42.0), color);
        
        // Draw capacity indicator
        let text_pos = pos + Vec2::new(0.0, 25.0);
        gizmos.circle_2d(text_pos, 8.0, Color::WHITE);
        
        // Draw usage text (simplified - just circles for current/max)
        for i in 0..cover_point.capacity {
            let dot_pos = text_pos + Vec2::new((i as f32 - 1.0) * 6.0, 0.0);
            let dot_color = if i < cover_point.current_users {
                Color::srgb(1.0, 0.3, 0.3) // Red for occupied
            } else {
                Color::srgb(0.3, 1.0, 0.3) // Green for available
            };
            gizmos.circle_2d(dot_pos, 2.0, dot_color);
        }
        
        // Draw cover direction arrow
        let arrow_end = pos + cover_point.cover_direction * 30.0;
        gizmos.line_2d(pos, arrow_end, Color::srgb(0.8, 0.8, 0.8));
        // Arrow head
        let arrow_side1 = arrow_end - cover_point.cover_direction * 8.0 + Vec2::new(-4.0, 4.0);
        let arrow_side2 = arrow_end - cover_point.cover_direction * 8.0 + Vec2::new(4.0, -4.0);
        gizmos.line_2d(arrow_end, arrow_side1, Color::srgb(0.8, 0.8, 0.8));
        gizmos.line_2d(arrow_end, arrow_side2, Color::srgb(0.8, 0.8, 0.8));
    }
    
    // Draw lines from enemies to their cover
    for (enemy_transform, in_cover) in in_cover_query.iter() {
        if let Ok((cover_transform, _)) = cover_query.get(in_cover.cover_entity) {
            gizmos.line_2d(
                enemy_transform.translation.truncate(),
                cover_transform.translation.truncate(),
                Color::srgb(0.6, 0.8, 0.6),
            );
        }
    }
}

// GOAP testing info display
pub fn goap_testing_info_system(
    input: Res<ButtonInput<KeyCode>>,
    mut show_info: Local<bool>,
    mut last_help_time: Local<f32>,
    time: Res<Time>,
) {
    *last_help_time += time.delta_secs();
    
    if input.just_pressed(KeyCode::KeyH) {
        *show_info = !*show_info;
        *last_help_time = 0.0;
        
        if *show_info {
            info!("=== GOAP TESTING CONTROLS ===");
            info!("T: Corporate Enemy (Red) | Y: Syndicate Enemy (Purple) | U: Police (Blue)");
            info!("C: Cover Point | V: Vehicle Cover | K: Cover Debug");
            info!("F4: GOAP Debug | H: Toggle this help");
            info!("");
            info!("EXPECTED BEHAVIOR:");
            info!("- Corporate vs Syndicate will fight each other");
            info!("- Police fight everyone");
            info!("- All factions fight player agents");
            info!("- Enemies use cover when outnumbered");
            info!("- GOAP shows decision-making process");
        } else {
            info!("GOAP testing help hidden. Press H to show again.");
        }
    }
    
    // Show periodic reminders
    if *show_info && *last_help_time > 30.0 {
        info!("Testing active - Press H to hide help, F4 for GOAP debug, K for cover debug");
        *last_help_time = 0.0;
    }
}

// Enhanced faction visualization
pub fn faction_visualization_system(
    mut gizmos: Gizmos,
    enemy_query: Query<(&Transform, &Faction, &Health), With<Enemy>>,
    agent_query: Query<(&Transform, &Faction), (With<Agent>, Without<Enemy>)>,
    input: Res<ButtonInput<KeyCode>>,
    mut show_factions: Local<bool>,
) {
    // Toggle faction visualization with J
    if input.just_pressed(KeyCode::KeyJ) {
        *show_factions = !*show_factions;
        info!("Faction visualization: {}", if *show_factions { "ON" } else { "OFF" });
    }
    
    if !*show_factions {
        return;
    }
    
    // Draw faction indicators above entities
    for (transform, faction) in agent_query.iter() {
        let pos = transform.translation.truncate() + Vec2::new(0.0, 35.0);
        gizmos.circle_2d(pos, 12.0, faction.color());
        
        // Draw "P" for player
        gizmos.line_2d(pos + Vec2::new(-3.0, -5.0), pos + Vec2::new(-3.0, 5.0), Color::WHITE);
        gizmos.line_2d(pos + Vec2::new(-3.0, 5.0), pos + Vec2::new(2.0, 5.0), Color::WHITE);
        gizmos.line_2d(pos + Vec2::new(2.0, 5.0), pos + Vec2::new(2.0, 0.0), Color::WHITE);
        gizmos.line_2d(pos + Vec2::new(2.0, 0.0), pos + Vec2::new(-3.0, 0.0), Color::WHITE);
    }
    
    for (transform, faction, health) in enemy_query.iter() {
        let pos = transform.translation.truncate() + Vec2::new(0.0, 35.0);
        let color = faction.color();
        
        // Adjust brightness based on health
        let health_ratio = (health.0 / 100.0).clamp(0.0, 1.0);
        let adjusted_color = Color::srgba(
            color.to_srgba().red * health_ratio + 0.3 * (1.0 - health_ratio),
            color.to_srgba().green * health_ratio + 0.3 * (1.0 - health_ratio),
            color.to_srgba().blue * health_ratio + 0.3 * (1.0 - health_ratio),
            1.0,
        );
        
        gizmos.circle_2d(pos, 12.0, adjusted_color);
        
        // Draw faction letter
        match faction {
            Faction::Corporate => {
                // Draw "C"
                gizmos.circle_2d(pos, 6.0, Color::BLACK);
                gizmos.circle_2d(pos + Vec2::new(2.0, 0.0), 6.0, adjusted_color);
            },
            Faction::Syndicate => {
                // Draw "S"  
                gizmos.line_2d(pos + Vec2::new(-3.0, 4.0), pos + Vec2::new(3.0, 4.0), Color::BLACK);
                gizmos.line_2d(pos + Vec2::new(-3.0, 0.0), pos + Vec2::new(3.0, 0.0), Color::BLACK);
                gizmos.line_2d(pos + Vec2::new(-3.0, -4.0), pos + Vec2::new(3.0, -4.0), Color::BLACK);
            },
            Faction::Police => {
                // Draw "P"
                gizmos.line_2d(pos + Vec2::new(-3.0, -5.0), pos + Vec2::new(-3.0, 5.0), Color::BLACK);
                gizmos.line_2d(pos + Vec2::new(-3.0, 5.0), pos + Vec2::new(2.0, 5.0), Color::BLACK);
                gizmos.line_2d(pos + Vec2::new(2.0, 5.0), pos + Vec2::new(2.0, 0.0), Color::BLACK);
                gizmos.line_2d(pos + Vec2::new(2.0, 0.0), pos + Vec2::new(-3.0, 0.0), Color::BLACK);
            },
            _ => {}
        }
    }
}

// Combat state visualization
pub fn combat_state_system(
    mut gizmos: Gizmos,
    enemy_query: Query<(&Transform, &AIState, &GoapAgent), With<Enemy>>,
    input: Res<ButtonInput<KeyCode>>,
    mut show_combat: Local<bool>,
) {
    // Toggle combat state visualization with M
    if input.just_pressed(KeyCode::KeyM) {
        *show_combat = !*show_combat;
        info!("Combat state visualization: {}", if *show_combat { "ON" } else { "OFF" });
    }
    
    if !*show_combat {
        return;
    }
    
    for (transform, ai_state, goap_agent) in enemy_query.iter() {
        let pos = transform.translation.truncate();
        
        // Draw AI state indicator
        let state_color = match ai_state.mode {
            crate::systems::ai::AIMode::Patrol => Color::srgb(0.2, 0.8, 0.2),
            crate::systems::ai::AIMode::Combat { .. } => Color::srgb(0.8, 0.2, 0.2),
            crate::systems::ai::AIMode::Investigate { .. } => Color::srgb(0.8, 0.8, 0.2),
            crate::systems::ai::AIMode::Search { .. } => Color::srgb(0.8, 0.5, 0.2),
            crate::systems::ai::AIMode::Panic => Color::srgb(0.8, 0.2, 0.8),
        };
        
        gizmos.circle_2d(pos + Vec2::new(-20.0, 20.0), 6.0, state_color);
        
        // Draw GOAP plan length
        let plan_length = goap_agent.current_plan.len();
        if plan_length > 0 {
            let bar_length = plan_length as f32 * 3.0;
            gizmos.line_2d(
                pos + Vec2::new(-15.0, -25.0),
                pos + Vec2::new(-15.0 + bar_length, -25.0),
                Color::srgb(0.3, 0.8, 0.8),
            );
        }
        
        // Draw goal indicator
        if let Some(goal) = &goap_agent.current_goal {
            let goal_color = match goal.name {
                "eliminate_threat" => Color::srgb(1.0, 0.2, 0.2),
                "investigate_disturbance" => Color::srgb(1.0, 0.8, 0.2),
                "patrol_area" => Color::srgb(0.2, 1.0, 0.2),
                "survival" => Color::srgb(1.0, 0.4, 0.0),
                _ => Color::WHITE,
            };
            gizmos.circle_2d(pos + Vec2::new(20.0, 20.0), 4.0, goal_color);
        }
    }
}
