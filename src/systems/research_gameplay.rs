// src/systems/research_gameplay.rs - Game systems for scientist interactions and research espionage
use bevy::prelude::*;
use crate::core::*;

// === SCIENTIST INTERACTION SYSTEM ===
pub fn scientist_interaction_system(
    mut commands: Commands,
    mut action_events: EventReader<ActionEvent>,
    mut scientist_query: Query<(Entity, &mut Scientist, &mut ScientistNPC, &Transform)>,
    agent_query: Query<&Transform, (With<Agent>, Without<Scientist>)>,
    mut global_data: ResMut<GlobalData>,
    mut research_progress: ResMut<ResearchProgress>,
) {
    for event in action_events.read() {
        match &event.action {
            Action::InteractWithScientist(scientist_entity) => {
                if let Ok((_, mut scientist, mut npc, _)) = scientist_query.get_mut(*scientist_entity) {
                    handle_scientist_interaction(&mut scientist, &mut npc, &mut global_data);
                }
            },
            Action::RecruitScientist(scientist_entity) => {
                if let Ok((_, mut scientist, mut npc, _)) = scientist_query.get_mut(*scientist_entity) {
                    attempt_scientist_recruitment(&mut scientist, &mut npc, &mut global_data);
                }
            },
            Action::KidnapScientist(scientist_entity) => {
                if let Ok((_, mut scientist, mut npc, _)) = scientist_query.get_mut(*scientist_entity) {
                    force_recruit_scientist(&mut scientist, &mut npc, &mut global_data);
                }
            },
            _ => {}
        }
    }
}

fn handle_scientist_interaction(
    scientist: &mut Scientist,
    npc: &mut ScientistNPC,
    global_data: &mut GlobalData,
) {
    if !npc.location_discovered {
        npc.location_discovered = true;
        info!("Discovered scientist: {:?} ({:?})", scientist.name, scientist.specialization);
        return;
    }

    if scientist.is_recruited {
        info!("{} is already working for you", scientist.name);
        return;
    }

    // Reduce recruitment difficulty through conversation
    if npc.recruitment_difficulty > 0 {
        npc.recruitment_difficulty -= 1;
        scientist.loyalty += 0.1; // Talking increases loyalty

        info!("Building rapport with {}... (difficulty: {})",
              scientist.name, npc.recruitment_difficulty);
    }
}

fn attempt_scientist_recruitment(
    scientist: &mut Scientist,
    npc: &mut ScientistNPC,
    global_data: &mut GlobalData,
) {
    if scientist.is_recruited {
        return;
    }

    let total_cost = scientist.recruitment_cost + (npc.recruitment_difficulty * 500);

    if global_data.credits >= total_cost && npc.recruitment_difficulty == 0 {
        global_data.credits -= total_cost;
        scientist.is_recruited = true;
        scientist.loyalty = 0.8; // Start with good loyalty

        info!("Successfully recruited {} for ${}", scientist.name, total_cost);
    } else if global_data.credits < total_cost {
        info!("Not enough credits to recruit {} (need ${})", scientist.name, total_cost);
    } else {
        info!("Build more rapport with {} before recruitment", scientist.name);
    }
}

fn force_recruit_scientist(
    scientist: &mut Scientist,
    npc: &mut ScientistNPC,
    global_data: &mut GlobalData,
) {
    if !scientist.is_recruited {
        scientist.is_recruited = true;
        scientist.loyalty = 0.2; // Very low loyalty when kidnapped
        npc.recruitment_difficulty = 0;

        info!("Forcibly recruited {} through neurovector control", scientist.name);
    }
}

// === RESEARCH FACILITY ESPIONAGE ===
pub fn research_facility_interaction_system(
    mut hack_events: EventReader<HackCompletedEvent>,
    mut facility_query: Query<&mut ResearchFacility>,
    mut research_progress: ResMut<ResearchProgress>,
    mut global_data: ResMut<GlobalData>,
) {
    for event in hack_events.read() {
        if let Ok(facility) = facility_query.get_mut(event.target) {
            handle_facility_hack(&facility, &mut research_progress, &mut global_data);
        }
    }
}

fn handle_facility_hack(
    facility: &ResearchFacility,
    research_progress: &mut ResearchProgress,
    global_data: &mut GlobalData,
) {
    info!("Successfully hacked {:?} research facility (Level {})",
          facility.owning_faction, facility.security_level);

    // Steal research data
    for project_id in &facility.available_data {
        let stolen_amount = facility.data_quality * (0.2 + rand::random::<f32>() * 0.3);
        let current = research_progress.stolen_data.get(project_id).unwrap_or(&0.0);
        let new_total = (current + stolen_amount).min(1.0);

        research_progress.stolen_data.insert(project_id.clone(), new_total);

        info!("Acquired research data for '{}': {:.0}% complete",
              project_id, new_total * 100.0);
    }

    // Bonus credits from industrial espionage
    let bonus_credits = facility.security_level * 200;
    global_data.credits += bonus_credits;

    info!("Industrial espionage bonus: ${}", bonus_credits);
}



// === SCIENTIST LOYALTY SYSTEM ===
pub fn scientist_loyalty_system(
    mut scientist_query: Query<&mut Scientist>,
    global_data: Res<GlobalData>,
    time: Res<Time>,
    mut last_day: Local<u32>,
) {
    // Process loyalty changes daily
    if global_data.current_day == *last_day {
        return;
    }
    *last_day = global_data.current_day;

    for mut scientist in scientist_query.iter_mut() {
        if !scientist.is_recruited {
            continue;
        }

        // Base loyalty decay
        scientist.loyalty -= 0.01;

        // Loyalty effects based on salary payment ability
        if global_data.credits >= scientist.daily_salary {
            scientist.loyalty += 0.02; // Paid on time = loyalty boost
        } else {
            scientist.loyalty -= 0.05; // Can't pay = big loyalty hit
        }

        // Random events
        if rand::random::<f32>() < 0.1 {
            match rand::random::<f32>() {
                x if x < 0.3 => {
                    scientist.loyalty += 0.1;
                    info!("{} had a breakthrough! (+loyalty)", scientist.name);
                },
                x if x < 0.6 => {
                    scientist.loyalty -= 0.08;
                    info!("{} is feeling overworked (-loyalty)", scientist.name);
                },
                _ => {
                    scientist.productivity_bonus += 0.05;
                    info!("{} improved their skills! (+productivity)", scientist.name);
                }
            }
        }

        // Clamp loyalty
        scientist.loyalty = scientist.loyalty.clamp(0.0, 1.0);

        // Handle defection
        if scientist.loyalty < 0.2 && rand::random::<f32>() < 0.1 {
            scientist.is_recruited = false;
            scientist.current_project = None;
            info!("⚠️ {} has defected due to low loyalty!", scientist.name);
        }
    }
}

// === RESEARCH SABOTAGE SYSTEM ===
pub fn research_sabotage_system(
    mut research_progress: ResMut<ResearchProgress>,
    scientist_query: Query<&Scientist>,
    global_data: Res<GlobalData>,
    time: Res<Time>,
) {
    // Random chance of enemy sabotage based on alert level
    if rand::random::<f32>() < 0.01 { // 1% chance per frame when running

        // PLACEHOLDER
        // let sabotage_chance = global_data.alert_level as f32 / 5.0 * 0.1; // Higher alert = more sabotage
        let sabotage_chance = 5.0 * 0.1; // Higher alert = more sabotage

        if rand::random::<f32>() < sabotage_chance && !research_progress.active_queue.is_empty() {
            let target_idx = rand::random::<usize>() % research_progress.active_queue.len();
            let target = &mut research_progress.active_queue[target_idx];

            // Sabotage effects
            match rand::random::<f32>() {
                x if x < 0.4 => {
                    // Slow down research
                    target.time_remaining += 1.0;
                    info!("⚠️ Research sabotage detected! {} delayed by 1 day", target.project_id);
                },
                x if x < 0.7 => {
                    // Lose progress
                    target.progress = (target.progress - 0.2).max(0.0);
                    target.time_remaining = target.completion_time * (1.0 - target.progress);
                    info!("⚠️ Research data corrupted! {} lost 20% progress", target.project_id);
                },
                _ => {
                    // Steal research data for enemies
                    info!("⚠️ Corporate spies infiltrated research! Data leaked to competitors");
                }
            }
        }
    }
}

// === ADVANCED SCIENTIST AI ===
pub fn scientist_productivity_system(
    mut scientist_query: Query<&mut Scientist>,
    research_progress: Res<ResearchProgress>,
    time: Res<Time>,
) {
    for mut scientist in scientist_query.iter_mut() {
        if !scientist.is_recruited || scientist.current_project.is_none() {
            continue;
        }

        // Dynamic productivity based on conditions
        let base_productivity = scientist.productivity_bonus;
        let mut current_productivity = base_productivity;

        // Loyalty affects productivity
        current_productivity *= 0.5 + (scientist.loyalty * 0.5);

        // Working on preferred specialization gives bonus
        if let Some(project_id) = &scientist.current_project {
            if let Some(active) = research_progress.active_queue.iter()
                .find(|a| a.project_id == *project_id) {
                // Check if project matches scientist's specialization through research_db lookup
                // This would require access to research_db, simplified here
                current_productivity *= 1.2; // Assume 20% bonus for now
            }
        }

        // Overwork penalty - working multiple projects reduces efficiency
        let projects_assigned = research_progress.active_queue.iter()
            .filter(|a| a.assigned_scientist == Some(Entity::PLACEHOLDER)) // Would need proper entity comparison
            .count();

        if projects_assigned > 1 {
            current_productivity *= 0.8_f32.powi(projects_assigned as i32 - 1);
        }

        // Update the scientist's effective productivity
        scientist.productivity_bonus = current_productivity;
    }
}

// === RESEARCH FACILITY MANAGEMENT ===
pub fn research_facility_security_system(
    mut facility_query: Query<&mut ResearchFacility>,
    time: Res<Time>,
    global_data: Res<GlobalData>,
) {
    for mut facility in facility_query.iter_mut() {
        // Security level increases over time after successful hacks
        if rand::random::<f32>() < 0.005 { // Small chance each frame
            facility.security_level = (facility.security_level + 1).min(10);
            info!("Corporate security tightened at research facility (Level {})",
                  facility.security_level);
        }

        // Higher alert levels make facilities more secure
        // Needs to be relative to the city we're in, not global
        // PLACEHOLDER
        // if global_data.alert_level > 3 && rand::random::<f32>() < 0.01 {
            facility.security_level = (facility.security_level + 1).min(10);
        // }
    }
}

// === UTILITY FUNCTIONS ===
pub fn calculate_research_efficiency(
    research_progress: &ResearchProgress,
    scientist_query: &Query<(Entity, &Scientist)>,
) -> f32 {
    if research_progress.active_queue.is_empty() {
        return 0.0;
    }

    let mut total_efficiency = 0.0;

    for active in &research_progress.active_queue {
        let mut project_efficiency = 1.0;

        if let Some(scientist_entity) = active.assigned_scientist {
            if let Ok((_, scientist)) = scientist_query.get(scientist_entity) {
                project_efficiency = scientist.productivity_bonus * scientist.loyalty;
            }
        } else {
            project_efficiency = 0.5; // No scientist = half speed
        }

        total_efficiency += project_efficiency;
    }

    total_efficiency / research_progress.active_queue.len() as f32
}

pub fn get_daily_research_costs(scientist_query: &Query<(Entity, &Scientist)>) -> u32 {
    scientist_query.iter()
        .filter(|(_, s)| s.is_recruited)
        .map(|(_, s)| s.daily_salary)
        .sum()
}

pub fn count_scientists_by_category(
    scientist_query: &Query<(Entity, &Scientist)>,
    category: ResearchCategory,
) -> usize {
    scientist_query.iter()
        .filter(|(_, s)| s.is_recruited && s.specialization == category)
        .count()
}

