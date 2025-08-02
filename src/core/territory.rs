// src/core/territory.rs - Territory Control and Tax Collection System
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use crate::core::*;
use std::collections::HashMap;

// === TERRITORY CONTROL ===
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerritoryControl {
    pub city_id: String,
    pub control_level: ControlLevel,
    pub control_strength: f32,      // 0.0 to 1.0
    pub tax_rate: f32,              // 0.0 to 0.5 (50% max)
    pub last_tax_collection: u32,   // Day
    pub resistance_level: f32,      // Increases with high taxes
    pub days_controlled: u32,
    pub total_tax_collected: u32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum ControlLevel {
    None,           // No control
    Contested,      // Weak control, unstable
    Established,    // Stable control
    Dominant,       // Strong control, lower resistance
}

impl ControlLevel {
    pub fn max_tax_rate(&self) -> f32 {
        match self {
            ControlLevel::None => 0.0,
            ControlLevel::Contested => 0.1,    // 10% max
            ControlLevel::Established => 0.25, // 25% max
            ControlLevel::Dominant => 0.4,     // 40% max
        }
    }
}

// === TERRITORY MANAGER RESOURCE ===
#[derive(Clone, Debug, Default, Resource, Serialize, Deserialize)]
pub struct TerritoryManager {
    pub controlled_cities: HashMap<String, TerritoryControl>,
    pub total_territory_income: u32,
    pub territory_count: usize,
}

impl TerritoryManager {
    pub fn establish_control(&mut self, city_id: String, current_day: u32) {
        let control = TerritoryControl {
            city_id: city_id.clone(),
            control_level: ControlLevel::Contested,
            control_strength: 0.3, // Start weak
            tax_rate: 0.05,        // Start with 5% tax
            last_tax_collection: current_day,
            resistance_level: 0.0,
            days_controlled: 0,
            total_tax_collected: 0,
        };

        self.controlled_cities.insert(city_id.clone(), control);
        self.territory_count += 1;
        info!("Established territory control in {}", city_id);
    }

    pub fn collect_taxes(&mut self, cities_db: &CitiesDatabase, current_day: u32) -> u32 {
        let mut total_collected = 0;

        for (city_id, territory) in self.controlled_cities.iter_mut() {
            if current_day > territory.last_tax_collection {
                if let Some(city) = cities_db.get_city(city_id) {
                    let days_since_collection = current_day - territory.last_tax_collection;
                    let base_income = city.population as f32 * 1000.0; // Base income per million population
                    let tax_income = (base_income * territory.tax_rate * days_since_collection as f32) as u32;

                    // Apply control strength modifier
                    let actual_income = (tax_income as f32 * territory.control_strength) as u32;

                    territory.total_tax_collected += actual_income;
                    territory.last_tax_collection = current_day;
                    total_collected += actual_income;

                    // Increase resistance based on tax rate
                    territory.resistance_level += territory.tax_rate * 0.1;
                    territory.resistance_level = territory.resistance_level.min(1.0);
                }
            }
        }

        self.total_territory_income += total_collected;
        total_collected
    }

    pub fn update_control(&mut self, current_day: u32) {
        for territory in self.controlled_cities.values_mut() {
            territory.days_controlled += 1;

            // Control naturally strengthens over time
            territory.control_strength += 0.01;
            territory.control_strength = territory.control_strength.min(1.0);

            // High resistance weakens control
            if territory.resistance_level > 0.7 {
                territory.control_strength -= 0.02;
            }

            // Resistance naturally decays over time
            territory.resistance_level *= 0.99;

            // Update control level based on strength
            territory.control_level = match territory.control_strength {
                0.0..=0.3 => ControlLevel::Contested,
                0.3..=0.7 => ControlLevel::Established,
                0.7..=1.0 => ControlLevel::Dominant,
                _ => ControlLevel::None,
            };

            // Risk of losing control if strength drops too low
            if territory.control_strength < 0.1 {
                info!("Warning: Control weakening in {}", territory.city_id);
            }
        }

        // Remove territories with no control
        self.controlled_cities.retain(|_, territory| {
            territory.control_level != ControlLevel::None
        });

        self.territory_count = self.controlled_cities.len();
    }

    pub fn set_tax_rate(&mut self, city_id: &str, new_rate: f32) -> bool {
        if let Some(territory) = self.controlled_cities.get_mut(city_id) {
            let max_rate = territory.control_level.max_tax_rate();
            if new_rate <= max_rate {
                territory.tax_rate = new_rate;
                info!("Set tax rate to {:.1}% in {}", new_rate * 100.0, city_id);
                return true;
            } else {
                warn!("Tax rate {:.1}% exceeds maximum {:.1}% for control level {:?}",
                      new_rate * 100.0, max_rate * 100.0, territory.control_level);
            }
        }
        false
    }

    pub fn get_territory(&self, city_id: &str) -> Option<&TerritoryControl> {
        self.controlled_cities.get(city_id)
    }

    pub fn is_controlled(&self, city_id: &str) -> bool {
        self.controlled_cities.contains_key(city_id)
    }
}

// === PROGRESSION TRACKING ===
#[derive(Clone, Debug, Default, Resource, Serialize, Deserialize)]
pub struct ProgressionTracker {
    pub campaign_progress: CampaignProgress,
    pub win_conditions: WinConditions,
    pub chapter_completion: HashMap<String, bool>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CampaignProgress {
    pub current_chapter: usize,
    pub total_chapters: usize,
    pub cities_liberated: usize,
    pub total_income_generated: u32,
    pub days_elapsed: u32,
}

impl Default for CampaignProgress {
    fn default() -> Self {
        Self {
            current_chapter: 0,
            total_chapters: 10, // Adjust based on your campaign
            cities_liberated: 0,
            total_income_generated: 0,
            days_elapsed: 0,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WinConditions {
    pub min_cities_controlled: usize,
    pub min_daily_income: u32,
    pub campaign_complete: bool,
}

impl Default for WinConditions {
    fn default() -> Self {
        Self {
            min_cities_controlled: 5,
            min_daily_income: 10000,
            campaign_complete: false,
        }
    }
}

impl ProgressionTracker {
    pub fn check_win_conditions(&mut self, territory_manager: &TerritoryManager) -> bool {
        let cities_controlled = territory_manager.territory_count;
        let daily_income = territory_manager.total_territory_income / territory_manager.territory_count.max(1) as u32;

        self.win_conditions.campaign_complete =
            cities_controlled >= self.win_conditions.min_cities_controlled &&
            daily_income >= self.win_conditions.min_daily_income &&
            self.campaign_progress.current_chapter >= self.campaign_progress.total_chapters;

        self.win_conditions.campaign_complete
    }

    pub fn advance_chapter(&mut self, chapter_city: String) {
        self.chapter_completion.insert(chapter_city, true);
        self.campaign_progress.current_chapter += 1;
        self.campaign_progress.cities_liberated += 1;

        info!("Campaign advanced to chapter {}/{}",
              self.campaign_progress.current_chapter,
              self.campaign_progress.total_chapters);
    }
}

// === NARRATIVE CAMPAIGN STRUCTURE ===
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CampaignChapter {
    pub id: usize,
    pub city_id: String,
    pub title: String,
    pub theme: ChapterTheme,
    pub story_beat: String,
    pub prerequisites: Vec<String>, // Required completed cities
    pub rewards: ChapterRewards,
}

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChapterRewards {
    pub credits: u32,
    pub research_unlocks: Vec<String>,
    pub special_equipment: Vec<String>,
}

// === CAMPAIGN DATABASE ===
#[derive(Debug, Default, Resource)]
pub struct CampaignDatabase {
    pub chapters: Vec<CampaignChapter>,
}

impl CampaignDatabase {
    pub fn initialize() -> Self {
        Self {
            chapters: vec![
                CampaignChapter {
                    id: 0,
                    city_id: "new_york".to_string(),
                    title: "First Strike".to_string(),
                    theme: ChapterTheme::Tutorial,
                    story_beat: "Your syndicate makes its first move against the corporations. New York's financial district holds the keys to their power.".to_string(),
                    prerequisites: vec![],
                    rewards: ChapterRewards {
                        credits: 5000,
                        research_unlocks: vec!["basic_hacking".to_string()],
                        special_equipment: vec!["scanner_mk1".to_string()],
                    },
                },
                CampaignChapter {
                    id: 1,
                    city_id: "tokyo".to_string(),
                    title: "Digital Shadows".to_string(),
                    theme: ChapterTheme::Corporate,
                    story_beat: "Nexus Corporation's Tokyo headquarters houses their AI research. Infiltrate their servers and steal the data that could change everything.".to_string(),
                    prerequisites: vec!["new_york".to_string()],
                    rewards: ChapterRewards {
                        credits: 8000,
                        research_unlocks: vec!["advanced_ai".to_string(), "neural_interface".to_string()],
                        special_equipment: vec!["stealth_suit".to_string()],
                    },
                },
                CampaignChapter {
                    id: 2,
                    city_id: "berlin".to_string(),
                    title: "Underground Networks".to_string(),
                    theme: ChapterTheme::Underground,
                    story_beat: "Berlin's criminal underground holds secrets about the corporations' illegal activities. Navigate the city's dangerous districts to uncover the truth.".to_string(),
                    prerequisites: vec!["new_york".to_string()],
                    rewards: ChapterRewards {
                        credits: 6000,
                        research_unlocks: vec!["black_market_contacts".to_string()],
                        special_equipment: vec!["assault_rifle".to_string()],
                    },
                },
            ],
        }
    }

    pub fn get_chapter(&self, chapter_id: usize) -> Option<&CampaignChapter> {
        self.chapters.get(chapter_id)
    }

    pub fn get_chapter_by_city(&self, city_id: &str) -> Option<&CampaignChapter> {
        self.chapters.iter().find(|chapter| chapter.city_id == city_id)
    }

    pub fn is_chapter_available(&self, chapter_id: usize, completed_cities: &std::collections::HashSet<String>) -> bool {
        if let Some(chapter) = self.get_chapter(chapter_id) {
            chapter.prerequisites.iter().all(|prereq| completed_cities.contains(prereq))
        } else {
            false
        }
    }
}

// === SYSTEM FUNCTIONS ===
pub fn territory_daily_update_system(
    mut territory_manager: ResMut<TerritoryManager>,
    mut global_data: ResMut<GlobalData>,
    mut progression_tracker: ResMut<ProgressionTracker>,
    cities_db: Res<CitiesDatabase>,
    mut day_changed: Local<u32>,
) {
    if global_data.current_day != *day_changed {
        *day_changed = global_data.current_day;

        // Update territory control
        territory_manager.update_control(global_data.current_day);

        // Collect taxes
        let tax_income = territory_manager.collect_taxes(&cities_db, global_data.current_day);
        if tax_income > 0 {
            global_data.credits += tax_income;
            info!("Collected {} credits in taxes", tax_income);
        }

        // Update progression
        progression_tracker.campaign_progress.days_elapsed = global_data.current_day;
        progression_tracker.campaign_progress.total_income_generated += tax_income;

        // Check win conditions
        if progression_tracker.check_win_conditions(&territory_manager) {
            info!("CAMPAIGN VICTORY ACHIEVED!");
            // Handle campaign completion
        }
    }
}
