// src/systems/ui/hub/research.rs - Enhanced research UI with queue management and scientists
use bevy::prelude::*;
use bevy_egui::egui;
use crate::core::*;
use std::collections::HashMap;

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
        show_research_queue(&mut columns[0], research_progress, research_db, scientist_query);
        
        // COLUMN 2: AVAILABLE PROJECTS  
        columns[1].heading("AVAILABLE PROJECTS");
        show_available_projects(&mut columns[1], global_data, research_progress, research_db, scientist_query);
        
        // COLUMN 3: SCIENTISTS
        columns[2].heading("RESEARCH TEAM");
        show_scientists(&mut columns[2], global_data, scientist_query);
    });
    
    ui.separator();
    
    // === BOTTOM SECTION: ESPIONAGE DATA ===
    if !research_progress.stolen_data.is_empty() {
        ui.collapsing("STOLEN RESEARCH DATA", |ui| {
            egui::ScrollArea::vertical().max_height(100.0).show(ui, |ui| {
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
        });
    }
}

fn show_research_queue(
    ui: &mut egui::Ui,
    research_progress: &mut ResearchProgress,
    research_db: &ResearchDatabase,
    scientist_query: &Query<(Entity, &Scientist)>,
) {
    if research_progress.active_queue.is_empty() {
        ui.colored_label(egui::Color32::GRAY, "No active research projects");
        return;
    }
    
    egui::ScrollArea::vertical().show(ui, |ui| {
        for (i, active) in research_progress.active_queue.iter().enumerate() {
            if let Some(project) = research_db.get_project(&active.project_id) {
                ui.group(|ui| {
                    // Project name and priority
                    ui.horizontal(|ui| {
                        let priority_color = match active.priority {
                            ResearchPriority::Low => egui::Color32::GRAY,
                            ResearchPriority::Normal => egui::Color32::WHITE,
                            ResearchPriority::High => egui::Color32::YELLOW,
                            ResearchPriority::Critical => egui::Color32::RED,
                        };
                        ui.colored_label(priority_color, format!("#{}", i + 1));
                        ui.label(&project.name);
                    });
                    
                    // Progress bar
                    let progress_bar = egui::ProgressBar::new(active.progress)
                        .text(format!("{:.0}%", active.progress * 100.0));
                    ui.add(progress_bar);
                    
                    // Time and scientist info
                    ui.horizontal(|ui| {
                        ui.weak(format!("{:.1} days left", active.time_remaining));
                        
                        if let Some(scientist_entity) = active.assigned_scientist {
                            if let Ok((_, scientist)) = scientist_query.get(scientist_entity) {
                                ui.separator();
                                ui.colored_label(egui::Color32::CYAN, &scientist.name);
                                ui.weak(format!("({:.1}x speed)", scientist.productivity_bonus));
                            }
                        } else {
                            ui.separator();
                            ui.colored_label(egui::Color32::RED, "No scientist assigned");
                        }
                    });
                    
                    // Management buttons
                    ui.horizontal(|ui| {
                        if ui.small_button("↑ Priority").clicked() {
                            // Increase priority logic here
                        }
                        if ui.small_button("⏸ Pause").clicked() {
                            // Pause/resume logic here  
                        }
                        if ui.small_button("❌ Cancel").clicked() {
                            // Cancel project logic here
                        }
                    });
                });
            }
        }
    });
}

fn show_available_projects(
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
    
    egui::ScrollArea::vertical().show(ui, |ui| {
        for project in available_projects {
            ui.group(|ui| {
                // Header
                ui.horizontal(|ui| {
                    let category_color = get_category_color(project.category);
                    ui.colored_label(category_color, format!("[{:?}]", project.category));
                    ui.heading(&project.name);
                });
                
                // Description and time
                ui.label(&project.description);
                ui.horizontal(|ui| {
                    ui.weak(format!("Time: {:.1} days", project.base_time_days));
                    ui.separator();
                    ui.weak(format!("Difficulty: {:?}", project.difficulty));
                });
                
                // Cost and stolen data bonus
                ui.horizontal(|ui| {
                    let can_afford = global_data.credits >= project.cost;
                    let cost_color = if can_afford { egui::Color32::WHITE } else { egui::Color32::RED };
                    ui.colored_label(cost_color, format!("Cost: ${}", project.cost));
                    
                    if let Some(&stolen_progress) = research_progress.stolen_data.get(&project.id) {
                        ui.separator();
                        ui.colored_label(egui::Color32::GREEN, format!("Data: {:.0}%", stolen_progress * 100.0));
                    }
                });
                
                // Benefits preview
                ui.collapsing("Benefits", |ui| {
                    for benefit in &project.benefits {
                        show_research_benefit(ui, benefit);
                    }
                });
                
                // Action buttons
                ui.horizontal(|ui| {
                    let can_start = global_data.credits >= project.cost && 
                                  research_progress.active_queue.len() < 5;
                    
                    // Start Research button
                    if ui.add_enabled(can_start, egui::Button::new("🔬 Start Research")).clicked() {
                        start_research_project(
                            &project.id,
                            find_best_scientist(&project.category, scientist_query),
                            global_data,
                            research_progress,
                            research_db,
                        );
                    }
                    
                    // Prototype button (if available)
                    if project.prototype_available && !research_progress.prototypes.contains(&project.id) {
                        let prototype_cost = project.cost * 3;
                        let can_prototype = global_data.credits >= prototype_cost;
                        
                        if ui.add_enabled(can_prototype, egui::Button::new("⚡ Prototype")).clicked() {
                            // Prototype acquisition logic would go here
                            if global_data.credits >= prototype_cost {
                                global_data.credits -= prototype_cost;
                                research_progress.prototypes.insert(project.id.clone());
                                info!("Acquired prototype: {}", project.name);
                            }
                        }
                        
                        if ui.small_button("?").on_hover_text("Use immediately at 3x cost, risky but fast").clicked() {}
                    }
                });
                
                // Prerequisites warning
                if !project.prerequisites.is_empty() {
                    ui.collapsing("Prerequisites", |ui| {
                        for req in &project.prerequisites {
                            let completed = research_progress.completed.contains(req);
                            let color = if completed { egui::Color32::GREEN } else { egui::Color32::RED };
                            let icon = if completed { "✓" } else { "✗" };
                            ui.colored_label(color, format!("{} {}", icon, req));
                        }
                    });
                }
            });
        }
    });
}

fn show_scientists(
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
    
    egui::ScrollArea::vertical().show(ui, |ui| {
        for (entity, scientist) in recruited_scientists {
            ui.group(|ui| {
                // Scientist name and specialization
                ui.horizontal(|ui| {
                    let spec_color = get_category_color(scientist.specialization);
                    ui.colored_label(spec_color, &scientist.name);
                    ui.weak(format!("({:?})", scientist.specialization));
                });
                
                // Stats
                ui.horizontal(|ui| {
                    ui.weak(format!("Productivity: {:.1}x", scientist.productivity_bonus));
                    ui.separator();
                    ui.weak(format!("Loyalty: {:.0}%", scientist.loyalty * 100.0));
                    ui.separator();
                    ui.weak(format!("Salary: ${}/day", scientist.daily_salary));
                });
                
                // Current assignment
                if let Some(project_id) = &scientist.current_project {
                    ui.colored_label(egui::Color32::GREEN, format!("Working on: {}", project_id));
                } else {
                    ui.colored_label(egui::Color32::YELLOW, "Available for assignment");
                }
                
                // Management buttons
                ui.horizontal(|ui| {
                    if ui.small_button("📋 Assign").clicked() {
                        // Assignment logic here
                    }
                    if ui.small_button("💰 Bonus").clicked() {
                        // Loyalty bonus logic here
                    }
                });
            });
        }
    });
    
    // Show unrecruited scientists found in world
    let unrecruited_scientists: Vec<_> = scientist_query.iter()
        .filter(|(_, s)| !s.is_recruited)
        .collect();
    
    if !unrecruited_scientists.is_empty() {
        ui.separator();
        ui.collapsing(format!("DISCOVERED SCIENTISTS ({})", unrecruited_scientists.len()), |ui| {
            for (entity, scientist) in unrecruited_scientists {
                ui.horizontal(|ui| {
                    let spec_color = get_category_color(scientist.specialization);
                    ui.colored_label(spec_color, &scientist.name);
                    ui.weak(format!("({:?})", scientist.specialization));
                    
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let can_recruit = global_data.credits >= scientist.recruitment_cost;
                        if ui.add_enabled(can_recruit, egui::Button::new("Recruit")).clicked() {
                            // Recruitment would be handled by event system
                            info!("Attempting to recruit: {}", scientist.name);
                        }
                        ui.weak(format!("${}", scientist.recruitment_cost));
                    });
                });
            }
        });
    }
}

fn show_research_benefit(ui: &mut egui::Ui, benefit: &ResearchBenefit) {
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

// === ADVANCED RESEARCH UI FEATURES ===

pub fn show_technology_tree(
    ui: &mut egui::Ui,
    research_progress: &ResearchProgress,
    research_db: &ResearchDatabase,
) {
    ui.heading("TECHNOLOGY TREE");
    
    // Group projects by category - use explicit HashMap import
    let mut categories: HashMap<ResearchCategory, Vec<&ResearchProject>> = HashMap::new();
    
    for project in &research_db.projects {
        categories.entry(project.category).or_insert_with(Vec::new).push(project);
    }
    
    // Show each category as a column
    ui.columns(4, |columns| {
        let cats = [ResearchCategory::Weapons, ResearchCategory::Equipment, 
                   ResearchCategory::Cybernetics, ResearchCategory::Intelligence];
        
        for (i, &category) in cats.iter().enumerate() {
            columns[i].vertical(|ui| {
                ui.colored_label(get_category_color(category), format!("{:?}", category));
                ui.separator();
                
                if let Some(projects) = categories.get(&category) {
                    for project in projects {
                        let status = if research_progress.completed.contains(&project.id) {
                            ("✓", egui::Color32::GREEN)
                        } else if research_progress.active_queue.iter().any(|a| a.project_id == project.id) {
                            ("⏳", egui::Color32::YELLOW)  
                        } else if research_db.projects.iter().find(|p| p.id == project.id)
                            .map_or(false, |p| p.prerequisites.iter().all(|req| research_progress.completed.contains(req))) {
                            ("○", egui::Color32::WHITE)
                        } else {
                            ("🔒", egui::Color32::GRAY)
                        };
                        
                        ui.horizontal(|ui| {
                            ui.colored_label(status.1, status.0);
                            ui.weak(&project.name);
                        });
                    }
                }
            });
        }
    });
}

pub fn show_research_analytics(
    ui: &mut egui::Ui,
    research_progress: &ResearchProgress,
    scientist_query: &Query<(Entity, &Scientist)>,
) {
    ui.collapsing("RESEARCH ANALYTICS", |ui| {
        // Investment summary
        ui.horizontal(|ui| {
            ui.label("Total Investment:");
            ui.colored_label(egui::Color32::YELLOW, format!("${}", research_progress.credits_invested));
            ui.separator();
            
            let daily_salaries: u32 = scientist_query.iter()
                .filter(|(_, s)| s.is_recruited)
                .map(|(_, s)| s.daily_salary)
                .sum();
            ui.label("Daily Costs:");
            ui.colored_label(egui::Color32::RED, format!("${}/day", daily_salaries));
        });
        
        // Productivity metrics
        let avg_productivity: f32 = scientist_query.iter()
            .filter(|(_, s)| s.is_recruited)
            .map(|(_, s)| s.productivity_bonus)
            .sum::<f32>() / scientist_query.iter().filter(|(_, s)| s.is_recruited).count().max(1) as f32;
        
        ui.horizontal(|ui| {
            ui.label("Team Productivity:");
            ui.colored_label(egui::Color32::CYAN, format!("{:.2}x average", avg_productivity));
        });
        
        // Queue efficiency
        if !research_progress.active_queue.is_empty() {
            let total_progress: f32 = research_progress.active_queue.iter().map(|a| a.progress).sum();
            let avg_progress = total_progress / research_progress.active_queue.len() as f32;
            
            ui.horizontal(|ui| {
                ui.label("Queue Progress:");
                ui.colored_label(egui::Color32::GREEN, format!("{:.1}% average", avg_progress * 100.0));
            });
        }
    });
}


// TO BE ADDED
pub fn auto_assign_scientists(
    mut research_progress: ResMut<ResearchProgress>,
    scientist_query: Query<(Entity, &Scientist)>,
    research_db: Res<ResearchDatabase>,
) {
    // Auto-assign available scientists to unassigned projects
    for active in &mut research_progress.active_queue {
        if active.assigned_scientist.is_none() {
            if let Some(project) = research_db.get_project(&active.project_id) {
                // Find best available scientist for this category
                let best_scientist = scientist_query.iter()
                    .filter(|(_, s)| {
                        s.is_recruited && 
                        s.current_project.is_none() &&
                        s.specialization == project.category
                    })
                    .max_by(|(_, a), (_, b)| {
                        a.productivity_bonus.partial_cmp(&b.productivity_bonus).unwrap()
                    });
                
                if let Some((entity, _)) = best_scientist {
                    active.assigned_scientist = Some(entity);
                    info!("Auto-assigned scientist to {}", active.project_id);
                }
            }
        }
    }
}

pub fn research_recommendations(
    research_progress: &ResearchProgress,
    research_db: &ResearchDatabase,
    global_data: &GlobalData,
) -> Vec<String> {
    let mut recommendations = Vec::new();
    
    // Recommend based on current credits
    if global_data.credits > 5000 {
        recommendations.push("Consider high-value Intelligence research".to_string());
    }
    
    // Recommend based on mission difficulty
    if global_data.alert_level > 3 {
        recommendations.push("Focus on stealth and counter-surveillance tech".to_string());
    }
    
    // Recommend prerequisite completion
    let available = research_db.get_available_projects(research_progress);
    if available.is_empty() {
        recommendations.push("Complete current research to unlock new projects".to_string());
    }
    
    recommendations
}

