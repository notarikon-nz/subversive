// src/systems/ui/hub/agents.rs - Updated with cybernetics and performance tracking
use bevy::prelude::*;
use crate::core::*;
use super::HubTab;

// Agent management state - moved here to avoid conflicts
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

pub fn handle_input(
    input: &ButtonInput<KeyCode>, 
    hub_state: &mut super::HubState,
    agent_state: &mut AgentManagementState,
    global_data: &mut ResMut<GlobalData>,
    cybernetics_db: &[CyberneticUpgrade],
) -> bool {
    let mut needs_rebuild = false;
    
    // Tab switching
    if input.just_pressed(KeyCode::KeyM) {
        hub_state.active_tab = HubTab::Manufacture;
        return true;
    }
    
    // View mode switching with 1-3 keys
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
    
    // Agent selection with LEFT/RIGHT
    if input.just_pressed(KeyCode::ArrowLeft) && agent_state.selected_agent_idx > 0 {
        agent_state.selected_agent_idx -= 1;
        agent_state.selected_cybernetic_idx = 0; // Reset cybernetic selection
        needs_rebuild = true;
    }
    if input.just_pressed(KeyCode::ArrowRight) && agent_state.selected_agent_idx < 2 {
        agent_state.selected_agent_idx += 1;
        agent_state.selected_cybernetic_idx = 0; // Reset cybernetic selection
        needs_rebuild = true;
    }
    
    // Cybernetic navigation with UP/DOWN (only in cybernetics view)
    if agent_state.view_mode == AgentViewMode::Cybernetics {
        // Available cybernetics
        let available_cybernetics = get_available_cybernetics(
            global_data, 
            agent_state.selected_agent_idx, 
            cybernetics_db
        );
        
        if input.just_pressed(KeyCode::ArrowUp) && agent_state.selected_cybernetic_idx > 0 {
            agent_state.selected_cybernetic_idx -= 1;
            needs_rebuild = true;
        }
        if input.just_pressed(KeyCode::ArrowDown) && agent_state.selected_cybernetic_idx < available_cybernetics.len().saturating_sub(1) {
            agent_state.selected_cybernetic_idx += 1;
            needs_rebuild = true;
        }
        
        // Install cybernetic with ENTER
        if input.just_pressed(KeyCode::Enter) && !available_cybernetics.is_empty() {
            if let Some(cybernetic) = available_cybernetics.get(agent_state.selected_cybernetic_idx) {
                if install_cybernetic(global_data, agent_state.selected_agent_idx, cybernetic) {
                    needs_rebuild = true;
                    info!("Installed {} on Agent {}", cybernetic.name, agent_state.selected_agent_idx + 1);
                }
            }
        }
    }
    
    needs_rebuild
}

pub fn create_content(
    parent: &mut ChildSpawnerCommands, 
    global_data: &GlobalData,
    agent_state: &AgentManagementState,  // <-- This should NOT be ResMut
    cybernetics_db: &[CyberneticUpgrade],
) {
    parent.spawn(Node {
        width: Val::Percent(100.0),
        flex_grow: 1.0,
        padding: UiRect::all(Val::Px(20.0)),
        flex_direction: FlexDirection::Column,
        row_gap: Val::Px(15.0),
        ..default()
    }).with_children(|content| {
        // Header with view mode tabs
        content.spawn((
            Text::new("AGENT MANAGEMENT"),
            TextFont { font_size: 24.0, ..default() },
            TextColor(Color::srgb(0.8, 0.8, 0.2)),
        ));
        
        // View mode selector
        content.spawn(Node {
            flex_direction: FlexDirection::Row,
            column_gap: Val::Px(20.0),
            margin: UiRect::bottom(Val::Px(10.0)),
            ..default()
        }).with_children(|tabs| {
            let tab_configs = [
                (AgentViewMode::Overview, "1: Overview"),
                (AgentViewMode::Cybernetics, "2: Cybernetics"),
                (AgentViewMode::Performance, "3: Performance"),
            ];
            
            for (mode, label) in tab_configs {
                let is_active = agent_state.view_mode == mode;
                let color = if is_active {
                    Color::srgb(0.8, 0.8, 0.2)
                } else {
                    Color::srgb(0.6, 0.6, 0.6)
                };
                
                tabs.spawn((
                    Text::new(label),
                    TextFont { font_size: 16.0, ..default() },
                    TextColor(color),
                ));
            }
        });
        
        // Agent selector
        content.spawn(Node {
            flex_direction: FlexDirection::Row,
            column_gap: Val::Px(20.0),
            margin: UiRect::bottom(Val::Px(15.0)),
            ..default()
        }).with_children(|agent_tabs| {
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
                
                let prefix = if is_selected { "> " } else { "  " };
                let status = if is_recovering { " (RECOVERING)" } else { "" };
                
                agent_tabs.spawn((
                    Text::new(format!("{}Agent {} (Lv{}){}", 
                            prefix, i + 1, global_data.agent_levels[i], status)),
                    TextFont { font_size: 16.0, ..default() },
                    TextColor(color),
                ));
            }
        });
        
        // Content area based on view mode
        match agent_state.view_mode {
            AgentViewMode::Overview => create_overview_content(content, global_data, agent_state.selected_agent_idx),
            AgentViewMode::Cybernetics => create_cybernetics_content(content, global_data, agent_state, cybernetics_db),
            AgentViewMode::Performance => create_performance_content(content, global_data, agent_state.selected_agent_idx),
        }
        
        // Controls help
        content.spawn((
            Text::new("\n←→: Select Agent | 1-3: View Mode | ↑↓: Navigate (Cybernetics) | ENTER: Install | M: Manufacture"),
            TextFont { font_size: 12.0, ..default() },
            TextColor(Color::srgb(0.7, 0.7, 0.7)),
        ));
    });
}

fn create_overview_content(
    parent: &mut ChildSpawnerCommands,
    global_data: &GlobalData,
    agent_idx: usize,
) {
    parent.spawn((
        Node {
            flex_direction: FlexDirection::Column,
            padding: UiRect::all(Val::Px(15.0)),
            row_gap: Val::Px(8.0),
            ..default()
        },
        BackgroundColor(Color::srgba(0.1, 0.1, 0.2, 0.3)),
    )).with_children(|overview| {
        let level = global_data.agent_levels[agent_idx];
        let exp = global_data.agent_experience[agent_idx];
        let next_level_exp = experience_for_level(level + 1);
        let loadout = global_data.get_agent_loadout(agent_idx);
        
        overview.spawn((
            Text::new(format!("AGENT {} PROFILE", agent_idx + 1)),
            TextFont { font_size: 18.0, ..default() },
            TextColor(Color::WHITE),
        ));
        
        // Basic stats
        overview.spawn((
            Text::new(format!("Level: {} | Experience: {}/{}", level, exp, next_level_exp)),
            TextFont { font_size: 14.0, ..default() },
            TextColor(Color::WHITE),
        ));
        
        // Recovery status
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
        
        overview.spawn((
            Text::new(recovery_status),
            TextFont { font_size: 14.0, ..default() },
            TextColor(status_color),
        ));
        
        // Equipment summary
        let weapon_name = if let Some(config) = loadout.weapon_configs.get(loadout.equipped_weapon_idx) {
            format!("{:?}", config.base_weapon)
        } else {
            "None".to_string()
        };
        
        overview.spawn((
            Text::new(format!("Primary Weapon: {}", weapon_name)),
            TextFont { font_size: 14.0, ..default() },
            TextColor(Color::WHITE),
        ));
        
        overview.spawn((
            Text::new(format!("Tools: {} equipped | Cybernetics: {} installed", 
                    loadout.tools.len(), loadout.cybernetics.len())),
            TextFont { font_size: 14.0, ..default() },
            TextColor(Color::WHITE),
        ));
        
        // Traits display (if we have agent upgrades data)
        overview.spawn((
            Text::new("\nTraits: [Not yet implemented - will show agent traits here]"),
            TextFont { font_size: 12.0, ..default() },
            TextColor(Color::srgb(0.6, 0.6, 0.6)),
        ));
    });
}

fn create_cybernetics_content(
    parent: &mut ChildSpawnerCommands,
    global_data: &GlobalData,
    agent_state: &AgentManagementState,
    cybernetics_db: &[CyberneticUpgrade],
) {
    parent.spawn((
        Node {
            flex_direction: FlexDirection::Column,
            padding: UiRect::all(Val::Px(15.0)),
            row_gap: Val::Px(8.0),
            ..default()
        },
        BackgroundColor(Color::srgba(0.2, 0.1, 0.2, 0.3)),
    )).with_children(|cyber_panel| {
        cyber_panel.spawn((
            Text::new(format!("CYBERNETIC UPGRADES - AGENT {}", agent_state.selected_agent_idx + 1)),
            TextFont { font_size: 18.0, ..default() },
            TextColor(Color::srgb(0.8, 0.2, 0.8)),
        ));
        
        // Show currently installed cybernetics
        let loadout = global_data.get_agent_loadout(agent_state.selected_agent_idx);
        if !loadout.cybernetics.is_empty() {
            cyber_panel.spawn((
                Text::new("INSTALLED:"),
                TextFont { font_size: 16.0, ..default() },
                TextColor(Color::WHITE),
            ));
            
            for cybernetic in &loadout.cybernetics {
                cyber_panel.spawn((
                    Text::new(format!("• {:?}", cybernetic)),
                    TextFont { font_size: 14.0, ..default() },
                    TextColor(Color::srgb(0.6, 0.8, 0.6)),
                ));
            }
        } else {
            cyber_panel.spawn((
                Text::new("No cybernetics installed"),
                TextFont { font_size: 14.0, ..default() },
                TextColor(Color::srgb(0.6, 0.6, 0.6)),
            ));
        }
        
        // Show available cybernetics
        let available_cybernetics = get_available_cybernetics(
            global_data, 
            agent_state.selected_agent_idx, 
            cybernetics_db
        );
        
        if !available_cybernetics.is_empty() {
            cyber_panel.spawn((
                Text::new("\nAVAILABLE UPGRADES:"),
                TextFont { font_size: 16.0, ..default() },
                TextColor(Color::WHITE),
            ));
            
            for (i, cybernetic) in available_cybernetics.iter().enumerate() {
                let is_selected = i == agent_state.selected_cybernetic_idx;
                let can_afford = global_data.credits >= cybernetic.cost;
                
                let color = if is_selected {
                    if can_afford {
                        Color::srgb(0.8, 0.8, 0.2)
                    } else {
                        Color::srgb(0.8, 0.3, 0.3)
                    }
                } else if can_afford {
                    Color::WHITE
                } else {
                    Color::srgb(0.5, 0.5, 0.5)
                };
                
                let prefix = if is_selected { "> " } else { "  " };
                let category_color = match cybernetic.category {
                    CyberneticCategory::Combat => "C",
                    CyberneticCategory::Stealth => "S", 
                    CyberneticCategory::Utility => "U",
                    CyberneticCategory::Survival => "H",
                };
                
                cyber_panel.spawn((
                    Text::new(format!("{}[{}] {} - {} credits", 
                            prefix, category_color, cybernetic.name, cybernetic.cost)),
                    TextFont { font_size: 14.0, ..default() },
                    TextColor(color),
                ));
                
                if is_selected {
                    cyber_panel.spawn((
                        Text::new(format!("    {}", cybernetic.description)),
                        TextFont { font_size: 12.0, ..default() },
                        TextColor(Color::srgb(0.8, 0.8, 0.8)),
                    ));
                    
                    // Show effects
                    let effects_text = cybernetic.effects.iter()
                        .map(|effect| format_cybernetic_effect(effect))
                        .collect::<Vec<_>>()
                        .join(", ");
                    
                    cyber_panel.spawn((
                        Text::new(format!("    Effects: {}", effects_text)),
                        TextFont { font_size: 12.0, ..default() },
                        TextColor(Color::srgb(0.6, 0.8, 0.6)),
                    ));
                    
                    if !cybernetic.prerequisites.is_empty() {
                        cyber_panel.spawn((
                            Text::new(format!("    Requires: {}", cybernetic.prerequisites.join(", "))),
                            TextFont { font_size: 12.0, ..default() },
                            TextColor(Color::srgb(0.8, 0.6, 0.2)),
                        ));
                    }
                }
            }
        } else {
            cyber_panel.spawn((
                Text::new("\nNo cybernetics available.\nComplete research or check prerequisites."),
                TextFont { font_size: 14.0, ..default() },
                TextColor(Color::srgb(0.6, 0.6, 0.6)),
            ));
        }
        
        cyber_panel.spawn((
            Text::new(format!("\nCredits Available: {}", global_data.credits)),
            TextFont { font_size: 14.0, ..default() },
            TextColor(Color::srgb(0.8, 0.8, 0.2)),
        ));
    });
}

fn create_performance_content(
    parent: &mut ChildSpawnerCommands,
    global_data: &GlobalData,
    agent_idx: usize,
) {
    parent.spawn((
        Node {
            flex_direction: FlexDirection::Column,
            padding: UiRect::all(Val::Px(15.0)),
            row_gap: Val::Px(8.0),
            ..default()
        },
        BackgroundColor(Color::srgba(0.1, 0.2, 0.1, 0.3)),
    )).with_children(|perf_panel| {
        perf_panel.spawn((
            Text::new(format!("PERFORMANCE STATS - AGENT {}", agent_idx + 1)),
            TextFont { font_size: 18.0, ..default() },
            TextColor(Color::srgb(0.2, 0.8, 0.2)),
        ));
        
        // For now, show basic stats we have access to
        let level = global_data.agent_levels[agent_idx];
        let exp = global_data.agent_experience[agent_idx];
        
        perf_panel.spawn((
            Text::new("MISSION HISTORY:"),
            TextFont { font_size: 16.0, ..default() },
            TextColor(Color::WHITE),
        ));
        
        perf_panel.spawn((
            Text::new(format!("Current Level: {}", level)),
            TextFont { font_size: 14.0, ..default() },
            TextColor(Color::WHITE),
        ));
        
        perf_panel.spawn((
            Text::new(format!("Total Experience: {}", exp)),
            TextFont { font_size: 14.0, ..default() },
            TextColor(Color::WHITE),
        ));
        
        // Calculate estimated missions from experience
        let estimated_missions = exp / 15; // Rough estimate based on 10-20 XP per mission
        perf_panel.spawn((
            Text::new(format!("Estimated Missions: {}", estimated_missions)),
            TextFont { font_size: 14.0, ..default() },
            TextColor(Color::WHITE),
        ));
        
        // Recovery history
        let recovery_status = if global_data.agent_recovery[agent_idx] > global_data.current_day {
            "Currently recovering from injuries"
        } else {
            "Fully operational"
        };
        
        perf_panel.spawn((
            Text::new(format!("Status: {}", recovery_status)),
            TextFont { font_size: 14.0, ..default() },
            TextColor(Color::WHITE),
        ));
        
        // Placeholder for future detailed tracking
        perf_panel.spawn((
            Text::new("\nDETAILED STATISTICS:"),
            TextFont { font_size: 16.0, ..default() },
            TextColor(Color::WHITE),
        ));
        
        perf_panel.spawn((
            Text::new("Missions Completed: [Tracking not yet implemented]"),
            TextFont { font_size: 14.0, ..default() },
            TextColor(Color::srgb(0.6, 0.6, 0.6)),
        ));
        
        perf_panel.spawn((
            Text::new("Enemies Eliminated: [Tracking not yet implemented]"),
            TextFont { font_size: 14.0, ..default() },
            TextColor(Color::srgb(0.6, 0.6, 0.6)),
        ));
        
        perf_panel.spawn((
            Text::new("Stealth Missions: [Tracking not yet implemented]"),
            TextFont { font_size: 14.0, ..default() },
            TextColor(Color::srgb(0.6, 0.6, 0.6)),
        ));
        
        perf_panel.spawn((
            Text::new("Survival Streak: [Tracking not yet implemented]"),
            TextFont { font_size: 14.0, ..default() },
            TextColor(Color::srgb(0.6, 0.6, 0.6)),
        ));
        
        perf_panel.spawn((
            Text::new("Veteran Bonuses: [Not yet unlocked]"),
            TextFont { font_size: 14.0, ..default() },
            TextColor(Color::srgb(0.6, 0.6, 0.6)),
        ));
    });
}

// Helper functions
fn get_available_cybernetics<'a>(
    global_data: &GlobalData,
    agent_idx: usize,
    cybernetics_db: &'a [CyberneticUpgrade],
) -> Vec<&'a CyberneticUpgrade> {
    let loadout = global_data.get_agent_loadout(agent_idx);
    let installed_ids: std::collections::HashSet<String> = loadout.cybernetics.iter()
        .map(|c| format!("{:?}", c)) // Simple comparison for now
        .collect();
    
    cybernetics_db.iter()
        .filter(|cyber| {
            // Not already installed
            !installed_ids.contains(&cyber.id) &&
            // Prerequisites met (simplified check)
            cyber.prerequisites.iter().all(|req| {
                // For now, just check if any cybernetic type matches
                loadout.cybernetics.iter().any(|installed| {
                    format!("{:?}", installed).to_lowercase().contains(&req.to_lowercase())
                })
            })
        })
        .collect()
}

fn install_cybernetic(
    global_data: &mut ResMut<GlobalData>,
    agent_idx: usize,
    cybernetic: &CyberneticUpgrade,
) -> bool {
    if global_data.credits >= cybernetic.cost {
        global_data.credits -= cybernetic.cost;
        
        // Convert to CyberneticType for storage (simplified)
        let cybernetic_type = match cybernetic.category {
            CyberneticCategory::Combat => CyberneticType::CombatEnhancer,
            CyberneticCategory::Stealth => CyberneticType::StealthModule,
            CyberneticCategory::Utility => CyberneticType::TechInterface,
            CyberneticCategory::Survival => CyberneticType::Neurovector, // Reuse for now
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