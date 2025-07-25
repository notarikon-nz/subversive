// src/systems/ui/hub/missions.rs - Simplified using UIBuilder
use bevy::prelude::*;
use crate::core::*;
use crate::systems::ui::builder::*;



pub fn handle_input(
    input: &ButtonInput<KeyCode>,
    commands: &mut Commands,
    next_state: &mut NextState<GameState>,
    global_data: &GlobalData,
    cities_progress: &CitiesProgress,
) -> bool {
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
    false
}

pub fn create_content(
    parent: &mut ChildSpawnerCommands, 
    global_data: &GlobalData, 
    cities_db: &CitiesDatabase,
    cities_progress: &CitiesProgress,
) {
    parent.spawn(UIBuilder::content_area()).with_children(|content| {
        
        if let Some(city) = cities_db.get_city(&global_data.cities_progress.current_city) {
            let city_state = global_data.cities_progress.get_city_state(&global_data.cities_progress.current_city);
            let briefing = generate_mission_briefing_for_city(global_data, cities_db, cities_progress, &global_data.cities_progress.current_city);
            
            // Header
            content.spawn(UIBuilder::row(0.0)).with_children(|header| {
                header.spawn(UIBuilder::text(&format!("MISSION BRIEFING: {}", city.name), 24.0, Color::srgb(0.8, 0.2, 0.2)));
                
                let threat_level = match city.corruption_level {
                    1..=3 => "LOW",
                    4..=6 => "MODERATE", 
                    7..=8 => "HIGH",
                    9..=10 => "EXTREME",
                    _ => "UNKNOWN"
                };
                
                header.spawn(UIBuilder::text(&format!("THREAT: {}", threat_level), 18.0, briefing.risks.casualty_risk.color()));
            });
            
            // Status info
            content.spawn(UIBuilder::row(30.0)).with_children(|info_row| {
                info_row.spawn(UIBuilder::text(&format!("Alert: {:?}", city_state.alert_level), 16.0, alert_color(city_state.alert_level)));
                info_row.spawn(UIBuilder::text(&format!("Time: {:?} | Visibility: {:.0}%", 
                        briefing.environment.time_of_day, 
                        briefing.environment.visibility * 100.0), 16.0, Color::WHITE));
            });
            
            create_objectives_section(content, &briefing.objectives);
            create_intelligence_section(content, &briefing.resistance, &briefing.environment);
            create_risk_assessment_section(content, &briefing.risks, global_data);
            create_deployment_section(content, global_data);
            create_rewards_section(content, &briefing.rewards, city, &city_state);
        }
    });
}

fn create_objectives_section(parent: &mut ChildSpawnerCommands, objectives: &[MissionObjective]) {
    let (panel_node, _) = UIBuilder::section_panel();
    parent.spawn((panel_node, BackgroundColor(Color::srgba(0.2, 0.1, 0.1, 0.3)))).with_children(|section| {
        section.spawn(UIBuilder::text("MISSION OBJECTIVES", 18.0, Color::srgb(0.8, 0.2, 0.2)));
        
        for objective in objectives {
            let prefix = if objective.required { "[REQUIRED]" } else { "[OPTIONAL]" };
            let color = if objective.required { Color::srgb(0.8, 0.3, 0.3) } else { Color::srgb(0.6, 0.6, 0.8) };
            let difficulty_stars = "★".repeat(objective.difficulty as usize);
            
            section.spawn(UIBuilder::text(&format!("{} {} {}", prefix, objective.name, difficulty_stars), 16.0, color));
            section.spawn(UIBuilder::text(&format!("  {}", objective.description), 14.0, Color::srgb(0.8, 0.8, 0.8)));
        }
    });
}

fn create_intelligence_section(parent: &mut ChildSpawnerCommands, resistance: &ResistanceProfile, environment: &EnvironmentData) {
    let (panel_node, _) = UIBuilder::section_panel();
    parent.spawn((panel_node, BackgroundColor(Color::srgba(0.1, 0.1, 0.2, 0.3)))).with_children(|section| {
        section.spawn(UIBuilder::text("INTELLIGENCE ASSESSMENT", 18.0, Color::srgb(0.2, 0.5, 0.8)));
        
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
        section.spawn(UIBuilder::text(&format!("Opposition: {}", enemy_types), 14.0, Color::srgb(0.8, 0.6, 0.6)));
        
        section.spawn(UIBuilder::text(&format!("Terrain: {:?} | Cover: {:.0}% | Civilians: {}", 
                environment.terrain, 
                environment.cover_density * 100.0, 
                match environment.civilian_presence {
                    0 => "None", 1..=2 => "Light", 3..=4 => "Moderate", _ => "Heavy"
                }), 14.0, Color::srgb(0.6, 0.8, 0.6)));
    });
}

fn create_risk_assessment_section(parent: &mut ChildSpawnerCommands, risks: &RiskAssessment, global_data: &GlobalData) {
    let (panel_node, _) = UIBuilder::section_panel();
    parent.spawn((panel_node, BackgroundColor(Color::srgba(0.2, 0.2, 0.1, 0.3)))).with_children(|section| {
        section.spawn(UIBuilder::text("RISK ASSESSMENT", 18.0, Color::srgb(0.8, 0.8, 0.2)));
        
        section.spawn(UIBuilder::row(25.0)).with_children(|risk_row| {
            risk_row.spawn(UIBuilder::text(&format!("Casualty: {}", risks.casualty_risk.text()), 14.0, risks.casualty_risk.color()));
            risk_row.spawn(UIBuilder::text(&format!("Detection: {}", risks.detection_risk.text()), 14.0, risks.detection_risk.color()));
            risk_row.spawn(UIBuilder::text(&format!("Equipment Loss: {}", risks.equipment_loss_risk.text()), 14.0, risks.equipment_loss_risk.color()));
        });
        
        let failure_color = if risks.mission_failure_chance > 0.5 { Color::srgb(0.8, 0.2, 0.2) } else { Color::WHITE };
        section.spawn(UIBuilder::text(&format!("Failure Probability: {:.0}%", risks.mission_failure_chance * 100.0), 14.0, failure_color));
        
        let avg_agent_level = global_data.agent_levels.iter().sum::<u8>() as f32 / 3.0;
        let readiness_color = if avg_agent_level >= risks.recommended_agent_level as f32 {
            Color::srgb(0.2, 0.8, 0.2)
        } else {
            Color::srgb(0.8, 0.5, 0.2)
        };
        
        section.spawn(UIBuilder::text(&format!("Recommended Level: {} (Squad: {:.1})", 
                risks.recommended_agent_level, avg_agent_level), 14.0, readiness_color));
    });
}

fn create_deployment_section(parent: &mut ChildSpawnerCommands, global_data: &GlobalData) {
    let (panel_node, _) = UIBuilder::section_panel();
    parent.spawn((panel_node, BackgroundColor(Color::srgba(0.2, 0.1, 0.2, 0.3)))).with_children(|section| {
        section.spawn(UIBuilder::text("SQUAD DEPLOYMENT STATUS", 18.0, Color::srgb(0.8, 0.2, 0.8)));
        
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
                            loadout.tools.len()), 14.0, Color::WHITE));
                }
            }
            
            section.spawn(UIBuilder::text("Press ENTER to launch mission", 16.0, Color::srgb(0.8, 0.8, 0.2)));
        } else {
            section.spawn(UIBuilder::error_text("No agents available - all recovering"));
            section.spawn(UIBuilder::small("Use 'W' on Global Map to advance time"));
        }
    });
}

fn create_rewards_section(parent: &mut ChildSpawnerCommands, rewards: &MissionRewards, city: &City, city_state: &CityState) {
    let (panel_node, _) = UIBuilder::section_panel();
    parent.spawn((panel_node, BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.3)))).with_children(|section| {
        section.spawn(UIBuilder::text("MISSION REWARDS", 18.0, Color::srgb(0.8, 0.8, 0.2)));
        
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
                rewards.experience_modifier, rewards.intel_value), 14.0, Color::WHITE));
        
        section.spawn(UIBuilder::text(&format!("Corporation: {:?} | Population: {}M", 
                city.controlling_corp, city.population), 12.0, Color::srgb(0.7, 0.7, 0.7)));
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