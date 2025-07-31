// src/systems/ui/hub/agents.rs - egui version
use bevy::prelude::*;
use bevy_egui::egui;
use crate::core::*;
use crate::systems::ui::hub::CyberneticsDatabase;

#[derive(Resource, Default)]
pub struct AgentManagementState {
    pub selected_agent_idx: usize,
    pub selected_cybernetic_idx: usize,
    pub view_mode: AgentViewMode,
}

#[derive(Default, PartialEq)]
pub enum AgentViewMode {
    #[default] 
    Overview, 
    Cybernetics, 
    Performance,
}

pub fn show_agents(
    ui: &mut egui::Ui,
    global_data: &mut GlobalData,
    cybernetics_db: &CyberneticsDatabase,
    input: &ButtonInput<KeyCode>,
) {
    // We'll use a simple local state since egui is immediate mode
    // In a real implementation, you'd want to store this in a Resource
    ui.heading("AGENT MANAGEMENT");
    
    ui.separator();
    
    // View mode selection
    ui.horizontal(|ui| {
        ui.selectable_value(&mut AgentViewMode::Overview, AgentViewMode::Overview, "1: Overview");
        ui.selectable_value(&mut AgentViewMode::Cybernetics, AgentViewMode::Cybernetics, "2: Cybernetics");
        ui.selectable_value(&mut AgentViewMode::Performance, AgentViewMode::Performance, "3: Performance");
    });
    
    ui.separator();
    
    // Agent selection
    let mut selected_agent = 0; // This should come from state
    ui.horizontal(|ui| {
        for i in 0..3 {
            let is_recovering = global_data.agent_recovery[i] > global_data.current_day;
            let text = format!("Agent {} (Lv{}){}", 
                i + 1, 
                global_data.agent_levels[i],
                if is_recovering { " (RECOVERING)" } else { "" }
            );
            
            let color = if is_recovering {
                egui::Color32::GRAY
            } else {
                egui::Color32::WHITE
            };
            
            if ui.selectable_label(selected_agent == i, egui::RichText::new(text).color(color)).clicked() {
                selected_agent = i;
            }
        }
    });
    
    ui.separator();
    
    // Content area with scrolling
    egui::ScrollArea::vertical().show(ui, |ui| {
        show_agent_overview(ui, global_data, selected_agent);
    });
}

fn show_agent_overview(ui: &mut egui::Ui, global_data: &GlobalData, agent_idx: usize) {
    ui.group(|ui| {
        ui.heading(format!("AGENT {} PROFILE", agent_idx + 1));
        
        let level = global_data.agent_levels[agent_idx];
        let exp = global_data.agent_experience[agent_idx];
        let next_level_exp = experience_for_level(level + 1);
        let loadout = global_data.get_agent_loadout(agent_idx);
        
        // Basic stats
        ui.label(format!("Level: {}", level));
        ui.label(format!("Experience: {}/{}", exp, next_level_exp));
        
        // Experience progress bar
        let progress = exp as f32 / next_level_exp as f32;
        ui.add(egui::ProgressBar::new(progress).text(format!("{:.1}%", progress * 100.0)));
        
        ui.separator();
        
        // Status with color coding
        let recovery_status = if global_data.agent_recovery[agent_idx] > global_data.current_day {
            let days_left = global_data.agent_recovery[agent_idx] - global_data.current_day;
            (format!("Status: RECOVERING ({} days remaining)", days_left), egui::Color32::YELLOW)
        } else {
            ("Status: READY FOR DEPLOYMENT".to_string(), egui::Color32::GREEN)
        };
        
        ui.colored_label(recovery_status.1, recovery_status.0);
        
        ui.separator();
        
        // Equipment summary
        ui.label("EQUIPMENT:");
        
        let weapon_name = if let Some(config) = loadout.weapon_configs.get(loadout.equipped_weapon_idx) {
            format!("{:?}", config.base_weapon)
        } else {
            "None".to_string()
        };
        
        ui.label(format!("• Primary Weapon: {}", weapon_name));
        ui.label(format!("• Tools: {}", loadout.tools.len()));
        ui.label(format!("• Cybernetics: {}", loadout.cybernetics.len()));
        
        // Show cybernetics if any
        if !loadout.cybernetics.is_empty() {
            ui.separator();
            ui.label("INSTALLED CYBERNETICS:");
            for cybernetic in &loadout.cybernetics {
                let cybernetic_name = match cybernetic {
                    CyberneticType::Neurovector => "Neurovector",
                    CyberneticType::NeuralInterface => "Neural Interface",
                    CyberneticType::CombatEnhancer => "Combat Enhancer",
                    CyberneticType::StealthModule => "Stealth Module",
                    CyberneticType::TechInterface => "Hacking Booster",
                    CyberneticType::ArmorPlating => "Armor Plating",
                    CyberneticType::ReflexEnhancer => "Reflex Enhancer",
                };
                ui.colored_label(egui::Color32::LIGHT_BLUE, format!("• {}", cybernetic_name));
            }
        }
    });
}

fn show_agent_cybernetics(
    ui: &mut egui::Ui,
    global_data: &mut GlobalData,
    cybernetics_db: &CyberneticsDatabase,
    agent_idx: usize,
) {
    ui.group(|ui| {
        ui.colored_label(
            egui::Color32::from_rgb(200, 100, 200), 
            format!("CYBERNETIC UPGRADES - AGENT {}", agent_idx + 1)
        );
        
        let loadout = global_data.get_agent_loadout(agent_idx);
        
        // Installed cybernetics
        if !loadout.cybernetics.is_empty() {
            ui.label("INSTALLED:");
            for cybernetic in &loadout.cybernetics {
                ui.colored_label(egui::Color32::GREEN, format!("• {:?}", cybernetic));
            }
            ui.separator();
        }
        
        // Available cybernetics
        let available_cybernetics = get_available_cybernetics(global_data, agent_idx, &cybernetics_db.cybernetics);
        
        if !available_cybernetics.is_empty() {
            ui.label("AVAILABLE UPGRADES:");
            
            egui::ScrollArea::vertical().max_height(300.0).show(ui, |ui| {
                for cybernetic in &available_cybernetics {
                    let can_afford = global_data.credits >= cybernetic.cost;
                    
                    ui.group(|ui| {
                        ui.horizontal(|ui| {
                            let category_tag = match cybernetic.category {
                                CyberneticCategory::Combat => "C",
                                CyberneticCategory::Stealth => "S", 
                                CyberneticCategory::Utility => "U",
                                CyberneticCategory::Survival => "H",
                            };
                            
                            ui.label(format!("[{}]", category_tag));
                            ui.label(&cybernetic.name);
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                let color = if can_afford { egui::Color32::YELLOW } else { egui::Color32::GRAY };
                                ui.colored_label(color, format!("{} credits", cybernetic.cost));
                            });
                        });
                        
                        ui.label(&cybernetic.description);
                        
                        // Effects
                        let effects_text = cybernetic.effects.iter()
                            .map(|effect| format_cybernetic_effect(effect))
                            .collect::<Vec<_>>()
                            .join(", ");
                        
                        ui.colored_label(egui::Color32::GREEN, format!("Effects: {}", effects_text));
                        
                        if ui.add_enabled(can_afford, egui::Button::new("Install")).clicked() {
                            install_cybernetic(global_data, agent_idx, cybernetic);
                        }
                    });
                }
            });
        } else {
            ui.weak("No cybernetics available.\nComplete research or check prerequisites.");
        }
        
        ui.separator();
        ui.colored_label(egui::Color32::YELLOW, format!("Credits: {}", global_data.credits));
    });
}

fn show_agent_performance(ui: &mut egui::Ui, global_data: &GlobalData, agent_idx: usize) {
    ui.group(|ui| {
        ui.colored_label(
            egui::Color32::GREEN, 
            format!("PERFORMANCE STATS - AGENT {}", agent_idx + 1)
        );
        
        ui.separator();
        ui.label("MISSION HISTORY:");
        
        let level = global_data.agent_levels[agent_idx];
        let exp = global_data.agent_experience[agent_idx];
        let next_level_exp = experience_for_level(level + 1);
        
        ui.label(format!("Level: {}", level));
        ui.label(format!("Experience: {}/{}", exp, next_level_exp));
        ui.label(format!("Estimated Missions: {}", exp / 15));
        
        let recovery_status = if global_data.agent_recovery[agent_idx] > global_data.current_day {
            "Currently recovering from injuries"
        } else {
            "Fully operational"
        };
        ui.label(format!("Status: {}", recovery_status));
        
        ui.separator();
        ui.label("DETAILED STATISTICS:");
        ui.weak("Tracking not yet implemented");
        
        // Placeholder for future stats
        ui.group(|ui| {
            ui.label("Mission Success Rate: 85%");
            ui.label("Stealth Missions: 12");
            ui.label("Combat Missions: 8");
            ui.label("Civilians Saved: 45");
            ui.label("Data Stolen: 2.3TB");
        });
    });
}

// Helper functions (same as before but simplified)
fn get_available_cybernetics<'a>(
    global_data: &GlobalData,
    agent_idx: usize,
    cybernetics_db: &'a [CyberneticUpgrade],
) -> Vec<&'a CyberneticUpgrade> {
    let loadout = global_data.get_agent_loadout(agent_idx);
    let installed_ids: std::collections::HashSet<String> = loadout.cybernetics.iter()
        .map(|c| format!("{:?}", c))
        .collect();
    
    cybernetics_db.iter()
        .filter(|cyber| !installed_ids.contains(&cyber.id))
        .collect()
}

fn install_cybernetic(
    global_data: &mut GlobalData,
    agent_idx: usize,
    cybernetic: &CyberneticUpgrade,
) -> bool {
    if global_data.credits >= cybernetic.cost {
        global_data.credits -= cybernetic.cost;
        
        let cybernetic_type = match cybernetic.category {
            CyberneticCategory::Combat => CyberneticType::CombatEnhancer,
            CyberneticCategory::Stealth => CyberneticType::StealthModule,
            CyberneticCategory::Utility => CyberneticType::TechInterface,
            CyberneticCategory::Survival => CyberneticType::Neurovector,
        };
        
        let mut loadout = global_data.get_agent_loadout(agent_idx).clone();
        loadout.cybernetics.push(cybernetic_type);
        global_data.save_agent_loadout(agent_idx, loadout);
        true
    } else {
        false
    }
}

fn format_cybernetic_effect(effect: &CyberneticEffect) -> String {
    match effect {
        CyberneticEffect::DamageBonus(bonus) => format!("+{:.0}% damage", bonus * 100.0),
        CyberneticEffect::AccuracyBonus(bonus) => format!("+{:.0}% accuracy", bonus * 100.0),
        CyberneticEffect::SpeedBonus(bonus) => format!("+{:.0}% speed", bonus * 100.0),
        CyberneticEffect::HealthBonus(bonus) => format!("+{:.0} health", bonus),
        CyberneticEffect::StealthBonus(bonus) => format!("+{:.0}% stealth", bonus * 100.0),
        CyberneticEffect::HackingBonus(bonus) => format!("+{:.0}% hack speed", bonus * 100.0),
        CyberneticEffect::ExperienceBonus(bonus) => format!("+{:.0}% XP", bonus * 100.0),
        CyberneticEffect::RecoveryReduction(days) => format!("-{} day recovery", days),
        CyberneticEffect::NoiseReduction(bonus) => format!("-{:.0}% noise", bonus * 100.0),
        CyberneticEffect::VisionBonus(bonus) => format!("+{:.0}% vision", bonus * 100.0),
    }
}