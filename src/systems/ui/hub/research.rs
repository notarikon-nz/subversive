use bevy::prelude::*;
use crate::core::*;

pub fn handle_input(_input: &ButtonInput<KeyCode>) -> bool {
    // TODO: Implement research tree navigation
    // - Arrow keys to navigate tech tree
    // - Enter to purchase research
    // - Research dependencies and progress tracking
    false
}

pub fn create_content(parent: &mut ChildBuilder, global_data: &GlobalData) {
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
        content.spawn(TextBundle::from_section(
            "RESEARCH & DEVELOPMENT",
            TextStyle { font_size: 24.0, color: Color::srgb(0.8, 0.8, 0.2), ..default() }
        ));
        
        content.spawn(TextBundle::from_section(
            "TODO: Implement research tree with unlockable upgrades:\n• Agent cybernetics\n• Weapon attachments\n• Mission equipment\n• Intelligence gathering tools",
            TextStyle { font_size: 16.0, color: Color::srgb(0.6, 0.6, 0.6), ..default() }
        ));
        
        content.spawn(TextBundle::from_section(
            format!("Available Credits: {}", global_data.credits),
            TextStyle { font_size: 16.0, color: Color::WHITE, ..default() }
        ));
        
        // Future: Add research tree visualization here
        content.spawn(TextBundle::from_section(
            "\nUpcoming Features:\n• Tech tree with dependencies\n• Research costs and timers\n• Unlock new missions and equipment",
            TextStyle { font_size: 14.0, color: Color::srgb(0.5, 0.7, 0.5), ..default() }
        ));
    });
}