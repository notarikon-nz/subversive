// src/systems/ui/hub/research.rs - Updated with functional research system
use bevy::prelude::*;
use crate::core::*;
use crate::core::research::*;

pub fn handle_input(
    input: &ButtonInput<KeyCode>,
    global_data: &mut GlobalData,
    research_progress: &mut ResearchProgress,
    research_db: &ResearchDatabase,
    unlocked_attachments: &mut UnlockedAttachments,
    selected_project: &mut usize,
) -> bool {
    let available_projects = research_db.get_available_projects(research_progress);
    let mut needs_rebuild = false;
    
    // Navigate projects with UP/DOWN
    if input.just_pressed(KeyCode::ArrowUp) && *selected_project > 0 {
        *selected_project -= 1;
        needs_rebuild = true;
    }
    
    if input.just_pressed(KeyCode::ArrowDown) && *selected_project < available_projects.len().saturating_sub(1) {
        *selected_project += 1;
        needs_rebuild = true;
    }
    
    // Purchase with ENTER
    if input.just_pressed(KeyCode::Enter) && !available_projects.is_empty() {
        if let Some(project) = available_projects.get(*selected_project) {
            if purchase_research(
                &project.id,
                global_data,
                research_progress,
                research_db,
                unlocked_attachments,
            ) {
                needs_rebuild = true;
                // Reset selection if we run out of projects
                let new_available = research_db.get_available_projects(research_progress);
                if *selected_project >= new_available.len() && !new_available.is_empty() {
                    *selected_project = new_available.len() - 1;
                }
            }
        }
    }
    
    needs_rebuild
}

pub fn create_content(
    parent: &mut ChildBuilder, 
    global_data: &GlobalData,
    research_progress: &ResearchProgress,
    research_db: &ResearchDatabase,
    selected_project: usize,
) {
    parent.spawn(NodeBundle {
        style: Style {
            width: Val::Percent(100.0),
            flex_grow: 1.0,
            padding: UiRect::all(Val::Px(20.0)),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(15.0),
            ..default()
        },
        ..default()
    }).with_children(|content| {
        // Header
        content.spawn(TextBundle::from_section(
            "RESEARCH & DEVELOPMENT",
            TextStyle { font_size: 24.0, color: Color::srgb(0.8, 0.8, 0.2), ..default() }
        ));
        
        // Credits and progress info
        content.spawn(NodeBundle {
            style: Style {
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(30.0),
                ..default()
            },
            ..default()
        }).with_children(|info| {
            info.spawn(TextBundle::from_section(
                format!("Available Credits: {}", global_data.credits),
                TextStyle { font_size: 16.0, color: Color::WHITE, ..default() }
            ));
            
            info.spawn(TextBundle::from_section(
                format!("Research Investment: {}", research_progress.credits_invested),
                TextStyle { font_size: 16.0, color: Color::srgb(0.6, 0.8, 0.6), ..default() }
            ));
            
            info.spawn(TextBundle::from_section(
                format!("Projects Completed: {}", research_progress.completed.len()),
                TextStyle { font_size: 16.0, color: Color::srgb(0.8, 0.6, 0.2), ..default() }
            ));
        });
        
        // Available research projects
        let available_projects = research_db.get_available_projects(research_progress);
        
        if !available_projects.is_empty() {
            content.spawn(TextBundle::from_section(
                "\nAVAILABLE RESEARCH:",
                TextStyle { font_size: 18.0, color: Color::WHITE, ..default() }
            ));
            
            // Show available projects with selection
            content.spawn(NodeBundle {
                style: Style {
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(8.0),
                    padding: UiRect::all(Val::Px(10.0)),
                    ..default()
                },
                background_color: Color::srgba(0.1, 0.1, 0.2, 0.5).into(),
                ..default()
            }).with_children(|projects_panel| {
                for (i, project) in available_projects.iter().enumerate() {
                    let is_selected = i == selected_project;
                    let can_afford = global_data.credits >= project.cost;
                    
                    let bg_color = if is_selected {
                        if can_afford {
                            Color::srgba(0.2, 0.6, 0.2, 0.3)
                        } else {
                            Color::srgba(0.6, 0.2, 0.2, 0.3)
                        }
                    } else {
                        Color::NONE
                    };
                    
                    let text_color = if can_afford {
                        Color::WHITE
                    } else {
                        Color::srgb(0.6, 0.6, 0.6)
                    };
                    
                    let prefix = if is_selected { "> " } else { "  " };
                    let category_color = match project.category {
                        ResearchCategory::Weapons => Color::srgb(0.8, 0.3, 0.3),
                        ResearchCategory::Cybernetics => Color::srgb(0.3, 0.3, 0.8),
                        ResearchCategory::Equipment => Color::srgb(0.3, 0.8, 0.3),
                        ResearchCategory::Intelligence => Color::srgb(0.8, 0.8, 0.3),
                    };
                    
                    projects_panel.spawn(NodeBundle {
                        style: Style {
                            flex_direction: FlexDirection::Column,
                            padding: UiRect::all(Val::Px(8.0)),
                            row_gap: Val::Px(4.0),
                            ..default()
                        },
                        background_color: bg_color.into(),
                        ..default()
                    }).with_children(|project_info| {
                        // Project name and cost
                        project_info.spawn(NodeBundle {
                            style: Style {
                                flex_direction: FlexDirection::Row,
                                justify_content: JustifyContent::SpaceBetween,
                                ..default()
                            },
                            ..default()
                        }).with_children(|title_row| {
                            title_row.spawn(TextBundle::from_section(
                                format!("{}{}", prefix, project.name),
                                TextStyle { font_size: 16.0, color: text_color, ..default() }
                            ));
                            
                            title_row.spawn(TextBundle::from_section(
                                format!("{} credits", project.cost),
                                TextStyle { font_size: 16.0, color: category_color, ..default() }
                            ));
                        });
                        
                        // Description
                        project_info.spawn(TextBundle::from_section(
                            format!("   {}", project.description),
                            TextStyle { font_size: 14.0, color: Color::srgb(0.8, 0.8, 0.8), ..default() }
                        ));
                        
                        // Benefits preview
                        if is_selected {
                            let benefits_text = project.benefits.iter()
                                .map(|benefit| match benefit {
                                    ResearchBenefit::UnlockAttachment(id) => format!("Unlock attachment: {}", id),
                                    ResearchBenefit::UnlockWeapon(weapon) => format!("Unlock weapon: {:?}", weapon),
                                    ResearchBenefit::UnlockTool(tool) => format!("Unlock tool: {:?}", tool),
                                    ResearchBenefit::UnlockCybernetic(cyber) => format!("Unlock cybernetic: {:?}", cyber),
                                    ResearchBenefit::CreditsPerMission(amount) => format!("+{} credits per mission", amount),
                                    ResearchBenefit::ExperienceBonus(pct) => format!("+{}% agent experience", pct),
                                    ResearchBenefit::AlertReduction(days) => format!("Alert decay +{} days", days),
                                })
                                .collect::<Vec<_>>()
                                .join("\n   • ");
                            
                            project_info.spawn(TextBundle::from_section(
                                format!("   Benefits:\n   • {}", benefits_text),
                                TextStyle { font_size: 12.0, color: Color::srgb(0.6, 0.8, 0.6), ..default() }
                            ));
                        }
                    });
                }
            });
            
            // Controls
            content.spawn(TextBundle::from_section(
                "\n↑↓: Navigate | ENTER: Purchase Research",
                TextStyle { font_size: 14.0, color: Color::srgb(0.7, 0.7, 0.7), ..default() }
            ));
            
        } else {
            content.spawn(TextBundle::from_section(
                "\nNo research projects available.\nComplete missions to earn credits for research.",
                TextStyle { font_size: 16.0, color: Color::srgb(0.6, 0.6, 0.6), ..default() }
            ));
        }
        
        // Completed research summary
        let completed_projects = research_db.get_completed_projects(research_progress);
        if !completed_projects.is_empty() {
            content.spawn(TextBundle::from_section(
                "\nCOMPLETED RESEARCH:",
                TextStyle { font_size: 18.0, color: Color::srgb(0.2, 0.8, 0.2), ..default() }
            ));
            
            for project in completed_projects.iter().take(5) { // Show last 5 completed
                let category_color = match project.category {
                    ResearchCategory::Weapons => Color::srgb(0.8, 0.3, 0.3),
                    ResearchCategory::Cybernetics => Color::srgb(0.3, 0.3, 0.8),
                    ResearchCategory::Equipment => Color::srgb(0.3, 0.8, 0.3),
                    ResearchCategory::Intelligence => Color::srgb(0.8, 0.8, 0.3),
                };
                
                content.spawn(TextBundle::from_section(
                    format!("✓ {} ({:?})", project.name, project.category),
                    TextStyle { font_size: 14.0, color: category_color, ..default() }
                ));
            }
            
            if completed_projects.len() > 5 {
                content.spawn(TextBundle::from_section(
                    format!("... and {} more", completed_projects.len() - 5),
                    TextStyle { font_size: 12.0, color: Color::srgb(0.6, 0.6, 0.6), ..default() }
                ));
            }
        }
    });
}
