// src/core/cities.rs - Global cities system for mission selection
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::core::*;

// === CITY DATA STRUCTURES ===
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct City {
    pub id: String,
    pub name: String,
    pub country: String,
    pub coordinates: CityCoordinates,
    pub population: u32,        // In millions
    pub corruption_level: u8,   // 1-10 scale
    pub controlling_corp: Corporation,
    pub traits: Vec<CityTrait>,
    pub connections: Vec<String>, // IDs of adjacent cities
}

impl City {
    pub fn get_chapter_theme(&self) -> ChapterTheme {
        match self.traits.first() {
            Some(CityTrait::FinancialHub) => ChapterTheme::Corporate,
            Some(CityTrait::DrugCartels) => ChapterTheme::Underground,
            Some(CityTrait::HighTechSurveillance) => ChapterTheme::Surveillance,
            Some(CityTrait::CivilianUnrest) => ChapterTheme::Revolution,
            Some(CityTrait::TechCenter) => ChapterTheme::Technology,
            _ => ChapterTheme::Corporate,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CityCoordinates {
    pub latitude: f32,
    pub longitude: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Corporation {
    Nexus,      // High-tech, surveillance
    Syndicate,  // Criminal operations
    Omnicorp,   // Industrial, manufacturing
    Helix,      // Biotech, pharmaceuticals
    Aegis,      // Security, military
    Independent, // No corporate control
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CityTrait {
    DrugCartels,
    HighTechSurveillance,
    CorporateHeadquarters,
    BlackMarket,
    PoliceBrutality,
    CivilianUnrest,
    HeavyIndustry,
    FinancialHub,
    MilitaryBase,
    Cybercrime,
    Underground,
    Corruption,
    TechCenter,
    Surveillance,

}

// === CITY STATE ===
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CityState {
    pub completed: bool,
    pub alert_level: AlertLevel,
    pub last_mission_day: u32,
    pub times_visited: u32,
}

impl Default for CityState {
    fn default() -> Self {
        Self {
            completed: false,
            alert_level: AlertLevel::Green,
            last_mission_day: 0,
            times_visited: 0,
        }
    }
}

// === CITIES DATABASE ===
#[derive(Debug, Default, Clone, Resource, Serialize, Deserialize)]
pub struct CitiesDatabase {
    pub cities: Vec<City>,
    pub starting_city: String,
}

#[derive(Clone, Resource, Serialize, Deserialize, Default)]
pub struct CitiesProgress {
    pub current_city: String,
    pub city_states: HashMap<String, CityState>,
    pub unlocked_cities: std::collections::HashSet<String>,
}

impl CitiesDatabase {
    pub fn load() -> Self {
        match std::fs::read_to_string("data/cities.json") {
            Ok(content) => {
                match serde_json::from_str::<CitiesDatabase>(&content) {
                    Ok(data) => {
                        info!("Loaded {} cities from data/cities.json", data.cities.len());
                        data
                    },
                    Err(e) => {
                        error!("Failed to parse cities.json: {}", e);
                        Self::default_cities()
                    }
                }
            },
            Err(_) => {
                warn!("cities.json not found, creating default");
                let default = Self::default_cities();
                default.save();
                default
            }
        }
    }

    pub fn save(&self) {
        if let Ok(json) = serde_json::to_string_pretty(self) {
            if std::fs::write("data/cities.json", json).is_ok() {
                info!("Saved cities database");
            }
        }
    }

    pub fn get_city(&self, id: &str) -> Option<&City> {
        self.cities.iter().find(|c| c.id == id)
    }

    pub fn get_all_cities(&self) -> Vec<&City> {
        self.cities.iter().collect()
    }

    pub fn get_accessible_cities(&self, global_data: &GlobalData) -> Vec<&City> {

        self.cities.iter()
            .filter(|city| {
                let is_starting = city.id == self.starting_city;
                let is_unlocked = global_data.cities_progress.unlocked_cities.contains(&city.id);
                let accessible = is_starting || is_unlocked;
                accessible
            })
            .collect()
    }

    pub fn unlock_connected_cities(&self, completed_city_id: &str, cities_progress: &mut CitiesProgress) -> Vec<String> {
        let mut newly_unlocked = Vec::new();

        if let Some(completed_city) = self.get_city(completed_city_id) {
            for connection_id in &completed_city.connections {
                if !cities_progress.unlocked_cities.contains(connection_id) {
                    cities_progress.unlocked_cities.insert(connection_id.clone());
                    newly_unlocked.push(connection_id.clone());
                    info!("Unlocked new city: {}", connection_id);
                }
            }
        }

        newly_unlocked
    }

    fn default_cities() -> Self {
        Self {
            starting_city: "new_york".to_string(),
            cities: create_default_cities(),
        }
    }
}

impl CitiesProgress {
    pub fn new(starting_city: String) -> Self {
        let mut unlocked_cities = std::collections::HashSet::new();
        unlocked_cities.insert(starting_city.clone()); // Starting city is always unlocked

        Self {
            current_city: starting_city,
            city_states: std::collections::HashMap::new(),
            unlocked_cities,
        }
    }

    pub fn get_city_state(&self, city_id: &str) -> CityState {
        self.city_states.get(city_id)
            .cloned()
            .unwrap_or_default()
    }

    pub fn get_city_state_mut(&mut self, city_id: &str) -> &mut CityState {
        self.city_states.entry(city_id.to_string()).or_insert_with(CityState::default)
    }

    pub fn complete_city(&mut self, city_id: &str, current_day: u32) {
        let state = self.get_city_state_mut(city_id);
        state.completed = true;
        state.last_mission_day = current_day;
        state.times_visited += 1;
    }

    pub fn raise_alert(&mut self, city_id: &str, current_day: u32) {
        let state = self.get_city_state_mut(city_id);
        state.alert_level = match state.alert_level {
            AlertLevel::Green => AlertLevel::Yellow,
            AlertLevel::Yellow => AlertLevel::Orange,
            AlertLevel::Orange => AlertLevel::Red,
            AlertLevel::Red => AlertLevel::Red,
        };
        state.last_mission_day = current_day;
    }
}

// === COORDINATE CONVERSION ===
#[derive(Clone, )]
pub struct MapProjection {
    pub map_width: f32,
    pub map_height: f32,
    pub center_x: f32,
    pub center_y: f32,
}

impl MapProjection {
    pub fn new(map_width: f32, map_height: f32) -> Self {
        Self {
            map_width,
            map_height,
            center_x: map_width / 2.0,
            center_y: map_height / 2.0,
        }
    }

    pub fn lat_lon_to_pixel(&self, coords: &CityCoordinates) -> Vec2 {
        // Simple equirectangular projection
        let x = self.center_x + ((coords.longitude - 30.0) / 160.0) * (self.map_width / 2.0);
        let y = self.center_y - ((coords.latitude - 10.0) / 60.0) * (self.map_height / 2.0);
        Vec2::new(x, y)
    }
}

// === CITY CREATION HELPERS ===
fn create_default_cities() -> Vec<City> {
    vec![
        // Americas
        create_city("new_york", "New York", "USA", 40.7128, -74.0060, 8, 4, Corporation::Nexus,
                   vec![CityTrait::FinancialHub, CityTrait::HighTechSurveillance],
                   vec!["chicago", "miami", "toronto"]),
    ]
}

fn create_city(
    id: &str,
    name: &str,
    country: &str,
    lat: f32,
    lon: f32,
    pop_millions: u32,
    corruption: u8,
    corp: Corporation,
    traits: Vec<CityTrait>,
    connections: Vec<&str>,
) -> City {
    City {
        id: id.to_string(),
        name: name.to_string(),
        country: country.to_string(),
        coordinates: CityCoordinates {
            latitude: lat,
            longitude: lon,
        },
        population: pop_millions,
        corruption_level: corruption,
        controlling_corp: corp,
        traits,
        connections: connections.into_iter().map(|s| s.to_string()).collect(),
    }
}

impl Corporation {
    pub fn color(&self) -> Color {
        match self {
            Corporation::Nexus => Color::srgb(0.2, 0.6, 1.0),     // Blue - high-tech
            Corporation::Syndicate => Color::srgb(0.8, 0.2, 0.8), // Purple - criminal
            Corporation::Omnicorp => Color::srgb(0.8, 0.6, 0.2),  // Orange - industrial
            Corporation::Helix => Color::srgb(0.2, 0.8, 0.2),     // Green - biotech
            Corporation::Aegis => Color::srgb(0.8, 0.2, 0.2),     // Red - military
            _ => Color::srgb(1.0, 1.0, 1.0),
        }
    }
}