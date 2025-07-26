// src/systems/ui/screens.rs - All the screen UIs updated for Bevy 0.16
use bevy::prelude::*;
use crate::core::*;
use crate::systems::ui::*;
use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};

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
    diagnostics: Res<DiagnosticsStore>,
    ui_state: Res<UIState>,
    mut fps_text_query: Query<(Entity, &mut Text), With<FpsText>>,
) {
    if !ui_state.fps_visible {
        // Clean up FPS text if it exists and should be hidden
        for (entity, _) in fps_text_query.iter() {
            commands.entity(entity).insert(MarkedForDespawn);
        }
        return;
    }

    let fps = diagnostics
        .get(&FrameTimeDiagnosticsPlugin::FPS)
        .and_then(|fps| fps.smoothed())
        .unwrap_or(0.0);
    
    let fps_text = format!("FPS: {:.1}", fps);
    
    // Try to update existing text first
    if let Ok((_, mut text)) = fps_text_query.single_mut() {
        **text = fps_text;
    } else {
        // Only create if it doesn't exist and should be visible
        commands.spawn((
            Text::new(fps_text),
            TextFont {
                font_size: 16.0,
                ..default()
            },
            TextColor(Color::WHITE),
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(10.0),
                left: Val::Px(10.0),
                ..default()
            },
            FpsText,
        ));
    }
}

// Pause system with mission abort
pub fn pause_system(
    mut commands: Commands,
    mut next_state: ResMut<NextState<GameState>>,
    mut post_mission: ResMut<PostMissionResults>,
    processed: ResMut<PostMissionProcessed>,
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
            commands.entity(entity).insert(MarkedForDespawn);
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
            commands.entity(entity).insert(MarkedForDespawn);
        }
    }
}

// Inventory system - with simple change detection
pub fn inventory_system(
    mut commands: Commands,
    inventory_state: Res<InventoryState>,
    agent_query: Query<(&Inventory, &WeaponState), (With<Agent>, Changed<Inventory>)>,
    all_agent_query: Query<(&Inventory, &WeaponState), With<Agent>>,
    inventory_ui_query: Query<Entity, (With<InventoryUI>, Without<MarkedForDespawn>)>,
) {
    if !inventory_state.ui_open {
        if !inventory_ui_query.is_empty() {
            for entity in inventory_ui_query.iter() {
                commands.entity(entity).insert(MarkedForDespawn);
            }
        }
        return;
    }
    
    let needs_update = inventory_ui_query.is_empty() || 
        (inventory_state.selected_agent.is_some() && 
         agent_query.get(inventory_state.selected_agent.unwrap()).is_ok());
    
    if needs_update {
        for entity in inventory_ui_query.iter() {
            commands.entity(entity).insert(MarkedForDespawn);
        }
        
        let (inventory, weapon_state) = inventory_state.selected_agent
            .and_then(|agent| all_agent_query.get(agent).ok())
            .map(|(inv, ws)| (Some(inv), Some(ws)))
            .unwrap_or((None, None));
        
        create_inventory_ui(&mut commands, inventory, weapon_state);
    }
}

fn create_inventory_ui(commands: &mut Commands, inventory: Option<&Inventory>, weapon_state: Option<&WeaponState>) {
    commands.spawn((
        Node {
            width: Val::Px(450.0),
            height: Val::Px(600.0), // Slightly taller for ammo display
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
        parent.spawn((
            Text::new("AGENT INVENTORY"),
            TextFont { font_size: 24.0, ..default() },
            TextColor(Color::WHITE),
        ));


        if let Some(inv) = inventory {
            parent.spawn((
                Text::new(format!("Credits: {}", inv.currency)),
                TextFont { font_size: 18.0, ..default() },
                TextColor(Color::srgb(0.8, 0.8, 0.2)),
            ));


    // Weapon display with ammo info
    if let Some(weapon_config) = &inv.equipped_weapon {
        let stats = weapon_config.stats();
        
        parent.spawn((
            Text::new(format!("EQUIPPED: {:?}", weapon_config.base_weapon)),
            TextFont { font_size: 16.0, ..default() },
            TextColor(Color::srgb(0.9, 0.5, 0.2)),
        ));
        
        if stats.accuracy != 0 || stats.range != 0 || stats.noise != 0 || 
           stats.reload_speed != 0 || stats.ammo_capacity != 0 {
            parent.spawn((
                Text::new(format!("Stats: Acc{:+} Rng{:+} Noise{:+} Reload{:+} Ammo{:+}", 
                        stats.accuracy, stats.range, stats.noise, stats.reload_speed, stats.ammo_capacity)),
                TextFont { font_size: 14.0, ..default() },
                TextColor(Color::srgb(0.6, 0.8, 0.6)),
            ));
        }
        
        let attached_count = weapon_config.attachments.len();
        
        if attached_count > 0 {
            parent.spawn((
                Text::new(format!("Attachments: {}/{}", 
                        attached_count, weapon_config.supported_slots().len())),
                TextFont { font_size: 14.0, ..default() },
                TextColor(Color::srgb(0.7, 0.7, 0.9)),
            ));
            
            for (slot, attachment) in &weapon_config.attachments {
                parent.spawn((
                    Text::new(format!("  {:?}: {}", slot, attachment.name)),
                    TextFont { font_size: 12.0, ..default() },
                    TextColor(Color::srgb(0.8, 0.8, 0.8)),
                ));
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
                
            
            // Add ammo display
            if let Some(weapon_state) = weapon_state {
                let ammo_color = if weapon_state.current_ammo == 0 {
                    Color::srgb(0.8, 0.2, 0.2) // Red when empty
                } else if weapon_state.needs_reload() {
                    Color::srgb(0.8, 0.6, 0.2) // Yellow when low
                } else {
                    Color::srgb(0.2, 0.8, 0.2) // Green when good
                };
                
                let reload_status = if weapon_state.is_reloading {
                    format!(" (Reloading: {:.1}s)", weapon_state.reload_timer)
                } else {
                    String::new()
                };
                
                parent.spawn((
                    Text::new(format!("AMMO: {}/{}{}", 
                            weapon_state.current_ammo, 
                            weapon_state.max_ammo,
                            reload_status)),
                    TextFont { font_size: 16.0, ..default() },
                    TextColor(ammo_color),
                ));
            }
            
            // Rest of inventory display (tools, cybernetics, etc.)
            if !inv.equipped_tools.is_empty() {
                parent.spawn((
                    Text::new(format!("EQUIPPED TOOLS: {:?}", inv.equipped_tools)),
                    TextFont { font_size: 16.0, ..default() },
                    TextColor(Color::srgb(0.3, 0.8, 0.3)),
                ));
            }
            
            // Continue with other inventory sections...
        } else {
            parent.spawn((
                Text::new("No agent selected"),
                TextFont { font_size: 16.0, ..default() },
                TextColor(Color::srgb(0.8, 0.3, 0.3)),
            ));
        }
        
        parent.spawn((
            Text::new("Press 'I' to close inventory\nPress 'R' to reload weapon\nGo to Manufacture tab to modify weapons"),
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
            commands.entity(entity).insert(MarkedForDespawn);
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
