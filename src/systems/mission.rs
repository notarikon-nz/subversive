use bevy::prelude::*;
use crate::core::*;

pub fn timer_system(
    mut mission_data: ResMut<MissionData>,
    mut next_state: ResMut<NextState<GameState>>,
    mut post_mission: ResMut<PostMissionResults>,
    game_mode: Res<GameMode>,
    time: Res<Time>,
) {
    if game_mode.paused { return; }
    
    mission_data.timer += time.delta_seconds();
    
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
    mut mission_data: ResMut<MissionData>,
    mut game_mode: ResMut<GameMode>,
    mut selection: ResMut<SelectionState>,
    mut inventory_state: ResMut<InventoryState>,
    global_data: Res<GlobalData>,
) {
    if restart_check.is_none() { return; }
    
    for entity in entities.iter() {
        commands.entity(entity).despawn_recursive();
    }
    
    *mission_data = MissionData::default();
    *game_mode = GameMode::default();
    *selection = SelectionState::default();
    *inventory_state = InventoryState::default();
    
    commands.remove_resource::<ShouldRestart>();
    
    crate::spawn_agents(&mut commands, 3, &*global_data);
    crate::spawn_civilians(&mut commands, 5);
    crate::spawn_enemy(&mut commands, &*global_data);
    crate::spawn_terminals(&mut commands);
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
) {
    // Handle input first
    if input.just_pressed(KeyCode::KeyR) {
        // Clear all post-mission UI
        for entity in ui_query.iter() {
            commands.entity(entity).despawn_recursive();
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
        update_mission_results(&mut global_data, &post_mission, &agent_query);
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
) {
    let region_idx = global_data.selected_region;
    global_data.current_day += 1;
    
    if post_mission.success {
        global_data.credits += post_mission.credits_earned;
        
        let exp_gained = 10 + (post_mission.enemies_killed * 5);
        let recovery_days = if post_mission.time_taken > 240.0 { 2 } else { 1 };
        
        for (i, _) in agent_query.iter().enumerate().take(3) {
            global_data.agent_experience[i] += exp_gained;
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
        parent.spawn(TextBundle::from_section(title, TextStyle {
            font_size: 48.0, color, ..default()
        }));
        
        parent.spawn(TextBundle::from_section(
            format!(
                "\nTime: {:.1}s\nEnemies: {}\nTerminals: {}\nCredits: {}\nAlert: {:?}",
                post_mission.time_taken,
                post_mission.enemies_killed,
                post_mission.terminals_accessed,
                post_mission.credits_earned,
                post_mission.alert_level
            ),
            TextStyle { font_size: 24.0, color: Color::WHITE, ..default() }
        ));
        
        if post_mission.success {
            let exp = 10 + (post_mission.enemies_killed * 5);
            parent.spawn(TextBundle::from_section(
                format!("\nEXP GAINED: {}", exp),
                TextStyle { font_size: 20.0, color: Color::srgb(0.2, 0.8, 0.8), ..default() }
            ));
            
            for i in 0..3 {
                parent.spawn(TextBundle::from_section(
                    format!("Agent {}: Lv{} ({}/{})", 
                        i + 1, 
                        global_data.agent_levels[i],
                        global_data.agent_experience[i],
                        experience_for_level(global_data.agent_levels[i] + 1)
                    ),
                    TextStyle { font_size: 16.0, color: Color::WHITE, ..default() }
                ));
            }
        }
        
        parent.spawn(TextBundle::from_section(
            "\nR: Return to Map | ESC: Quit",
            TextStyle { font_size: 16.0, color: Color::srgb(0.7, 0.7, 0.7), ..default() }
        ));
    });
}

#[derive(Component)]
pub struct PostMissionUI;