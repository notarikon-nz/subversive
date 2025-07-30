// src/core/research.rs - Enhanced research system with Scientists, queues, and espionage
// 0.2.12
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::{HashSet, HashMap};
use crate::core::*;
use crate::core::factions::{Faction};
use crate::systems::scanner::{Scannable};
use crate::systems::day_night::*;

// === RESEARCH PROGRESS & QUEUE ===
#[derive(Resource, Default, Clone, Serialize, Deserialize)]
pub struct ResearchProgress {
    pub completed: HashSet<String>,
    pub credits_invested: u32,
    pub active_queue: Vec<ActiveResearch>,
    pub stolen_data: HashMap<String, f32>, // project_id -> progress stolen (0.0-1.0)
    pub prototypes: HashSet<String>, // Items available as prototype only
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ResearchCategory {
    Weapons, Cybernetics, Equipment, Intelligence,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResearchBenefit {
    UnlockAttachment(String),
    UnlockWeapon(WeaponType),
    UnlockTool(ToolType),
    UnlockCybernetic(CyberneticType),
    CreditsPerMission(u32),
    ExperienceBonus(u32),
    AlertReduction(u32),
}


#[derive(Clone, Serialize, Deserialize)]
pub struct ActiveResearch {
    pub project_id: String,
    pub assigned_scientist: Option<Entity>,
    pub progress: f32, // 0.0 to 1.0
    pub completion_time: f32, // Total time needed in days
    pub time_remaining: f32,
    pub priority: ResearchPriority,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ResearchPriority {
    Low,
    Normal, 
    High,
    Critical,
}

// === SCIENTIST SYSTEM ===
#[derive(Component, Clone, Serialize, Deserialize)]
pub struct Scientist {
    pub name: String,
    pub specialization: ResearchCategory,
    pub productivity_bonus: f32, // 1.0 = normal, 2.0 = twice as fast
    pub recruitment_cost: u32,
    pub daily_salary: u32,
    pub loyalty: f32, // 0.0-1.0, affects chance of being poached
    pub is_recruited: bool,
    pub current_project: Option<String>,
}

impl Scientist {
    pub fn new(name: String, specialization: ResearchCategory) -> Self {
        let productivity = 0.8 + rand::random::<f32>() * 0.8; // 0.8-1.6x
        let cost = match specialization {
            ResearchCategory::Weapons => 2000 + (productivity * 1000.0) as u32,
            ResearchCategory::Cybernetics => 2500 + (productivity * 1200.0) as u32,
            ResearchCategory::Equipment => 1500 + (productivity * 800.0) as u32,
            ResearchCategory::Intelligence => 3000 + (productivity * 1500.0) as u32,
        };
        
        Self {
            name,
            specialization,
            productivity_bonus: productivity,
            recruitment_cost: cost,
            daily_salary: cost / 20, // ~20 days to pay for recruitment
            loyalty: 0.5 + rand::random::<f32>() * 0.3, // 0.5-0.8
            is_recruited: false,
            current_project: None,
        }
    }
}

// === ENHANCED PROJECT DEFINITION ===
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchProject {
    pub id: String,
    pub name: String,
    pub description: String,
    pub cost: u32,
    pub base_time_days: f32, // Base research time in game days
    pub category: ResearchCategory,
    pub prerequisites: Vec<String>,
    pub benefits: Vec<ResearchBenefit>,
    pub prototype_available: bool, // Can use immediately but expensive/risky
    pub difficulty: ResearchDifficulty,
    pub stolen_data_value: f32, // How much stolen data helps (0.0-1.0)
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ResearchDifficulty {
    Basic,    // No scientist needed, 1x time
    Advanced, // Scientist recommended, 1.5x time without
    Expert,   // Scientist required, 2x time without
    Cutting,  // Top scientist required, fails without
}

// === CORPORATE ESPIONAGE ===
#[derive(Component)]
pub struct ResearchFacility {
    pub owning_faction: Faction,
    pub security_level: u32, // 1-5, affects hack difficulty
    pub available_data: Vec<String>, // project IDs that can be stolen
    pub data_quality: f32, // 0.0-1.0, how much progress stolen data provides
}

#[derive(Component)]
pub struct ScientistNPC {
    pub scientist_data: Scientist,
    pub recruitment_difficulty: u32, // Credits + persuasion needed
    pub location_discovered: bool,
}

// === RESEARCH DATABASE ===
#[derive(Resource, Default, Clone, Serialize, Deserialize)]
pub struct ResearchDatabase {
    pub projects: Vec<ResearchProject>,
}

impl ResearchDatabase {
    pub fn load() -> Self {
        std::fs::read_to_string("data/research.json")
            .map_err(|e| error!("Failed to load research.json: {}", e))
            .and_then(|content| {
                serde_json::from_str::<Self>(&content)
                    .map_err(|e| error!("Failed to parse research.json: {}", e))
            })
            .unwrap_or_default()
    }
    
    pub fn get_project(&self, id: &str) -> Option<&ResearchProject> {
        self.projects.iter().find(|p| p.id == id)
    }
    
    pub fn get_available_projects(&self, progress: &ResearchProgress) -> Vec<&ResearchProject> {
        self.projects.iter()
            .filter(|p| self.is_project_available(p, progress))
            .collect()
    }
    
    fn is_project_available(&self, project: &ResearchProject, progress: &ResearchProgress) -> bool {
        !progress.completed.contains(&project.id) &&
        !progress.active_queue.iter().any(|a| a.project_id == project.id) &&
        project.prerequisites.iter().all(|req| progress.completed.contains(req))
    }
}

// === QUEUE MANAGEMENT ===
impl ResearchProgress {
    pub fn add_to_queue(&mut self, project_id: String, project: &ResearchProject, scientist: Option<Entity>) -> bool {
        if self.active_queue.len() >= 5 { return false; } // Max 5 concurrent projects
        
        let mut time_needed = project.base_time_days;
        
        // Apply difficulty modifier if no scientist
        if scientist.is_none() {
            time_needed *= match project.difficulty {
                ResearchDifficulty::Basic => 1.0,
                ResearchDifficulty::Advanced => 1.5,
                ResearchDifficulty::Expert => 2.0,
                ResearchDifficulty::Cutting => return false, // Can't start without scientist
            };
        }
        
        // Apply stolen data bonus
        if let Some(&stolen_progress) = self.stolen_data.get(&project_id) {
            time_needed *= 1.0 - (stolen_progress * 0.6); // Up to 60% time reduction
        }
        
        self.active_queue.push(ActiveResearch {
            project_id: project_id.clone(),
            assigned_scientist: scientist,
            progress: 0.0,
            completion_time: time_needed,
            time_remaining: time_needed,
            priority: ResearchPriority::Normal,
        });
        
        true
    }
    
    pub fn remove_from_queue(&mut self, project_id: &str) {
        self.active_queue.retain(|a| a.project_id != project_id);
    }
    
    pub fn get_active_research(&mut self, project_id: &str) -> Option<&mut ActiveResearch> {
        self.active_queue.iter_mut().find(|a| a.project_id == project_id)
    }
}

// === MAIN RESEARCH SYSTEMS ===

// Daily research progress system
pub fn research_progress_system(
    mut progress: ResMut<ResearchProgress>,
    research_db: Res<ResearchDatabase>,
    scientist_query: Query<(Entity, &Scientist)>,
    mut unlocked_attachments: ResMut<UnlockedAttachments>,
    mut global_data: ResMut<GlobalData>,
    time: Res<Time>,
    mut last_time: Local<f32>,
) {
    // Process research every game day (24 game hours)
    // PLACEHOLDER
    *last_time += time.delta_secs();
    if *last_time < 86400.0 { // 24 hours * 60 minutes - one minute = hour in game
        return;
    }
    *last_time = 0.0;
    
    let mut completed_projects = Vec::new();
    
    for active in &mut progress.active_queue {
        // Calculate daily progress
        let mut daily_progress = 1.0; // Base 1 day of progress
        
        // Apply scientist bonus
        if let Some(scientist_entity) = active.assigned_scientist {
            if let Ok((_, scientist)) = scientist_query.get(scientist_entity) {
                daily_progress *= scientist.productivity_bonus;
                
                // Loyalty bonus - loyal scientists work harder
                daily_progress *= 1.0 + (scientist.loyalty - 0.5) * 0.2;
            }
        }
        
        // Apply priority modifier
        daily_progress *= match active.priority {
            ResearchPriority::Low => 0.7,
            ResearchPriority::Normal => 1.0,
            ResearchPriority::High => 1.3,
            ResearchPriority::Critical => 1.6,
        };
        
        // Update progress
        active.time_remaining -= daily_progress;
        active.progress = 1.0 - (active.time_remaining / active.completion_time).max(0.0);
        
        // Check completion
        if active.time_remaining <= 0.0 {
            completed_projects.push(active.project_id.clone());
        }
    }
    
    // Complete finished projects
    for project_id in completed_projects {
        complete_research(&project_id, &mut progress, &research_db, &mut unlocked_attachments, &mut global_data);
    }
    
    // Pay scientist salaries
    pay_scientist_salaries(&mut global_data, &scientist_query);
}

fn complete_research(
    project_id: &str,
    progress: &mut ResearchProgress,
    research_db: &ResearchDatabase,
    unlocked_attachments: &mut UnlockedAttachments,
    global_data: &mut GlobalData,
) {
    if let Some(project) = research_db.get_project(project_id) {
        // Mark as completed
        progress.completed.insert(project_id.to_string());
        progress.remove_from_queue(project_id);
        
        // Apply benefits
        apply_research_benefits(progress, research_db, unlocked_attachments, global_data);
        
        // Remove prototype status if it was there
        progress.prototypes.remove(project_id);
        
        info!("Research completed: {}", project.name);
    }
}

fn pay_scientist_salaries(global_data: &mut GlobalData, scientist_query: &Query<(Entity, &Scientist)>) {
    let mut daily_cost = 0;
    
    for (_, scientist) in scientist_query.iter() {
        if scientist.is_recruited {
            daily_cost += scientist.daily_salary;
        }
    }
    
    if daily_cost > 0 {
        global_data.credits = global_data.credits.saturating_sub(daily_cost);
        info!("Paid scientist salaries: ${}", daily_cost);
    }
}

// === SCIENTIST MANAGEMENT ===
pub fn scientist_recruitment_system(
    commands: Commands,
    mut scientist_query: Query<(Entity, &mut Scientist, &mut ScientistNPC)>,
    mut action_events: EventReader<ActionEvent>,
    mut global_data: ResMut<GlobalData>,
) {
    for event in action_events.read() {
        if let Action::RecruitScientist(scientist_entity) = event.action {
            if let Ok((entity, mut scientist, mut npc)) = scientist_query.get_mut(scientist_entity) {
                if !scientist.is_recruited && global_data.credits >= scientist.recruitment_cost {
                    global_data.credits -= scientist.recruitment_cost;
                    scientist.is_recruited = true;
                    npc.recruitment_difficulty = 0; // Already recruited
                    
                    info!("Recruited scientist: {:?} ({:?})", scientist.name, scientist.specialization);
                }
            }
        }
    }
}

// === ESPIONAGE SYSTEM ===
pub fn research_espionage_system(
    mut progress: ResMut<ResearchProgress>,
    mut hack_events: EventReader<HackCompletedEvent>,
    facility_query: Query<&ResearchFacility>,
) {
    for event in hack_events.read() {
        if let Ok(facility) = facility_query.get(event.target) {
            // Successful hack of research facility
            for project_id in &facility.available_data {
                let stolen_amount = facility.data_quality * (0.3 + rand::random::<f32>() * 0.4); // 30-70%
                
                let current = progress.stolen_data.get(project_id).unwrap_or(&0.0);
                let new_amount = (current + stolen_amount).min(1.0);
                
                progress.stolen_data.insert(project_id.clone(), new_amount);
                
                info!("Stole research data for {}: {:.1}% total", project_id, new_amount * 100.0);
            }
        }
    }
}

// === PROTOTYPE SYSTEM ===
pub fn prototype_access_system(
    commands: Commands,
    mut action_events: EventReader<ActionEvent>,
    mut progress: ResMut<ResearchProgress>,
    research_db: Res<ResearchDatabase>,
    mut global_data: ResMut<GlobalData>,
    mut unlocked_attachments: ResMut<UnlockedAttachments>,
) {
    for event in action_events.read() {
        if let Action::UsePrototype(project_id) = &event.action {
            if let Some(project) = research_db.get_project(project_id) {
                if project.prototype_available && !progress.completed.contains(project_id) {
                    // Can use prototype for 3x cost and some risk
                    let prototype_cost = project.cost * 3;
                    
                    if global_data.credits >= prototype_cost {
                        global_data.credits -= prototype_cost;
                        progress.prototypes.insert(project_id.clone());
                        
                        // Apply benefits immediately but mark as prototype
                        apply_prototype_benefits(project, &mut unlocked_attachments);
                        
                        info!("Acquired prototype: {} for ${}", project.name, prototype_cost);
                    }
                }
            }
        }
    }
}

fn apply_prototype_benefits(project: &ResearchProject, unlocked_attachments: &mut UnlockedAttachments) {
    for benefit in &project.benefits {
        match benefit {
            ResearchBenefit::UnlockAttachment(attachment_id) => {
                unlocked_attachments.attachments.insert(format!("{}_prototype", attachment_id));
            },
            // Other benefits applied at reduced effectiveness or with limitations
            _ => {} // Handle other prototype benefits as needed
        }
    }
}

// === UTILITY FUNCTIONS ===
pub fn start_research_project(
    project_id: &str,
    scientist_entity: Option<Entity>,
    global_data: &mut GlobalData,
    progress: &mut ResearchProgress,
    research_db: &ResearchDatabase,
) -> bool {
    let Some(project) = research_db.get_project(project_id) else {
        return false;
    };
    
    if global_data.credits < project.cost {
        return false;
    }
    
    if !progress.add_to_queue(project_id.to_string(), project, scientist_entity) {
        return false;
    }
    
    global_data.credits -= project.cost;
    info!("Started research: {} (${}, {:.1} days)", project.name, project.cost, project.base_time_days);
    true
}

pub fn assign_scientist_to_project(
    scientist_entity: Entity,
    project_id: &str,
    progress: &mut ResearchProgress,
    mut scientist_query: Query<&mut Scientist>,
) -> bool {
    if let Ok(mut scientist) = scientist_query.get_mut(scientist_entity) {
        if scientist.is_recruited {
            // Remove from previous project
            if let Some(old_project) = &scientist.current_project {
                if let Some(active) = progress.get_active_research(old_project) {
                    active.assigned_scientist = None;
                }
            }
            
            // Assign to new project
            if let Some(active) = progress.get_active_research(project_id) {
                active.assigned_scientist = Some(scientist_entity);
                scientist.current_project = Some(project_id.to_string());
                return true;
            }
        }
    }
    false
}

// === SPAWN FUNCTIONS ===
pub fn spawn_scientist_npc(
    commands: &mut Commands,
    position: Vec2,
    specialization: ResearchCategory,
    sprites: &GameSprites,
) {
    let scientist_names = [
        "Dr. Chen", "Dr. Volkov", "Dr. Martinez", "Dr. Kim", "Dr. Hassan",
        "Prof. Anderson", "Dr. Okafor", "Dr. Singh", "Dr. Mueller", "Dr. Tanaka"
    ];
    
    let name = scientist_names[rand::random::<usize>() % scientist_names.len()].to_string();
    let scientist = Scientist::new(name, specialization);
    
    let (sprite, _) = crate::core::sprites::create_civilian_sprite(sprites);
    
    commands.spawn((
        sprite,
        Transform::from_translation(position.extend(1.0)),
        Scientist::clone(&scientist),
        ScientistNPC {
            scientist_data: scientist,
            recruitment_difficulty: 3, // Requires multiple interactions
            location_discovered: false,
        },
        Civilian,
        Faction::Civilian,
        Health(80.0),
        MovementSpeed(60.0),
        NeurovectorTarget, // Can be mind-controlled
        Scannable,
        bevy_rapier2d::prelude::RigidBody::Dynamic,
        bevy_rapier2d::prelude::Collider::ball(8.0),
        bevy_rapier2d::prelude::Velocity::default(),
        bevy_rapier2d::prelude::GravityScale(0.0),
    ));
}

pub fn spawn_research_facility(
    commands: &mut Commands,
    position: Vec2,
    faction: Faction,
    security_level: u32,
    available_projects: Vec<String>,
) {
    commands.spawn((
        Sprite {
            color: Color::srgb(0.3, 0.4, 0.6),
            custom_size: Some(Vec2::new(80.0, 60.0)),
            ..default()
        },
        Transform::from_translation(position.extend(1.0)),
        ResearchFacility {
            owning_faction: faction,
            security_level,
            available_data: available_projects,
            data_quality: 0.6 + (security_level as f32 * 0.08), // Higher security = better data
        },
        bevy_rapier2d::prelude::RigidBody::Fixed,
        bevy_rapier2d::prelude::Collider::cuboid(40.0, 30.0),
        Hackable {
            device_type: DeviceType::Terminal,
            hack_time: 5.0 + security_level as f32 * 2.0, // Harder facilities take longer
            security_level: security_level as u8,
            is_hacked: false,
            disabled_duration: 0.0,
            hack_effects: Vec::new(),
            network_id: None,
            requires_tool: None,
        },
        Scannable,
    ));
}

// === COMPATIBILITY FUNCTIONS ===
// Keep existing functions for backward compatibility
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
                    _ => {} // Other benefits handled elsewhere
                }
            }
        }
    }
}

pub fn apply_research_unlocks(
    progress: &ResearchProgress,
    research_db: &ResearchDatabase,
    unlocked_attachments: &mut UnlockedAttachments,
) {
    apply_research_benefits(progress, research_db, unlocked_attachments, &mut GlobalData::default());
}

pub fn purchase_research(
    project_id: &str,
    global_data: &mut GlobalData,
    progress: &mut ResearchProgress,
    research_db: &ResearchDatabase,
    unlocked_attachments: &mut UnlockedAttachments,
) -> bool {
    // For backward compatibility - instant completion
    let Some(project) = research_db.get_project(project_id) else {
        return false;
    };
    
    if global_data.credits < project.cost || progress.completed.contains(project_id) {
        return false;
    }
    
    global_data.credits -= project.cost;
    progress.completed.insert(project_id.to_string());
    
    apply_research_benefits(progress, research_db, unlocked_attachments, global_data);
    
    info!("Instantly completed research: {}", project.name);
    true
}

// Calculate bonus credits from research
pub fn calculate_research_credit_bonus(
    progress: &ResearchProgress,
    research_db: &ResearchDatabase,
) -> u32 {
    let mut total_bonus = 0;
    
    process_research_benefits(progress, research_db, |benefit| {
        if let ResearchBenefit::CreditsPerMission(amount) = benefit {
            total_bonus += amount;
        }
    });
    
    total_bonus
}

// Calculate experience bonus from research
pub fn calculate_research_xp_bonus(
    progress: &ResearchProgress,
    research_db: &ResearchDatabase,
    base_xp: u32,
) -> u32 {
    let mut bonus_percentage = 0;
    
    process_research_benefits(progress, research_db, |benefit| {
        if let ResearchBenefit::ExperienceBonus(percentage) = benefit {
            bonus_percentage += percentage;
        }
    });
    
    if bonus_percentage > 0 {
        base_xp + (base_xp * bonus_percentage / 100)
    } else {
        base_xp
    }
}

fn process_research_benefits<F>(
    progress: &ResearchProgress,
    research_db: &ResearchDatabase,
    mut processor: F,
) where
    F: FnMut(&ResearchBenefit),
{
    for project_id in &progress.completed {
        if let Some(project) = research_db.get_project(project_id) {
            project.benefits.iter().for_each(&mut processor);
        }
    }
}