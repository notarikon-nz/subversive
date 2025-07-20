// src/systems/mission.rs - Updated for Bevy 0.16
use bevy::prelude::*;
use crate::core::*;
use crate::systems::*;

pub fn timer_system(
    mut mission_data: ResMut<MissionData>,
    mut next_state: ResMut<NextState<GameState>>,
    mut post_mission: ResMut<PostMissionResults>,
    game_mode: Res<GameMode>,
    time: Res<Time>,
) {
    if game_mode.paused { return; }
    
    mission_data.timer += time.delta_secs();
    
    if mission_data.timer >= mission_data.time_limit {
        *post_mission = PostMissionResults {
            success: false,
            time_taken: mission_data.timer,
            enemies_killed: mission_data.enemies_killed,
            terminals_accessed: mission_data.terminals_accessed,
            credits_earned: 0,
            alert_level: mission_data.alert_level,
        };
        next_state.set(GameState::PostMission);
    }
}

pub fn check_completion(
    mut next_state: ResMut<NextState<GameState>>,
    mission_data: Res<MissionData>,
    mut post_mission: ResMut<PostMissionResults>,
    agent_query: Query<&Inventory, With<Agent>>,
) {
    let objectives_complete = mission_data.objectives_completed >= mission_data.total_objectives;
    let agents_alive = !agent_query.is_empty();
    
    if objectives_complete {
        let credits_earned = agent_query.iter().map(|inv| inv.currency).sum();
        *post_mission = PostMissionResults {
            success: true,
            time_taken: mission_data.timer,
            enemies_killed: mission_data.enemies_killed,
            terminals_accessed: mission_data.terminals_accessed,
            credits_earned,
            alert_level: mission_data.alert_level,
        };
        next_state.set(GameState::PostMission);
    } else if !agents_alive {
        *post_mission = PostMissionResults::default();
        next_state.set(GameState::PostMission);
    }
}

pub fn restart_system(
    mut commands: Commands,
    restart_check: Option<Res<ShouldRestart>>,
    entities: Query<Entity, (Without<Camera>, Without<Window>)>,
    sprites: Res<GameSprites>,
    mut mission_data: ResMut<MissionData>,
    mut game_mode: ResMut<GameMode>,
    mut selection: ResMut<SelectionState>,
    mut inventory_state: ResMut<InventoryState>,
    global_data: Res<GlobalData>,
) {
    if restart_check.is_none() { return; }
    
    info!("Restarting mission - despawning {} entities", entities.iter().count());
    
    // FIXED: Use Result instead of Option
    for entity in entities.iter() {
        if let Ok(mut entity_commands) = commands.get_entity(entity) {
            commands.safe_despawn_recursive(entity);
        }
    }
    
    *mission_data = MissionData::default();
    *game_mode = GameMode::default();
    *selection = SelectionState::default();
    *inventory_state = InventoryState::default();
    
    commands.remove_resource::<ShouldRestart>();
    
    let scene_name = match global_data.selected_region {
        0 => "mission1",
        1 => "mission2", 
        2 => "mission3",
        _ => "mission1",
    };
    
    let scene = crate::systems::scenes::load_scene(scene_name);
    crate::systems::scenes::spawn_from_scene(&mut commands, &scene, &*global_data, &sprites);
    
    info!("Mission restart complete");
}

pub fn process_mission_results(
    mut global_data: ResMut<GlobalData>,
    mut processed: ResMut<PostMissionProcessed>,
    post_mission: Res<PostMissionResults>,
    agent_query: Query<&Agent>,
) {
    if processed.0 { return; }
    
    let region_idx = global_data.selected_region;
    global_data.current_day += 1;
    let current_day = global_data.current_day; // Store the value
    
    if post_mission.success {
        global_data.credits += post_mission.credits_earned;
        
        let exp_gained = 10 + (post_mission.enemies_killed * 5);
        let recovery_days = if post_mission.time_taken > 240.0 { 2 } else { 1 };
        
        for (i, _) in agent_query.iter().enumerate().take(3) {
            global_data.agent_experience[i] += exp_gained;
            global_data.agent_recovery[i] = current_day + recovery_days;
            
            let required_exp = experience_for_level(global_data.agent_levels[i] + 1);
            if global_data.agent_experience[i] >= required_exp && global_data.agent_levels[i] < 10 {
                global_data.agent_levels[i] += 1;
            }
        }
        
        if post_mission.enemies_killed > 0 || post_mission.time_taken >= 180.0 {
            global_data.regions[region_idx].raise_alert(current_day);
        }
    } else {
        global_data.regions[region_idx].raise_alert(current_day);
        global_data.regions[region_idx].raise_alert(current_day);
        for i in 0..3 {
            global_data.agent_recovery[i] = current_day + 3;
        }
    }
    
    for region in &mut global_data.regions {
        region.update_alert(current_day);
    }
    
    processed.0 = true;
}

pub fn post_mission_system(
    mut commands: Commands,
    post_mission: Res<PostMissionResults>,
    mut next_state: ResMut<NextState<GameState>>,
    mut global_data: ResMut<GlobalData>,
    mut processed: ResMut<PostMissionProcessed>,
    mut ui_state: ResMut<UIState>,
    agent_query: Query<&Agent>,
    input: Res<ButtonInput<KeyCode>>,
    ui_query: Query<Entity, With<PostMissionUI>>,
    research_db: Res<ResearchDatabase>,
) {
    // Handle input first
    if input.just_pressed(KeyCode::KeyR) {
        // Clear all post-mission UI
        for entity in ui_query.iter() {
            commands.safe_despawn_recursive(entity);
        }
        
        // Reset flags
        processed.0 = false;
        ui_state.global_map_open = false;
        
        // Transition to global map
        next_state.set(GameState::GlobalMap);
        return;
    }
    
    if input.just_pressed(KeyCode::Escape) {
        std::process::exit(0);
    }
    
    // Process mission results only once
    if !processed.0 {
        update_mission_results(&mut global_data, &post_mission, &agent_query, research_db);
        processed.0 = true;
    }
    
    // Only create UI if it doesn't exist
    if ui_query.is_empty() {
        create_results_ui(&mut commands, &post_mission, &global_data);
    }
}

fn update_mission_results(
    global_data: &mut GlobalData,
    post_mission: &PostMissionResults,
    agent_query: &Query<&Agent>,
    research_db: Res<ResearchDatabase>,
) {
    let region_idx = global_data.selected_region;
    global_data.current_day += 1;
    
    if post_mission.success {
        let base_exp = 10 + (post_mission.enemies_killed * 5);
        let research_exp = calculate_research_xp_bonus(&global_data.research_progress, &research_db, base_exp);
        let credit_bonus = calculate_research_credit_bonus(&global_data.research_progress, &research_db);

        global_data.credits += post_mission.credits_earned + credit_bonus;
       
        let exp_gained = 10 + (post_mission.enemies_killed * 5);
        let recovery_days = if post_mission.time_taken > 240.0 { 2 } else { 1 };
        
        for (i, _) in agent_query.iter().enumerate().take(3) {
            global_data.agent_experience[i] += research_exp;
            global_data.agent_recovery[i] = global_data.current_day + recovery_days;
            
            let required_exp = experience_for_level(global_data.agent_levels[i] + 1);
            if global_data.agent_experience[i] >= required_exp && global_data.agent_levels[i] < 10 {
                global_data.agent_levels[i] += 1;
            }
        }
        
        if post_mission.enemies_killed > 0 || post_mission.time_taken >= 180.0 {
            global_data.regions[region_idx].raise_alert(global_data.current_day);
        }

    } else {
        global_data.regions[region_idx].raise_alert(global_data.current_day);
        global_data.regions[region_idx].raise_alert(global_data.current_day);
        for i in 0..3 {
            global_data.agent_recovery[i] = global_data.current_day + 3;
        }
    }

    for region in &mut global_data.regions {
        region.update_alert(global_data.current_day);
    }
}

fn create_results_ui(
    commands: &mut Commands,
    post_mission: &PostMissionResults,
    global_data: &GlobalData,
) {
    let (title, color) = if post_mission.success {
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
        PostMissionUI,
    )).with_children(|parent| {
        parent.spawn((
            Text::new(title),
            TextFont { font_size: 48.0, ..default() },
            TextColor(color),
        ));
        
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
        
        if post_mission.success {
            let exp = 10 + (post_mission.enemies_killed * 5);
            parent.spawn((
                Text::new(format!("\nEXP GAINED: {}", exp)),
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

#[derive(Component)]
pub struct PostMissionUI;