// src/systems/minimap.rs
use bevy::prelude::*;
use crate::core::*;
use crate::core::factions::*;

// === MINIMAP COMPONENTS ===
#[derive(Component)]
pub struct MinimapContainer;

#[derive(Component)]
pub struct MinimapCanvas;

#[derive(Component)]
pub struct MinimapDot {
    pub entity_ref: Entity,
}

// === MINIMAP RESOURCE ===
#[derive(Resource)]
pub struct MinimapSettings {
    pub size: f32,
    pub range: f32,
    pub show_colors: bool,
    pub show_terminals: bool,
    pub position: Vec2, // Screen position (top-right)
}

impl Default for MinimapSettings {
    fn default() -> Self {
        Self {
            size: 200.0,
            range: 300.0,
            show_colors: false,
            show_terminals: true,
            position: Vec2::new(-220.0, -220.0), // Top-right with margin
        }
    }
}

// === SETUP MINIMAP UI ===
pub fn setup_minimap(mut commands: Commands) {
    let settings = MinimapSettings::default();
    
    // Main minimap container
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(20.0),
                right: Val::Px(20.0),
                width: Val::Px(settings.size),
                height: Val::Px(settings.size),
                ..default()
            },
            BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.8)),
            BorderColor(Color::srgb(0.3, 0.3, 0.3)),
            MinimapContainer,
        ))
        .with_children(|parent| {
            // Canvas for entity dots
            parent.spawn((
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    ..default()
                },
                MinimapCanvas,
            ));
        });

    commands.insert_resource(settings);
}

// === UPDATE MINIMAP SYSTEM ===
pub fn update_minimap_system(
    mut commands: Commands,
    settings: Res<MinimapSettings>,
    minimap_canvas: Query<Entity, With<MinimapCanvas>>,
    existing_dots: Query<Entity, (With<MinimapDot>, Without<MarkedForDespawn>)>,
    
    // Entity queries
    agents: Query<(Entity, &Transform), (With<Agent>, Without<Dead>)>,
    enemies: Query<(Entity, &Transform, &Faction), (With<Enemy>, Without<Dead>)>,
    civilians: Query<(Entity, &Transform), (With<Civilian>, Without<Dead>)>,
    terminals: Query<(Entity, &Transform), With<Terminal>>,
    
    camera: Query<&Transform, (With<Camera2d>, Without<Agent>, Without<Enemy>, Without<Civilian>)>,
    global_data: Res<GlobalData>,
) {
    // Get camera position for centering minimap
    let camera_pos = if let Ok(cam_transform) = camera.single() {
        cam_transform.translation.truncate()
    } else {
        Vec2::ZERO // Fallback if no camera found
    };
    
    // Clear existing dots
    for dot_entity in existing_dots.iter() {
        commands.entity(dot_entity).insert(MarkedForDespawn);
    }
    
    let Ok(canvas_entity) = minimap_canvas.single() else { return; };
    
    // Get main agent position for range calculation (use first agent if multiple)
    let main_agent_pos = agents.iter().next().map(|(_, t)| t.translation.truncate())
        .unwrap_or(camera_pos);
    
    // Update range based on agent upgrades
    let current_range = calculate_minimap_range(&global_data, &settings);

    // Collect all the dots we need to spawn first
let mut dots_to_spawn = Vec::new();

// Add agent dots (always visible)
for (entity, transform) in agents.iter() {
    let world_pos = transform.translation.truncate();
    if let Some(minimap_pos) = world_to_minimap_pos(world_pos, main_agent_pos, current_range, settings.size) {
        dots_to_spawn.push((entity, minimap_pos, get_agent_color(&settings)));
    }
}

// Add enemy dots
for (entity, transform, faction) in enemies.iter() {
    let world_pos = transform.translation.truncate();
    if world_pos.distance(main_agent_pos) <= current_range {
        if let Some(minimap_pos) = world_to_minimap_pos(world_pos, main_agent_pos, current_range, settings.size) {
            let color = if settings.show_colors {
                get_faction_color(faction)
            } else {
                Color::WHITE
            };
            dots_to_spawn.push((entity, minimap_pos, color));
        }
    }
}

// Add civilian dots
for (entity, transform) in civilians.iter() {
    let world_pos = transform.translation.truncate();
    if world_pos.distance(main_agent_pos) <= current_range {
        if let Some(minimap_pos) = world_to_minimap_pos(world_pos, main_agent_pos, current_range, settings.size) {
            let color = if settings.show_colors {
                Color::srgb(0.7, 0.7, 0.7)
            } else {
                Color::WHITE
            };
            dots_to_spawn.push((entity, minimap_pos, color));
        }
    }
}

// Add terminal dots
if settings.show_terminals {
    for (entity, transform) in terminals.iter() {
        let world_pos = transform.translation.truncate();
        if world_pos.distance(main_agent_pos) <= current_range {
            if let Some(minimap_pos) = world_to_minimap_pos(world_pos, main_agent_pos, current_range, settings.size) {
                dots_to_spawn.push((entity, minimap_pos, Color::srgb(0.2, 0.6, 1.0)));
            }
        }
    }
}

// Now spawn all dots in one go
commands.entity(canvas_entity).with_children(|parent| {
    for (entity_ref, pos, color) in dots_to_spawn {
        parent.spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(pos.x + 100.0),
                top: Val::Px(-pos.y + 100.0),
                width: Val::Px(4.0),
                height: Val::Px(4.0),
                ..default()
            },
            BackgroundColor(color),
            MinimapDot { entity_ref },
        ));
    }
});

}

// === HELPER FUNCTIONS ===
fn world_to_minimap_pos(world_pos: Vec2, center_pos: Vec2, range: f32, minimap_size: f32) -> Option<Vec2> {
    let relative_pos = world_pos - center_pos;
    let distance = relative_pos.length();
    
    if distance > range {
        return None; // Outside minimap range
    }
    
    // Convert to minimap coordinates (0,0 = center of minimap)
    let normalized = relative_pos / range; // -1 to 1
    let minimap_pos = normalized * (minimap_size * 0.4); // Use 80% of minimap size for entities
    
    Some(minimap_pos)
}

fn spawn_minimap_dot(parent: &mut Commands, entity_ref: Entity, pos: Vec2, color: Color, canvas_entity: Entity) {
    parent.entity(canvas_entity).with_children(|child_builder| {
        child_builder.spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(pos.x + 100.0),
                top: Val::Px(-pos.y + 100.0),
                width: Val::Px(4.0),
                height: Val::Px(4.0),
                ..default()
            },
            BackgroundColor(color),
            MinimapDot { entity_ref },
        ));
    });
}

fn calculate_minimap_range(global_data: &GlobalData, base_settings: &MinimapSettings) -> f32 {
    // Base range
    let mut range = base_settings.range;
    
    // Check if any agent has sensor upgrades
    // PLACEHOLDER
    let has_enhanced_sensors = global_data.agent_loadouts.iter().any(|agent_loadout| {
        agent_loadout.tools.contains(&ToolType::EnhancedSensors)
    });
    
    let has_satellite_uplink = global_data.agent_loadouts.iter().any(|agent_loadout| {
        agent_loadout.tools.contains(&ToolType::SatelliteUplink)
    });
    
    // Apply range boosts
    if has_enhanced_sensors {
        range *= 1.5;
    }
    
    if has_satellite_uplink {
        range *= 2.0;
    }
    
    range
}

fn get_agent_color(settings: &MinimapSettings) -> Color {
    if settings.show_colors {
        Color::srgb(0.2, 1.0, 0.2) // Green for agents
    } else {
        Color::WHITE
    }
}

fn get_faction_color(faction: &Faction) -> Color {
    match faction {
        Faction::Player => Color::srgb(0.2, 1.0, 0.2),     // Green
        Faction::Corporate => Color::srgb(1.0, 0.6, 0.2),  // Orange
        Faction::Syndicate => Color::srgb(1.0, 0.2, 0.2),  // Red
        Faction::Police => Color::srgb(0.2, 0.2, 1.0),     // Blue
        Faction::Civilian => Color::srgb(0.7, 0.7, 0.7),   // Gray
        _ => Color::WHITE,
    }
}

// === RESEARCH INTEGRATION ===
pub fn apply_minimap_research_benefits(
    mut settings: ResMut<MinimapSettings>,
    global_data: Res<GlobalData>,
) {
    if global_data.is_changed() {
        // PLACEHOLDER
        // Check if any agent has tactical scanner for color coding
        settings.show_colors = global_data.agent_loadouts.iter().any(|agent_loadout| {
            agent_loadout.tools.contains(&ToolType::TacticalScanner)
        });
        
        // Check if any agent has network scanner for terminal display
        settings.show_terminals = global_data.agent_loadouts.iter().any(|agent_loadout| {
            agent_loadout.tools.contains(&ToolType::NetworkScanner)
        });
    }
}

// === MINIMAP TOGGLE SYSTEM ===
pub fn minimap_toggle_system(
    mut minimap_container: Query<&mut Visibility, With<MinimapContainer>>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    if keyboard.just_pressed(KeyCode::KeyM) {
        if let Ok(mut visibility) = minimap_container.single_mut() {
            *visibility = match *visibility {
                Visibility::Visible => Visibility::Hidden,
                _ => Visibility::Visible,
            };
        }
    }
}

pub fn cleanup_minimap_ui(
    mut commands: Commands,
    minimap_query: Query<Entity, With<MinimapContainer>>,
) {
    for entity in minimap_query.iter() {
        commands.entity(entity).insert(MarkedForDespawn);
    }
}
