// src/systems/ui/hub/research.rs - Fixed research UI without collapsing widgets
use bevy::prelude::*;
use bevy_egui::egui;
use crate::core::*;
use std::collections::HashMap;
use crate::systems::input::{MenuInput};

#[derive(Resource, Default)]
pub struct ResearchUIState {
    pub selected_section: ResearchSection,
    pub selected_project: usize,
    pub selected_scientist: usize,
    pub scroll_position: f32,
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub enum ResearchSection {
    #[default]
    Queue,
    Available,
    Scientists,
}

pub fn show_research(
    ui: &mut egui::Ui,
    global_data: &mut GlobalData,
    research_progress: &mut ResearchProgress,
    research_db: &ResearchDatabase,
    unlocked_attachments: &mut UnlockedAttachments,
    scientist_query: &Query<(Entity, &Scientist)>,
    input: &ButtonInput<KeyCode>,
) {
    ui.heading("RESEARCH & DEVELOPMENT");
    
    // === TOP STATS BAR ===
    ui.horizontal(|ui| {
        ui.colored_label(egui::Color32::YELLOW, format!("Credits: ${}", global_data.credits));
        ui.separator();
        
        let active_count = research_progress.active_queue.len();
        let queue_color = if active_count >= 5 { egui::Color32::RED } else { egui::Color32::GREEN };
        ui.colored_label(queue_color, format!("Active Projects: {}/5", active_count));
        ui.separator();
        
        let recruited_scientists = scientist_query.iter().filter(|(_, s)| s.is_recruited).count();
        ui.colored_label(egui::Color32::CYAN, format!("Scientists: {}", recruited_scientists));
        ui.separator();
        
        ui.colored_label(egui::Color32::WHITE, format!("Completed: {}", research_progress.completed.len()));
    });
    
    ui.separator();
    
    // === THREE-COLUMN LAYOUT ===
    ui.columns(3, |columns| {
        // COLUMN 1: ACTIVE RESEARCH QUEUE
        columns[0].heading("ACTIVE RESEARCH");
        show_research_queue_flat(&mut columns[0], research_progress, research_db, scientist_query);
        
        // COLUMN 2: AVAILABLE PROJECTS  
        columns[1].heading("AVAILABLE PROJECTS");
        show_available_projects_flat(&mut columns[1], global_data, research_progress, research_db, scientist_query);
        
        // COLUMN 3: SCIENTISTS
        columns[2].heading("RESEARCH TEAM");
        show_scientists_flat(&mut columns[2], global_data, scientist_query);
    });
    
    // === BOTTOM SECTION: STOLEN DATA (Always Visible) ===
    if !research_progress.stolen_data.is_empty() {
        ui.separator();
        ui.heading("STOLEN RESEARCH DATA");
        show_stolen_data_flat(ui, research_progress, research_db);
    }
}

fn show_research_queue_flat(
    ui: &mut egui::Ui,
    research_progress: &mut ResearchProgress,
    research_db: &ResearchDatabase,
    scientist_query: &Query<(Entity, &Scientist)>,
) {
    if research_progress.active_queue.is_empty() {
        ui.colored_label(egui::Color32::GRAY, "No active research projects");
        return;
    }
    
    egui::ScrollArea::vertical()
        .max_height(400.0)
        .show(ui, |ui| {
            for (i, active) in research_progress.active_queue.iter().enumerate() {
                if let Some(project) = research_db.get_project(&active.project_id) {
                    ui.group(|ui| {
                        // Project header with priority
                        ui.horizontal(|ui| {
                            let priority_color = match active.priority {
                                ResearchPriority::Low => egui::Color32::GRAY,
                                ResearchPriority::Normal => egui::Color32::WHITE,
                                ResearchPriority::High => egui::Color32::YELLOW,
                                ResearchPriority::Critical => egui::Color32::RED,
                            };
                            ui.colored_label(priority_color, format!("#{}", i + 1));
                            ui.strong(&project.name);
                        });
                        
                        // Progress bar
                        let progress_bar = egui::ProgressBar::new(active.progress)
                            .text(format!("{:.0}%", active.progress * 100.0));
                        ui.add(progress_bar);
                        
                        // Time remaining
                        ui.label(format!("Days Left: {:.1}", active.time_remaining));
                        
                        // Assigned scientist info
                        if let Some(scientist_entity) = active.assigned_scientist {
                            if let Ok((_, scientist)) = scientist_query.get(scientist_entity) {
                                ui.horizontal(|ui| {
                                    ui.colored_label(egui::Color32::CYAN, format!("Scientist: {}", scientist.name));
                                    ui.label(format!("Speed: {:.1}x", scientist.productivity_bonus));
                                });
                            }
                        } else {
                            ui.colored_label(egui::Color32::RED, "⚠ No scientist assigned");
                        }
                        
                        // Project benefits (always visible)
                        ui.label("Benefits:");
                        ui.indent(format!("benefits_{}", i), |ui| {
                            for benefit in &project.benefits {
                                show_research_benefit_inline(ui, benefit);
                            }
                        });
                        
                        // Action buttons
                        ui.horizontal(|ui| {
                            if ui.small_button("↑ Priority").clicked() {
                                // Priority increase logic
                            }
                            if ui.small_button("⏸ Pause").clicked() {
                                // Pause logic
                            }
                            if ui.small_button("❌ Cancel").clicked() {
                                // Cancel logic
                            }
                        });
                    });
                    ui.add_space(4.0);
                }
            }
        });
}

fn show_available_projects_flat(
    ui: &mut egui::Ui,
    global_data: &mut GlobalData,
    research_progress: &mut ResearchProgress,
    research_db: &ResearchDatabase,
    scientist_query: &Query<(Entity, &Scientist)>,
) {
    let available_projects = research_db.get_available_projects(research_progress);
    
    if available_projects.is_empty() {
        ui.colored_label(egui::Color32::GRAY, "No projects available");
        ui.weak("Complete prerequisites or finish current research");
        return;
    }
    
    egui::ScrollArea::vertical()
        .max_height(400.0)
        .show(ui, |ui| {
            for (proj_idx, project) in available_projects.iter().enumerate() {
                ui.group(|ui| {
                    // Project header
                    ui.horizontal(|ui| {
                        let category_color = get_category_color(project.category);
                        ui.colored_label(category_color, format!("{:?}", project.category));
                        ui.strong(&project.name);
                    });
                    
                    // Description
                    ui.label(&project.description);
                    
                    // Stats row
                    ui.horizontal(|ui| {
                        ui.label(format!("Time: {:.1} days", project.base_time_days));
                        ui.separator();
                        ui.label(format!("Difficulty: {:?}", project.difficulty));
                    });
                    
                    // Cost and stolen data
                    ui.horizontal(|ui| {
                        let can_afford = global_data.credits >= project.cost;
                        let cost_color = if can_afford { egui::Color32::WHITE } else { egui::Color32::RED };
                        ui.colored_label(cost_color, format!("Cost: ${}", project.cost));
                        
                        if let Some(&stolen_progress) = research_progress.stolen_data.get(&project.id) {
                            ui.separator();
                            ui.colored_label(egui::Color32::GREEN, format!("Stolen Data: {:.0}%", stolen_progress * 100.0));
                        }
                    });
                    
                    // Benefits (always visible, no collapsing)
                    ui.label("Benefits:");
                    ui.indent(format!("proj_benefits_{}", proj_idx), |ui| {
                        for benefit in &project.benefits {
                            show_research_benefit_inline(ui, benefit);
                        }
                    });
                    
                    // Prerequisites (if any)
                    if !project.prerequisites.is_empty() {
                        ui.label("Prerequisites:");
                        ui.indent(format!("proj_prereq_{}", proj_idx), |ui| {
                            for req in &project.prerequisites {
                                let completed = research_progress.completed.contains(req);
                                let (icon, color) = if completed { ("✓", egui::Color32::GREEN) } else { ("✗", egui::Color32::RED) };
                                ui.colored_label(color, format!("{} {}", icon, req));
                            }
                        });
                    }
                    
                    // Action buttons
                    ui.horizontal(|ui| {
                        let can_start = global_data.credits >= project.cost && research_progress.active_queue.len() < 5;
                        
                        if ui.add_enabled(can_start, egui::Button::new("🔬 Start Research")).clicked() {
                            start_research_project(
                                &project.id,
                                find_best_scientist(&project.category, scientist_query),
                                global_data,
                                research_progress,
                                research_db,
                            );
                        }
                        
                        // Prototype button
                        if project.prototype_available && !research_progress.prototypes.contains(&project.id) {
                            let prototype_cost = project.cost * 3;
                            let can_prototype = global_data.credits >= prototype_cost;
                            
                            if ui.add_enabled(can_prototype, egui::Button::new(format!("⚡ Prototype (${}))", prototype_cost))).clicked() {
                                if global_data.credits >= prototype_cost {
                                    global_data.credits -= prototype_cost;
                                    research_progress.prototypes.insert(project.id.clone());
                                    info!("Acquired prototype: {}", project.name);
                                }
                            }
                        }
                    });
                });
                ui.add_space(4.0);
            }
        });
}

fn show_scientists_flat(
    ui: &mut egui::Ui,
    global_data: &mut GlobalData,
    scientist_query: &Query<(Entity, &Scientist)>,
) {
    let recruited_scientists: Vec<_> = scientist_query.iter()
        .filter(|(_, s)| s.is_recruited)
        .collect();
    
    if recruited_scientists.is_empty() {
        ui.colored_label(egui::Color32::GRAY, "No scientists recruited");
        ui.weak("Find and recruit scientists in missions");
        return;
    }
    
    egui::ScrollArea::vertical()
        .max_height(200.0)
        .show(ui, |ui| {
            for (entity, scientist) in recruited_scientists {
                ui.group(|ui| {
                    // Name and specialization
                    ui.horizontal(|ui| {
                        let spec_color = get_category_color(scientist.specialization);
                        ui.colored_label(spec_color, &scientist.name);
                        ui.weak(format!("({:?})", scientist.specialization));
                    });
                    
                    // Stats in readable format
                    ui.label(format!("Productivity: {:.1}x", scientist.productivity_bonus));
                    ui.label(format!("Loyalty: {:.0}%", scientist.loyalty * 100.0));
                    ui.label(format!("Salary: ${}/day", scientist.daily_salary));
                    
                    // Assignment status
                    if let Some(project_id) = &scientist.current_project {
                        ui.colored_label(egui::Color32::GREEN, format!("Working: {}", project_id));
                    } else {
                        ui.colored_label(egui::Color32::YELLOW, "Available");
                    }
                    
                    // Quick actions
                    ui.horizontal(|ui| {
                        if ui.small_button("📋 Assign").clicked() {
                            // Assignment logic
                        }
                        if ui.small_button("💰 Bonus").clicked() {
                            // Loyalty bonus
                        }
                    });
                });
                ui.add_space(2.0);
            }
        });
    
    // Show discovered but unrecruited scientists
    let unrecruited_scientists: Vec<_> = scientist_query.iter()
        .filter(|(_, s)| !s.is_recruited)
        .collect();
    
    if !unrecruited_scientists.is_empty() {
        ui.separator();
        ui.strong(format!("DISCOVERED SCIENTISTS ({})", unrecruited_scientists.len()));
        
        egui::ScrollArea::vertical()
            .max_height(150.0)
            .show(ui, |ui| {
                for (entity, scientist) in unrecruited_scientists {
                    ui.horizontal(|ui| {
                        let spec_color = get_category_color(scientist.specialization);
                        ui.colored_label(spec_color, &scientist.name);
                        ui.weak(format!("({:?})", scientist.specialization));
                        
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            let can_recruit = global_data.credits >= scientist.recruitment_cost;
                            if ui.add_enabled(can_recruit, egui::Button::new("Recruit")).clicked() {
                                info!("Attempting to recruit: {}", scientist.name);
                            }
                            ui.weak(format!("${}", scientist.recruitment_cost));
                        });
                    });
                }
            });
    }
}

fn show_stolen_data_flat(
    ui: &mut egui::Ui,
    research_progress: &ResearchProgress,
    research_db: &ResearchDatabase,
) {
    egui::ScrollArea::vertical()
        .max_height(100.0)
        .show(ui, |ui| {
            for (project_id, progress) in &research_progress.stolen_data {
                if let Some(project) = research_db.get_project(project_id) {
                    ui.horizontal(|ui| {
                        ui.colored_label(egui::Color32::from_rgb(150, 100, 200), "📊");
                        ui.label(&project.name);
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.colored_label(egui::Color32::GREEN, format!("{:.0}% stolen", progress * 100.0));
                        });
                    });
                }
            }
        });
}

fn show_research_benefit_inline(ui: &mut egui::Ui, benefit: &ResearchBenefit) {
    let (icon, text, color) = match benefit {
        ResearchBenefit::UnlockAttachment(id) => 
            ("🔧", format!("Unlock attachment: {}", id), egui::Color32::from_rgb(150, 200, 150)),
        ResearchBenefit::UnlockWeapon(weapon) => 
            ("🔫", format!("Unlock weapon: {:?}", weapon), egui::Color32::from_rgb(200, 150, 150)),
        ResearchBenefit::UnlockTool(tool) => 
            ("🛠", format!("Unlock tool: {:?}", tool), egui::Color32::from_rgb(150, 150, 200)),
        ResearchBenefit::UnlockCybernetic(cyber) => 
            ("🧠", format!("Unlock cybernetic: {:?}", cyber), egui::Color32::from_rgb(200, 150, 200)),
        ResearchBenefit::CreditsPerMission(amount) => 
            ("💰", format!("+${} per mission", amount), egui::Color32::YELLOW),
        ResearchBenefit::ExperienceBonus(pct) => 
            ("⭐", format!("+{}% agent experience", pct), egui::Color32::from_rgb(200, 200, 150)),
        ResearchBenefit::AlertReduction(days) => 
            ("🛡", format!("Alert decay +{} days", days), egui::Color32::from_rgb(150, 200, 200)),
    };
    
    ui.horizontal(|ui| {
        ui.label(icon);
        ui.colored_label(color, text);
    });
}

fn get_category_color(category: ResearchCategory) -> egui::Color32 {
    match category {
        ResearchCategory::Weapons => egui::Color32::from_rgb(200, 80, 80),
        ResearchCategory::Cybernetics => egui::Color32::from_rgb(80, 80, 200), 
        ResearchCategory::Equipment => egui::Color32::from_rgb(80, 200, 80),
        ResearchCategory::Intelligence => egui::Color32::from_rgb(200, 200, 80),
    }
}

fn find_best_scientist(
    category: &ResearchCategory,
    scientist_query: &Query<(Entity, &Scientist)>,
) -> Option<Entity> {
    scientist_query.iter()
        .filter(|(_, s)| s.is_recruited && s.specialization == *category && s.current_project.is_none())
        .max_by(|(_, a), (_, b)| a.productivity_bonus.partial_cmp(&b.productivity_bonus).unwrap())
        .map(|(entity, _)| entity)
}

fn start_research_project(
    project_id: &str,
    scientist_entity: Option<Entity>,
    global_data: &mut GlobalData,
    research_progress: &mut ResearchProgress,
    research_db: &ResearchDatabase,
) {
    if let Some(project) = research_db.get_project(project_id) {
        if global_data.credits >= project.cost && research_progress.active_queue.len() < 5 {
            global_data.credits -= project.cost;
            
            let active_research = ActiveResearch {
                project_id: project_id.to_string(),
                progress: 0.0,
                time_remaining: project.base_time_days,
                completion_time: project.base_time_days,
                assigned_scientist: scientist_entity,
                priority: ResearchPriority::Normal,
            };
            
            research_progress.active_queue.push(active_research);
            info!("Started research project: {}", project.name);
        }
    }
}

// Keyboard/Gamepad navigation system (add to your main systems)
pub fn research_navigation_system(
    mut ui_state: ResMut<ResearchUIState>,
    keyboard: Res<ButtonInput<KeyCode>>,
    gamepads: Query<&Gamepad>,
) {
    let input = MenuInput::new(&keyboard, &gamepads);

    // Tab navigation with Q/E or shoulder buttons
    if input.option {
        ui_state.selected_section = match ui_state.selected_section {
            ResearchSection::Queue => ResearchSection::Scientists,
            ResearchSection::Available => ResearchSection::Queue,
            ResearchSection::Scientists => ResearchSection::Available,
        };
    }
    
    if keyboard.just_pressed(KeyCode::KeyE) {
        ui_state.selected_section = match ui_state.selected_section {
            ResearchSection::Queue => ResearchSection::Available,
            ResearchSection::Available => ResearchSection::Scientists,
            ResearchSection::Scientists => ResearchSection::Queue,
        };
    }
    
    // Vertical navigation with arrow keys or D-pad
    if input.up {
        match ui_state.selected_section {
            ResearchSection::Queue => ui_state.selected_project = ui_state.selected_project.saturating_sub(1),
            ResearchSection::Available => ui_state.selected_project = ui_state.selected_project.saturating_sub(1),
            ResearchSection::Scientists => ui_state.selected_scientist = ui_state.selected_scientist.saturating_sub(1),
        }
    }
    
    if input.down {
        match ui_state.selected_section {
            ResearchSection::Queue => ui_state.selected_project = ui_state.selected_project.saturating_add(1),
            ResearchSection::Available => ui_state.selected_project = ui_state.selected_project.saturating_add(1),
            ResearchSection::Scientists => ui_state.selected_scientist = ui_state.selected_scientist.saturating_add(1),
        }
    }
    
    // Action with Space/Enter or A button
    if input.select || keyboard.just_pressed(KeyCode::Enter) {
        // Trigger action based on current selection
        info!("Action triggered for selected item in {:?} section", ui_state.selected_section);
    }
}