// src/systems/ui/hub/missions.rs - egui version
use bevy::prelude::*;
use bevy_egui::egui;
use crate::core::*;

pub fn show_missions(
    ui: &mut bevy_egui::egui::Ui,
    global_data: &GlobalData,
    cities_db: &CitiesDatabase,
    commands: &mut Commands,
    next_state: &mut NextState<GameState>,
    input: &ButtonInput<KeyCode>,
) {
    ui.heading("MISSION BRIEFING");
    
    ui.separator();
    
    if let Some(city) = cities_db.get_city(&global_data.cities_progress.current_city) {
        let city_state = global_data.cities_progress.get_city_state(&global_data.cities_progress.current_city);
        let briefing = generate_mission_briefing_for_city(global_data, cities_db, &global_data.cities_progress, &global_data.cities_progress.current_city);
        
        // Mission header
        ui.horizontal(|ui| {
            ui.colored_label(egui::Color32::RED, format!("{}", city.name));
            
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let threat_level = match city.corruption_level {
                    1..=3 => ("LOW", egui::Color32::GREEN),
                    4..=6 => ("MODERATE", egui::Color32::YELLOW), 
                    7..=8 => ("HIGH", egui::Color32::from_rgb(255, 165, 0)),
                    9..=10 => ("EXTREME", egui::Color32::RED),
                    _ => ("UNKNOWN", egui::Color32::GRAY)
                };
                ui.colored_label(threat_level.1, format!("THREAT: {}", threat_level.0));
            });
        });
        
        // Status info
        ui.horizontal(|ui| {
            let alert_color = match city_state.alert_level {
                AlertLevel::Green => egui::Color32::GREEN,
                AlertLevel::Yellow => egui::Color32::YELLOW,
                AlertLevel::Orange => egui::Color32::from_rgb(255, 165, 0),
                AlertLevel::Red => egui::Color32::RED,
            };
            ui.colored_label(alert_color, format!("Alert: {:?}", city_state.alert_level));
            ui.separator();
            ui.label(format!("Time: {:?} | Visibility: {:.0}%", 
                    briefing.environment.time_of_day, 
                    briefing.environment.visibility * 100.0));
        });
        
        ui.separator();
        
        // Scrollable content area
        egui::ScrollArea::vertical().show(ui, |ui| {
            // Objectives section
            ui.group(|ui| {
                ui.colored_label(egui::Color32::RED, "MISSION OBJECTIVES");
                ui.separator();
                
                for objective in &briefing.objectives {
                    let prefix_color = if objective.required { egui::Color32::RED } else { egui::Color32::BLUE };
                    let prefix = if objective.required { "[REQUIRED]" } else { "[OPTIONAL]" };
                    let difficulty_stars = "★".repeat(objective.difficulty as usize);
                    
                    ui.horizontal(|ui| {
                        ui.colored_label(prefix_color, prefix);
                        ui.label(&objective.name);
                        ui.colored_label(egui::Color32::YELLOW, difficulty_stars);
                    });
                    ui.indent("obj_desc", |ui| {
                        ui.weak(&objective.description);
                    });
                }
            });
            
            ui.separator();
            
            // Intelligence section
            ui.group(|ui| {
                ui.colored_label(egui::Color32::BLUE, "INTELLIGENCE ASSESSMENT");
                ui.separator();
                
                ui.horizontal(|ui| {
                    ui.label(format!("Enemy Forces: {}", briefing.resistance.enemy_count));
                    ui.separator();
                    ui.label(format!("Security Level: {}/5", briefing.resistance.security_level));
                    ui.separator();
                    ui.label(format!("Alert Sensitivity: {:.0}%", briefing.resistance.alert_sensitivity * 100.0));
                });
                
                let enemy_types = briefing.resistance.enemy_types.iter()
                    .map(|e| format!("{:?}", e))
                    .collect::<Vec<_>>()
                    .join(", ");
                ui.colored_label(egui::Color32::from_rgb(200, 150, 150), format!("Opposition: {}", enemy_types));
                
                ui.label(format!("Terrain: {:?} | Cover: {:.0}% | Civilians: {}", 
                        briefing.environment.terrain, 
                        briefing.environment.cover_density * 100.0, 
                        match briefing.environment.civilian_presence {
                            0 => "None", 1..=2 => "Light", 3..=4 => "Moderate", _ => "Heavy"
                        }));
            });
            
            ui.separator();
            
            // Risk assessment
            ui.group(|ui| {
                ui.colored_label(egui::Color32::YELLOW, "RISK ASSESSMENT");
                ui.separator();
                
                ui.horizontal(|ui| {
                    ui.colored_label(briefing.risks.casualty_risk.color(), format!("Casualty: {}", briefing.risks.casualty_risk.text()));
                    ui.separator();
                    ui.colored_label(briefing.risks.detection_risk.color(), format!("Detection: {}", briefing.risks.detection_risk.text()));
                    ui.separator();
                    ui.colored_label(briefing.risks.equipment_loss_risk.color(), format!("Equipment Loss: {}", briefing.risks.equipment_loss_risk.text()));
                });
                
                let failure_color = if briefing.risks.mission_failure_chance > 0.5 { egui::Color32::RED } else { egui::Color32::WHITE };
                ui.colored_label(failure_color, format!("Failure Probability: {:.0}%", briefing.risks.mission_failure_chance * 100.0));
                
                let avg_agent_level = global_data.agent_levels.iter().sum::<u8>() as f32 / 3.0;
                let readiness_color = if avg_agent_level >= briefing.risks.recommended_agent_level as f32 {
                    egui::Color32::GREEN
                } else {
                    egui::Color32::from_rgb(200, 125, 50)
                };
                
                ui.colored_label(readiness_color, format!("Recommended Level: {} (Squad: {:.1})", 
                        briefing.risks.recommended_agent_level, avg_agent_level));
            });
            
            ui.separator();
            
            // Deployment status
            ui.group(|ui| {
                ui.colored_label(egui::Color32::from_rgb(200, 100, 200), "SQUAD DEPLOYMENT STATUS");
                ui.separator();
                
                let ready_agents = (0..3).filter(|&i| global_data.agent_recovery[i] <= global_data.current_day).count();
                
                if ready_agents > 0 {
                    ui.colored_label(egui::Color32::GREEN, format!("Deployment Ready: {} agents available", ready_agents));
                    
                    for i in 0..3 {
                        if global_data.agent_recovery[i] <= global_data.current_day {
                            let loadout = global_data.get_agent_loadout(i);
                            let weapon_name = if let Some(config) = loadout.weapon_configs.get(loadout.equipped_weapon_idx) {
                                format!("{:?}", config.base_weapon)
                            } else {
                                "No Weapon".to_string()
                            };
                            
                            ui.label(format!("Agent {}: Lv{} | {} | {} tools", 
                                    i + 1, 
                                    global_data.agent_levels[i],
                                    weapon_name,
                                    loadout.tools.len()));
                        }
                    }
                    
                    ui.separator();
                    
                    // Launch button
                    if ui.button("🚀 LAUNCH MISSION (ENTER)").clicked() || input.just_pressed(KeyCode::Enter) {
                        commands.insert_resource(MissionLaunchData {
                            city_id: global_data.cities_progress.current_city.clone(),
                            region_id: global_data.selected_region,
                        });

                        commands.insert_resource(ShouldRestart);
                        next_state.set(GameState::Mission);
                    }
                } else {
                    ui.colored_label(egui::Color32::RED, "No agents available - all recovering");
                    ui.weak("Use 'W' on Global Map to advance time");
                }
            });
            
            ui.separator();
            
            // Rewards section
            ui.group(|ui| {
                ui.colored_label(egui::Color32::YELLOW, "MISSION REWARDS");
                ui.separator();
                
                let difficulty_bonus = match city.corruption_level {
                    1..=3 => 1.0, 4..=6 => 1.2, 7..=8 => 1.5, 9..=10 => 2.0, _ => 1.0
                };
                let total_credits = (briefing.rewards.base_credits as f32 * difficulty_bonus) as u32;
                
                ui.horizontal(|ui| {
                    ui.colored_label(egui::Color32::YELLOW, format!("Base Credits: {}", total_credits));
                    ui.separator();
                    ui.colored_label(egui::Color32::GREEN, format!("Bonus Potential: +{}", briefing.rewards.bonus_credits));
                    ui.separator();
                    ui.colored_label(egui::Color32::BLUE, format!("Equipment Drop: {:.0}%", briefing.rewards.equipment_chance * 100.0));
                });
                
                ui.label(format!("XP Modifier: {:.1}x | Intel: {}/5", 
                        briefing.rewards.experience_modifier, briefing.rewards.intel_value));
                
                ui.weak(format!("Corporation: {:?} | Population: {}M", 
                        city.controlling_corp, city.population));
            });
        });
    } else {
        ui.group(|ui| {
            ui.colored_label(egui::Color32::GRAY, "No city selected");
            ui.label("Select a city from the Global Map to view mission details.");
        });
    }
}

// Helper trait for risk level colors
trait RiskLevelExt {
    fn color(&self) -> egui::Color32;
    fn text(&self) -> &str;
}

impl RiskLevelExt for RiskLevel {
    fn color(&self) -> egui::Color32 {
        match self {
            RiskLevel::Low => egui::Color32::GREEN,
            RiskLevel::Medium => egui::Color32::YELLOW,
            RiskLevel::High => egui::Color32::from_rgb(255, 165, 0),
            RiskLevel::Critical => egui::Color32::RED,
            _ => egui::Color32::WHITE,
        }
    }
    
    fn text(&self) -> &str {
        match self {
            RiskLevel::Low => "LOW",
            RiskLevel::Medium => "MEDIUM",
            RiskLevel::High => "HIGH", 
            RiskLevel::Critical => "CRITICAL",
            _ => "UNKNOWN",
        }
    }
}