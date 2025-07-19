use bevy::prelude::*;
use crate::core::*;

pub fn handle_input(
    input: &ButtonInput<KeyCode>,
    commands: &mut Commands,
    next_state: &mut NextState<GameState>,
    global_data: &GlobalData,
) -> bool {
    if input.just_pressed(KeyCode::Enter) {
        let ready_agents = (0..3).filter(|&i| global_data.agent_recovery[i] <= global_data.current_day).count();
        if ready_agents > 0 {
            commands.insert_resource(ShouldRestart);
            next_state.set(GameState::Mission);
            info!("Launching mission with {} agents", ready_agents);
        } else {
            info!("No agents ready for deployment!");
        }
    }
    false
}

pub fn create_content(parent: &mut ChildBuilder, global_data: &GlobalData, hub_state: &super::HubState) {
    parent.spawn(NodeBundle {
        style: Style {
            width: Val::Percent(100.0),
            flex_grow: 1.0,
            padding: UiRect::all(Val::Px(20.0)),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(15.0),
            ..default()
        },
        ..default()
    }).with_children(|content| {
        let region = &global_data.regions[hub_state.selected_region];
        
        content.spawn(TextBundle::from_section(
            format!("MISSION BRIEFING: {}", region.name),
            TextStyle { font_size: 24.0, color: Color::srgb(0.8, 0.2, 0.2), ..default() }
        ));
        
        content.spawn(TextBundle::from_section(
            format!("Threat Level: {} | Alert Status: {:?}", region.threat_level, region.alert_level),
            TextStyle { font_size: 18.0, color: Color::WHITE, ..default() }
        ));
        
        let ready_agents = (0..3).filter(|&i| global_data.agent_recovery[i] <= global_data.current_day).count();
        
        if ready_agents > 0 {
            content.spawn(TextBundle::from_section(
                format!("\nSquad Status: {} agents ready for deployment", ready_agents),
                TextStyle { font_size: 16.0, color: Color::srgb(0.2, 0.8, 0.2), ..default() }
            ));
            
            content.spawn(TextBundle::from_section(
                "Press ENTER to launch mission",
                TextStyle { font_size: 16.0, color: Color::srgb(0.8, 0.8, 0.2), ..default() }
            ));
        } else {
            content.spawn(TextBundle::from_section(
                "\nSquad Status: No agents available (all recovering)",
                TextStyle { font_size: 16.0, color: Color::srgb(0.8, 0.2, 0.2), ..default() }
            ));
            
            content.spawn(TextBundle::from_section(
                "Wait for agents to recover or use 'W' to advance time",
                TextStyle { font_size: 14.0, color: Color::srgb(0.6, 0.6, 0.6), ..default() }
            ));
        }
        
        // Mission details
        content.spawn(TextBundle::from_section(
            format!("\nMission Difficulty Modifier: {:.1}x", region.mission_difficulty_modifier()),
            TextStyle { font_size: 14.0, color: Color::WHITE, ..default() }
        ));
        
        content.spawn(TextBundle::from_section(
            "\nTODO: Detailed mission briefing:\n• Objectives and requirements\n• Expected resistance\n• Equipment recommendations\n• Risk assessment",
            TextStyle { font_size: 12.0, color: Color::srgb(0.6, 0.6, 0.6), ..default() }
        ));
    });
}