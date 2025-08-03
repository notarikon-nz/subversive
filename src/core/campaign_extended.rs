// src/core/campaign_extended.rs - Extended campaign for 46 cities
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::core::*;

// === EXTENDED CAMPAIGN STRUCTURE ===
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtendedCampaign {
    pub acts: Vec<CampaignAct>,
    pub total_chapters: usize,
    pub victory_conditions: ExtendedVictoryConditions,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CampaignAct {
    pub id: usize,
    pub title: String,
    pub theme: ActTheme,
    pub description: String,
    pub chapters: Vec<CampaignChapter>,
    pub act_requirements: ActRequirements,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActTheme {
    Foundation,     // Acts 1-2: Establishing operations (Americas)
    Expansion,      // Acts 3-4: Global reach (Europe/Asia)
    Confrontation,  // Acts 5-6: Corporate war (All regions)
    Liberation,     // Act 7: Final push
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActRequirements {
    pub min_territories: usize,
    pub min_daily_income: u32,
    pub required_research: Vec<String>,
    pub previous_act_completion: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtendedVictoryConditions {
    pub min_territories_per_region: HashMap<String, usize>,
    pub total_liberation_threshold: f32, // Percentage of world population freed
    pub final_corporate_targets: Vec<String>, // Must control these cities
}

// === CAMPAIGN DATABASE EXTENDED ===
#[derive(Debug, Resource, Deserialize, Serialize)]
pub struct ExtendedCampaignDatabase {
    pub campaign: ExtendedCampaign,
    pub regional_campaigns: HashMap<String, RegionalCampaign>,
}

impl ExtendedCampaignDatabase {
    pub fn load() -> Self {
        match std::fs::read_to_string("data/campaign.json") {
            Ok(content) => serde_json::from_str(&content).unwrap_or_else(|_| Self::initialize()),
            Err(_) => Self::initialize()
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegionalCampaign {
    pub region_name: String,
    pub corporate_focus: Corporation,
    pub narrative_theme: String,
    pub key_cities: Vec<String>,
    pub regional_objectives: Vec<RegionalObjective>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegionalObjective {
    pub objective_type: ObjectiveType,
    pub target_cities: Vec<String>,
    pub description: String,
    pub rewards: ChapterRewards,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ObjectiveType {
    EstablishBeachhead,     // First city in region
    SecureSupplyLines,      // Connect to adjacent controlled cities
    CorporateHeadquarters,  // Target specific corp HQs
    RegionalDomination,     // Control majority of region
    LibratePopulation,      // Free high-population centers
}

impl ExtendedCampaignDatabase {
    pub fn initialize() -> Self {
        Self {
            campaign: create_extended_campaign(),
            regional_campaigns: create_regional_campaigns(),
        }
    }

    // Add missing compatibility methods
    pub fn get_chapter(&self, chapter_id: usize) -> Option<&CampaignChapter> {
        self.campaign.acts.iter()
            .flat_map(|act| &act.chapters)
            .find(|chapter| chapter.id == chapter_id)
    }
    
    pub fn get_chapter_by_city(&self, city_id: &str) -> Option<&CampaignChapter> {
        self.campaign.acts.iter()
            .flat_map(|act| &act.chapters)
            .find(|chapter| chapter.city_id == city_id)
    }
    
    pub fn is_chapter_available(&self, chapter_id: usize, completed_cities: &std::collections::HashSet<String>) -> bool {
        if let Some(chapter) = self.get_chapter(chapter_id) {
            chapter.prerequisites.iter().all(|prereq| completed_cities.contains(prereq))
        } else {
            false
        }
    }
 

    pub fn get_available_targets(&self, controlled_cities: &std::collections::HashSet<String>) -> Vec<String> {
        let mut targets = Vec::new();

        // Always include connected cities (cities adjacent to controlled ones)
        for city_id in controlled_cities {
            if let Some(connections) = get_city_connections(city_id) {
                for connection in connections {
                    if !controlled_cities.contains(&connection) {
                        targets.push(connection);
                    }
                }
            }
        }

        // Add regional objectives based on current progress
        for (_, regional_campaign) in &self.regional_campaigns {
            let regional_control = controlled_cities.iter()
                .filter(|city| regional_campaign.key_cities.contains(city))
                .count();
      
            if regional_control > 0 && regional_control < regional_campaign.key_cities.len() {
                for city in &regional_campaign.key_cities {
                    if !controlled_cities.contains(city) {
                        targets.push(city.clone());
                    }
                }
            }
        }

        targets.sort();
        targets.dedup();
        targets
    }

    pub fn get_current_act(&self, chapter_progress: usize) -> Option<&CampaignAct> {
        for act in &self.campaign.acts {
            let act_start = act.chapters.first()?.id;
            let act_end = act.chapters.last()?.id;
            if chapter_progress >= act_start && chapter_progress <= act_end {
                return Some(act);
            }
        }
        None
    }
  
}

// === CAMPAIGN CREATION FUNCTIONS ===
fn create_extended_campaign() -> ExtendedCampaign {
    ExtendedCampaign {
        acts: vec![
            // ACT 1: AMERICAN FOUNDATION (Chapters 1-8)
            CampaignAct {
                id: 1,
                title: "Corporate Foothold".to_string(),
                theme: ActTheme::Foundation,
                description: "Establish your syndicate's presence in the Americas and build the foundation for global operations.".to_string(),
                chapters: vec![
                    create_chapter(1, "new_york", "First Strike", ChapterTheme::Tutorial,
                        "Your syndicate makes its first move against the megacorps. New York's financial networks hold the keys to corporate power."),
                    create_chapter(2, "chicago", "Industrial Might", ChapterTheme::Corporate,
                        "Chicago's industrial complex and Aegis Corporation's manufacturing base. Secure heavy weapons production."),
                    create_chapter(3, "toronto", "Northern Expansion", ChapterTheme::Corporate,
                        "Expand into Canada's financial sector. Nexus Corporation's northern operations are vulnerable."),
                    create_chapter(4, "miami", "Gateway Drugs", ChapterTheme::Underground,
                        "Miami's drug cartels control the Caribbean supply routes. Form alliances or eliminate competition."),
                    create_chapter(5, "los_angeles", "Hollywood Hackers", ChapterTheme::Technology,
                        "LA's entertainment industry masks serious cyber-crime operations. Infiltrate the digital underworld."),
                    create_chapter(6, "mexico_city", "Cartel Politics", ChapterTheme::Underground,
                        "Navigate Mexico City's complex cartel politics. The Syndicate's stronghold won't fall easily."),
                    create_chapter(7, "sao_paulo", "South American Media", ChapterTheme::Revolution,
                        "Control São Paulo's massive media infrastructure. Broadcast your message across South America."),
                    create_chapter(8, "bogota", "Mountain Fortress", ChapterTheme::Underground,
                        "Bogotá's high-altitude location and civil unrest make it a perfect revolutionary stronghold."),
                ],
                act_requirements: ActRequirements {
                    min_territories: 0,
                    min_daily_income: 0,
                    required_research: vec![],
                    previous_act_completion: false,
                },
            },
  
            // ACT 2: EUROPEAN INFILTRATION (Chapters 9-16)
            CampaignAct {
                id: 2,
                title: "Old World Networks".to_string(),
                theme: ActTheme::Foundation,
                description: "Infiltrate Europe's ancient power structures and established corporate hierarchies.".to_string(),
                chapters: vec![
                    create_chapter(9, "london", "Eyes Everywhere", ChapterTheme::Surveillance,
                        "London's omnipresent surveillance network. Turn their own cameras against them."),
                    create_chapter(10, "paris", "Revolution Rising", ChapterTheme::Revolution,
                        "Paris knows revolution. Organize the first major anti-corporate uprising since the old days."),
                    create_chapter(11, "berlin", "Underground Networks", ChapterTheme::Underground,
                        "Berlin's underground scenes hide the continent's most dangerous hackers and black market dealers."),
                    create_chapter(12, "amsterdam", "Financial Haven", ChapterTheme::Corporate,
                        "Amsterdam's banking sector launders money for half the world's criminal organizations."),
                    create_chapter(13, "rome", "Ancient Corruption", ChapterTheme::Underground,
                        "Rome's ancient corruption networks run deeper than you imagine. Helix Corporation has deep roots here."),
                    create_chapter(14, "moscow", "Bear's Den", ChapterTheme::Technology,
                        "Moscow's military-industrial complex controls weapons that could change the war's balance."),
                    create_chapter(15, "istanbul", "Bridge Between Worlds", ChapterTheme::Underground,
                        "Istanbul connects Europe to Asia. Control this hub, control the flow of information and contraband."),
                    create_chapter(16, "stockholm", "Nordic Model", ChapterTheme::Surveillance,
                        "Stockholm's advanced surveillance state is the model other cities want to copy. Break it first."),
                ],
                act_requirements: ActRequirements {
                    min_territories: 4,
                    min_daily_income: 15000,
                    required_research: vec!["advanced_hacking".to_string(), "surveillance_countermeasures".to_string()],
                    previous_act_completion: true,
                },
            },
  
            // ACT 3: ASIAN EXPANSION (Chapters 17-28)
            CampaignAct {
                id: 3,
                title: "Digital Dragons".to_string(),
                theme: ActTheme::Expansion,
                description: "Challenge the tech giants in their home territory. Asia's megacities hold the future of human consciousness.".to_string(),
                chapters: vec![
                    create_chapter(17, "tokyo", "Silicon Shadows", ChapterTheme::Technology,
                        "Nexus Corporation's Tokyo headquarters houses their experimental AI research. The neural interface prototypes could give your agents superhuman capabilities."),
                    create_chapter(18, "seoul", "K-Pop Resistance", ChapterTheme::Technology,
                        "Seoul's youth culture masks a sophisticated cyber-resistance movement. They've been waiting for someone like you."),
                    create_chapter(19, "shanghai", "Industrial Heart", ChapterTheme::Corporate,
                        "Shanghai's massive industrial output feeds corporate power worldwide. Sabotage the supply chains."),
                    create_chapter(20, "hong_kong", "Financial Fortress", ChapterTheme::Corporate,
                        "Hong Kong's financial networks process corporate transactions globally. Take control of the money flow."),
                    create_chapter(21, "singapore", "Data Fortress", ChapterTheme::Technology,
                        "Singapore's status as Asia's data hub makes it perfect for a massive cyber-warfare campaign."),
                    create_chapter(22, "bangkok", "Golden Triangle", ChapterTheme::Underground,
                        "Bangkok sits at the center of Southeast Asia's criminal networks. The Syndicate's influence runs deep here."),
                    create_chapter(23, "manila", "Island Networks", ChapterTheme::Underground,
                        "Manila's island geography creates natural safe havens for resistance operations."),
                    create_chapter(24, "jakarta", "Archipelago Rebellion", ChapterTheme::Revolution,
                        "Indonesia's thousands of islands could hide a revolutionary army. If you can unite them."),
                    create_chapter(25, "mumbai", "Bollywood Dreams", ChapterTheme::Revolution,
                        "Mumbai's entertainment industry influences a billion minds. Control the narrative, control the people."),
                    create_chapter(26, "delhi", "Seat of Power", ChapterTheme::Corporate,
                        "Delhi's government connections to Omnicorp run deeper than anyone realizes. Expose the corruption."),
                    create_chapter(27, "karachi", "Port of Storms", ChapterTheme::Underground,
                        "Karachi's chaotic port districts hide weapons shipments that could arm a revolution."),
                    create_chapter(28, "dhaka", "River Networks", ChapterTheme::Revolution,
                        "Dhaka's river networks provide hidden transport routes for revolutionary supplies."),
                ],
                act_requirements: ActRequirements {
                    min_territories: 10,
                    min_daily_income: 50000,
                    required_research: vec!["neural_interface".to_string(), "advanced_ai".to_string()],
                    previous_act_completion: true,
                },
            },
  
            // ACT 4: MIDDLE EAST & AFRICA (Chapters 29-36)
            CampaignAct {
                id: 4,
                title: "Desert Storm".to_string(),
                theme: ActTheme::Expansion,
                description: "The cradle of civilization becomes the battleground for humanity's future.".to_string(),
                chapters: vec![
                    create_chapter(29, "dubai", "Golden Cage", ChapterTheme::Corporate,
                        "Dubai's golden towers hide the darkest corporate secrets. Nexus's financial crimes are documented here."),
                    create_chapter(30, "tehran", "Shadow Operations", ChapterTheme::Technology,
                        "Tehran's underground tech scene develops weapons too dangerous for legitimate markets."),
                    create_chapter(31, "cairo", "Ancient Secrets", ChapterTheme::Underground,
                        "Cairo's ancient tunnels hide modern resistance networks that have fought oppression for millennia."),
                    create_chapter(32, "lagos", "Oil and Blood", ChapterTheme::Revolution,
                        "Lagos's oil wealth built corporate power. Time to redistribute that wealth to the people."),
                    create_chapter(33, "nairobi", "Safari Networks", ChapterTheme::Underground,
                        "Nairobi's wildlife conservation cover masks one of Africa's most sophisticated smuggling operations."),
                    create_chapter(34, "johannesburg", "Mining the Future", ChapterTheme::Corporate,
                        "Johannesburg's mines extract the rare metals that power corporate technology. Cut off their supply."),
                    create_chapter(35, "casablanca", "Maghreb Connection", ChapterTheme::Underground,
                        "Casablanca links African and European criminal networks. Control this hub, control continental flow."),
                    create_chapter(36, "addis_ababa", "Highland Fortress", ChapterTheme::Technology,
                        "Ethiopia's highlands hide Aegis Corporation's most secretive military research facilities."),
                ],
                act_requirements: ActRequirements {
                    min_territories: 20,
                    min_daily_income: 100000,
                    required_research: vec!["global_networks".to_string(), "advanced_weapons".to_string()],
                    previous_act_completion: true,
                },
            },
  
            // ACT 5: PACIFIC RIM (Chapters 37-40)
            CampaignAct {
                id: 5,
                title: "Ring of Fire".to_string(),
                theme: ActTheme::Confrontation,
                description: "The Pacific Rim's island nations become the final testing ground for liberation technology.".to_string(),
                chapters: vec![
                    create_chapter(37, "sydney", "Southern Cross", ChapterTheme::Corporate,
                        "Australia's isolation made it the perfect testing ground for corporate control experiments."),
                    create_chapter(38, "melbourne", "Industrial Underground", ChapterTheme::Technology,
                        "Melbourne's underground culture has been preparing for this war longer than anyone realizes."),
                    create_chapter(39, "auckland", "Edge of the World", ChapterTheme::Underground,
                        "New Zealand's remoteness hides the server farms that store corporate secrets from around the globe."),
                    create_chapter(40, "lima", "Mountain Rebellion", ChapterTheme::Revolution,
                        "Peru's mountainous terrain provides perfect cover for coordinating the final continental push."),
                ],
                act_requirements: ActRequirements {
                    min_territories: 30,
                    min_daily_income: 200000,
                    required_research: vec!["global_coordination".to_string(), "mass_liberation".to_string()],
                    previous_act_completion: true,
                },
            },
  
            // ACT 6: FINAL LIBERATION (Chapters 41-46)
            CampaignAct {
                id: 6,
                title: "Global Revolution".to_string(),
                theme: ActTheme::Liberation,
                description: "Coordinate simultaneous strikes across all continents. The final battle for human consciousness begins.".to_string(),
                chapters: vec![
                    create_chapter(41, "rio_janeiro", "Carnival of Revolution", ChapterTheme::Revolution,
                        "Rio's carnival traditions mask the coordination of South America's final liberation."),
                    create_chapter(42, "buenos_aires", "Southern Command", ChapterTheme::Corporate,
                        "Buenos Aires becomes the southern command center for coordinating global operations."),
                    create_chapter(43, "prague", "Heart of Europe", ChapterTheme::Underground,
                        "Prague's central location makes it the coordination hub for European liberation forces."),
                    create_chapter(44, "warsaw", "Eastern Front", ChapterTheme::Technology,
                        "Warsaw's position coordinates the liberation of Eastern Europe and connection to Asian forces."),
                    create_chapter(45, "athens", "Cradle of Democracy", ChapterTheme::Revolution,
                        "Athens, where democracy was born, becomes the symbol of humanity's return to self-governance."),
                    create_chapter(46, "barcelona", "Final Hour", ChapterTheme::Liberation,
                        "Barcelona coordinates the final simultaneous strike. All your territories, all your allies, one decisive moment."),
                ],
                act_requirements: ActRequirements {
                    min_territories: 40,
                    min_daily_income: 500000,
                    required_research: vec!["neural_liberation".to_string(), "global_uprising".to_string()],
                    previous_act_completion: true,
                },
            },
        ],
        total_chapters: 46,
        victory_conditions: ExtendedVictoryConditions {
            min_territories_per_region: [
                ("americas".to_string(), 8),
                ("europe".to_string(), 8),
                ("asia".to_string(), 12),
                ("middle_east_africa".to_string(), 8),
                ("pacific".to_string(), 4),
            ].into_iter().collect(),
            total_liberation_threshold: 0.75, // 75% of world population
            final_corporate_targets: vec![
                "tokyo".to_string(),    // Nexus HQ
                "new_york".to_string(), // Financial center
                "london".to_string(),   // Surveillance center
                "shanghai".to_string(), // Industrial center
                "dubai".to_string(),    // Resource center
            ],
        },
    }
}

fn create_regional_campaigns() -> HashMap<String, RegionalCampaign> {
    let mut campaigns = HashMap::new();

    // Americas - Focus on Syndicate territory and financial systems
    campaigns.insert("americas".to_string(), RegionalCampaign {
        region_name: "Americas".to_string(),
        corporate_focus: Corporation::Syndicate,
        narrative_theme: "Establishing the foundation of resistance in corporate heartland".to_string(),
        key_cities: vec!["new_york".to_string(), "chicago".to_string(), "los_angeles".to_string(),
                        "mexico_city".to_string(), "sao_paulo".to_string(), "bogota".to_string(),
                        "toronto".to_string(), "miami".to_string()],
        regional_objectives: vec![
            RegionalObjective {
                objective_type: ObjectiveType::EstablishBeachhead,
                target_cities: vec!["new_york".to_string()],
                description: "Establish first foothold in corporate financial networks".to_string(),
                rewards: ChapterRewards { credits: 5000, research_unlocks: vec![], special_equipment: vec![] },
            },
        ],
    });

    // Europe - Focus on Nexus and surveillance
    campaigns.insert("europe".to_string(), RegionalCampaign {
        region_name: "Europe".to_string(),
        corporate_focus: Corporation::Nexus,
        narrative_theme: "Breaking ancient power structures and surveillance states".to_string(),
        key_cities: vec!["london".to_string(), "paris".to_string(), "berlin".to_string(),
                        "amsterdam".to_string(), "rome".to_string(), "moscow".to_string(),
                        "istanbul".to_string(), "stockholm".to_string()],
        regional_objectives: vec![],
    });

    // Asia - Focus on technology and manufacturing
    campaigns.insert("asia".to_string(), RegionalCampaign {
        region_name: "Asia".to_string(),
        corporate_focus: Corporation::Omnicorp,
        narrative_theme: "Challenging tech giants and breaking manufacturing chains".to_string(),
        key_cities: vec!["tokyo".to_string(), "shanghai".to_string(), "hong_kong".to_string(),
                        "seoul".to_string(), "singapore".to_string(), "bangkok".to_string(),
                        "manila".to_string(), "jakarta".to_string(), "mumbai".to_string(),
                        "delhi".to_string(), "karachi".to_string(), "dhaka".to_string()],
        regional_objectives: vec![],
    });

    campaigns
}

fn create_chapter(id: usize, city_id: &str, title: &str, theme: ChapterTheme, story_beat: &str) -> CampaignChapter {
    CampaignChapter {
        id,
        city_id: city_id.to_string(),
        title: title.to_string(),
        theme,
        story_beat: story_beat.to_string(),
        prerequisites: if id == 1 { vec![] } else { vec![] }, // Will be filled based on connections
        rewards: ChapterRewards {
            credits: 5000 + (id as u32 * 1000), // Scaling rewards
            research_unlocks: vec![],
            special_equipment: vec![],
        },
    }
}

// Helper function to get city connections (would integrate with your cities.json)
fn get_city_connections(city_id: &str) -> Option<Vec<String>> {
    // This would read from your cities.json file
    // For now, return empty for compilation
    Some(vec![])
}