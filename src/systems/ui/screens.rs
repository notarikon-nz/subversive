// src/systems/ui/screens.rs - All the screen UIs updated for Bevy 0.16
use bevy::prelude::*;
use crate::core::*;

// Re-export components for compatibility
#[derive(Component)]
pub struct InventoryUI;

#[derive(Component)]
pub struct PostMissionScreen;

#[derive(Component)]
pub struct GlobalMapScreen;

#[derive(Component)]
pub struct PauseScreen;

#[derive(Component)]
pub struct FpsText;

// FPS system
pub fn fps_system(
    mut commands: Commands,
    ui_state: Res<UIState>,
    fps_query: Query<Entity, With<FpsText>>,
    mut fps_text_query: Query<&mut Text, With<FpsText>>,
    time: Res<Time>,
) {
    if ui_state.fps_visible && fps_query.is_empty() {
        commands.spawn((
            Text::new("FPS: --"),
            TextFont {
                font_size: 20.0,
                ..default()
            },
            TextColor(Color::srgb(0.0, 1.0, 0.0)),
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(10.0),
                left: Val::Px(10.0),
                ..default()
            },
            FpsText,
        ));
    } else if !ui_state.fps_visible && !fps_query.is_empty() {
        for entity in fps_query.iter() {
            commands.entity(entity).despawn_recursive();
        }
    } else if ui_state.fps_visible && !fps_query.is_empty() {
        let fps = 1.0 / time.delta_secs();
        if let Ok(mut text) = fps_text_query.get_single_mut() {
            **text = format!("FPS: {:.0}", fps);
        }
    }
}

// Pause system with mission abort
pub fn pause_system(
    mut commands: Commands,
    mut next_state: ResMut<NextState<GameState>>,
    mut post_mission: ResMut<PostMissionResults>,
    mut processed: ResMut<PostMissionProcessed>,
    game_mode: Res<GameMode>,
    input: Res<ButtonInput<KeyCode>>,
    screen_query: Query<Entity, With<PauseScreen>>,
    mission_data: Res<MissionData>,
) {
    let should_show = game_mode.paused;
    let ui_exists = !screen_query.is_empty();
    
    // Handle abort input when paused
    if should_show && input.just_pressed(KeyCode::KeyQ) {
        // Set mission as failed/aborted
        *post_mission = PostMissionResults {
            success: false,
            time_taken: mission_data.timer,
            enemies_killed: mission_data.enemies_killed,
            terminals_accessed: mission_data.terminals_accessed,
            credits_earned: 0, // No credits for abort
            alert_level: mission_data.alert_level,
        };
        
        // Clear pause UI
        for entity in screen_query.iter() {
            commands.entity(entity).despawn_recursive();
        }
        
        // Go to post-mission to handle agent recovery properly
        next_state.set(GameState::PostMission);
        return;
    }
    
    if should_show && !ui_exists {
        commands.spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.5)),
            ZIndex(100),
            PauseScreen,
        )).with_children(|parent| {
            parent.spawn((
                Text::new("PAUSED"),
                TextFont { font_size: 32.0, ..default() },
                TextColor(Color::WHITE),
            ));
            
            parent.spawn((
                Text::new("\nSPACE: Resume Mission\nQ: Abort Mission"),
                TextFont { font_size: 20.0, ..default() },
                TextColor(Color::srgb(0.8, 0.8, 0.8)),
            ));
            
            parent.spawn((
                Text::new("\n(Aborting will count as mission failure)"),
                TextFont { font_size: 14.0, ..default() },
                TextColor(Color::srgb(0.8, 0.3, 0.3)),
            ));
        });
    } else if !should_show && ui_exists {
        for entity in screen_query.iter() {
            commands.entity(entity).despawn_recursive();
        }
    }
}

// Inventory system - with simple change detection
pub fn inventory_system(
    mut commands: Commands,
    inventory_state: Res<InventoryState>,
    agent_query: Query<&Inventory, (With<Agent>, Changed<Inventory>)>,
    all_agent_query: Query<&Inventory, With<Agent>>,
    inventory_ui_query: Query<Entity, With<InventoryUI>>,
) {
    if !inventory_state.ui_open {
        if !inventory_ui_query.is_empty() {
            for entity in inventory_ui_query.iter() {
                commands.entity(entity).despawn_recursive();
            }
        }
        return;
    }
    
    // Only rebuild if inventory changed or UI doesn't exist
    let needs_update = inventory_ui_query.is_empty() || 
        (inventory_state.selected_agent.is_some() && 
         agent_query.get(inventory_state.selected_agent.unwrap()).is_ok());
    
    if needs_update {
        // Clear existing UI
        for entity in inventory_ui_query.iter() {
            commands.entity(entity).despawn_recursive();
        }
        
        let inventory = inventory_state.selected_agent
            .and_then(|agent| all_agent_query.get(agent).ok());
        
        create_inventory_ui(&mut commands, inventory);
    }
}

fn create_inventory_ui(commands: &mut Commands, inventory: Option<&Inventory>) {
    commands.spawn((
        Node {
            width: Val::Px(450.0),  // Slightly wider for attachment info
            height: Val::Px(550.0),
            position_type: PositionType::Absolute,
            left: Val::Px(50.0),
            top: Val::Px(50.0),
            flex_direction: FlexDirection::Column,
            padding: UiRect::all(Val::Px(10.0)),
            row_gap: Val::Px(10.0),
            ..default()
        },
        BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.9)),
        ZIndex(50),
        InventoryUI,
    )).with_children(|parent| {
        // Title
        parent.spawn((
            Text::new("AGENT INVENTORY"),
            TextFont { font_size: 24.0, ..default() },
            TextColor(Color::WHITE),
        ));
        
        if let Some(inv) = inventory {
            // Currency
            parent.spawn((
                Text::new(format!("Credits: {}", inv.currency)),
                TextFont { font_size: 18.0, ..default() },
                TextColor(Color::srgb(0.8, 0.8, 0.2)),
            ));
            
            // Equipped weapon with attachment stats
            if let Some(weapon_config) = &inv.equipped_weapon {
                let stats = weapon_config.calculate_total_stats();
                
                parent.spawn((
                    Text::new(format!("EQUIPPED: {:?}", weapon_config.base_weapon)),
                    TextFont { font_size: 16.0, ..default() },
                    TextColor(Color::srgb(0.9, 0.5, 0.2)),
                ));
                
                // Show attachment stats if any are non-zero
                if stats.accuracy != 0 || stats.range != 0 || stats.noise != 0 || 
                   stats.reload_speed != 0 || stats.ammo_capacity != 0 {
                    parent.spawn((
                        Text::new(format!("Stats: Acc{:+} Rng{:+} Noise{:+} Reload{:+} Ammo{:+}", 
                                stats.accuracy, stats.range, stats.noise, stats.reload_speed, stats.ammo_capacity)),
                        TextFont { font_size: 14.0, ..default() },
                        TextColor(Color::srgb(0.6, 0.8, 0.6)),
                    ));
                }
                
                // Show attached components
                let attached_count = weapon_config.attachments.values()
                    .filter(|att| att.is_some())
                    .count();
                
                if attached_count > 0 {
                    parent.spawn((
                        Text::new(format!("Attachments: {}/{}", attached_count, weapon_config.supported_slots.len())),
                        TextFont { font_size: 14.0, ..default() },
                        TextColor(Color::srgb(0.7, 0.7, 0.9)),
                    ));
                    
                    // List attached items
                    for (slot, attachment_opt) in &weapon_config.attachments {
                        if let Some(attachment) = attachment_opt {
                            parent.spawn((
                                Text::new(format!("  {:?}: {}", slot, attachment.name)),
                                TextFont { font_size: 12.0, ..default() },
                                TextColor(Color::srgb(0.8, 0.8, 0.8)),
                            ));
                        }
                    }
                } else {
                    parent.spawn((
                        Text::new("No attachments equipped"),
                        TextFont { font_size: 14.0, ..default() },
                        TextColor(Color::srgb(0.5, 0.5, 0.5)),
                    ));
                }
                
            } else {
                parent.spawn((
                    Text::new("EQUIPPED WEAPON: None"),
                    TextFont { font_size: 16.0, ..default() },
                    TextColor(Color::srgb(0.8, 0.3, 0.3)),
                ));
            }
            
            // Equipped tools
            if !inv.equipped_tools.is_empty() {
                parent.spawn((
                    Text::new(format!("EQUIPPED TOOLS: {:?}", inv.equipped_tools)),
                    TextFont { font_size: 16.0, ..default() },
                    TextColor(Color::srgb(0.3, 0.8, 0.3)),
                ));
            }
            
            // Weapons section - show as configs now
            if !inv.weapons.is_empty() {
                parent.spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(2.0),
                        ..default()
                    },
                )).with_children(|weapons| {
                    weapons.spawn((
                        Text::new("WEAPONS:"),
                        TextFont { font_size: 16.0, ..default() },
                        TextColor(Color::srgb(0.8, 0.3, 0.3)),
                    ));
                    for weapon_config in &inv.weapons {
                        let attached_count = weapon_config.attachments.values()
                            .filter(|att| att.is_some())
                            .count();
                        weapons.spawn((
                            Text::new(format!("• {:?} ({}/{})", weapon_config.base_weapon, attached_count, weapon_config.supported_slots.len())),
                            TextFont { font_size: 14.0, ..default() },
                            TextColor(Color::WHITE),
                        ));
                    }
                });
            }
            
            // Tools section (unchanged logic, new syntax)
            if !inv.tools.is_empty() {
                parent.spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(2.0),
                        ..default()
                    },
                )).with_children(|tools| {
                    tools.spawn((
                        Text::new("TOOLS:"),
                        TextFont { font_size: 16.0, ..default() },
                        TextColor(Color::srgb(0.3, 0.8, 0.3)),
                    ));
                    for tool in &inv.tools {
                        tools.spawn((
                            Text::new(format!("• {:?}", tool)),
                            TextFont { font_size: 14.0, ..default() },
                            TextColor(Color::WHITE),
                        ));
                    }
                });
            }
            
            // Cybernetics section (unchanged logic, new syntax)
            if !inv.cybernetics.is_empty() {
                parent.spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(2.0),
                        ..default()
                    },
                )).with_children(|cyber| {
                    cyber.spawn((
                        Text::new("CYBERNETICS:"),
                        TextFont { font_size: 16.0, ..default() },
                        TextColor(Color::srgb(0.3, 0.3, 0.8)),
                    ));
                    for cybernetic in &inv.cybernetics {
                        cyber.spawn((
                            Text::new(format!("• {:?}", cybernetic)),
                            TextFont { font_size: 14.0, ..default() },
                            TextColor(Color::WHITE),
                        ));
                    }
                });
            }
            
            // Intel section (unchanged logic, new syntax)
            if !inv.intel_documents.is_empty() {
                parent.spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(2.0),
                        ..default()
                    },
                )).with_children(|intel| {
                    intel.spawn((
                        Text::new("INTEL DOCUMENTS:"),
                        TextFont { font_size: 16.0, ..default() },
                        TextColor(Color::srgb(0.8, 0.8, 0.3)),
                    ));
                    for (i, document) in inv.intel_documents.iter().enumerate() {
                        let preview = if document.len() > 40 {
                            format!("{}...", &document[..37])
                        } else {
                            document.clone()
                        };
                        intel.spawn((
                            Text::new(format!("• Doc {}: {}", i + 1, preview)),
                            TextFont { font_size: 11.0, ..default() },
                            TextColor(Color::WHITE),
                        ));
                    }
                });
            }
        } else {
            parent.spawn((
                Text::new("No agent selected"),
                TextFont { font_size: 16.0, ..default() },
                TextColor(Color::srgb(0.8, 0.3, 0.3)),
            ));
        }
        
        // Instructions
        parent.spawn((
            Text::new("Press 'I' to close inventory\nGo to Manufacture tab to modify weapons"),
            TextFont { font_size: 12.0, ..default() },
            TextColor(Color::srgb(0.7, 0.7, 0.7)),
        ));
    });
}

// Post mission system - restored full functionality
pub fn post_mission_ui_system(
    mut commands: Commands,
    mut next_state: ResMut<NextState<GameState>>,
    mut processed: ResMut<PostMissionProcessed>,
    post_mission: Res<PostMissionResults>,
    global_data: Res<GlobalData>,
    input: Res<ButtonInput<KeyCode>>,
    screen_query: Query<Entity, With<PostMissionScreen>>,
) {
    if input.just_pressed(KeyCode::KeyR) {
        for entity in screen_query.iter() {
            commands.entity(entity).despawn_recursive();
        }
        processed.0 = false;
        next_state.set(GameState::GlobalMap);
        return;
    }
    
    if input.just_pressed(KeyCode::Escape) {
        std::process::exit(0);
    }
    
    if screen_query.is_empty() && processed.0 {
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
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            position_type: PositionType::Absolute,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            flex_direction: FlexDirection::Column,
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.8)),
        ZIndex(200),
        PostMissionScreen,
    )).with_children(|parent| {
        // Title
        parent.spawn((
            Text::new(title),
            TextFont { font_size: 48.0, ..default() },
            TextColor(title_color),
        ));
        
        // Stats
        parent.spawn((
            Text::new(format!(
                "\nTime: {:.1}s\nEnemies: {}\nTerminals: {}\nCredits: {}\nAlert: {:?}",
                post_mission.time_taken,
                post_mission.enemies_killed,
                post_mission.terminals_accessed,
                post_mission.credits_earned,
                post_mission.alert_level
            )),
            TextFont { font_size: 24.0, ..default() },
            TextColor(Color::WHITE),
        ));
        
        // Agent progression (only if successful)
        if post_mission.success {
            let exp_gained = 10 + (post_mission.enemies_killed * 5);
            parent.spawn((
                Text::new(format!("\nEXP GAINED: {}", exp_gained)),
                TextFont { font_size: 20.0, ..default() },
                TextColor(Color::srgb(0.2, 0.8, 0.8)),
            ));
            
            for i in 0..3 {
                parent.spawn((
                    Text::new(format!("Agent {}: Lv{} ({}/{})", 
                        i + 1, 
                        global_data.agent_levels[i],
                        global_data.agent_experience[i],
                        experience_for_level(global_data.agent_levels[i] + 1)
                    )),
                    TextFont { font_size: 16.0, ..default() },
                    TextColor(Color::WHITE),
                ));
            }
        }
        
        parent.spawn((
            Text::new("\nR: Return to Map | ESC: Quit"),
            TextFont { font_size: 16.0, ..default() },
            TextColor(Color::srgb(0.7, 0.7, 0.7)),
        ));
    });
}
