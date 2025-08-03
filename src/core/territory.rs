// src/core/territory.rs - Neo-Singapore District Control and Liberation System
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use crate::core::*;
use std::collections::HashMap;

//OLD 
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChapterTheme {
    Tutorial,
    Corporate,
    Underground,
    Surveillance,
    Revolution,
    Conspiracy,
    Technology,
    Environment,
    Liberation,
    Final,
}

// === DISTRICT CONTROL ===
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistrictControl {
    pub district_id: String,
    pub control_level: ControlLevel,
    pub control_strength: f32,         // 0.0 to 1.0
    pub liberation_status: LiberationStatus,
    pub population_support: f32,       // 0.0 to 1.0 - civilian support level
    pub corporate_presence: f32,       // 0.0 to 1.0 - remaining corporate control
    pub surveillance_level: f32,       // 0.0 to 1.0 - active surveillance
    pub economic_activity: f32,        // 0.0 to 1.0 - district economic health
    pub days_controlled: u32,
    pub total_credits_generated: u32,
    pub resistance_cells: u32,         // Number of active resistance cells
    pub corporate_responses: Vec<CorporateResponse>, // Recent corporate countermeasures
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum ControlLevel {
    Corporate,      // Full corporate control
    Contested,      // Active fighting for control
    Liberated,      // Player controlled but unstable
    Secured,        // Stable player control
    Autonomous,     // Self-governing, minimal surveillance
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum LiberationStatus {
    Oppressed,      // Heavy corporate surveillance/control
    Awakening,      // Population becoming aware
    Resisting,      // Active resistance movements
    Fighting,       // Open conflict
    Liberated,      // Corporate forces expelled
    Thriving,       // Autonomous community established
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorporateResponse {
    pub corporation: Corporation,
    pub response_type: ResponseType,
    pub severity: u8,              // 1-5
    pub day_activated: u32,
    pub duration_days: u32,
    pub affected_districts: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResponseType {
    IncreasedSurveillance,    // More cameras, patrols
    EconomicSanctions,        // Reduced trade, services
    SecurityCrackdown,        // Heavy police/military presence
    PropagandaCampaign,       // Counter-narrative in media
    Sabotage,                 // Corporate agents cause disruption
    Evacuation,               // Corporate personnel/assets withdrawn
    CounterIntelligence,      // Hunt for resistance cells
}

impl ControlLevel {
    pub fn income_multiplier(&self) -> f32 {
        match self {
            ControlLevel::Corporate => 0.0,     // No income from corporate districts
            ControlLevel::Contested => 0.2,     // Minimal income during fighting
            ControlLevel::Liberated => 0.6,     // Moderate income, some instability
            ControlLevel::Secured => 1.0,       // Full income potential
            ControlLevel::Autonomous => 1.2,    // Bonus from autonomous organization
        }
    }

    pub fn max_resistance_cells(&self) -> u32 {
        match self {
            ControlLevel::Corporate => 0,
            ControlLevel::Contested => 2,
            ControlLevel::Liberated => 5,
            ControlLevel::Secured => 8,
            ControlLevel::Autonomous => 12,
        }
    }
}

// === TERRITORY MANAGER RESOURCE ===
#[derive(Clone, Debug, Default, Resource, Serialize, Deserialize)]
pub struct TerritoryManager {
    pub controlled_districts: HashMap<String, DistrictControl>,
    pub total_daily_income: u32,
    pub liberated_population: u32,
    pub total_population: u32,
    pub corporate_alert_level: u8,      // 1-5, affects AI response intensity
    pub global_liberation_progress: f32, // 0.0 to 1.0
}

impl TerritoryManager {
    pub fn establish_control(&mut self, district_id: String, district_data: &SingaporeDistrict, current_day: u32) {
        let control = DistrictControl {
            district_id: district_id.clone(),
            control_level: ControlLevel::Contested,
            control_strength: 0.3, // Start with weak control
            liberation_status: LiberationStatus::Fighting,
            population_support: 0.2, // Low initial support
            corporate_presence: 0.8, // High corporate presence initially
            surveillance_level: 0.9, // Heavy surveillance
            economic_activity: 0.4,  // Disrupted by conflict
            days_controlled: 0,
            total_credits_generated: 0,
            resistance_cells: 1,     // Start with one cell
            corporate_responses: vec![],
        };

        self.controlled_districts.insert(district_id.clone(), control);
        self.update_global_metrics();
        
        info!("Established control in district: {}", district_id);
    }

    pub fn collect_daily_income(&mut self, districts_db: &HashMap<String, SingaporeDistrict>, current_day: u32) -> u32 {

        info!("collect_daily_income");
        let mut total_collected = 0;

        let mut_self = &mut self.clone();
        for (district_id, control) in self.controlled_districts.iter_mut() {
            if let Some(district) = districts_db.get(district_id) {
                // Base income from population and economic activity
                let base_income = (district.population as f32 * 0.1 * control.economic_activity) as u32;
                
                // Apply control level multiplier
                let control_multiplier = control.control_level.income_multiplier();
                
                // Apply population support modifier
                let support_modifier = 0.5 + (control.population_support * 0.5);
                
                // Calculate final income
                let daily_income = (base_income as f32 * control_multiplier * support_modifier) as u32;
                
                control.total_credits_generated += daily_income;
                total_collected += daily_income;

                // Update economic activity based on control stability
                mut_self.update_economic_activity(control);
            }
        }

        self.total_daily_income = total_collected;
        total_collected
    }

    fn update_economic_activity(&self, control: &mut DistrictControl) {
        match control.control_level {
            ControlLevel::Corporate => {
                // Corporate districts maintain economic activity
                control.economic_activity = (control.economic_activity + 0.02).min(1.0);
            },
            ControlLevel::Contested => {
                // Economic activity suffers during conflict
                control.economic_activity = (control.economic_activity - 0.05).max(0.2);
            },
            ControlLevel::Liberated => {
                // Gradual economic recovery
                control.economic_activity = (control.economic_activity + 0.01).min(0.8);
            },
            ControlLevel::Secured => {
                // Strong economic recovery
                control.economic_activity = (control.economic_activity + 0.03).min(1.0);
            },
            ControlLevel::Autonomous => {
                // Economic bonus from self-organization
                control.economic_activity = (control.economic_activity + 0.05).min(1.2);
            },
        }
    }

    pub fn update_districts(&mut self, current_day: u32) {

        info!("update_districts");

        let mut_self = &mut self.clone();
        for control in self.controlled_districts.values_mut() {
            control.days_controlled += 1;

            // Natural progression of liberation
            mut_self.update_liberation_progress(control);
            
            // Update surveillance levels
            mut_self.update_surveillance(control);
            
            // Corporate presence decay in liberated areas
            if control.control_level != ControlLevel::Corporate {
                control.corporate_presence *= 0.98; // Gradual reduction
            }

            // Population support grows with successful control
            if control.control_level == ControlLevel::Secured || control.control_level == ControlLevel::Autonomous {
                control.population_support = (control.population_support + 0.01).min(1.0);
            }

            // Process corporate responses
            mut_self.process_corporate_responses(control, current_day);
        }

        self.update_global_metrics();
        self.update_corporate_alert_level();
    }

    fn update_liberation_progress(&self, control: &mut DistrictControl) {
        // Liberation status progression based on control metrics
        let liberation_score = control.population_support + 
                             (1.0 - control.corporate_presence) + 
                             (1.0 - control.surveillance_level);

        control.liberation_status = match liberation_score {
            0.0..=0.5 => LiberationStatus::Oppressed,
            0.5..=1.0 => LiberationStatus::Awakening,
            1.0..=1.5 => LiberationStatus::Resisting,
            1.5..=2.0 => LiberationStatus::Fighting,
            2.0..=2.5 => LiberationStatus::Liberated,
            2.5..=3.0 => LiberationStatus::Thriving,
            _ => LiberationStatus::Thriving,
        };

        // Update control level based on liberation status
        control.control_level = match control.liberation_status {
            LiberationStatus::Oppressed => ControlLevel::Corporate,
            LiberationStatus::Awakening | LiberationStatus::Resisting => ControlLevel::Contested,
            LiberationStatus::Fighting | LiberationStatus::Liberated => ControlLevel::Liberated,
            LiberationStatus::Thriving => {
                if control.population_support > 0.8 {
                    ControlLevel::Autonomous
                } else {
                    ControlLevel::Secured
                }
            },
        };
    }

    fn update_surveillance(&self, control: &mut DistrictControl) {
        // Surveillance decreases as corporate presence weakens
        let target_surveillance = control.corporate_presence * 0.8;
        
        if control.surveillance_level > target_surveillance {
            control.surveillance_level = (control.surveillance_level - 0.05).max(target_surveillance);
        }
    }

    fn process_corporate_responses(&self, control: &mut DistrictControl, current_day: u32) {
        // Remove expired responses
        control.corporate_responses.retain(|response| {
            current_day < response.day_activated + response.duration_days
        });

        // Apply active response effects
        for response in &control.corporate_responses {
            match response.response_type {
                ResponseType::IncreasedSurveillance => {
                    control.surveillance_level = (control.surveillance_level + 0.1).min(1.0);
                },
                ResponseType::EconomicSanctions => {
                    control.economic_activity = (control.economic_activity - 0.1).max(0.1);
                },
                ResponseType::SecurityCrackdown => {
                    control.population_support = (control.population_support - 0.05).max(0.0);
                    control.surveillance_level = (control.surveillance_level + 0.15).min(1.0);
                },
                ResponseType::PropagandaCampaign => {
                    control.population_support = (control.population_support - 0.03).max(0.0);
                },
                ResponseType::Sabotage => {
                    control.economic_activity = (control.economic_activity - 0.15).max(0.0);
                },
                ResponseType::CounterIntelligence => {
                    // Reduce resistance cells
                    if control.resistance_cells > 0 {
                        control.resistance_cells = (control.resistance_cells - 1).max(1);
                    }
                },
                _ => {},
            }
        }
    }

    fn update_global_metrics(&mut self) {
        // Calculate total liberated population
        self.liberated_population = self.controlled_districts.values()
            .filter(|control| control.control_level != ControlLevel::Corporate)
            .map(|control| (control.population_support * 100000.0) as u32) // Estimate population per district
            .sum();

        // Update global liberation progress
        if self.total_population > 0 {
            self.global_liberation_progress = self.liberated_population as f32 / self.total_population as f32;
        }
    }

    fn update_corporate_alert_level(&mut self) {
        let liberated_districts = self.controlled_districts.values()
            .filter(|control| control.control_level != ControlLevel::Corporate)
            .count();

        self.corporate_alert_level = match liberated_districts {
            0..=2 => 1,
            3..=5 => 2,
            6..=10 => 3,
            11..=15 => 4,
            _ => 5,
        };
    }

    pub fn trigger_corporate_response(&mut self, corporation: Corporation, response_type: ResponseType, 
                                    target_districts: Vec<String>, current_day: u32) {
        let severity = self.corporate_alert_level;
        let duration = match response_type {
            ResponseType::IncreasedSurveillance => 14,
            ResponseType::EconomicSanctions => 7,
            ResponseType::SecurityCrackdown => 10,
            ResponseType::PropagandaCampaign => 21,
            ResponseType::Sabotage => 3,
            ResponseType::Evacuation => 1,
            ResponseType::CounterIntelligence => 30,
        };

        let response = CorporateResponse {
            corporation,
            response_type,
            severity,
            day_activated: current_day,
            duration_days: duration,
            affected_districts: target_districts.clone(),
        };

        // Apply response to affected districts
        for district_id in target_districts {
            if let Some(control) = self.controlled_districts.get_mut(&district_id) {
                control.corporate_responses.push(response.clone());
            }
        }

        info!("Corporate response triggered");
    }

    pub fn get_district(&self, district_id: &str) -> Option<&DistrictControl> {
        self.controlled_districts.get(district_id)
    }

    pub fn is_liberated(&self, district_id: &str) -> bool {
        if let Some(control) = self.controlled_districts.get(district_id) {
            matches!(control.control_level, ControlLevel::Liberated | ControlLevel::Secured | ControlLevel::Autonomous)
        } else {
            false
        }
    }

    pub fn get_liberation_score(&self) -> f32 {
        self.global_liberation_progress
    }
}

// === CAMPAIGN PROGRESSION ===
#[derive(Clone, Debug, Default, Resource, Serialize, Deserialize)]
pub struct CampaignProgressionTracker {
    pub campaign_progress: NeoSingaporeCampaignProgress,
    pub victory_conditions: NeoSingaporeVictory,
    pub operation_completion: HashMap<String, bool>,
    pub act_completion: HashMap<u8, bool>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NeoSingaporeCampaignProgress {
    pub current_operation: usize,
    pub total_operations: usize,
    pub current_act: u8,
    pub districts_liberated: usize,
    pub total_credits_generated: u32,
    pub days_elapsed: u32,
    pub population_freed: u32,
}

impl Default for NeoSingaporeCampaignProgress {
    fn default() -> Self {
        Self {
            current_operation: 1,
            total_operations: 32,
            current_act: 1,
            districts_liberated: 0,
            total_credits_generated: 0,
            days_elapsed: 0,
            population_freed: 0,
        }
    }
}

impl CampaignProgressionTracker {
    pub fn check_victory_conditions(&mut self, territory_manager: &TerritoryManager) -> bool {
        let liberation_score = territory_manager.get_liberation_score();
        let districts_controlled = territory_manager.controlled_districts.len();
        
        // Check if all major corporate HQs are neutralized
        let hqs_destroyed = self.victory_conditions.corporate_hq_destroyed.len() == 4; // All 4 corporations
        
        // Check if minimum district control is achieved
        let district_threshold_met = districts_controlled >= self.victory_conditions.min_district_control;
        
        // Check if population liberation threshold is met
        let population_threshold_met = liberation_score >= self.victory_conditions.population_liberation;

        let victory_achieved = hqs_destroyed && district_threshold_met && population_threshold_met;

        if victory_achieved && !self.victory_conditions.all_conditions_met() {
            info!("NEO-SINGAPORE LIBERATED! Victory conditions achieved!");
            info!("Liberation Score: {:.1}%", liberation_score * 100.0);
            info!("Districts Controlled: {}", districts_controlled);
            info!("Population Freed: {}", territory_manager.liberated_population);
        }

        victory_achieved
    }

    pub fn complete_operation(&mut self, operation_id: usize, district_id: String) {
        self.operation_completion.insert(district_id.clone(), true);
        self.campaign_progress.current_operation = operation_id + 1;
        self.campaign_progress.districts_liberated += 1;
        
        info!("Operation {} completed in district {}", operation_id, district_id);
        
        // Check for act completion
        self.check_act_completion(operation_id);
    }

    fn check_act_completion(&mut self, operation_id: usize) {
        let act = match operation_id {
            1..=8 => 1,
            9..=16 => 2,
            17..=24 => 3,
            25..=32 => 4,
            _ => return,
        };

        if !self.act_completion.contains_key(&act) {
            let act_operations_complete = match act {
                1 => (1..=8).all(|id| self.operation_completion.values().filter(|&&completed| completed).count() >= id),
                2 => (9..=16).all(|id| self.operation_completion.values().filter(|&&completed| completed).count() >= id),
                3 => (17..=24).all(|id| self.operation_completion.values().filter(|&&completed| completed).count() >= id),
                4 => (25..=32).all(|id| self.operation_completion.values().filter(|&&completed| completed).count() >= id),
                _ => false,
            };

            if act_operations_complete {
                self.act_completion.insert(act, true);
                self.campaign_progress.current_act = act + 1;
                info!("🎬 Act {} completed! Advancing to Act {}", act, act + 1);
            }
        }
    }
}

impl NeoSingaporeVictory {
    pub fn all_conditions_met(&self) -> bool {
        self.corporate_hq_destroyed.len() == 4 &&
        self.population_liberation >= 0.8 &&
        self.min_district_control >= 20
    }
}

// === SYSTEM FUNCTIONS ===
pub fn territory_daily_update_system(
    mut territory_manager: ResMut<TerritoryManager>,
    mut global_data: ResMut<GlobalData>,
    mut progression_tracker: ResMut<CampaignProgressionTracker>,
    campaign_db: ResMut<NeoSingaporeCampaignDatabase>,
    mut day_changed: Local<u32>,
) {
    if global_data.current_day == *day_changed {
        return;
    }

    info!("territory_daily_update_system");
    *day_changed = global_data.current_day;

    // Update district control and liberation progress
    territory_manager.update_districts(global_data.current_day);

    // Collect daily income from liberated districts
    let daily_income = territory_manager.collect_daily_income(&campaign_db.districts, global_data.current_day);
    if daily_income > 0 {
        global_data.credits += daily_income;
        info!("Daily income from liberated districts: {} credits", daily_income);
    }

    // Update campaign progress
    progression_tracker.campaign_progress.days_elapsed = global_data.current_day;
    progression_tracker.campaign_progress.total_credits_generated += daily_income;
    progression_tracker.campaign_progress.population_freed = territory_manager.liberated_population;

    // Check victory conditions
    if progression_tracker.check_victory_conditions(&territory_manager) {
        // Handle campaign victory
        // TODO: Trigger victory cutscene/ending
    }

    // Trigger random corporate responses based on alert level
    if fastrand::f32() < (territory_manager.corporate_alert_level as f32 * 0.02) {
        trigger_random_corporate_response(&mut territory_manager, global_data.current_day);
    }

}


fn trigger_random_corporate_response(territory_manager: &mut TerritoryManager, current_day: u32) {
    let corporations = [Corporation::Nexus, Corporation::Omnicorp, Corporation::Helix, Corporation::Aegis];
    let response_types = [
        ResponseType::IncreasedSurveillance,
        ResponseType::EconomicSanctions,
        ResponseType::SecurityCrackdown,
        ResponseType::PropagandaCampaign,
        ResponseType::CounterIntelligence,
    ];

    let random_corporation = fastrand::usize(0..corporations.len());
    let corp = &corporations[random_corporation];
    let random_response = fastrand::usize(0..response_types.len());
    let response = &response_types[random_response];
    
    // Target random liberated districts
    let liberated_districts: Vec<String> = territory_manager.controlled_districts
        .iter()
        .filter(|(_, control)| control.control_level != ControlLevel::Corporate)
        .map(|(id, _)| id.clone())
        .collect();

    if !liberated_districts.is_empty() {
        let target_count = (liberated_districts.len() / 3).max(1);
        let mut targets = Vec::new();
        
        for _ in 0..target_count {
            if let Some(district) = liberated_districts.get(fastrand::usize(0..liberated_districts.len())) {
                if !targets.contains(district) {  // Avoid duplicates
                    targets.push(district.clone());
                }
            }
        }

        if !targets.is_empty() {
            territory_manager.trigger_corporate_response(corp.clone(), response.clone(), targets, current_day);
        }
    }
}