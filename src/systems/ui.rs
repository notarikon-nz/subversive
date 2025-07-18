use bevy::prelude::*;
use crate::core::*;

pub fn system(
    mut gizmos: Gizmos,
    game_mode: Res<GameMode>,
    selected_query: Query<(&Transform, &NeurovectorCapability), (With<Agent>, With<Selected>)>,
    target_query: Query<&Transform, With<NeurovectorTarget>>,
    controlled_query: Query<&Transform, With<NeurovectorControlled>>,
    enemy_query: Query<(&Transform, &Vision), With<Enemy>>,
    neurovector_query: Query<(&Transform, &NeurovectorCapability), With<Agent>>,
) {
    // Draw neurovector ranges for selected agents
    for (transform, neurovector) in selected_query.iter() {
        let color = if neurovector.current_cooldown > 0.0 {
            Color::srgba(0.8, 0.3, 0.3, 0.3)
        } else {
            Color::srgba(0.3, 0.3, 0.8, 0.3)
        };
        
        gizmos.circle_2d(transform.translation.truncate(), neurovector.range, color);
    }

    // Highlight targets when in neurovector targeting mode
    if let Some(TargetingMode::Neurovector { agent }) = &game_mode.targeting {
        if let Ok((agent_transform, neurovector)) = neurovector_query.get(*agent) {
            for target_transform in target_query.iter() {
                let distance = agent_transform.translation.truncate()
                    .distance(target_transform.translation.truncate());
                
                if distance <= neurovector.range {
                    gizmos.circle_2d(
                        target_transform.translation.truncate(),
                        20.0,
                        Color::srgb(0.8, 0.8, 0.3),
                    );
                }
            }
        }
    }

    // Draw control connections
    for (agent_transform, neurovector) in neurovector_query.iter() {
        for &controlled_entity in &neurovector.controlled {
            if let Ok(controlled_transform) = controlled_query.get(controlled_entity) {
                gizmos.line_2d(
                    agent_transform.translation.truncate(),
                    controlled_transform.translation.truncate(),
                    Color::srgb(0.8, 0.3, 0.8),
                );
            }
        }
    }

    // Draw enemy vision cones
    for (transform, vision) in enemy_query.iter() {
        draw_vision_cone(&mut gizmos, transform.translation.truncate(), vision);
    }
}

#[derive(Component)]
pub struct PauseUI;

pub fn pause_system(
    mut commands: Commands,
    game_mode: Res<GameMode>,
    pause_ui_query: Query<Entity, With<PauseUI>>,
) {
    if game_mode.paused {
        // Only create pause UI if it doesn't exist
        if pause_ui_query.is_empty() {
            commands.spawn((
                NodeBundle {
                    style: Style {
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        position_type: PositionType::Absolute,
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    background_color: Color::srgba(0.0, 0.0, 0.0, 0.5).into(),
                    z_index: ZIndex::Global(100),
                    ..default()
                },
                PauseUI, // Marker component
            )).with_children(|parent| {
                parent.spawn(TextBundle::from_section(
                    "PAUSED\nPress SPACE to resume",
                    TextStyle {
                        font_size: 32.0,
                        color: Color::WHITE,
                        ..default()
                    },
                ));
            });
        }
    } else {
        // Clear pause UI when not paused
        for entity in pause_ui_query.iter() {
            commands.entity(entity).despawn_recursive();
        }
    }
}

pub fn inventory_system(
    mut commands: Commands,
    inventory_state: Res<InventoryState>,
    agent_query: Query<&Inventory, With<Agent>>,
    inventory_ui_query: Query<Entity, (With<Node>, With<InventoryUI>)>,
) {
    // Clear existing inventory UI
    for entity in inventory_ui_query.iter() {
        commands.entity(entity).despawn_recursive();
    }
    
    if !inventory_state.ui_open {
        return;
    }
    
    // Get inventory data
    let inventory = if let Some(agent_entity) = inventory_state.selected_agent {
        agent_query.get(agent_entity).ok()
    } else {
        None
    };
    
    // Create inventory panel with marker component
    commands.spawn((
        NodeBundle {
            style: Style {
                width: Val::Px(400.0),
                height: Val::Px(500.0),
                position_type: PositionType::Absolute,
                left: Val::Px(50.0),
                top: Val::Px(50.0),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(10.0)),
                ..default()
            },
            background_color: Color::srgba(0.1, 0.1, 0.1, 0.9).into(),
            z_index: ZIndex::Global(50),
            ..default()
        },
        InventoryUI, // Marker component
    )).with_children(|parent| {
        // Title
        parent.spawn(TextBundle::from_section(
            "AGENT INVENTORY",
            TextStyle {
                font_size: 24.0,
                color: Color::WHITE,
                ..default()
            },
        ));
        
        if let Some(inv) = inventory {
            // Currency display
            parent.spawn(TextBundle::from_section(
                format!("Credits: {}", inv.currency),
                TextStyle {
                    font_size: 18.0,
                    color: Color::srgb(0.8, 0.8, 0.2),
                    ..default()
                },
            ));
            
            // Equipped weapon section
            if let Some(weapon) = &inv.equipped_weapon {
                parent.spawn(TextBundle::from_section(
                    format!("EQUIPPED WEAPON: {:?}", weapon),
                    TextStyle {
                        font_size: 16.0,
                        color: Color::srgb(0.9, 0.5, 0.2),
                        ..default()
                    },
                ));
            } else {
                parent.spawn(TextBundle::from_section(
                    "EQUIPPED WEAPON: None",
                    TextStyle {
                        font_size: 16.0,
                        color: Color::srgb(0.8, 0.3, 0.3),
                        ..default()
                    },
                ));
            }
            
            // Equipped tools section
            if !inv.equipped_tools.is_empty() {
                parent.spawn(TextBundle::from_section(
                    format!("EQUIPPED TOOLS: {:?}", inv.equipped_tools),
                    TextStyle {
                        font_size: 16.0,
                        color: Color::srgb(0.3, 0.8, 0.3),
                        ..default()
                    },
                ));
            }
            
            // Weapons section
            if !inv.weapons.is_empty() {
                parent.spawn(TextBundle::from_section(
                    "WEAPONS:",
                    TextStyle {
                        font_size: 16.0,
                        color: Color::srgb(0.8, 0.3, 0.3),
                        ..default()
                    },
                ));
                
                for weapon in &inv.weapons {
                    parent.spawn(TextBundle::from_section(
                        format!("• {:?}", weapon),
                        TextStyle {
                            font_size: 14.0,
                            color: Color::WHITE,
                            ..default()
                        },
                    ));
                }
            }
            
            // Tools section
            if !inv.tools.is_empty() {
                parent.spawn(TextBundle::from_section(
                    "TOOLS:",
                    TextStyle {
                        font_size: 16.0,
                        color: Color::srgb(0.3, 0.8, 0.3),
                        ..default()
                    },
                ));
                
                for tool in &inv.tools {
                    parent.spawn(TextBundle::from_section(
                        format!("• {:?}", tool),
                        TextStyle {
                            font_size: 14.0,
                            color: Color::WHITE,
                            ..default()
                        },
                    ));
                }
            }
            
            // Cybernetics section
            if !inv.cybernetics.is_empty() {
                parent.spawn(TextBundle::from_section(
                    "CYBERNETICS:",
                    TextStyle {
                        font_size: 16.0,
                        color: Color::srgb(0.3, 0.3, 0.8),
                        ..default()
                    },
                ));
                
                for cybernetic in &inv.cybernetics {
                    parent.spawn(TextBundle::from_section(
                        format!("• {:?}", cybernetic),
                        TextStyle {
                            font_size: 14.0,
                            color: Color::WHITE,
                            ..default()
                        },
                    ));
                }
            }
            
            // Intel documents section
            if !inv.intel_documents.is_empty() {
                parent.spawn(TextBundle::from_section(
                    "INTEL DOCUMENTS:",
                    TextStyle {
                        font_size: 16.0,
                        color: Color::srgb(0.8, 0.8, 0.3),
                        ..default()
                    },
                ));
                
                for (i, document) in inv.intel_documents.iter().enumerate() {
                    let preview = if document.len() > 50 {
                        format!("{}...", &document[..47])
                    } else {
                        document.clone()
                    };
                    
                    parent.spawn(TextBundle::from_section(
                        format!("• Document {}: {}", i + 1, preview),
                        TextStyle {
                            font_size: 12.0,
                            color: Color::WHITE,
                            ..default()
                        },
                    ));
                }
            }
        } else {
            parent.spawn(TextBundle::from_section(
                "No agent selected",
                TextStyle {
                    font_size: 16.0,
                    color: Color::srgb(0.8, 0.3, 0.3),
                    ..default()
                },
            ));
        }
        
        // Instructions
        parent.spawn(TextBundle::from_section(
            "\nPress 'I' to close inventory",
            TextStyle {
                font_size: 12.0,
                color: Color::srgb(0.7, 0.7, 0.7),
                ..default()
            },
        ));
    });
}

fn draw_vision_cone(gizmos: &mut Gizmos, position: Vec2, vision: &Vision) {
    let half_angle = vision.angle / 2.0;
    let segments = 16;
    
    let color = Color::srgba(1.0, 1.0, 0.3, 0.2);
    
    // Draw cone outline
    for i in 0..segments {
        let t1 = i as f32 / segments as f32;
        let t2 = (i + 1) as f32 / segments as f32;
        
        let angle1 = -half_angle + (vision.angle * t1);
        let angle2 = -half_angle + (vision.angle * t2);
        
        let dir1 = Vec2::new(
            vision.direction.x * angle1.cos() - vision.direction.y * angle1.sin(),
            vision.direction.x * angle1.sin() + vision.direction.y * angle1.cos(),
        );
        
        let dir2 = Vec2::new(
            vision.direction.x * angle2.cos() - vision.direction.y * angle2.sin(),
            vision.direction.x * angle2.sin() + vision.direction.y * angle2.cos(),
        );
        
        let point1 = position + dir1 * vision.range;
        let point2 = position + dir2 * vision.range;
        
        gizmos.line_2d(point1, point2, color);
    }
    
    // Draw cone edges
    let left_dir = Vec2::new(
        vision.direction.x * half_angle.cos() - vision.direction.y * half_angle.sin(),
        vision.direction.x * half_angle.sin() + vision.direction.y * half_angle.cos(),
    );
    
    let right_dir = Vec2::new(
        vision.direction.x * half_angle.cos() + vision.direction.y * half_angle.sin(),
        -vision.direction.x * half_angle.sin() + vision.direction.y * half_angle.cos(),
    );
    
    gizmos.line_2d(position, position + left_dir * vision.range, color);
    gizmos.line_2d(position, position + right_dir * vision.range, color);
}