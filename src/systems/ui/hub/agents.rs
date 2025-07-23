// src/systems/ui/hub/agents.rs - Simplified using UIBuilder
use bevy::prelude::*;
use crate::core::*;
use crate::systems::ui::builder::*;

#[derive(Resource, Default)]
pub struct AgentManagementState {
    pub selected_agent_idx: usize,
    pub selected_cybernetic_idx: usize,
    pub view_mode: AgentViewMode,
}

#[derive(Default, PartialEq)]
pub enum AgentViewMode {
    #[default] Overview, Cybernetics, Performance,
}

pub fn handle_input(
    input: &ButtonInput<KeyCode>, 
    hub_state: &mut super::HubState,
    agent_state: &mut AgentManagementState,
    global_data: &mut ResMut<GlobalData>,
    cybernetics_db: &[CyberneticUpgrade],
) -> bool {
    let mut needs_rebuild = false;
    
    if input.just_pressed(KeyCode::KeyM) {
        hub_state.active_tab = super::HubTab::Manufacture;
        return true;
    }
    
    if input.just_pressed(KeyCode::Digit1) {
        agent_state.view_mode = AgentViewMode::Overview;
        needs_rebuild = true;
    }
    if input.just_pressed(KeyCode::Digit2) {
        agent_state.view_mode = AgentViewMode::Cybernetics;
        needs_rebuild = true;
    }
    if input.just_pressed(KeyCode::Digit3) {
        agent_state.view_mode = AgentViewMode::Performance;
        needs_rebuild = true;
    }
    
    if input.just_pressed(KeyCode::ArrowLeft) && agent_state.selected_agent_idx > 0 {
        agent_state.selected_agent_idx -= 1;
        agent_state.selected_cybernetic_idx = 0;
        needs_rebuild = true;
    }
    if input.just_pressed(KeyCode::ArrowRight) && agent_state.selected_agent_idx < 2 {
        agent_state.selected_agent_idx += 1;
        agent_state.selected_cybernetic_idx = 0;
        needs_rebuild = true;
    }
    
    if agent_state.view_mode == AgentViewMode::Cybernetics {
        let available_cybernetics = get_available_cybernetics(global_data, agent_state.selected_agent_idx, cybernetics_db);
        
        if input.just_pressed(KeyCode::ArrowUp) && agent_state.selected_cybernetic_idx > 0 {
            agent_state.selected_cybernetic_idx -= 1;
            needs_rebuild = true;
        }
        if input.just_pressed(KeyCode::ArrowDown) && agent_state.selected_cybernetic_idx < available_cybernetics.len().saturating_sub(1) {
            agent_state.selected_cybernetic_idx += 1;
            needs_rebuild = true;
        }
        
        if input.just_pressed(KeyCode::Enter) && !available_cybernetics.is_empty() {
            if let Some(cybernetic) = available_cybernetics.get(agent_state.selected_cybernetic_idx) {
                if install_cybernetic(global_data, agent_state.selected_agent_idx, cybernetic) {
                    needs_rebuild = true;
                }
            }
        }
    }
    
    needs_rebuild
}

pub fn create_content(
    parent: &mut ChildSpawnerCommands, 
    global_data: &GlobalData,
    agent_state: &AgentManagementState,
    cybernetics_db: &[CyberneticUpgrade],
) {
    parent.spawn(UIBuilder::content_area()).with_children(|content| {
        content.spawn(UIBuilder::title("AGENT MANAGEMENT"));
        
        // View mode tabs
        content.spawn(UIBuilder::row(20.0)).with_children(|tabs| {
            let modes = [(AgentViewMode::Overview, "1: Overview"),
                        (AgentViewMode::Cybernetics, "2: Cybernetics"),
                        (AgentViewMode::Performance, "3: Performance")];
            
            for (mode, label) in modes {
                let color = if agent_state.view_mode == mode {
                    Color::srgb(0.8, 0.8, 0.2)
                } else {
                    Color::srgb(0.6, 0.6, 0.6)
                };
                tabs.spawn(UIBuilder::text(label, 16.0, color));
            }
        });
        
        // Agent selector
        content.spawn(UIBuilder::row(20.0)).with_children(|agent_tabs| {
            for i in 0..3 {
                let is_selected = i == agent_state.selected_agent_idx;
                let is_recovering = global_data.agent_recovery[i] > global_data.current_day;
                
                let color = if is_selected {
                    Color::srgb(0.2, 0.8, 0.2)
                } else if is_recovering {
                    Color::srgb(0.5, 0.5, 0.5)
                } else {
                    Color::WHITE
                };
                
                let text = UIBuilder::selection_item(
                    is_selected,
                    "",
                    &format!("Agent {} (Lv{}){}", 
                            i + 1, 
                            global_data.agent_levels[i],
                            if is_recovering { " (RECOVERING)" } else { "" })
                );
                
                agent_tabs.spawn(UIBuilder::text(&text, 16.0, color));
            }
        });
        
        // Content based on view mode
        match agent_state.view_mode {
            AgentViewMode::Overview => create_overview_content(content, global_data, agent_state.selected_agent_idx),
            AgentViewMode::Cybernetics => create_cybernetics_content(content, global_data, agent_state, cybernetics_db),
            AgentViewMode::Performance => create_performance_content(content, global_data, agent_state.selected_agent_idx),
        }
        
        content.spawn(UIBuilder::nav_controls("←→: Select Agent | 1-3: View Mode | ↑↓: Navigate | ENTER: Install"));
    });
}

fn create_overview_content(parent: &mut ChildSpawnerCommands, global_data: &GlobalData, agent_idx: usize) {
    let (panel_node, panel_bg) = UIBuilder::section_panel();
    parent.spawn((panel_node, panel_bg)).with_children(|overview| {
        let level = global_data.agent_levels[agent_idx];
        let exp = global_data.agent_experience[agent_idx];
        let next_level_exp = experience_for_level(level + 1);
        let loadout = global_data.get_agent_loadout(agent_idx);
        
        overview.spawn(UIBuilder::subtitle(&format!("AGENT {} PROFILE", agent_idx + 1)));
        
        // Direct UIBuilder calls instead of StatsBuilder
        overview.spawn(UIBuilder::text(&format!("Level: {}", level), 14.0, Color::WHITE));
        overview.spawn(UIBuilder::text(&format!("Experience: {}/{}", exp, next_level_exp), 14.0, Color::WHITE));
        
        let recovery_status = if global_data.agent_recovery[agent_idx] > global_data.current_day {
            let days_left = global_data.agent_recovery[agent_idx] - global_data.current_day;
            format!("Status: RECOVERING ({} days remaining)", days_left)
        } else {
            "Status: READY FOR DEPLOYMENT".to_string()
        };
        
        let status_color = if global_data.agent_recovery[agent_idx] > global_data.current_day {
            Color::srgb(0.8, 0.5, 0.2)
        } else {
            Color::srgb(0.2, 0.8, 0.2)
        };
        
        overview.spawn(UIBuilder::text(&recovery_status, 14.0, status_color));
        
        let weapon_name = if let Some(config) = loadout.weapon_configs.get(loadout.equipped_weapon_idx) {
            format!("{:?}", config.base_weapon)
        } else {
            "None".to_string()
        };
        
        overview.spawn(UIBuilder::text(&format!("Primary Weapon: {}", weapon_name), 14.0, Color::WHITE));
        overview.spawn(UIBuilder::text(&format!("Tools: {}", loadout.tools.len()), 14.0, Color::WHITE));
        overview.spawn(UIBuilder::text(&format!("Cybernetics: {}", loadout.cybernetics.len()), 14.0, Color::WHITE));
    });
}

fn create_cybernetics_content(
    parent: &mut ChildSpawnerCommands,
    global_data: &GlobalData,
    agent_state: &AgentManagementState,
    cybernetics_db: &[CyberneticUpgrade],
) {
    let (panel_node, panel_bg) = UIBuilder::section_panel();
    parent.spawn((panel_node, BackgroundColor(Color::srgba(0.2, 0.1, 0.2, 0.3)))).with_children(|cyber_panel| {
        cyber_panel.spawn(UIBuilder::text(&format!("CYBERNETIC UPGRADES - AGENT {}", agent_state.selected_agent_idx + 1), 18.0, Color::srgb(0.8, 0.2, 0.8)));
        
        let loadout = global_data.get_agent_loadout(agent_state.selected_agent_idx);
        if !loadout.cybernetics.is_empty() {
            cyber_panel.spawn(UIBuilder::subtitle("INSTALLED:"));
            for cybernetic in &loadout.cybernetics {
                cyber_panel.spawn(UIBuilder::text(&format!("• {:?}", cybernetic), 14.0, Color::srgb(0.6, 0.8, 0.6)));
            }
        } else {
            cyber_panel.spawn(UIBuilder::text("No cybernetics installed", 14.0, Color::srgb(0.6, 0.6, 0.6)));
        }
        
        let available_cybernetics = get_available_cybernetics(global_data, agent_state.selected_agent_idx, cybernetics_db);
        
        if !available_cybernetics.is_empty() {
            cyber_panel.spawn(UIBuilder::subtitle("AVAILABLE UPGRADES:"));
            
            for (i, cybernetic) in available_cybernetics.iter().enumerate() {
                let is_selected = i == agent_state.selected_cybernetic_idx;
                let can_afford = global_data.credits >= cybernetic.cost;
                
                let color = if is_selected {
                    if can_afford { Color::srgb(0.8, 0.8, 0.2) } else { Color::srgb(0.8, 0.3, 0.3) }
                } else if can_afford {
                    Color::WHITE
                } else {
                    Color::srgb(0.5, 0.5, 0.5)
                };
                
                let category_tag = match cybernetic.category {
                    CyberneticCategory::Combat => "C",
                    CyberneticCategory::Stealth => "S", 
                    CyberneticCategory::Utility => "U",
                    CyberneticCategory::Survival => "H",
                };
                
                let text = UIBuilder::selection_item(
                    is_selected, 
                    &format!("[{}] ", category_tag), 
                    &format!("{} - {} credits", cybernetic.name, cybernetic.cost)
                );
                
                cyber_panel.spawn(UIBuilder::text(&text, 14.0, color));
                
                if is_selected {
                    cyber_panel.spawn(UIBuilder::text(&format!("    {}", cybernetic.description), 12.0, Color::srgb(0.8, 0.8, 0.8)));
                    
                    let effects_text = cybernetic.effects.iter()
                        .map(|effect| format_cybernetic_effect(effect))
                        .collect::<Vec<_>>()
                        .join(", ");
                    
                    cyber_panel.spawn(UIBuilder::text(&format!("    Effects: {}", effects_text), 12.0, Color::srgb(0.6, 0.8, 0.6)));
                }
            }
        } else {
            cyber_panel.spawn(UIBuilder::text("No cybernetics available.\nComplete research or check prerequisites.", 14.0, Color::srgb(0.6, 0.6, 0.6)));
        }
        
        cyber_panel.spawn(UIBuilder::text(&UIBuilder::credits_display(global_data.credits), 14.0, Color::srgb(0.8, 0.8, 0.2)));
    });
}

fn create_performance_content(parent: &mut ChildSpawnerCommands, global_data: &GlobalData, agent_idx: usize) {
    let (panel_node, panel_bg) = UIBuilder::section_panel();
    parent.spawn((panel_node, BackgroundColor(Color::srgba(0.1, 0.2, 0.1, 0.3)))).with_children(|perf_panel| {
        perf_panel.spawn(UIBuilder::text(&format!("PERFORMANCE STATS - AGENT {}", agent_idx + 1), 18.0, Color::srgb(0.2, 0.8, 0.2)));
        
        perf_panel.spawn(UIBuilder::subtitle("MISSION HISTORY:"));
        
        let mut stats = StatsBuilder::new(perf_panel);
        stats.level(global_data.agent_levels[agent_idx], global_data.agent_experience[agent_idx], experience_for_level(global_data.agent_levels[agent_idx] + 1));
        stats.stat("Estimated Missions", &(global_data.agent_experience[agent_idx] / 15).to_string(), None);
        
        let recovery_status = if global_data.agent_recovery[agent_idx] > global_data.current_day {
            "Currently recovering from injuries"
        } else {
            "Fully operational"
        };
        stats.stat("Status", recovery_status, None);
        
        perf_panel.spawn(UIBuilder::subtitle("DETAILED STATISTICS:"));
        perf_panel.spawn(UIBuilder::text("Tracking not yet implemented", 14.0, Color::srgb(0.6, 0.6, 0.6)));
    });
}

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
    global_data: &mut ResMut<GlobalData>,
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