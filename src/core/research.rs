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

#[derive(Resource)]
pub struct ResearchDatabase {
    pub projects: Vec<ResearchProject>,
}

impl ResearchDatabase {
    pub fn load() -> Self {
        // Create research projects inspired by original Syndicate but simplified
        let projects = vec![
            // === WEAPONS RESEARCH ===
            ResearchProject {
                id: "basic_optics".to_string(),
                name: "Basic Optics".to_string(),
                description: "Develop improved weapon sighting systems".to_string(),
                cost: 1000,
                category: ResearchCategory::Weapons,
                prerequisites: vec![],
                benefits: vec![
                    ResearchBenefit::UnlockAttachment("red_dot".to_string()),
                    ResearchBenefit::UnlockAttachment("iron_sights".to_string()),
                ],
            },
            
            ResearchProject {
                id: "suppression_tech".to_string(),
                name: "Suppression Technology".to_string(),
                description: "Develop sound dampening for covert operations".to_string(),
                cost: 1500,
                category: ResearchCategory::Weapons,
                prerequisites: vec!["basic_optics".to_string()],
                benefits: vec![
                    ResearchBenefit::UnlockAttachment("suppressor".to_string()),
                    ResearchBenefit::UnlockAttachment("flash_hider".to_string()),
                ],
            },
            
            ResearchProject {
                id: "advanced_magazines".to_string(),
                name: "Advanced Magazines".to_string(),
                description: "Improve ammunition feeding systems".to_string(),
                cost: 2000,
                category: ResearchCategory::Weapons,
                prerequisites: vec!["suppression_tech".to_string()],
                benefits: vec![
                    ResearchBenefit::UnlockAttachment("extended_mag".to_string()),
                    ResearchBenefit::UnlockAttachment("fast_mag".to_string()),
                ],
            },
            
            ResearchProject {
                id: "heavy_weapons".to_string(),
                name: "Heavy Weapons Platform".to_string(),
                description: "Develop support weapons for high-threat missions".to_string(),
                cost: 3000,
                category: ResearchCategory::Weapons,
                prerequisites: vec!["advanced_magazines".to_string()],
                benefits: vec![
                    ResearchBenefit::UnlockWeapon(WeaponType::Minigun),
                    ResearchBenefit::UnlockAttachment("bipod".to_string()),
                ],
            },
            
            // === CYBERNETICS RESEARCH ===
            ResearchProject {
                id: "neurovector_implants".to_string(),
                name: "Neurovector Implants".to_string(),
                description: "Basic mind control technology for civilian manipulation".to_string(),
                cost: 2500,
                category: ResearchCategory::Cybernetics,
                prerequisites: vec![],
                benefits: vec![
                    ResearchBenefit::UnlockCybernetic(CyberneticType::Neurovector),
                ],
            },
            
            ResearchProject {
                id: "combat_enhancers".to_string(),
                name: "Combat Enhancers".to_string(),
                description: "Improve agent reflexes and combat effectiveness".to_string(),
                cost: 3500,
                category: ResearchCategory::Cybernetics,
                prerequisites: vec!["neurovector_implants".to_string()],
                benefits: vec![
                    ResearchBenefit::UnlockCybernetic(CyberneticType::CombatEnhancer),
                    ResearchBenefit::ExperienceBonus(25), // 25% more XP
                ],
            },
            
            // === EQUIPMENT RESEARCH ===
            ResearchProject {
                id: "surveillance_gear".to_string(),
                name: "Surveillance Gear".to_string(),
                description: "Advanced reconnaissance and hacking tools".to_string(),
                cost: 1200,
                category: ResearchCategory::Equipment,
                prerequisites: vec![],
                benefits: vec![
                    ResearchBenefit::UnlockTool(ToolType::Scanner),
                    ResearchBenefit::UnlockTool(ToolType::Hacker),
                ],
            },
            
            ResearchProject {
                id: "infiltration_kit".to_string(),
                name: "Infiltration Kit".to_string(),
                description: "Tools for covert entry and stealth operations".to_string(),
                cost: 1800,
                category: ResearchCategory::Equipment,
                prerequisites: vec!["surveillance_gear".to_string()],
                benefits: vec![
                    ResearchBenefit::UnlockTool(ToolType::Lockpick),
                    ResearchBenefit::UnlockCybernetic(CyberneticType::StealthModule),
                ],
            },
            
            // === INTELLIGENCE RESEARCH ===
            ResearchProject {
                id: "corporate_intelligence".to_string(),
                name: "Corporate Intelligence".to_string(),
                description: "Improve mission planning and regional influence".to_string(),
                cost: 2200,
                category: ResearchCategory::Intelligence,
                prerequisites: vec!["surveillance_gear".to_string()],
                benefits: vec![
                    ResearchBenefit::CreditsPerMission(200), // Extra 200 credits per mission
                    ResearchBenefit::AlertReduction(1), // Alert decays 1 day faster
                ],
            },
            
            ResearchProject {
                id: "tech_interface".to_string(),
                name: "Tech Interface".to_string(),
                description: "Advanced hacking and electronic warfare capabilities".to_string(),
                cost: 4000,
                category: ResearchCategory::Intelligence,
                prerequisites: vec!["corporate_intelligence".to_string(), "infiltration_kit".to_string()],
                benefits: vec![
                    ResearchBenefit::UnlockCybernetic(CyberneticType::TechInterface),
                    ResearchBenefit::CreditsPerMission(300),
                ],
            },
        ];
        
        Self { projects }
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
    
    pub fn get_project(&self, id: &str) -> Option<&ResearchProject> {
        self.projects.iter().find(|p| p.id == id)
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
