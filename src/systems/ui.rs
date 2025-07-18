use bevy::prelude::*;
use crate::core::*;
use crate::systems::*;

use crate::core::{PostMissionProcessed, PostMissionResults, GlobalData, UIState, GameState, experience_for_level};


pub fn system(
    mut gizmos: Gizmos,
    game_mode: Res<GameMode>,
    selected_query: Query<(&Transform, &NeurovectorCapability), (With<Agent>, With<Selected>)>,
    all_selected_query: Query<&Transform, (With<Agent>, With<Selected>)>, // For selection indicators
    target_query: Query<&Transform, With<NeurovectorTarget>>,
    controlled_query: Query<&Transform, With<NeurovectorControlled>>,
    enemy_query: Query<(&Transform, &Vision), With<Enemy>>,
    neurovector_query: Query<(&Transform, &NeurovectorCapability), With<Agent>>,
    selection: Res<SelectionState>,
) {
    // Draw selection indicators for all selected agents
    for transform in all_selected_query.iter() {
        let pos = transform.translation.truncate();
        
        // Draw selection circle
        gizmos.circle_2d(pos, 18.0, Color::srgb(0.2, 0.8, 0.2));
        
        // Draw selection corners for visual clarity
        let size = 12.0;
        let positions = [
            pos + Vec2::new(-size, -size),
            pos + Vec2::new(size, -size),
            pos + Vec2::new(size, size),
            pos + Vec2::new(-size, size),
        ];
        
        for corner in positions {
            gizmos.rect_2d(corner, 0.0, Vec2::new(3.0, 3.0), Color::srgb(0.2, 0.8, 0.2));
        }
    }

    // Draw neurovector ranges for selected agents
    for (transform, neurovector) in selected_query.iter() {
        let color = if neurovector.current_cooldown > 0.0 {
            Color::srgba(0.8, 0.3, 0.3, 0.3)
        } else {
            Color::srgba(0.3, 0.3, 0.8, 0.3)
        };
        
        gizmos.circle_2d(transform.translation.truncate(), neurovector.range, color);
    }

    // Draw formation indicators when multiple agents are selected
    if selection.selected.len() > 1 {
        draw_formation_indicators(&mut gizmos, &all_selected_query, &selection);
    }

    // Highlight targets when in neurovector targeting mode
    if let Some(TargetingMode::Neurovector { agent }) = &game_mode.targeting {
        if let Ok((agent_transform, neurovector)) = neurovector_query.get(*agent) {
            for target_transform in target_query.iter() {
                let distance = agent_transform.translation.truncate()
                    .distance(target_transform.translation.truncate());
                
                if distance <= neurovector.range {
                    gizmos.circle_2d(
                        target_transform.translation.truncate(),
                        20.0,
                        Color::srgb(0.8, 0.8, 0.3),
                    );
                }
            }
        }
    }

    // Draw control connections
    for (agent_transform, neurovector) in neurovector_query.iter() {
        for &controlled_entity in &neurovector.controlled {
            if let Ok(controlled_transform) = controlled_query.get(controlled_entity) {
                gizmos.line_2d(
                    agent_transform.translation.truncate(),
                    controlled_transform.translation.truncate(),
                    Color::srgb(0.8, 0.3, 0.8),
                );
            }
        }
    }

    // Draw enemy vision cones
    for (transform, vision) in enemy_query.iter() {
        draw_vision_cone(&mut gizmos, transform.translation.truncate(), vision);
    }
}

fn draw_formation_indicators(
    gizmos: &mut Gizmos,
    selected_query: &Query<&Transform, (With<Agent>, With<Selected>)>,
    selection: &SelectionState,
) {
    if selection.selected.len() < 2 { return; }
    
    // Calculate center of selected agents
    let mut center = Vec2::ZERO;
    let mut count = 0;
    
    for transform in selected_query.iter() {
        center += transform.translation.truncate();
        count += 1;
    }
    
    if count > 0 {
        center /= count as f32;
        
        // Draw formation center
        gizmos.circle_2d(center, 8.0, Color::srgba(0.8, 0.8, 0.2, 0.5));
        
        // Draw lines connecting selected agents
        let positions: Vec<Vec2> = selected_query.iter()
            .map(|t| t.translation.truncate())
            .collect();
        
        for i in 0..positions.len() {
            for j in (i + 1)..positions.len() {
                gizmos.line_2d(
                    positions[i],
                    positions[j],
                    Color::srgba(0.8, 0.8, 0.2, 0.3),
                );
            }
        }
    }
}


#[derive(Component)]
pub struct PauseUI;

// === PAUSE UI ===
#[derive(Component)]
pub struct PauseScreen;

pub fn pause_system(
    mut commands: Commands,
    game_mode: Res<GameMode>,
    screen_query: Query<Entity, With<PauseScreen>>,
) {
    let should_show = game_mode.paused;
    let ui_exists = !screen_query.is_empty();
    
    if should_show && !ui_exists {
        commands.spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                background_color: Color::srgba(0.0, 0.0, 0.0, 0.5).into(),
                z_index: ZIndex::Global(100),
                ..default()
            },
            PauseScreen,
        )).with_children(|parent| {
            parent.spawn(TextBundle::from_section(
                "PAUSED\nPress SPACE to resume",
                TextStyle { font_size: 32.0, color: Color::WHITE, ..default() }
            ));
        });
    } else if !should_show && ui_exists {
        for entity in screen_query.iter() {
            commands.entity(entity).despawn_recursive();
        }
    }
}

/*
pub fn pause_system(
    mut commands: Commands,
    game_mode: Res<GameMode>,
    mut ui_state: ResMut<UIState>,
    pause_ui_query: Query<Entity, With<PauseUI>>,
) {
    let should_show = game_mode.paused;
    
    if ui_state.pause_open != should_show {
        ui_state.pause_open = should_show;
        
        // Clear existing UI
        for entity in pause_ui_query.iter() {
            commands.entity(entity).despawn_recursive();
        }
        
        if should_show {
            commands.spawn((
                NodeBundle {
                    style: Style {
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        position_type: PositionType::Absolute,
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    background_color: Color::srgba(0.0, 0.0, 0.0, 0.5).into(),
                    z_index: ZIndex::Global(100),
                    ..default()
                },
                PauseUI,
            )).with_children(|parent| {
                parent.spawn(TextBundle::from_section(
                    "PAUSED\nPress SPACE to resume",
                    TextStyle {
                        font_size: 32.0,
                        color: Color::WHITE,
                        ..default()
                    },
                ));
            });
        }
    }
}
*/


pub fn inventory_system(
    mut commands: Commands,
    inventory_state: Res<InventoryState>,
    agent_query: Query<&Inventory, (With<Agent>, Changed<Inventory>)>, // Only when changed
    all_agent_query: Query<&Inventory, With<Agent>>, // For initial display
    inventory_ui_query: Query<Entity, With<InventoryUI>>,
) {
    // Handle open/close
    if !inventory_state.ui_open {
        if !inventory_ui_query.is_empty() {
            for entity in inventory_ui_query.iter() {
                commands.entity(entity).despawn_recursive();
            }
        }
        return;
    }
    
    // Check if we need to update - either UI doesn't exist or inventory changed
    let needs_update = inventory_ui_query.is_empty() || 
        (inventory_state.selected_agent.is_some() && 
         agent_query.get(inventory_state.selected_agent.unwrap()).is_ok());
    
    if needs_update {
        // Clear existing UI
        for entity in inventory_ui_query.iter() {
            commands.entity(entity).despawn_recursive();
        }
        
        // Get current inventory data
        let inventory = inventory_state.selected_agent
            .and_then(|agent| all_agent_query.get(agent).ok());
        
        // Recreate UI with updated data
        create_inventory_ui(&mut commands, inventory);
    }
}

fn create_inventory_ui(commands: &mut Commands, inventory: Option<&Inventory>) {
    commands.spawn((
        NodeBundle {
            style: Style {
                width: Val::Px(400.0),
                height: Val::Px(500.0),
                position_type: PositionType::Absolute,
                left: Val::Px(50.0),
                top: Val::Px(50.0),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(10.0)),
                row_gap: Val::Px(10.0),
                ..default()
            },
            background_color: Color::srgba(0.1, 0.1, 0.1, 0.9).into(),
            z_index: ZIndex::Global(50),
            ..default()
        },
        InventoryUI,
    )).with_children(|parent| {
        // Title
        parent.spawn(TextBundle::from_section(
            "AGENT INVENTORY",
            TextStyle { font_size: 24.0, color: Color::WHITE, ..default() }
        ));
        
        if let Some(inv) = inventory {
            // Currency
            parent.spawn(TextBundle::from_section(
                format!("Credits: {}", inv.currency),
                TextStyle { font_size: 18.0, color: Color::srgb(0.8, 0.8, 0.2), ..default() }
            ));
            
            // Equipped weapon
            let weapon_text = if let Some(weapon) = &inv.equipped_weapon {
                format!("EQUIPPED WEAPON: {:?}", weapon)
            } else {
                "EQUIPPED WEAPON: None".to_string()
            };
            parent.spawn(TextBundle::from_section(
                weapon_text,
                TextStyle { font_size: 16.0, color: Color::srgb(0.9, 0.5, 0.2), ..default() }
            ));
            
            // Equipped tools
            if !inv.equipped_tools.is_empty() {
                parent.spawn(TextBundle::from_section(
                    format!("EQUIPPED TOOLS: {:?}", inv.equipped_tools),
                    TextStyle { font_size: 16.0, color: Color::srgb(0.3, 0.8, 0.3), ..default() }
                ));
            }
            
            // Weapons section
            if !inv.weapons.is_empty() {
                parent.spawn(NodeBundle {
                    style: Style {
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(2.0),
                        ..default()
                    },
                    ..default()
                }).with_children(|weapons| {
                    weapons.spawn(TextBundle::from_section(
                        "WEAPONS:",
                        TextStyle { font_size: 16.0, color: Color::srgb(0.8, 0.3, 0.3), ..default() }
                    ));
                    for weapon in &inv.weapons {
                        weapons.spawn(TextBundle::from_section(
                            format!("• {:?}", weapon),
                            TextStyle { font_size: 14.0, color: Color::WHITE, ..default() }
                        ));
                    }
                });
            }
            
            // Tools section
            if !inv.tools.is_empty() {
                parent.spawn(NodeBundle {
                    style: Style {
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(2.0),
                        ..default()
                    },
                    ..default()
                }).with_children(|tools| {
                    tools.spawn(TextBundle::from_section(
                        "TOOLS:",
                        TextStyle { font_size: 16.0, color: Color::srgb(0.3, 0.8, 0.3), ..default() }
                    ));
                    for tool in &inv.tools {
                        tools.spawn(TextBundle::from_section(
                            format!("• {:?}", tool),
                            TextStyle { font_size: 14.0, color: Color::WHITE, ..default() }
                        ));
                    }
                });
            }
            
            // Cybernetics section
            if !inv.cybernetics.is_empty() {
                parent.spawn(NodeBundle {
                    style: Style {
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(2.0),
                        ..default()
                    },
                    ..default()
                }).with_children(|cyber| {
                    cyber.spawn(TextBundle::from_section(
                        "CYBERNETICS:",
                        TextStyle { font_size: 16.0, color: Color::srgb(0.3, 0.3, 0.8), ..default() }
                    ));
                    for cybernetic in &inv.cybernetics {
                        cyber.spawn(TextBundle::from_section(
                            format!("• {:?}", cybernetic),
                            TextStyle { font_size: 14.0, color: Color::WHITE, ..default() }
                        ));
                    }
                });
            }
            
            // Intel section
            if !inv.intel_documents.is_empty() {
                parent.spawn(NodeBundle {
                    style: Style {
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(2.0),
                        ..default()
                    },
                    ..default()
                }).with_children(|intel| {
                    intel.spawn(TextBundle::from_section(
                        "INTEL DOCUMENTS:",
                        TextStyle { font_size: 16.0, color: Color::srgb(0.8, 0.8, 0.3), ..default() }
                    ));
                    for (i, document) in inv.intel_documents.iter().enumerate() {
                        let preview = if document.len() > 50 {
                            format!("{}...", &document[..47])
                        } else {
                            document.clone()
                        };
                        intel.spawn(TextBundle::from_section(
                            format!("• Doc {}: {}", i + 1, preview),
                            TextStyle { font_size: 12.0, color: Color::WHITE, ..default() }
                        ));
                    }
                });
            }
        } else {
            parent.spawn(TextBundle::from_section(
                "No agent selected",
                TextStyle { font_size: 16.0, color: Color::srgb(0.8, 0.3, 0.3), ..default() }
            ));
        }
        
        // Instructions
        parent.spawn(TextBundle::from_section(
            "Press 'I' to close inventory",
            TextStyle { font_size: 12.0, color: Color::srgb(0.7, 0.7, 0.7), ..default() }
        ));
    });
}

// === POST MISSION UI ===
#[derive(Component)]
pub struct PostMissionScreen;

pub fn post_mission_ui_system(
    mut commands: Commands,
    mut next_state: ResMut<NextState<GameState>>,
    mut processed: ResMut<PostMissionProcessed>,
    mut ui_state: ResMut<UIState>,
    post_mission: Res<PostMissionResults>,
    global_data: Res<GlobalData>,
    input: Res<ButtonInput<KeyCode>>,
    screen_query: Query<Entity, With<PostMissionScreen>>,
) {
    // Handle input
    if input.just_pressed(KeyCode::KeyR) {
        for entity in screen_query.iter() {
            commands.entity(entity).despawn_recursive();
        }
        processed.0 = false;
        ui_state.global_map_open = false;
        next_state.set(GameState::GlobalMap);
        return;
    }
    
    if input.just_pressed(KeyCode::Escape) {
        std::process::exit(0);
    }
    
    // Create UI if it doesn't exist
    if screen_query.is_empty() {
        create_post_mission_ui(&mut commands, &post_mission, &global_data);
    }
}

fn create_post_mission_ui(
    commands: &mut Commands,
    post_mission: &PostMissionResults,
    global_data: &GlobalData,
) {
    let (title, title_color) = if post_mission.success {
        ("MISSION SUCCESS", Color::srgb(0.2, 0.8, 0.2))
    } else {
        ("MISSION FAILED", Color::srgb(0.8, 0.2, 0.2))
    };
    
    commands.spawn((
        NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                row_gap: Val::Px(20.0),
                ..default()
            },
            background_color: Color::srgba(0.0, 0.0, 0.0, 0.8).into(),
            ..default()
        },
        PostMissionScreen,
    )).with_children(|parent| {
        // Title
        parent.spawn(TextBundle::from_section(
            title,
            TextStyle { font_size: 48.0, color: title_color, ..default() }
        ));
        
        // Stats grid
        parent.spawn(NodeBundle {
            style: Style {
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(10.0),
                ..default()
            },
            ..default()
        }).with_children(|stats| {
            let stat_entries = [
                ("Time", format!("{:.1}s", post_mission.time_taken)),
                ("Enemies", post_mission.enemies_killed.to_string()),
                ("Terminals", post_mission.terminals_accessed.to_string()),
                ("Credits", post_mission.credits_earned.to_string()),
                ("Alert", format!("{:?}", post_mission.alert_level)),
            ];
            
            for (label, value) in stat_entries {
                stats.spawn(NodeBundle {
                    style: Style {
                        flex_direction: FlexDirection::Row,
                        column_gap: Val::Px(20.0),
                        ..default()
                    },
                    ..default()
                }).with_children(|row| {
                    row.spawn(TextBundle::from_section(
                        format!("{}:", label),
                        TextStyle { font_size: 24.0, color: Color::srgb(0.7, 0.7, 0.7), ..default() }
                    ));
                    row.spawn(TextBundle::from_section(
                        value,
                        TextStyle { font_size: 24.0, color: Color::WHITE, ..default() }
                    ));
                });
            }
        });
        
        // Agent progression (if successful)
        if post_mission.success {
            parent.spawn(NodeBundle {
                style: Style {
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(5.0),
                    ..default()
                },
                ..default()
            }).with_children(|progression| {
                let exp = 10 + (post_mission.enemies_killed * 5);
                progression.spawn(TextBundle::from_section(
                    format!("EXP GAINED: {}", exp),
                    TextStyle { font_size: 20.0, color: Color::srgb(0.2, 0.8, 0.8), ..default() }
                ));
                
                for i in 0..3 {
                    progression.spawn(TextBundle::from_section(
                        format!("Agent {}: Lv{} ({}/{})", 
                            i + 1, 
                            global_data.agent_levels[i],
                            global_data.agent_experience[i],
                            experience_for_level(global_data.agent_levels[i] + 1)
                        ),
                        TextStyle { font_size: 16.0, color: Color::WHITE, ..default() }
                    ));
                }
            });
        }
        
        // Controls
        parent.spawn(TextBundle::from_section(
            "R: Return to Map | ESC: Quit",
            TextStyle { font_size: 16.0, color: Color::srgb(0.7, 0.7, 0.7), ..default() }
        ));
    });
}

/*
pub fn post_mission_system(
    mut commands: Commands,
    post_mission: Res<PostMissionResults>,
    mut next_state: ResMut<NextState<GameState>>,
    mut global_data: ResMut<GlobalData>,
    mut processed: ResMut<PostMissionProcessed>,
    agent_query: Query<&Agent>,
    input: Res<ButtonInput<KeyCode>>,
    ui_query: Query<Entity, With<PostMissionUI>>,
) {
    // Only process mission results once
    if !processed.0 {
        update_global_data_with_mission_results(&mut global_data, &post_mission, &agent_query);
        processed.0 = true;
    }
    
    // Clear existing UI
    for entity in ui_query.iter() {
        commands.entity(entity).despawn_recursive();
    }
    
    // Create results screen
    commands.spawn((
        NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                position_type: PositionType::Absolute,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                ..default()
            },
            background_color: Color::srgba(0.0, 0.0, 0.0, 0.8).into(),
            z_index: ZIndex::Global(200),
            ..default()
        },
        PostMissionUI,
    )).with_children(|parent| {
        // Title
        let title = if post_mission.success { "MISSION SUCCESS" } else { "MISSION FAILED" };
        let title_color = if post_mission.success { Color::srgb(0.2, 0.8, 0.2) } else { Color::srgb(0.8, 0.2, 0.2) };
        
        parent.spawn(TextBundle::from_section(
            title,
            TextStyle {
                font_size: 48.0,
                color: title_color,
                ..default()
            },
        ));
        
        // Stats
        parent.spawn(TextBundle::from_section(
            format!(
                "\nTime: {:.1}s\nEnemies Eliminated: {}\nTerminals Accessed: {}\nCredits Earned: {}\nAlert Level: {:?}",
                post_mission.time_taken,
                post_mission.enemies_killed,
                post_mission.terminals_accessed,
                post_mission.credits_earned,
                post_mission.alert_level
            ),
            TextStyle {
                font_size: 24.0,
                color: Color::WHITE,
                ..default()
            },
        ));
        
        // Agent progression (only if successful)
        if post_mission.success {
            let exp_gained = 10 + (post_mission.enemies_killed * 5);
            parent.spawn(TextBundle::from_section(
                format!("\nAGENT PROGRESSION:\nExperience Gained: {}", exp_gained),
                TextStyle {
                    font_size: 20.0,
                    color: Color::srgb(0.2, 0.8, 0.8),
                    ..default()
                },
            ));
            
            for i in 0..3 {
                let level_text = format!(
                    "Agent {}: Level {} ({}/{})",
                    i + 1,
                    global_data.agent_levels[i],
                    global_data.agent_experience[i],
                    experience_for_level(global_data.agent_levels[i] + 1)
                );
                
                parent.spawn(TextBundle::from_section(
                    level_text,
                    TextStyle {
                        font_size: 16.0,
                        color: Color::WHITE,
                        ..default()
                    },
                ));
            }
        }
        
        parent.spawn(TextBundle::from_section(
            "\nPress R to return to map\nPress ESC to quit",
            TextStyle {
                font_size: 16.0,
                color: Color::srgb(0.7, 0.7, 0.7),
                ..default()
            },
        ));
    });
    
    // Handle input
    if input.just_pressed(KeyCode::KeyR) {
        processed.0 = false; // Reset for next mission
        next_state.set(GameState::GlobalMap);
        info!("Returning to global map...");
    }
    
    if input.just_pressed(KeyCode::Escape) {
        std::process::exit(0);
    }
}
*/


fn update_global_data_with_mission_results(
    global_data: &mut GlobalData,
    post_mission: &PostMissionResults,
    agent_query: &Query<&Agent>,
) {
    let selected_region = global_data.selected_region;
    
    if post_mission.success {
        global_data.credits += post_mission.credits_earned;
        global_data.current_day += 1;
        let new_day = global_data.current_day;
        
        // Award experience and set recovery time based on mission difficulty
        let experience_gained = 10 + (post_mission.enemies_killed * 5);
        let recovery_days = if post_mission.time_taken > 240.0 { 2 } else { 1 }; // Longer missions = more recovery
        
        for (i, _agent) in agent_query.iter().enumerate() {
            if i < 3 {
                global_data.agent_experience[i] += experience_gained;
                global_data.agent_recovery[i] = new_day + recovery_days;
                
                // Check for level up
                let current_level = global_data.agent_levels[i];
                let required_exp = experience_for_level(current_level + 1);
                if global_data.agent_experience[i] >= required_exp && current_level < 10 {
                    global_data.agent_levels[i] += 1;
                    info!("Agent {} leveled up to level {}!", i + 1, global_data.agent_levels[i]);
                }
            }
        }
        
        // Successful stealth missions may reduce alert (if fast and no kills)
        if post_mission.enemies_killed == 0 && post_mission.time_taken < 180.0 {
            // Perfect stealth - no alert increase, may even reduce
        } else {
            // Normal success still raises alert slightly
            global_data.regions[selected_region].raise_alert(new_day);
        }
    } else {
        // Failed missions raise alert significantly and advance time
        global_data.current_day += 1;
        let new_day = global_data.current_day;
        global_data.regions[selected_region].raise_alert(new_day);
        global_data.regions[selected_region].raise_alert(new_day); // Double penalty for failure
        
        for i in 0..3 {
            global_data.agent_recovery[i] = new_day + 3; // Longer recovery on failure
        }
    }
    
    // Update all region alert levels for decay
    let current_day = global_data.current_day;
    for region in &mut global_data.regions {
        region.update_alert(current_day);
    }
}

#[derive(Component)]
pub struct PostMissionUI;

pub fn fps_system(
    mut commands: Commands,
    ui_state: Res<UIState>,
    fps_query: Query<Entity, With<FpsText>>,
    mut fps_text_query: Query<&mut Text, With<FpsText>>,
    time: Res<Time>,
) {
    // Handle FPS counter visibility
    if ui_state.fps_visible && fps_query.is_empty() {
        // Create FPS counter
        info!("Creating FPS counter");
        commands.spawn((
            TextBundle::from_section(
                "FPS: --",
                TextStyle {
                    font_size: 20.0,
                    color: Color::srgb(0.0, 1.0, 0.0),
                    ..default()
                },
            )
            .with_style(Style {
                position_type: PositionType::Absolute,
                top: Val::Px(10.0),
                left: Val::Px(10.0),
                ..default()
            }),
            FpsText,
        ));
    } else if !ui_state.fps_visible && !fps_query.is_empty() {
        // Remove FPS counter
        info!("Removing FPS counter");
        for entity in fps_query.iter() {
            commands.entity(entity).despawn_recursive();
        }
    } else if ui_state.fps_visible && !fps_query.is_empty() {
        // Update FPS counter
        let fps = 1.0 / time.delta_seconds();
        if let Ok(mut text) = fps_text_query.get_single_mut() {
            text.sections[0].value = format!("FPS: {:.0}", fps);
        }
    }
}

/*
pub fn global_map_system(
    mut commands: Commands,
    mut next_state: ResMut<NextState<GameState>>,
    mut global_data: ResMut<GlobalData>,
    mut ui_state: ResMut<UIState>,
    input: Res<ButtonInput<KeyCode>>,
    ui_query: Query<Entity, With<GlobalMapUI>>,
) {
    // Handle input first
    let mut needs_rebuild = false;
    
    if input.just_pressed(KeyCode::ArrowUp) {
        if global_data.selected_region > 0 {
            global_data.selected_region -= 1;
            needs_rebuild = true;
        }
    }
    
    if input.just_pressed(KeyCode::ArrowDown) {
        if global_data.selected_region < global_data.regions.len() - 1 {
            global_data.selected_region += 1;
            needs_rebuild = true;
        }
    }
    
    if input.just_pressed(KeyCode::KeyW) {
        global_data.current_day += 1;
        
        // Update all region alert levels for decay
        let current_day = global_data.current_day;
        for region in &mut global_data.regions {
            region.update_alert(current_day);
        }
        
        needs_rebuild = true;
        info!("Waited 1 day. Current day: {}", current_day);
    }
    
    if input.just_pressed(KeyCode::Enter) {
        let ready_agents = (0..3).filter(|&i| global_data.agent_recovery[i] <= global_data.current_day).count();
        if ready_agents > 0 {
            commands.insert_resource(ShouldRestart);
            next_state.set(GameState::Mission);
            ui_state.global_map_open = false;
            info!("Starting mission with {} agents in: {}", ready_agents, global_data.regions[global_data.selected_region].name);
            return;
        } else {
            info!("No agents ready for deployment!");
        }
    }
    
    if input.just_pressed(KeyCode::Escape) {
        std::process::exit(0);
    }
    
    // Rebuild UI if needed
    if !ui_state.global_map_open || needs_rebuild {
        ui_state.global_map_open = true;
        
        // Clear existing UI
        for entity in ui_query.iter() {
            commands.entity(entity).despawn_recursive();
        }
        
        create_global_map_ui(&mut commands, &global_data);
    }
}

fn create_global_map_ui(commands: &mut Commands, global_data: &GlobalData) {
    
    // Create global map UI
    commands.spawn((
        NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                position_type: PositionType::Absolute,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                ..default()
            },
            background_color: Color::srgb(0.1, 0.1, 0.2).into(),
            ..default()
        },
        GlobalMapUI,
    )).with_children(|parent| {
        // Title and day counter
        parent.spawn(TextBundle::from_section(
            format!("SUBVERSIVE - GLOBAL MAP\nDay {}", global_data.current_day),
            TextStyle {
                font_size: 36.0,
                color: Color::WHITE,
                ..default()
            },
        ));
        
        // Credits
        parent.spawn(TextBundle::from_section(
            format!("Credits: {}", global_data.credits),
            TextStyle {
                font_size: 20.0,
                color: Color::srgb(0.8, 0.8, 0.2),
                ..default()
            },
        ));
        
        // Region selection
        parent.spawn(TextBundle::from_section(
            "\nSELECT REGION:",
            TextStyle {
                font_size: 24.0,
                color: Color::WHITE,
                ..default()
            },
        ));
        
        // Agent roster
        parent.spawn(TextBundle::from_section(
            "\nAGENT ROSTER:",
            TextStyle {
                font_size: 20.0,
                color: Color::WHITE,
                ..default()
            },
        ));
        
        for i in 0..3 {
            let level = global_data.agent_levels[i];
            let is_recovering = global_data.agent_recovery[i] > global_data.current_day;
            let recovery_days = if is_recovering { 
                global_data.agent_recovery[i] - global_data.current_day 
            } else { 0 };
            
            let level_color = if is_recovering {
                Color::srgb(0.5, 0.5, 0.5) // Gray when recovering
            } else {
                match level {
                    1 => Color::srgb(0.2, 0.8, 0.2),     // Green - Rookie
                    2 => Color::srgb(0.2, 0.2, 0.8),     // Blue - Veteran
                    3 => Color::srgb(0.8, 0.2, 0.8),     // Purple - Elite
                    _ => Color::srgb(0.8, 0.8, 0.2),     // Gold - Master
                }
            };
            
            let status_text = if is_recovering {
                format!("Agent {}: Level {} - RECOVERING ({} days)", i + 1, level, recovery_days)
            } else {
                format!("Agent {}: Level {} - READY", i + 1, level)
            };
            
            parent.spawn(TextBundle::from_section(
                status_text,
                TextStyle {
                    font_size: 16.0,
                    color: level_color,
                    ..default()
                },
            ));
        }
        for (i, region) in global_data.regions.iter().enumerate() {
            let is_selected = i == global_data.selected_region;
            let color = if is_selected { Color::srgb(0.2, 0.8, 0.2) } else { Color::WHITE };
            let prefix = if is_selected { "> " } else { "  " };
            
            parent.spawn(TextBundle::from_section(
                format!("{}{} (Threat: {})", prefix, region.name, region.threat_level),
                TextStyle {
                    font_size: 18.0,
                    color,
                    ..default()
                },
            ));
        }
        
        // Controls
        let any_agents_recovering = (0..3).any(|i| global_data.agent_recovery[i] > global_data.current_day);
        let controls_text = if any_agents_recovering {
            "\nUP/DOWN: Select Region\nW: Wait 1 Day\nENTER: Start Mission (Recovering agents won't deploy)\nESC: Quit"
        } else {
            "\nUP/DOWN: Select Region\nW: Wait 1 Day\nENTER: Start Mission\nESC: Quit"
        };
        
        parent.spawn(TextBundle::from_section(
            controls_text,
            TextStyle {
                font_size: 16.0,
                color: Color::srgb(0.7, 0.7, 0.7),
                ..default()
            },
        ));

        parent.spawn(TextBundle::from_section(
            "\nF5: Save Game",
            TextStyle {
                font_size: 14.0,
                color: Color::srgb(0.5, 0.8, 0.5),
                ..default()
            },
        ));        
    });
}
*/

// === GLOBAL MAP UI ===
#[derive(Component)]
pub struct GlobalMapScreen;

pub fn global_map_system(
    mut commands: Commands,
    mut next_state: ResMut<NextState<GameState>>,
    mut global_data: ResMut<GlobalData>,
    input: Res<ButtonInput<KeyCode>>,
    screen_query: Query<Entity, With<GlobalMapScreen>>,
) {
    // Handle input
    if input.just_pressed(KeyCode::ArrowUp) && global_data.selected_region > 0 {
        global_data.selected_region -= 1;
        rebuild_global_map(&mut commands, &screen_query, &global_data);
    }
    
    if input.just_pressed(KeyCode::ArrowDown) && global_data.selected_region < global_data.regions.len() - 1 {
        global_data.selected_region += 1;
        rebuild_global_map(&mut commands, &screen_query, &global_data);
    }
    
    if input.just_pressed(KeyCode::KeyW) {
        global_data.current_day += 1;
        let current_day = global_data.current_day; // Store the value
        for region in &mut global_data.regions {
            region.update_alert(current_day);
        }
        rebuild_global_map(&mut commands, &screen_query, &global_data);
    }
    
    if input.just_pressed(KeyCode::Enter) {
        let ready_agents = (0..3).filter(|&i| global_data.agent_recovery[i] <= global_data.current_day).count();
        if ready_agents > 0 {
            commands.insert_resource(ShouldRestart);
            next_state.set(GameState::Mission);
        }
    }
    
    if input.just_pressed(KeyCode::Escape) {
        std::process::exit(0);
    }
    
    // Create UI if it doesn't exist
    if screen_query.is_empty() {
        create_global_map_ui(&mut commands, &global_data);
    }
}

fn rebuild_global_map(
    commands: &mut Commands,
    screen_query: &Query<Entity, With<GlobalMapScreen>>,
    global_data: &GlobalData,
) {
    for entity in screen_query.iter() {
        commands.entity(entity).despawn_recursive();
    }
    create_global_map_ui(commands, global_data);
}

fn create_global_map_ui(commands: &mut Commands, global_data: &GlobalData) {
    commands.spawn((
        NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                row_gap: Val::Px(20.0),
                ..default()
            },
            background_color: Color::srgb(0.1, 0.1, 0.2).into(),
            ..default()
        },
        GlobalMapScreen,
    )).with_children(|parent| {
        // Title
        parent.spawn(TextBundle::from_section(
            format!("SUBVERSIVE - Day {}", global_data.current_day),
            TextStyle { font_size: 36.0, color: Color::WHITE, ..default() }
        ));
        
        // Credits
        parent.spawn(TextBundle::from_section(
            format!("Credits: {}", global_data.credits),
            TextStyle { font_size: 20.0, color: Color::srgb(0.8, 0.8, 0.2), ..default() }
        ));
        
        // Agent roster
        parent.spawn(NodeBundle {
            style: Style {
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(5.0),
                ..default()
            },
            ..default()
        }).with_children(|agents| {
            agents.spawn(TextBundle::from_section(
                "AGENT ROSTER:",
                TextStyle { font_size: 20.0, color: Color::WHITE, ..default() }
            ));
            
            for i in 0..3 {
                let is_recovering = global_data.agent_recovery[i] > global_data.current_day;
                let color = if is_recovering { Color::srgb(0.5, 0.5, 0.5) } else { Color::srgb(0.2, 0.8, 0.2) };
                let status = if is_recovering {
                    format!("Agent {}: Level {} - RECOVERING", i + 1, global_data.agent_levels[i])
                } else {
                    format!("Agent {}: Level {} - READY", i + 1, global_data.agent_levels[i])
                };
                
                agents.spawn(TextBundle::from_section(
                    status,
                    TextStyle { font_size: 16.0, color, ..default() }
                ));
            }
        });
        
        // Regions
        parent.spawn(NodeBundle {
            style: Style {
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(5.0),
                ..default()
            },
            ..default()
        }).with_children(|regions| {
            regions.spawn(TextBundle::from_section(
                "SELECT REGION:",
                TextStyle { font_size: 24.0, color: Color::WHITE, ..default() }
            ));
            
            for (i, region) in global_data.regions.iter().enumerate() {
                let is_selected = i == global_data.selected_region;
                let color = if is_selected { Color::srgb(0.2, 0.8, 0.2) } else { Color::WHITE };
                let prefix = if is_selected { "> " } else { "  " };
                
                regions.spawn(TextBundle::from_section(
                    format!("{}{} (Threat: {})", prefix, region.name, region.threat_level),
                    TextStyle { font_size: 18.0, color, ..default() }
                ));
            }
        });
        
        // Controls
        parent.spawn(TextBundle::from_section(
            "UP/DOWN: Select | W: Wait Day | ENTER: Start Mission | F5: Save | ESC: Quit",
            TextStyle { font_size: 16.0, color: Color::srgb(0.7, 0.7, 0.7), ..default() }
        ));
    });
}



fn draw_vision_cone(gizmos: &mut Gizmos, position: Vec2, vision: &Vision) {
    let half_angle = vision.angle / 2.0;
    let segments = 16;
    
    let color = Color::srgba(1.0, 1.0, 0.3, 0.2);
    
    // Draw cone outline
    for i in 0..segments {
        let t1 = i as f32 / segments as f32;
        let t2 = (i + 1) as f32 / segments as f32;
        
        let angle1 = -half_angle + (vision.angle * t1);
        let angle2 = -half_angle + (vision.angle * t2);
        
        let dir1 = Vec2::new(
            vision.direction.x * angle1.cos() - vision.direction.y * angle1.sin(),
            vision.direction.x * angle1.sin() + vision.direction.y * angle1.cos(),
        );
        
        let dir2 = Vec2::new(
            vision.direction.x * angle2.cos() - vision.direction.y * angle2.sin(),
            vision.direction.x * angle2.sin() + vision.direction.y * angle2.cos(),
        );
        
        let point1 = position + dir1 * vision.range;
        let point2 = position + dir2 * vision.range;
        
        gizmos.line_2d(point1, point2, color);
    }
    
    // Draw cone edges
    let left_dir = Vec2::new(
        vision.direction.x * half_angle.cos() - vision.direction.y * half_angle.sin(),
        vision.direction.x * half_angle.sin() + vision.direction.y * half_angle.cos(),
    );
    
    let right_dir = Vec2::new(
        vision.direction.x * half_angle.cos() + vision.direction.y * half_angle.sin(),
        -vision.direction.x * half_angle.sin() + vision.direction.y * half_angle.cos(),
    );
    
    gizmos.line_2d(position, position + left_dir * vision.range, color);
    gizmos.line_2d(position, position + right_dir * vision.range, color);
}

// Add this system to handle state transitions
pub fn cleanup_mission_ui(
    mut commands: Commands,
    mut inventory_state: ResMut<InventoryState>,
    mut game_mode: ResMut<GameMode>,
    inventory_ui_query: Query<Entity, With<InventoryUI>>,
    pause_ui_query: Query<Entity, With<PauseScreen>>,
) {
    // Close inventory
    inventory_state.ui_open = false;
    inventory_state.selected_agent = None;
    
    // Clear targeting modes
    game_mode.targeting = None;
    game_mode.paused = false;
    
    // Despawn any open UI windows
    for entity in inventory_ui_query.iter() {
        commands.entity(entity).despawn_recursive();
    }
    
    for entity in pause_ui_query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

// Also add cleanup when entering global map
pub fn cleanup_global_map_ui(
    mut commands: Commands,
    mut inventory_state: ResMut<InventoryState>,
    inventory_ui_query: Query<Entity, With<InventoryUI>>,
    post_mission_ui_query: Query<Entity, With<PostMissionScreen>>,
) {
    // Make sure inventory is closed in global map
    inventory_state.ui_open = false;
    inventory_state.selected_agent = None;
    
    // Clean up any lingering UI
    for entity in inventory_ui_query.iter() {
        commands.entity(entity).despawn_recursive();
    }
    
    for entity in post_mission_ui_query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}