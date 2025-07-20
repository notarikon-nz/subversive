// src/systems/ui/hub/missions.rs - Updated for Bevy 0.16
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

pub fn create_content(parent: &mut ChildSpawnerCommands, global_data: &GlobalData, hub_state: &super::HubState) {
    let briefing = generate_mission_briefing(global_data, hub_state.selected_region);
    
    parent.spawn(Node {
        width: Val::Percent(100.0),
        flex_grow: 1.0,
        padding: UiRect::all(Val::Px(20.0)),
        flex_direction: FlexDirection::Column,
        row_gap: Val::Px(15.0),
        overflow: Overflow {
            x: OverflowAxis::Visible,
            y: OverflowAxis::Scroll,
        },
        ..default()
    }).with_children(|content| {
        let region = &global_data.regions[hub_state.selected_region];
        
        // Header
        content.spawn(Node {
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::SpaceBetween,
            align_items: AlignItems::Center,
            ..default()
        }).with_children(|header| {
            header.spawn((
                Text::new(format!("MISSION BRIEFING: {}", region.name)),
                TextFont { font_size: 24.0, ..default() },
                TextColor(Color::srgb(0.8, 0.2, 0.2)),
            ));
            
            header.spawn((
                Text::new(format!("THREAT LEVEL: {}", region.threat_level)),
                TextFont { font_size: 18.0, ..default() },
                TextColor(briefing.risks.casualty_risk.color()),
            ));
        });
        
        // Alert status and environmental info
        content.spawn(Node {
            flex_direction: FlexDirection::Row,
            column_gap: Val::Px(30.0),
            ..default()
        }).with_children(|info_row| {
            info_row.spawn((
                Text::new(format!("Alert Status: {:?}", region.alert_level)),
                TextFont { font_size: 16.0, ..default() },
                TextColor(alert_color(region.alert_level)),
            ));
            
            info_row.spawn((
                Text::new(format!("Time: {:?} | Visibility: {:.0}%", 
                        briefing.environment.time_of_day, 
                        briefing.environment.visibility * 100.0)),
                TextFont { font_size: 16.0, ..default() },
                TextColor(Color::WHITE),
            ));
        });
        
        // Mission Objectives Section
        create_objectives_section(content, &briefing.objectives[..]);
        
        // Intelligence Assessment Section  
        create_intelligence_section(content, &briefing.resistance, &briefing.environment);
        
        // Risk Assessment Section
        create_risk_assessment_section(content, &briefing.risks, global_data);
        
        // Equipment Recommendations Section
        create_equipment_recommendations_section(content, &briefing.risks, global_data);
        
        // Squad Status and Deployment
        create_deployment_section(content, global_data);
        
        // Mission Rewards
        create_rewards_section(content, &briefing.rewards, region);
    });
}

fn create_objectives_section(parent: &mut ChildSpawnerCommands, objectives: &[MissionObjective]) {
    parent.spawn((
        Node {
            flex_direction: FlexDirection::Column,
            padding: UiRect::all(Val::Px(15.0)),
            row_gap: Val::Px(8.0),
            ..default()
        },
        BackgroundColor(Color::srgba(0.2, 0.1, 0.1, 0.3)),
    )).with_children(|section| {
        section.spawn((
            Text::new("MISSION OBJECTIVES"),
            TextFont { font_size: 18.0, ..default() },
            TextColor(Color::srgb(0.8, 0.2, 0.2)),
        ));
        
        for objective in objectives {
            let prefix = if objective.required { "[REQUIRED]" } else { "[OPTIONAL]" };
            let color = if objective.required { 
                Color::srgb(0.8, 0.3, 0.3) 
            } else { 
                Color::srgb(0.6, 0.6, 0.8) 
            };
            
            let difficulty_stars = "★".repeat(objective.difficulty as usize);
            
            section.spawn((
                Text::new(format!("{} {} {}", prefix, objective.name, difficulty_stars)),
                TextFont { font_size: 16.0, ..default() },
                TextColor(color),
            ));
            
            section.spawn((
                Text::new(format!("  {}", objective.description)),
                TextFont { font_size: 14.0, ..default() },
                TextColor(Color::srgb(0.8, 0.8, 0.8)),
            ));
        }
    });
}

fn create_intelligence_section(parent: &mut ChildSpawnerCommands, resistance: &ResistanceProfile, environment: &EnvironmentData) {
    parent.spawn((
        Node {
            flex_direction: FlexDirection::Column,
            padding: UiRect::all(Val::Px(15.0)),
            row_gap: Val::Px(8.0),
            ..default()
        },
        BackgroundColor(Color::srgba(0.1, 0.1, 0.2, 0.3)),
    )).with_children(|section| {
        section.spawn((
            Text::new("INTELLIGENCE ASSESSMENT"),
            TextFont { font_size: 18.0, ..default() },
            TextColor(Color::srgb(0.2, 0.5, 0.8)),
        ));
        
        // Resistance details
        section.spawn(Node {
            flex_direction: FlexDirection::Row,
            column_gap: Val::Px(20.0),
            ..default()
        }).with_children(|intel_row| {
            intel_row.spawn((
                Text::new(format!("Enemy Forces: {} units", resistance.enemy_count)),
                TextFont { font_size: 14.0, ..default() },
                TextColor(Color::WHITE),
            ));
            
            intel_row.spawn((
                Text::new(format!("Security Level: {}/5", resistance.security_level)),
                TextFont { font_size: 14.0, ..default() },
                TextColor(Color::WHITE),
            ));
            
            intel_row.spawn((
                Text::new(format!("Alert Sensitivity: {:.0}%", resistance.alert_sensitivity * 100.0)),
                TextFont { font_size: 14.0, ..default() },
                TextColor(Color::WHITE),
            ));
        });
        
        // Enemy composition
        let enemy_types = resistance.enemy_types.iter()
            .map(|e| format!("{:?}", e))
            .collect::<Vec<_>>()
            .join(", ");
        section.spawn((
            Text::new(format!("Expected Opposition: {}", enemy_types)),
            TextFont { font_size: 14.0, ..default() },
            TextColor(Color::srgb(0.8, 0.6, 0.6)),
        ));
        
        // Environment details
        section.spawn((
            Text::new(format!("Terrain: {:?} | Cover Density: {:.0}% | Civilians: {}", 
                    environment.terrain, 
                    environment.cover_density * 100.0, 
                    match environment.civilian_presence {
                        0 => "None",
                        1..=2 => "Light",
                        3..=4 => "Moderate", 
                        _ => "Heavy"
                    })),
            TextFont { font_size: 14.0, ..default() },
            TextColor(Color::srgb(0.6, 0.8, 0.6)),
        ));
    });
}

fn create_risk_assessment_section(parent: &mut ChildSpawnerCommands, risks: &RiskAssessment, global_data: &GlobalData) {
    parent.spawn((
        Node {
            flex_direction: FlexDirection::Column,
            padding: UiRect::all(Val::Px(15.0)),
            row_gap: Val::Px(8.0),
            ..default()
        },
        BackgroundColor(Color::srgba(0.2, 0.2, 0.1, 0.3)),
    )).with_children(|section| {
        section.spawn((
            Text::new("RISK ASSESSMENT"),
            TextFont { font_size: 18.0, ..default() },
            TextColor(Color::srgb(0.8, 0.8, 0.2)),
        ));
        
        // Risk matrix
        section.spawn(Node {
            flex_direction: FlexDirection::Row,
            column_gap: Val::Px(25.0),
            ..default()
        }).with_children(|risk_row| {
            risk_row.spawn((
                Text::new(format!("Casualty Risk: {}", risks.casualty_risk.text())),
                TextFont { font_size: 14.0, ..default() },
                TextColor(risks.casualty_risk.color()),
            ));
            
            risk_row.spawn((
                Text::new(format!("Detection Risk: {}", risks.detection_risk.text())),
                TextFont { font_size: 14.0, ..default() },
                TextColor(risks.detection_risk.color()),
            ));
            
            risk_row.spawn((
                Text::new(format!("Equipment Loss: {}", risks.equipment_loss_risk.text())),
                TextFont { font_size: 14.0, ..default() },
                TextColor(risks.equipment_loss_risk.color()),
            ));
        });
        
        // Mission analysis
        section.spawn((
            Text::new(format!("Mission Failure Probability: {:.0}%", risks.mission_failure_chance * 100.0)),
            TextFont { font_size: 14.0, ..default() },
            TextColor(if risks.mission_failure_chance > 0.5 { 
                Color::srgb(0.8, 0.2, 0.2) 
            } else { 
                Color::WHITE 
            }),
        ));
        
        // Agent readiness assessment
        let avg_agent_level = global_data.agent_levels.iter().sum::<u8>() as f32 / 3.0;
        let readiness_color = if avg_agent_level >= risks.recommended_agent_level as f32 {
            Color::srgb(0.2, 0.8, 0.2)
        } else {
            Color::srgb(0.8, 0.5, 0.2)
        };
        
        section.spawn((
            Text::new(format!("Recommended Agent Level: {} (Squad Average: {:.1})", 
                    risks.recommended_agent_level, avg_agent_level)),
            TextFont { font_size: 14.0, ..default() },
            TextColor(readiness_color),
        ));
    });
}

fn create_equipment_recommendations_section(parent: &mut ChildSpawnerCommands, risks: &RiskAssessment, global_data: &GlobalData) {
    parent.spawn((
        Node {
            flex_direction: FlexDirection::Column,
            padding: UiRect::all(Val::Px(15.0)),
            row_gap: Val::Px(8.0),
            ..default()
        },
        BackgroundColor(Color::srgba(0.1, 0.2, 0.1, 0.3)),
    )).with_children(|section| {
        section.spawn((
            Text::new("EQUIPMENT RECOMMENDATIONS"),
            TextFont { font_size: 18.0, ..default() },
            TextColor(Color::srgb(0.2, 0.8, 0.2)),
        ));
        
        // Recommended loadout
        section.spawn((
            Text::new("Recommended Equipment:"),
            TextFont { font_size: 16.0, ..default() },
            TextColor(Color::WHITE),
        ));
        
        for item in &risks.recommended_loadout {
            section.spawn((
                Text::new(format!("• {}", item)),
                TextFont { font_size: 14.0, ..default() },
                TextColor(Color::srgb(0.8, 0.8, 0.8)),
            ));
        }
        
        // Attachment recommendations based on mission type
        section.spawn((
            Text::new("\nSuggested Weapon Modifications:"),
            TextFont { font_size: 16.0, ..default() },
            TextColor(Color::WHITE),
        ));
        
        let attachment_suggestions = match risks.detection_risk {
            RiskLevel::High | RiskLevel::Extreme => vec![
                "Sound Suppressor - Critical for stealth",
                "Red Dot Sight - Improved accuracy in CQB",
                "Extended Magazine - Sustained firefights"
            ],
            RiskLevel::Medium => vec![
                "Tactical Sight - Better target acquisition", 
                "Flash Hider - Reduce muzzle flash",
                "Grip - Stability for longer engagements"
            ],
            RiskLevel::Low => vec![
                "Any available attachments",
                "Focus on agent training over equipment"
            ],
        };
        
        for suggestion in attachment_suggestions {
            section.spawn((
                Text::new(format!("• {}", suggestion)),
                TextFont { font_size: 14.0, ..default() },
                TextColor(Color::srgb(0.6, 0.8, 0.6)),
            ));
        }
    });
}

fn create_deployment_section(parent: &mut ChildSpawnerCommands, global_data: &GlobalData) {
    parent.spawn((
        Node {
            flex_direction: FlexDirection::Column,
            padding: UiRect::all(Val::Px(15.0)),
            row_gap: Val::Px(8.0),
            ..default()
        },
        BackgroundColor(Color::srgba(0.2, 0.1, 0.2, 0.3)),
    )).with_children(|section| {
        section.spawn((
            Text::new("SQUAD DEPLOYMENT STATUS"),
            TextFont { font_size: 18.0, ..default() },
            TextColor(Color::srgb(0.8, 0.2, 0.8)),
        ));
        
        let ready_agents = (0..3).filter(|&i| global_data.agent_recovery[i] <= global_data.current_day).count();
        
        if ready_agents > 0 {
            section.spawn((
                Text::new(format!("Deployment Ready: {} agents available", ready_agents)),
                TextFont { font_size: 16.0, ..default() },
                TextColor(Color::srgb(0.2, 0.8, 0.2)),
            ));
            
            // Show ready agents with their loadouts
            for i in 0..3 {
                if global_data.agent_recovery[i] <= global_data.current_day {
                    let loadout = global_data.get_agent_loadout(i);
                    let weapon_name = if let Some(config) = loadout.weapon_configs.get(loadout.equipped_weapon_idx) {
                        format!("{:?}", config.base_weapon)
                    } else {
                        "No Weapon".to_string()
                    };
                    
                    section.spawn((
                        Text::new(format!("Agent {}: Lv{} | {} | {} tools", 
                                i + 1, 
                                global_data.agent_levels[i],
                                weapon_name,
                                loadout.tools.len())),
                        TextFont { font_size: 14.0, ..default() },
                        TextColor(Color::WHITE),
                    ));
                }
            }
            
            section.spawn((
                Text::new("\nPress ENTER to launch mission"),
                TextFont { font_size: 16.0, ..default() },
                TextColor(Color::srgb(0.8, 0.8, 0.2)),
            ));
        } else {
            section.spawn((
                Text::new("No agents available - all recovering from previous missions"),
                TextFont { font_size: 16.0, ..default() },
                TextColor(Color::srgb(0.8, 0.2, 0.2)),
            ));
            
            section.spawn((
                Text::new("Use 'W' on Global Map to advance time or wait for recovery"),
                TextFont { font_size: 14.0, ..default() },
                TextColor(Color::srgb(0.6, 0.6, 0.6)),
            ));
        }
    });
}

fn create_rewards_section(parent: &mut ChildSpawnerCommands, rewards: &MissionRewards, region: &Region) {
    parent.spawn((
        Node {
            flex_direction: FlexDirection::Column,
            padding: UiRect::all(Val::Px(15.0)),
            row_gap: Val::Px(8.0),
            ..default()
        },
        BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.3)),
    )).with_children(|section| {
        section.spawn((
            Text::new("MISSION REWARDS"),
            TextFont { font_size: 18.0, ..default() },
            TextColor(Color::srgb(0.8, 0.8, 0.2)),
        ));
        
        let difficulty_bonus = region.mission_difficulty_modifier();
        let total_credits = (rewards.base_credits as f32 * difficulty_bonus) as u32;
        
        section.spawn(Node {
            flex_direction: FlexDirection::Row,
            column_gap: Val::Px(20.0),
            ..default()
        }).with_children(|rewards_row| {
            rewards_row.spawn((
                Text::new(format!("Base Credits: {}", total_credits)),
                TextFont { font_size: 14.0, ..default() },
                TextColor(Color::srgb(0.8, 0.8, 0.2)),
            ));
            
            rewards_row.spawn((
                Text::new(format!("Bonus Potential: +{}", rewards.bonus_credits)),
                TextFont { font_size: 14.0, ..default() },
                TextColor(Color::srgb(0.6, 0.8, 0.6)),
            ));
            
            rewards_row.spawn((
                Text::new(format!("Equipment Drop: {:.0}%", rewards.equipment_chance * 100.0)),
                TextFont { font_size: 14.0, ..default() },
                TextColor(Color::srgb(0.6, 0.6, 0.8)),
            ));
        });
        
        section.spawn((
            Text::new(format!("Experience Modifier: {:.1}x | Intel Value: {}/5", 
                    rewards.experience_modifier, rewards.intel_value)),
            TextFont { font_size: 14.0, ..default() },
            TextColor(Color::WHITE),
        ));
    });
}

fn alert_color(alert_level: AlertLevel) -> Color {
    match alert_level {
        AlertLevel::Green => Color::srgb(0.2, 0.8, 0.2),
        AlertLevel::Yellow => Color::srgb(0.8, 0.8, 0.2),
        AlertLevel::Orange => Color::srgb(0.8, 0.5, 0.2),
        AlertLevel::Red => Color::srgb(0.8, 0.2, 0.2),
    }
}