// src/systems/ui/hub/global_map.rs - Updated for Bevy 0.16
use bevy::prelude::*;
use crate::core::*;
use super::HubTab;

pub fn handle_input(
    input: &ButtonInput<KeyCode>,
    global_data: &mut GlobalData,
    hub_state: &mut super::HubState,
) -> bool {
    let mut needs_rebuild = false;

    if input.just_pressed(KeyCode::ArrowUp) && hub_state.selected_region > 0 {
        hub_state.selected_region -= 1;
        global_data.selected_region = hub_state.selected_region;
        needs_rebuild = true;
    }
    
    if input.just_pressed(KeyCode::ArrowDown) && hub_state.selected_region < global_data.regions.len() - 1 {
        hub_state.selected_region += 1;
        global_data.selected_region = hub_state.selected_region;
        needs_rebuild = true;
    }

    if input.just_pressed(KeyCode::KeyW) {
        global_data.current_day += 1;
        let current_day = global_data.current_day;
        for region in &mut global_data.regions {
            region.update_alert(current_day);
        }
        needs_rebuild = true;
        info!("Waited 1 day. Current day: {}", current_day);
    }

    if input.just_pressed(KeyCode::Enter) {
        hub_state.active_tab = HubTab::Missions;
        needs_rebuild = true;
    }

    needs_rebuild
}

pub fn create_content(parent: &mut ChildSpawnerCommands, global_data: &GlobalData, hub_state: &super::HubState) {
    parent.spawn(Node {
        width: Val::Percent(100.0),
        flex_grow: 1.0,
        padding: UiRect::all(Val::Px(20.0)),
        flex_direction: FlexDirection::Column,
        row_gap: Val::Px(15.0),
        ..default()
    }).with_children(|content| {
        // Agent status
        content.spawn((
            Text::new("AGENT STATUS:"),
            TextFont { font_size: 20.0, ..default() },
            TextColor(Color::WHITE),
        ));
        
        for i in 0..3 {
            let level = global_data.agent_levels[i];
            let is_recovering = global_data.agent_recovery[i] > global_data.current_day;
            let recovery_days = if is_recovering { 
                global_data.agent_recovery[i] - global_data.current_day 
            } else { 0 };
            
            let color = if is_recovering { Color::srgb(0.5, 0.5, 0.5) } else { Color::srgb(0.2, 0.8, 0.2) };
            let status = if is_recovering {
                format!("Agent {}: Level {} - RECOVERING ({} days)", i + 1, level, recovery_days)
            } else {
                format!("Agent {}: Level {} - READY", i + 1, level)
            };
            
            content.spawn((
                Text::new(status),
                TextFont { font_size: 16.0, ..default() },
                TextColor(color),
            ));
        }
        
        // World regions
        content.spawn((
            Text::new("\nWORLD REGIONS:"),
            TextFont { font_size: 20.0, ..default() },
            TextColor(Color::WHITE),
        ));
        
        for (i, region) in global_data.regions.iter().enumerate() {
            let is_selected = i == hub_state.selected_region;
            let color = if is_selected { Color::srgb(0.2, 0.8, 0.2) } else { Color::WHITE };
            let prefix = if is_selected { "> " } else { "  " };
            
            content.spawn((
                Text::new(format!("{}{} (Threat: {}, Alert: {:?})", 
                        prefix, region.name, region.threat_level, region.alert_level)),
                TextFont { font_size: 18.0, ..default() },
                TextColor(color),
            ));
        }
    });
}
