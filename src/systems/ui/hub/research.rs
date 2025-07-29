// src/systems/ui/hub/research.rs - egui version
use bevy::prelude::*;
use bevy_egui::egui;
use crate::core::*;

pub fn show_research(
    ui: &mut egui::Ui,
    global_data: &mut GlobalData,
    research_progress: &mut ResearchProgress,
    research_db: &ResearchDatabase,
    unlocked_attachments: &mut UnlockedAttachments,
    input: &ButtonInput<KeyCode>,
) {
    ui.heading("RESEARCH & DEVELOPMENT");
    
    ui.separator();
    
    // Header stats
    ui.horizontal(|ui| {
        ui.colored_label(egui::Color32::YELLOW, format!("Available Credits: {}", global_data.credits));
        ui.separator();
        ui.colored_label(egui::Color32::GREEN, format!("Research Investment: {}", research_progress.credits_invested));
        ui.separator();
        ui.colored_label(egui::Color32::from_rgb(200, 150, 50), format!("Projects Completed: {}", research_progress.completed.len()));
    });
    
    ui.separator();
    
    // Available projects
    let available_projects = research_db.get_available_projects(research_progress);
    
    if !available_projects.is_empty() {
        ui.heading("AVAILABLE RESEARCH:");
        
        egui::ScrollArea::vertical().show(ui, |ui| {
            for project in &available_projects {
                let can_afford = global_data.credits >= project.cost;
                
                ui.group(|ui| {
                    // Header with name and cost
                    ui.horizontal(|ui| {
                        let category_color = match project.category {
                            ResearchCategory::Weapons => egui::Color32::from_rgb(200, 80, 80),
                            ResearchCategory::Cybernetics => egui::Color32::from_rgb(80, 80, 200),
                            ResearchCategory::Equipment => egui::Color32::from_rgb(80, 200, 80),
                            ResearchCategory::Intelligence => egui::Color32::from_rgb(200, 200, 80),
                        };
                        
                        ui.colored_label(category_color, format!("[{:?}]", project.category));
                        ui.heading(&project.name);
                        
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            let cost_color = if can_afford { egui::Color32::WHITE } else { egui::Color32::GRAY };
                            ui.colored_label(cost_color, format!("{} credits", project.cost));
                        });
                    });
                    
                    // Description
                    ui.label(&project.description);
                    
                    // Benefits
                    ui.collapsing("Benefits", |ui| {
                        for benefit in &project.benefits {
                            let benefit_text = match benefit {
                                ResearchBenefit::UnlockAttachment(id) => format!("• Unlock attachment: {}", id),
                                ResearchBenefit::UnlockWeapon(weapon) => format!("• Unlock weapon: {:?}", weapon),
                                ResearchBenefit::UnlockTool(tool) => format!("• Unlock tool: {:?}", tool),
                                ResearchBenefit::UnlockCybernetic(cyber) => format!("• Unlock cybernetic: {:?}", cyber),
                                ResearchBenefit::CreditsPerMission(amount) => format!("• +{} credits per mission", amount),
                                ResearchBenefit::ExperienceBonus(pct) => format!("• +{}% agent experience", pct),
                                ResearchBenefit::AlertReduction(days) => format!("• Alert decay +{} days", days),
                            };
                            ui.colored_label(egui::Color32::GREEN, benefit_text);
                        }
                    });
                    
                    // Purchase button
                    if ui.add_enabled(can_afford, egui::Button::new("Purchase Research")).clicked() {
                        purchase_research(
                            &project.id,
                            global_data,
                            research_progress,
                            research_db,
                            unlocked_attachments,
                        );
                    }
                });
            }
        });
    } else {
        ui.group(|ui| {
            ui.colored_label(egui::Color32::GRAY, "No research projects available.");
            ui.label("Complete missions to earn credits for research.");
        });
    }
    
    ui.separator();
    
    // Completed research
    let completed_projects = research_db.get_completed_projects(research_progress);
    if !completed_projects.is_empty() {
        ui.collapsing(format!("COMPLETED RESEARCH ({})", completed_projects.len()), |ui| {
            egui::ScrollArea::vertical().max_height(200.0).show(ui, |ui| {
                for project in completed_projects.iter().take(10) { // Show last 10
                    let category_color = match project.category {
                        ResearchCategory::Weapons => egui::Color32::from_rgb(200, 80, 80),
                        ResearchCategory::Cybernetics => egui::Color32::from_rgb(80, 80, 200),
                        ResearchCategory::Equipment => egui::Color32::from_rgb(80, 200, 80),
                        ResearchCategory::Intelligence => egui::Color32::from_rgb(200, 200, 80),
                    };
                    
                    ui.horizontal(|ui| {
                        ui.label("✓");
                        ui.colored_label(category_color, &project.name);
                        ui.weak(format!("({:?})", project.category));
                    });
                }
                
                if completed_projects.len() > 10 {
                    ui.weak(format!("... and {} more", completed_projects.len() - 10));
                }
            });
        });
    }
}