use bevy::prelude::*;
use crate::core::*;
use super::HubTab;

pub fn handle_input(input: &ButtonInput<KeyCode>, hub_state: &mut super::HubState) -> bool {
    if input.just_pressed(KeyCode::KeyM) {
        hub_state.active_tab = HubTab::Manufacture;
        return true;
    }
    // TODO: Implement agent management input
    // - Arrow keys to select agents
    // - Enter to modify equipment
    // - Save/Load squad presets
    false
}

pub fn create_content(parent: &mut ChildSpawnerCommands, global_data: &GlobalData) {
    parent.spawn(Node {
            width: Val::Percent(100.0),
            flex_grow: 1.0,
            padding: UiRect::all(Val::Px(20.0)),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(15.0),
            ..default()
    }).with_children(|content| {

        content.spawn((
            Text::new("AGENT MANAGEMENT"),
            TextFont { font_size: 24.0, ..default() },
            TextColor(Color::srgb(0.8, 0.8, 0.2)),
        ));

        content.spawn((
            Text::new("Press 'M' to access Manufacture tab for weapon customization"),
            TextFont { font_size: 16.0, ..default() },
            TextColor(Color::srgb(0.8, 0.8, 0.2)),
        ));
        
        for i in 0..3 {
            let level = global_data.agent_levels[i];
            let exp = global_data.agent_experience[i];
            let next_level_exp = experience_for_level(level + 1);
            
            content.spawn((
                Text::new(format!("Agent {}: Level {} ({}/{} XP)", i + 1, level, exp, next_level_exp)),
                TextFont { font_size: 16.0, ..default() },
                TextColor(Color::WHITE),
            ));            
        }

        content.spawn((
            Text::new("\nTODO: Implement squad management features:\n• Equipment presets\n• Skill specializations\n• Agent recovery tracking\n• Performance statistics"),
            TextFont { font_size: 14.0, ..default() },
            TextColor(Color::srgb(0.6, 0.6, 0.6)),
        ));
    });
}