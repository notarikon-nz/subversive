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
#[derive(Resource, Serialize, Deserialize)]
pub struct CitiesDatabase {
    pub cities: Vec<City>,
    pub starting_city: String,
}

#[derive(Clone, Resource, Serialize, Deserialize, Default)]
pub struct CitiesProgress {
    pub city_states: HashMap<String, CityState>,
    pub current_city: String,
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
    
    pub fn get_accessible_cities(&self, progress: &CitiesProgress) -> Vec<&City> {
        let mut accessible = Vec::new();
        
        // Starting city is always accessible
        if let Some(starting_city) = self.get_city(&self.starting_city) {
            accessible.push(starting_city);
        }
        
        // Cities connected to completed cities are accessible
        for city in &self.cities {
            if progress.city_states.get(&city.id).map_or(false, |s| s.completed) {
                // Add all connected cities
                for connection_id in &city.connections {
                    if let Some(connected_city) = self.get_city(connection_id) {
                        if !accessible.iter().any(|c| c.id == connected_city.id) {
                            accessible.push(connected_city);
                        }
                    }
                }
            }
        }
        
        accessible
    }
    
    fn default_cities() -> Self {
        Self {
            starting_city: "new_york".to_string(),
            cities: create_default_cities(),
        }
    }
}

impl CitiesProgress {
    pub fn get_city_state(&self, city_id: &str) -> CityState {
        self.city_states.get(city_id)
            .cloned()
            .unwrap_or_default()
    }
    
    pub fn get_city_state_mut(&mut self, city_id: &str) -> &mut CityState {
        self.city_states.entry(city_id.to_string()).or_default()
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
        let x = self.center_x + (coords.longitude / 180.0) * (self.map_width / 2.0);
        let y = self.center_y - (coords.latitude / 90.0) * (self.map_height / 2.0);
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
        
        create_city("los_angeles", "Los Angeles", "USA", 34.0522, -118.2437, 4, 5, Corporation::Omnicorp,
                   vec![CityTrait::BlackMarket, CityTrait::Cybercrime],
                   vec!["chicago", "mexico_city"]),
        
        create_city("mexico_city", "Mexico City", "Mexico", 19.4326, -99.1332, 9, 7, Corporation::Syndicate,
                   vec![CityTrait::DrugCartels, CityTrait::Corruption],
                   vec!["los_angeles", "panama_city"]),
        
        create_city("chicago", "Chicago", "USA", 41.8781, -87.6298, 3, 3, Corporation::Aegis,
                   vec![CityTrait::HeavyIndustry, CityTrait::PoliceBrutality],
                   vec!["new_york", "los_angeles", "toronto"]),
        
        create_city("miami", "Miami", "USA", 25.7617, -80.1918, 2, 6, Corporation::Syndicate,
                   vec![CityTrait::DrugCartels, CityTrait::BlackMarket],
                   vec!["new_york", "panama_city"]),
        
        create_city("toronto", "Toronto", "Canada", 43.6532, -79.3832, 3, 2, Corporation::Nexus,
                   vec![CityTrait::FinancialHub],
                   vec!["new_york", "chicago"]),
        
        create_city("panama_city", "Panama City", "Panama", 8.5380, -79.5205, 1, 8, Corporation::Syndicate,
                   vec![CityTrait::DrugCartels, CityTrait::Corruption],
                   vec!["mexico_city", "miami", "bogota"]),
        
        create_city("bogota", "Bogotá", "Colombia", 4.7110, -74.0721, 7, 9, Corporation::Syndicate,
                   vec![CityTrait::DrugCartels, CityTrait::CivilianUnrest],
                   vec!["panama_city", "lima"]),
        
        create_city("lima", "Lima", "Peru", -12.0464, -77.0428, 10, 7, Corporation::Omnicorp,
                   vec![CityTrait::HeavyIndustry, CityTrait::CivilianUnrest],
                   vec!["bogota", "sao_paulo"]),
        
        create_city("sao_paulo", "São Paulo", "Brazil", -23.5505, -46.6333, 12, 6, Corporation::Helix,
                   vec![CityTrait::HeavyIndustry, CityTrait::Cybercrime],
                   vec!["lima", "rio_janeiro", "buenos_aires"]),
        
        create_city("rio_janeiro", "Rio de Janeiro", "Brazil", -22.9068, -43.1729, 6, 8, Corporation::Syndicate,
                   vec![CityTrait::DrugCartels, CityTrait::CivilianUnrest],
                   vec!["sao_paulo"]),
        
        create_city("buenos_aires", "Buenos Aires", "Argentina", -34.6118, -58.3960, 15, 5, Corporation::Omnicorp,
                   vec![CityTrait::FinancialHub, CityTrait::PoliceBrutality],
                   vec!["sao_paulo"]),
        
        // Europe
        create_city("london", "London", "UK", 51.5074, -0.1278, 9, 3, Corporation::Nexus,
                   vec![CityTrait::FinancialHub, CityTrait::HighTechSurveillance],
                   vec!["paris", "amsterdam", "berlin"]),
        
        create_city("paris", "Paris", "France", 48.8566, 2.3522, 11, 4, Corporation::Helix,
                   vec![CityTrait::CorporateHeadquarters, CityTrait::Underground],
                   vec!["london", "berlin", "barcelona"]),
        
        create_city("berlin", "Berlin", "Germany", 52.5200, 13.4050, 4, 2, Corporation::Omnicorp,
                   vec![CityTrait::HeavyIndustry, CityTrait::HighTechSurveillance],
                   vec!["london", "paris", "prague", "warsaw"]),
        
        create_city("moscow", "Moscow", "Russia", 55.7558, 37.6176, 12, 6, Corporation::Aegis,
                   vec![CityTrait::MilitaryBase, CityTrait::Cybercrime],
                   vec!["warsaw", "istanbul"]),
        
        create_city("istanbul", "Istanbul", "Turkey", 41.0082, 28.9784, 15, 5, Corporation::Syndicate,
                   vec![CityTrait::BlackMarket, CityTrait::Corruption],
                   vec!["moscow", "athens", "cairo"]),
        
        create_city("rome", "Rome", "Italy", 41.9028, 12.4964, 3, 6, Corporation::Helix,
                   vec![CityTrait::CorporateHeadquarters, CityTrait::Underground],
                   vec!["barcelona", "athens"]),
        
        create_city("barcelona", "Barcelona", "Spain", 41.3851, 2.1734, 5, 4, Corporation::Nexus,
                   vec![CityTrait::FinancialHub],
                   vec!["paris", "rome", "casablanca"]),
        
        create_city("amsterdam", "Amsterdam", "Netherlands", 52.3676, 4.9041, 1, 2, Corporation::Nexus,
                   vec![CityTrait::FinancialHub, CityTrait::BlackMarket],
                   vec!["london", "berlin"]),
        
        create_city("prague", "Prague", "Czech Republic", 50.0755, 14.4378, 1, 3, Corporation::Omnicorp,
                   vec![CityTrait::Underground, CityTrait::Cybercrime],
                   vec!["berlin", "warsaw"]),
        
        create_city("warsaw", "Warsaw", "Poland", 52.2297, 21.0122, 2, 4, Corporation::Aegis,
                   vec![CityTrait::MilitaryBase],
                   vec!["berlin", "prague", "moscow"]),
        
        create_city("athens", "Athens", "Greece", 37.9838, 23.7275, 3, 7, Corporation::Syndicate,
                   vec![CityTrait::BlackMarket, CityTrait::Underground],
                   vec!["rome", "istanbul"]),
        
        create_city("stockholm", "Stockholm", "Sweden", 59.3293, 18.0686, 1, 1, Corporation::Nexus,
                   vec![CityTrait::HighTechSurveillance],
                   vec!["amsterdam"]),
        
        // Asia
        create_city("tokyo", "Tokyo", "Japan", 35.6762, 139.6503, 14, 2, Corporation::Nexus,
                   vec![CityTrait::HighTechSurveillance, CityTrait::CorporateHeadquarters],
                   vec!["seoul", "shanghai"]),
        
        create_city("shanghai", "Shanghai", "China", 31.2304, 121.4737, 26, 3, Corporation::Omnicorp,
                   vec![CityTrait::HeavyIndustry, CityTrait::HighTechSurveillance],
                   vec!["tokyo", "hong_kong", "seoul"]),
        
        create_city("hong_kong", "Hong Kong", "China", 22.3193, 114.1694, 7, 2, Corporation::Nexus,
                   vec![CityTrait::FinancialHub, CityTrait::Cybercrime],
                   vec!["shanghai", "singapore", "manila"]),
        
        create_city("seoul", "Seoul", "South Korea", 37.5665, 126.9780, 10, 2, Corporation::Nexus,
                   vec![CityTrait::HighTechSurveillance, CityTrait::Cybercrime],
                   vec!["tokyo", "shanghai"]),
        
        create_city("singapore", "Singapore", "Singapore", 1.3521, 103.8198, 6, 1, Corporation::Nexus,
                   vec![CityTrait::HighTechSurveillance, CityTrait::FinancialHub],
                   vec!["hong_kong", "bangkok", "jakarta"]),
        
        create_city("bangkok", "Bangkok", "Thailand", 13.7563, 100.5018, 10, 6, Corporation::Syndicate,
                   vec![CityTrait::BlackMarket, CityTrait::Corruption],
                   vec!["singapore", "manila"]),
        
        create_city("mumbai", "Mumbai", "India", 19.0760, 72.8777, 20, 7, Corporation::Helix,
                   vec![CityTrait::CivilianUnrest, CityTrait::BlackMarket],
                   vec!["delhi", "dubai"]),
        
        create_city("delhi", "Delhi", "India", 28.7041, 77.1025, 30, 8, Corporation::Omnicorp,
                   vec![CityTrait::HeavyIndustry, CityTrait::CivilianUnrest],
                   vec!["mumbai", "karachi"]),
        
        create_city("jakarta", "Jakarta", "Indonesia", -6.2088, 106.8456, 10, 8, Corporation::Syndicate,
                   vec![CityTrait::Corruption, CityTrait::CivilianUnrest],
                   vec!["singapore", "manila", "sydney"]),
        
        create_city("manila", "Manila", "Philippines", 14.5995, 120.9842, 13, 9, Corporation::Syndicate,
                   vec![CityTrait::Corruption, CityTrait::DrugCartels],
                   vec!["hong_kong", "bangkok", "jakarta"]),
        
        create_city("dubai", "Dubai", "UAE", 25.2048, 55.2708, 3, 3, Corporation::Nexus,
                   vec![CityTrait::FinancialHub, CityTrait::HighTechSurveillance],
                   vec!["mumbai", "tehran", "cairo"]),
        
        create_city("tehran", "Tehran", "Iran", 35.6892, 51.3890, 9, 7, Corporation::Aegis,
                   vec![CityTrait::MilitaryBase, CityTrait::Underground],
                   vec!["dubai", "moscow"]),
        
        create_city("karachi", "Karachi", "Pakistan", 24.8607, 67.0011, 15, 9, Corporation::Syndicate,
                   vec![CityTrait::DrugCartels, CityTrait::CivilianUnrest],
                   vec!["delhi", "mumbai"]),
        
        create_city("dhaka", "Dhaka", "Bangladesh", 23.8103, 90.4125, 22, 8, Corporation::Helix,
                   vec![CityTrait::CivilianUnrest, CityTrait::Corruption],
                   vec!["delhi"]),
        
        // Africa
        create_city("cairo", "Cairo", "Egypt", 30.0444, 31.2357, 10, 6, Corporation::Aegis,
                   vec![CityTrait::MilitaryBase, CityTrait::Underground],
                   vec!["istanbul", "dubai", "addis_ababa"]),
        
        create_city("lagos", "Lagos", "Nigeria", 6.5244, 3.3792, 15, 8, Corporation::Syndicate,
                   vec![CityTrait::DrugCartels, CityTrait::CivilianUnrest],
                   vec!["casablanca", "nairobi"]),
        
        create_city("nairobi", "Nairobi", "Kenya", -1.2921, 36.8219, 4, 6, Corporation::Helix,
                   vec![CityTrait::Underground, CityTrait::CivilianUnrest],
                   vec!["lagos", "cairo", "addis_ababa", "johannesburg"]),
        
        create_city("johannesburg", "Johannesburg", "South Africa", -26.2041, 28.0473, 5, 5, Corporation::Omnicorp,
                   vec![CityTrait::HeavyIndustry, CityTrait::CivilianUnrest],
                   vec!["nairobi"]),
        
        create_city("casablanca", "Casablanca", "Morocco", 33.5731, -7.5898, 4, 5, Corporation::Syndicate,
                   vec![CityTrait::BlackMarket, CityTrait::Corruption],
                   vec!["barcelona", "lagos"]),
        
        create_city("addis_ababa", "Addis Ababa", "Ethiopia", 9.1450, 38.7451, 3, 6, Corporation::Aegis,
                   vec![CityTrait::MilitaryBase],
                   vec!["cairo", "nairobi"]),
        
        // Oceania
        create_city("sydney", "Sydney", "Australia", -33.8688, 151.2093, 5, 2, Corporation::Nexus,
                   vec![CityTrait::FinancialHub],
                   vec!["melbourne", "jakarta", "auckland"]),
        
        create_city("melbourne", "Melbourne", "Australia", -37.8136, 144.9631, 5, 2, Corporation::Omnicorp,
                   vec![CityTrait::HeavyIndustry],
                   vec!["sydney"]),
        
        create_city("auckland", "Auckland", "New Zealand", -36.8485, 174.7633, 2, 1, Corporation::Nexus,
                   vec![CityTrait::Underground],
                   vec!["sydney"]),
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
        }
    }
}