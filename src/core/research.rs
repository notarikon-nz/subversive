// src/core/research.rs - Simple research system inspired by original Syndicate
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use crate::core::*;

#[derive(Resource, Default, Clone, Serialize, Deserialize)]
pub struct ResearchProgress {
    pub completed: HashSet<String>,
    pub credits_invested: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchProject {
    pub id: String,
    pub name: String,
    pub description: String,
    pub cost: u32,
    pub category: ResearchCategory,
    pub prerequisites: Vec<String>, // IDs of required research
    pub benefits: Vec<ResearchBenefit>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResearchCategory {
    Weapons,      // Unlock attachments and weapon types
    Cybernetics,  // Agent augmentations  
    Equipment,    // Tools and gadgets
    Intelligence, // Mission advantages
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResearchBenefit {
    UnlockAttachment(String),           // Unlock specific attachment
    UnlockWeapon(WeaponType),          // Unlock weapon type
    UnlockTool(ToolType),              // Unlock tool type
    UnlockCybernetic(CyberneticType),  // Unlock cybernetic
    CreditsPerMission(u32),            // Bonus credits per mission
    ExperienceBonus(u32),              // Extra XP percentage
    AlertReduction(u32),               // Reduce regional alert faster
}

impl ResearchProject {
    pub fn is_available(&self, progress: &ResearchProgress) -> bool {
        self.prerequisites.iter().all(|req| progress.completed.contains(req))
    }
    
    pub fn is_completed(&self, progress: &ResearchProgress) -> bool {
        progress.completed.contains(&self.id)
    }
}

#[derive(Resource, Default, Clone, Serialize, Deserialize)]
pub struct ResearchDatabase {
    pub projects: Vec<ResearchProject>,
}


impl ResearchDatabase {
    pub fn load() -> Self {
        match std::fs::read_to_string("data/research.json") {
            Ok(content) => {
                match serde_json::from_str::<ResearchDatabase>(&content) {  // Add type annotation
                    Ok(data) => {
                        info!("Loaded {} research projects from data/research.json", data.projects.len());
                        data
                    },
                    Err(e) => {
                        error!("Failed to parse research.json: {}", e);
                        Self { projects: Vec::new() }
                    }
                }
            },
            Err(e) => {
                error!("Failed to load data/research.json: {}", e);
                Self { projects: Vec::new() }
            }
        }
    }
    
    // Add missing methods that are used elsewhere in the codebase
    pub fn get_project(&self, id: &str) -> Option<&ResearchProject> {
        self.projects.iter().find(|p| p.id == id)
    }
    
    pub fn get_available_projects(&self, progress: &ResearchProgress) -> Vec<&ResearchProject> {
        self.projects.iter()
            .filter(|p| p.is_available(progress) && !p.is_completed(progress))
            .collect()
    }
    
    pub fn get_completed_projects(&self, progress: &ResearchProgress) -> Vec<&ResearchProject> {
        self.projects.iter()
            .filter(|p| p.is_completed(progress))
            .collect()
    }
}


// Apply research benefits to game state
pub fn apply_research_benefits(
    progress: &ResearchProgress,
    research_db: &ResearchDatabase,
    unlocked_attachments: &mut UnlockedAttachments,
    global_data: &mut GlobalData,
) {
    for project_id in &progress.completed {
        if let Some(project) = research_db.get_project(project_id) {
            for benefit in &project.benefits {
                match benefit {
                    ResearchBenefit::UnlockAttachment(attachment_id) => {
                        unlocked_attachments.attachments.insert(attachment_id.clone());
                    },
                    ResearchBenefit::UnlockWeapon(_weapon_type) => {
                        // Weapons are available to purchase - handled in equipment systems
                    },
                    ResearchBenefit::UnlockTool(_tool_type) => {
                        // Tools are available to find - handled in mission systems  
                    },
                    ResearchBenefit::UnlockCybernetic(_cybernetic_type) => {
                        // Cybernetics are available to install - handled in agent systems
                    },
                    ResearchBenefit::CreditsPerMission(_amount) => {
                        // Applied during mission completion
                    },
                    ResearchBenefit::ExperienceBonus(_percentage) => {
                        // Applied during XP calculation
                    },
                    ResearchBenefit::AlertReduction(_days) => {
                        // Applied during region alert decay
                    },
                }
            }
        }
    }
}

// NEW: Startup-safe version that only handles immediate unlocks
pub fn apply_research_unlocks(
    progress: &ResearchProgress,
    research_db: &ResearchDatabase,
    unlocked_attachments: &mut UnlockedAttachments,
) {
    for project_id in &progress.completed {
        if let Some(project) = research_db.get_project(project_id) {
            for benefit in &project.benefits {
                match benefit {
                    ResearchBenefit::UnlockAttachment(attachment_id) => {
                        unlocked_attachments.attachments.insert(attachment_id.clone());
                        info!("Research: Unlocked attachment {}", attachment_id);
                    },
                    // Other benefits don't need immediate application at startup
                    _ => {}
                }
            }
        }
    }
}

// Calculate bonus credits from research
pub fn calculate_research_credit_bonus(
    progress: &ResearchProgress,
    research_db: &ResearchDatabase,
) -> u32 {
    let mut total_bonus = 0;
    
    for project_id in &progress.completed {
        if let Some(project) = research_db.get_project(project_id) {
            for benefit in &project.benefits {
                if let ResearchBenefit::CreditsPerMission(amount) = benefit {
                    total_bonus += amount;
                }
            }
        }
    }
    
    total_bonus
}

// Calculate experience bonus from research
pub fn calculate_research_xp_bonus(
    progress: &ResearchProgress,
    research_db: &ResearchDatabase,
    base_xp: u32,
) -> u32 {
    let mut bonus_percentage = 0;
    
    for project_id in &progress.completed {
        if let Some(project) = research_db.get_project(project_id) {
            for benefit in &project.benefits {
                if let ResearchBenefit::ExperienceBonus(percentage) = benefit {
                    bonus_percentage += percentage;
                }
            }
        }
    }
    
    if bonus_percentage > 0 {
        base_xp + (base_xp * bonus_percentage / 100)
    } else {
        base_xp
    }
}

// Get alert reduction bonus from research
pub fn get_research_alert_reduction(
    progress: &ResearchProgress,
    research_db: &ResearchDatabase,
) -> u32 {
    let mut total_reduction = 0;
    
    for project_id in &progress.completed {
        if let Some(project) = research_db.get_project(project_id) {
            for benefit in &project.benefits {
                if let ResearchBenefit::AlertReduction(days) = benefit {
                    total_reduction += days;
                }
            }
        }
    }
    
    total_reduction
}

// Simple research purchase system
pub fn purchase_research(
    project_id: &str,
    global_data: &mut GlobalData,
    progress: &mut ResearchProgress,
    research_db: &ResearchDatabase,
    unlocked_attachments: &mut UnlockedAttachments,
) -> bool {
    if let Some(project) = research_db.get_project(project_id) {
        // Check if available and affordable
        if project.is_available(progress) && !project.is_completed(progress) && global_data.credits >= project.cost {
            // Purchase research
            global_data.credits -= project.cost;
            progress.completed.insert(project_id.to_string());
            progress.credits_invested += project.cost;
            
            // Apply immediate benefits
            apply_research_benefits(progress, research_db, unlocked_attachments, global_data);
            
            info!("Research completed: {} for {} credits", project.name, project.cost);
            return true;
        }
    }
    
    false
}
