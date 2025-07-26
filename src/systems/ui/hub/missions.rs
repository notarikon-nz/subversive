// src/systems/ui/hub/missions.rs - Simplified using UIBuilder
use bevy::prelude::*;
use bevy::input::mouse::*;
use crate::core::*;
use crate::systems::ui::builder::*;
use crate::systems::ui::hub::*;


// WILL EVENTUALLY BE UNIVERSAL
#[derive(Component)]
pub struct ScrollContainer {
    pub scroll_y: f32,
    pub max_scroll: f32,
    pub container_height: f32,
}

impl Default for ScrollContainer {
    fn default() -> Self {
        Self {
            scroll_y: 0.0,
            max_scroll: 0.0,
            container_height: 400.0, // Height of the scrollable area
        }
    }
}

#[derive(Component)]
pub struct ScrollableContent;

#[derive(Component)]
pub struct ScrollbarThumb;

pub fn handle_input(
    input: &ButtonInput<KeyCode>,
    commands: &mut Commands,
    next_state: &mut NextState<GameState>,
    global_data: &GlobalData,
    cities_progress: &CitiesProgress,
    mut scroll_events: EventReader<MouseWheel>,
    windows: &Query<&Window>,  // Changed from Query<&Window>
    cameras: &Query<(&Camera, &GlobalTransform)>,  // Changed from Query<...>
    scroll_params: ParamSet<(
        Query<(Entity, &mut ScrollContainer, &GlobalTransform)>,
        Query<&mut Node, With<ScrollableContent>>,
        Query<&mut Node, With<ScrollbarThumb>>,
    )>,
    hub_states: &mut HubStates,
) -> bool {
    // LAUNCH MISSION
    if input.just_pressed(KeyCode::Enter) {
        let ready_agents = (0..3).filter(|&i| global_data.agent_recovery[i] <= global_data.current_day).count();

        if ready_agents > 0 {
            commands.insert_resource(MissionLaunchData {
                city_id: global_data.cities_progress.current_city.clone(),
                region_id: global_data.selected_region,
            });

            commands.insert_resource(ShouldRestart);
            next_state.set(GameState::Mission);
        }
    }
    // Handle mouse wheel scrolling
    let mut scroll_changed = false;
    for scroll_event in scroll_events.read() {
        let scroll_delta = scroll_event.y * 30.0;
        let old_scroll = hub_states.mission_scroll;
        
        // Update scroll with bounds
        hub_states.mission_scroll = (hub_states.mission_scroll - scroll_delta)
            .max(0.0)
            .min(hub_states.mission_max_scroll); // This will be calculated dynamically
        
        if old_scroll != hub_states.mission_scroll {
            scroll_changed = true;
        }
    }
    
    scroll_changed
}

const SIZE: f32 = 12.0;

pub fn create_content(
    parent: &mut ChildSpawnerCommands, 
    global_data: &GlobalData, 
    cities_db: &CitiesDatabase,
    cities_progress: &CitiesProgress, // can remove
    hub_states: &HubStates,
) {
    parent.spawn(UIBuilder::content_area()).with_children(|content| {
        
        if let Some(city) = cities_db.get_city(&global_data.cities_progress.current_city) {
            let city_state = global_data.cities_progress.get_city_state(&global_data.cities_progress.current_city);
            let briefing = generate_mission_briefing_for_city(global_data, cities_db, cities_progress, &global_data.cities_progress.current_city);
            
            // Header
            content.spawn(
                UIBuilder::row(0.0), // Node
            ).with_children(|header| {
                header.spawn(UIBuilder::text(&format!("MISSION BRIEFING: {}", city.name), SIZE, Color::srgb(0.8, 0.2, 0.2)));
                
                let threat_level = match city.corruption_level {
                    1..=3 => "LOW",
                    4..=6 => "MODERATE", 
                    7..=8 => "HIGH",
                    9..=10 => "EXTREME",
                    _ => "UNKNOWN"
                };
                
                header.spawn(UIBuilder::text(&format!("THREAT: {}", threat_level), SIZE, briefing.risks.casualty_risk.color()));
            });
            
            // Status info
            content.spawn(UIBuilder::row(30.0)).with_children(|info_row| {
                info_row.spawn(UIBuilder::text(&format!("Alert: {:?}", city_state.alert_level), SIZE, alert_color(city_state.alert_level)));
                info_row.spawn(UIBuilder::text(&format!("Time: {:?} | Visibility: {:.0}%", 
                        briefing.environment.time_of_day, 
                        briefing.environment.visibility * 100.0), SIZE, Color::WHITE));
            });
            
            // Main scrollable section with scrollbar
            content.spawn((
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Px(400.0),
                    flex_direction: FlexDirection::Row,
                    overflow: Overflow::clip(),
                    ..default()
                },
            )).with_children(|scroll_wrapper| {
                
    // Scrollable content area (95% width) - This maintains the layout
    scroll_wrapper.spawn((
        Node {
            width: Val::Percent(95.0),
            height: Val::Percent(100.0),
            position_type: PositionType::Relative, // Keep this relative to maintain layout
            overflow: Overflow::clip(),
            ..default()
        },
    )).with_children(|scroll_area| {
        
        // Inner scrollable content with absolute positioning
        scroll_area.spawn((
            Node {
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(15.0),
                position_type: PositionType::Absolute, // Absolute within the relative parent
                top: Val::Px(-hub_states.mission_scroll),
                left: Val::Px(0.0),
                right: Val::Px(0.0),
                // Don't set bottom - let it size naturally
                ..default()
            },
            ScrollableContent,
        )).with_children(|scrollable_content| {
            create_objectives_section(scrollable_content, &briefing.objectives);
            create_intelligence_section(scrollable_content, &briefing.resistance, &briefing.environment);
            create_risk_assessment_section(scrollable_content, &briefing.risks, global_data);
            create_deployment_section(scrollable_content, global_data);
            create_rewards_section(scrollable_content, &briefing.rewards, city, &city_state);
        });
    });
                
                // Scrollbar area (5% width)
                scroll_wrapper.spawn((
                    Node {
                        width: Val::Px(16.0),
                        height: Val::Percent(100.0),
                        flex_direction: FlexDirection::Column,
                        justify_content: JustifyContent::FlexStart,
                        overflow: Overflow::clip_y(),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.2, 0.2, 0.2)),
                )).with_children(|scrollbar_track| {
                    
                    // Calculate thumb position based on scroll
                    let container_height: f32 = 400.0;
                    let content_height: f32 = 500.0; // Estimate - you might want to calculate this
                    let max_scroll = (content_height - container_height).max(0.0);
                    let thumb_height = (container_height / content_height * container_height).min(container_height);
                    let thumb_position = if max_scroll > 0.0 {
                        (hub_states.mission_scroll / max_scroll) * (container_height - thumb_height)
                    } else {
                        0.0
                    };

                    // Scrollbar thumb
                    scrollbar_track.spawn((
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Px(thumb_height),
                            position_type: PositionType::Absolute,
                            top: Val::Px(thumb_position),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.5, 0.5, 0.5)),
                        ScrollbarThumb,
                    ));
                });
            });
            // end scrollable content area

        }
    });
}

fn create_objectives_section(parent: &mut ChildSpawnerCommands, objectives: &[MissionObjective]) {
    let (panel_node, _) = UIBuilder::section_panel();
    parent.spawn((panel_node, BackgroundColor(Color::srgba(0.2, 0.1, 0.1, 0.3)))).with_children(|section| {
        section.spawn(UIBuilder::text("MISSION OBJECTIVES", SIZE, Color::srgb(0.8, 0.2, 0.2)));
        
        for objective in objectives {
            let prefix = if objective.required { "[REQUIRED]" } else { "[OPTIONAL]" };
            let color = if objective.required { Color::srgb(0.8, 0.3, 0.3) } else { Color::srgb(0.6, 0.6, 0.8) };
            let difficulty_stars = "★".repeat(objective.difficulty as usize);
            
            section.spawn(UIBuilder::text(&format!("{} {} {}", prefix, objective.name, difficulty_stars), SIZE, color));
            section.spawn(UIBuilder::text(&format!("  {}", objective.description), SIZE, Color::srgb(0.8, 0.8, 0.8)));
        }
    });
}

fn create_intelligence_section(parent: &mut ChildSpawnerCommands, resistance: &ResistanceProfile, environment: &EnvironmentData) {
    let (panel_node, _) = UIBuilder::section_panel();
    parent.spawn((panel_node, BackgroundColor(Color::srgba(0.1, 0.1, 0.2, 0.3)))).with_children(|section| {
        section.spawn(UIBuilder::text("INTELLIGENCE ASSESSMENT", SIZE, Color::srgb(0.2, 0.5, 0.8)));
        
        section.spawn(UIBuilder::row(20.0)).with_children(|intel_row| {
            let mut stats = StatsBuilder::new(intel_row);
            stats.stat("Enemy Forces", &resistance.enemy_count.to_string(), None);
            stats.stat("Security Level", &format!("{}/5", resistance.security_level), None);
            stats.stat("Alert Sensitivity", &format!("{:.0}%", resistance.alert_sensitivity * 100.0), None);
        });
        
        let enemy_types = resistance.enemy_types.iter()
            .map(|e| format!("{:?}", e))
            .collect::<Vec<_>>()
            .join(", ");
        section.spawn(UIBuilder::text(&format!("Opposition: {}", enemy_types), SIZE, Color::srgb(0.8, 0.6, 0.6)));
        
        section.spawn(UIBuilder::text(&format!("Terrain: {:?} | Cover: {:.0}% | Civilians: {}", 
                environment.terrain, 
                environment.cover_density * 100.0, 
                match environment.civilian_presence {
                    0 => "None", 1..=2 => "Light", 3..=4 => "Moderate", _ => "Heavy"
                }), SIZE, Color::srgb(0.6, 0.8, 0.6)));
    });
}

fn create_risk_assessment_section(parent: &mut ChildSpawnerCommands, risks: &RiskAssessment, global_data: &GlobalData) {
    let (panel_node, _) = UIBuilder::section_panel();
    parent.spawn((panel_node, BackgroundColor(Color::srgba(0.2, 0.2, 0.1, 0.3)))).with_children(|section| {
        section.spawn(UIBuilder::text("RISK ASSESSMENT", SIZE, Color::srgb(0.8, 0.8, 0.2)));
        
        section.spawn(UIBuilder::row(25.0)).with_children(|risk_row| {
            risk_row.spawn(UIBuilder::text(&format!("Casualty: {}", risks.casualty_risk.text()), SIZE, risks.casualty_risk.color()));
            risk_row.spawn(UIBuilder::text(&format!("Detection: {}", risks.detection_risk.text()), SIZE, risks.detection_risk.color()));
            risk_row.spawn(UIBuilder::text(&format!("Equipment Loss: {}", risks.equipment_loss_risk.text()), SIZE, risks.equipment_loss_risk.color()));
        });
        
        let failure_color = if risks.mission_failure_chance > 0.5 { Color::srgb(0.8, 0.2, 0.2) } else { Color::WHITE };
        section.spawn(UIBuilder::text(&format!("Failure Probability: {:.0}%", risks.mission_failure_chance * 100.0), SIZE, failure_color));
        
        let avg_agent_level = global_data.agent_levels.iter().sum::<u8>() as f32 / 3.0;
        let readiness_color = if avg_agent_level >= risks.recommended_agent_level as f32 {
            Color::srgb(0.2, 0.8, 0.2)
        } else {
            Color::srgb(0.8, 0.5, 0.2)
        };
        
        section.spawn(UIBuilder::text(&format!("Recommended Level: {} (Squad: {:.1})", 
                risks.recommended_agent_level, avg_agent_level), SIZE, readiness_color));
    });
}

fn create_deployment_section(parent: &mut ChildSpawnerCommands, global_data: &GlobalData) {
    let (panel_node, _) = UIBuilder::section_panel();
    parent.spawn((panel_node, BackgroundColor(Color::srgba(0.2, 0.1, 0.2, 0.3)))).with_children(|section| {
        section.spawn(UIBuilder::text("SQUAD DEPLOYMENT STATUS", SIZE, Color::srgb(0.8, 0.2, 0.8)));
        
        let ready_agents = (0..3).filter(|&i| global_data.agent_recovery[i] <= global_data.current_day).count();
        
        if ready_agents > 0 {
            section.spawn(UIBuilder::success_text(&format!("Deployment Ready: {} agents available", ready_agents)));
            
            for i in 0..3 {
                if global_data.agent_recovery[i] <= global_data.current_day {
                    let loadout = global_data.get_agent_loadout(i);
                    let weapon_name = if let Some(config) = loadout.weapon_configs.get(loadout.equipped_weapon_idx) {
                        format!("{:?}", config.base_weapon)
                    } else {
                        "No Weapon".to_string()
                    };
                    
                    section.spawn(UIBuilder::text(&format!("Agent {}: Lv{} | {} | {} tools", 
                            i + 1, 
                            global_data.agent_levels[i],
                            weapon_name,
                            loadout.tools.len()), SIZE, Color::WHITE));
                }
            }
            
            section.spawn(UIBuilder::text("Press ENTER to launch mission", SIZE, Color::srgb(0.8, 0.8, 0.2)));
        } else {
            section.spawn(UIBuilder::error_text("No agents available - all recovering"));
            section.spawn(UIBuilder::small("Use 'W' on Global Map to advance time"));
        }
    });
}

fn create_rewards_section(parent: &mut ChildSpawnerCommands, rewards: &MissionRewards, city: &City, city_state: &CityState) {
    let (panel_node, _) = UIBuilder::section_panel();
    parent.spawn((panel_node, BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.3)))).with_children(|section| {
        section.spawn(UIBuilder::text("MISSION REWARDS", SIZE, Color::srgb(0.8, 0.8, 0.2)));
        
        let difficulty_bonus = match city.corruption_level {
            1..=3 => 1.0, 4..=6 => 1.2, 7..=8 => 1.5, 9..=10 => 2.0, _ => 1.0
        };
        let total_credits = (rewards.base_credits as f32 * difficulty_bonus) as u32;
        
        section.spawn(UIBuilder::row(20.0)).with_children(|rewards_row| {
            let mut stats = StatsBuilder::new(rewards_row);
            stats.stat("Base Credits", &total_credits.to_string(), Some(Color::srgb(0.8, 0.8, 0.2)));
            stats.stat("Bonus Potential", &format!("+{}", rewards.bonus_credits), Some(Color::srgb(0.6, 0.8, 0.6)));
            stats.stat("Equipment Drop", &format!("{:.0}%", rewards.equipment_chance * 100.0), Some(Color::srgb(0.6, 0.6, 0.8)));
        });
        
        section.spawn(UIBuilder::text(&format!("XP Modifier: {:.1}x | Intel: {}/5", 
                rewards.experience_modifier, rewards.intel_value), SIZE, Color::WHITE));
        
        section.spawn(UIBuilder::text(&format!("Corporation: {:?} | Population: {}M", 
                city.controlling_corp, city.population), SIZE, Color::srgb(0.7, 0.7, 0.7)));
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