// src/systems/testing_spawn.rs - On-demand enemy spawning for GOAP testing
use bevy::prelude::*;
use bevy_mod_imgui::prelude::*;
use bevy_rapier2d::prelude::*;
use crate::core::*;
use crate::core::factions::Faction;
use crate::systems::ai::*;

// Add this component for selected enemy debug
#[derive(Component)]
pub struct DebugSelected;

#[derive(Component)]
pub struct DebugInfoText;

#[derive(Resource)]
pub struct ImguiState {
    pub demo_window_open: bool,
}

pub fn testing_spawn_system(
    mut commands: Commands,
    input: Res<ButtonInput<KeyCode>>,
    sprites: Res<GameSprites>,
    global_data: Res<GlobalData>,
    game_state: Res<State<GameState>>,
    cameras: Query<&Transform, With<Camera>>,
    windows: Query<&Window>,
    camera_global_transform: Query<(&Camera, &GlobalTransform)>,
    enemy_query: Query<(Entity, &Transform), (With<Enemy>, Without<Dead>)>,
    selected_query: Query<Entity, With<DebugSelected>>,
    mouse: Res<ButtonInput<MouseButton>>,
) {
    if *game_state.get() != GameState::Mission {
        return;
    }

    let camera_pos = if let Ok(camera_transform) = cameras.single() {
        camera_transform.translation.truncate()
    } else {
        Vec2::ZERO
    };
    
    // Original spawn controls
    if input.just_pressed(KeyCode::KeyT) {
        let spawn_pos = camera_pos + Vec2::new(100.0, 50.0);
        spawn_test_enemy(&mut commands, spawn_pos, Faction::Corporate, &sprites, &global_data);
    }
    
    if input.just_pressed(KeyCode::KeyY) {
        let spawn_pos = camera_pos + Vec2::new(-100.0, 50.0);
        spawn_test_enemy(&mut commands, spawn_pos, Faction::Syndicate, &sprites, &global_data);
    }
    
    if input.just_pressed(KeyCode::KeyU) {
        let spawn_pos = camera_pos + Vec2::new(0.0, 100.0);
        spawn_test_enemy(&mut commands, spawn_pos, Faction::Police, &sprites, &global_data);
    }
    
    // NEW: Enemy selection for debug with right-click
    if input.just_pressed(KeyCode::ShiftLeft) && mouse.just_pressed(MouseButton::Left) {
        if let Some(world_pos) = get_world_mouse_position(&windows, &camera_global_transform) {
            // Clear previous selection
            for entity in selected_query.iter() {
                commands.entity(entity).remove::<DebugSelected>();
            }
            
            // Find closest enemy to mouse
            let mut closest_enemy = None;
            let mut closest_distance = f32::INFINITY;
            
            for (entity, transform) in enemy_query.iter() {
                let distance = world_pos.distance(transform.translation.truncate());
                if distance < 50.0 && distance < closest_distance {
                    closest_distance = distance;
                    closest_enemy = Some(entity);
                }
            }
            
            if let Some(enemy) = closest_enemy {
                commands.entity(enemy).insert(DebugSelected);
            }
        }
    }
    
    // Clear selection with X
    if input.just_pressed(KeyCode::KeyX) {
        for entity in selected_query.iter() {
            commands.entity(entity).remove::<DebugSelected>();
        }
    }
}
// Enhanced debug display system - shows GOAP info on screen
pub fn goap_debug_display_system(
    mut commands: Commands,
    selected_query: Query<(Entity, &Transform, &GoapAgent, &AIState, &Faction, &Health), (With<Enemy>, With<DebugSelected>)>,
    debug_text_query: Query<Entity, With<DebugInfoText>>,
    all_enemy_query: Query<(&Transform, &Faction), (With<Enemy>, Without<Dead>)>,
    agent_query: Query<&Transform, With<Agent>>,
) {
    // Clean up old debug text
    for entity in debug_text_query.iter() {
        commands.entity(entity).despawn();
    }
    
    // Show debug info for selected enemy
    if let Ok((entity, transform, goap_agent, ai_state, faction, health)) = selected_query.single() {
        let enemy_pos = transform.translation.truncate();
        
        // Count nearby entities by faction
        let mut nearby_info = String::new();
        for faction_type in [Faction::Player, Faction::Corporate, Faction::Syndicate, Faction::Police] {
            let count = if faction_type == Faction::Player {
                agent_query.iter()
                    .filter(|t| enemy_pos.distance(t.translation.truncate()) <= 150.0)
                    .count()
            } else {
                all_enemy_query.iter()
                    .filter(|(t, &f)| f == faction_type && enemy_pos.distance(t.translation.truncate()) <= 150.0)
                    .count()
            };
            
            if count > 0 {
                nearby_info.push_str(&format!("{:?}: {} ", faction_type, count));
            }
        }
        
        // Check for hostiles in range
        let hostile_count = agent_query.iter()
            .filter(|t| enemy_pos.distance(t.translation.truncate()) <= 150.0)
            .count() +
            all_enemy_query.iter()
                .filter(|(t, &other_faction)| {
                    faction.is_hostile_to(&other_faction) &&
                    enemy_pos.distance(t.translation.truncate()) <= 150.0
                })
                .count();
        
        let debug_info = format!(
            "ENEMY DEBUG (Entity {:?})\n\
            Faction: {:?} | Health: {:.0}\n\
            AI Mode: {:?}\n\
            GOAP Goal: {}\n\
            Plan Length: {}\n\
            World State:\n\
            • HasTarget: {}\n\
            • TargetVisible: {}\n\
            • IsAlert: {}\n\
            • HeardSound: {}\n\
            • IsPanicked: {}\n\
            • Outnumbered: {}\n\
            \nNearby Entities (150u): {}\n\
            Hostile Count: {}",
            entity.index(),
            faction,
            health.0,
            ai_state.mode,
            goap_agent.current_goal.as_ref().map(|g| g.name).unwrap_or("None"),
            goap_agent.current_plan.len(),
            goap_agent.world_state.get(&WorldKey::HasTarget).unwrap_or(&false),
            goap_agent.world_state.get(&WorldKey::TargetVisible).unwrap_or(&false),
            goap_agent.world_state.get(&WorldKey::IsAlert).unwrap_or(&false),
            goap_agent.world_state.get(&WorldKey::HeardSound).unwrap_or(&false),
            goap_agent.world_state.get(&WorldKey::IsPanicked).unwrap_or(&false),
            goap_agent.world_state.get(&WorldKey::Outnumbered).unwrap_or(&false),
            nearby_info,
            hostile_count
        );
        
        // Spawn debug text UI
        commands.spawn((
            Text::new(debug_info),
            TextFont { font_size: 14.0, ..default() },
            TextColor(Color::WHITE),
            Node {
                position_type: PositionType::Absolute,
                right: Val::Px(10.0),
                top: Val::Px(10.0),
                width: Val::Px(350.0),
                padding: UiRect::all(Val::Px(10.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.8)),
            ZIndex(1000),
            DebugInfoText,
        ));
    } else if selected_query.is_empty() {
        // Show help text when nothing selected
        commands.spawn((
            Text::new("GOAP Debug Help:\nShift+Click: Select Enemy\nX: Clear Selection\nT/Y/U: Spawn enemies"),
            TextFont { font_size: 12.0, ..default() },
            TextColor(Color::srgb(0.7, 0.7, 0.7)),
            Node {
                position_type: PositionType::Absolute,
                right: Val::Px(10.0),
                top: Val::Px(10.0),
                padding: UiRect::all(Val::Px(10.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.6)),
            ZIndex(1000),
            DebugInfoText,
        ));
    }
}

pub fn imgui_example_ui(mut context: NonSendMut<ImguiContext>, state: ResMut<ImguiState>) {
    let ui = context.ui();
    let window = ui.window("GOAP Debug Help");
    window
        .size([300.0,100.0], imgui::Condition::FirstUseEver)
        .position([0.0, 0.0], imgui::Condition::FirstUseEver)
        .build(|| {
            // ui.text(format!("Frame time: {:.2}ms", time.delta_seconds() * 1000.0));
            ui.text("Shift+Click: Select Enemy");
            ui.text("X: Clear Selection");
            ui.text("T/Y/U: Spawn enemies");
            ui.separator();
            let mouse_pos = ui.io().mouse_pos;
            ui.text(format!("Mouse Position: ({:.1},{:.1})", mouse_pos[0], mouse_pos[1]));
            ui.separator();
            // ui.text(format!("Faction: {}", if *show_factions { "ON" } else { "OFF" }));
            // ui.text(format!("Cover: {}", if *show_cover { "ON" } else { "OFF" }));
        });
}

// Visual indicator for selected enemy
pub fn debug_selection_visual_system(
    mut gizmos: Gizmos,
    selected_query: Query<&Transform, (With<Enemy>, With<DebugSelected>)>,
    time: Res<Time>,
) {
    for transform in selected_query.iter() {
        let pos = transform.translation.truncate();
        let pulse = (time.elapsed_secs() * 3.0).sin() * 0.3 + 0.7;
        let color = Color::srgba(1.0, 1.0, 0.0, pulse);
        
        // Pulsing selection ring
        gizmos.circle_2d(pos, 35.0, color);
        gizmos.circle_2d(pos, 40.0, Color::srgba(1.0, 1.0, 0.0, 0.3));
        
        // Selection arrows
        let arrow_size = 15.0;
        gizmos.line_2d(pos + Vec2::new(0.0, 50.0), pos + Vec2::new(0.0, 45.0), color);
        gizmos.line_2d(pos + Vec2::new(-5.0, 50.0), pos + Vec2::new(0.0, 45.0), color);
        gizmos.line_2d(pos + Vec2::new(5.0, 50.0), pos + Vec2::new(0.0, 45.0), color);
    }
}
fn spawn_test_enemy(
    commands: &mut Commands,
    position: Vec2,
    faction: Faction,
    sprites: &GameSprites,
    global_data: &GlobalData,
) {
    // Same as before but ensuring proper faction setup
    let (sprite, _) = crate::core::sprites::create_enemy_sprite(sprites);
    
    let patrol_points = vec![
        position,
        position + Vec2::new(80.0, 0.0),
        position + Vec2::new(80.0, 80.0),
        position + Vec2::new(0.0, 80.0),
    ];
    
    let difficulty = global_data.regions[global_data.selected_region].mission_difficulty_modifier();
    
    let base_weapon = match faction {
        Faction::Corporate => WeaponType::Rifle,
        Faction::Syndicate => WeaponType::Minigun,
        Faction::Police => WeaponType::Pistol,
        _ => WeaponType::Rifle,
    };
    
    let mut inventory = Inventory::default();
    inventory.equipped_weapon = Some(WeaponConfig::new(base_weapon.clone()));
    
    commands.spawn_empty()
    .insert((
        sprite,
        Transform::from_translation(position.extend(1.0)),
        Enemy,
        faction, // CRITICAL: Make sure faction is applied
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

pub fn simple_visual_debug_system(
    mut gizmos: Gizmos,
    agent_query: Query<&Transform, With<Agent>>,
    enemy_query: Query<(&Transform, &Faction), (With<Enemy>, With<Faction>)>,
    civilian_query: Query<&Transform, With<Civilian>>,
    input: Res<ButtonInput<KeyCode>>,
    show_debug: Local<bool>,
) {
    for transform in agent_query.iter() {
        let pos = transform.translation.truncate();
        gizmos.circle_2d(pos, 20.0, Color::srgb(0.0, 1.0, 0.0)); // Green for agents
        gizmos.circle_2d(pos, 15.0, Color::srgb(1.0, 1.0, 1.0)); // White inner
    }
    
    for (transform, faction) in enemy_query.iter() {
        let pos = transform.translation.truncate();
        let color = faction.color();
        gizmos.circle_2d(pos, 20.0, color); // Faction color
        gizmos.circle_2d(pos, 15.0, Color::srgb(0.0, 0.0, 0.0)); // Black inner
    }
    
    for transform in civilian_query.iter() {
        let pos = transform.translation.truncate();
        gizmos.circle_2d(pos, 20.0, Color::srgb(1.0, 1.0, 0.0)); // Yellow for civilians
        gizmos.circle_2d(pos, 15.0, Color::srgb(1.0, 1.0, 1.0)); // White inner
    }
}

pub fn patrol_debug_system(
    enemy_query: Query<(Entity, &Patrol, Option<&MoveTarget>), With<Enemy>>,
    input: Res<ButtonInput<KeyCode>>,
) {
    if input.just_pressed(KeyCode::F2) {
        for (entity, patrol, move_target) in enemy_query.iter() {
            info!("Enemy {}: Patrol points: {:?}, Current target: {:?}, Has MoveTarget: {}", 
                  entity.index(), 
                  patrol.points.len(),
                  patrol.current_target(),
                  move_target.is_some());
        }
    }
}