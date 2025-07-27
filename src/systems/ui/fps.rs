use bevy::prelude::*;
use crate::systems::ui::*;
use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};

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