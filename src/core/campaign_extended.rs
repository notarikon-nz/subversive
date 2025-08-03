// src/core/neosingapore_campaign.rs - Neo-Singapore focused campaign
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use crate::core::*;

// === NEO-SINGAPORE CAMPAIGN STRUCTURE ===
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeoSingaporeCampaign {
    pub acts: Vec<CampaignAct>,
    pub total_operations: usize,
    pub victory_conditions: NeoSingaporeVictory,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CampaignAct {
    pub id: usize,
    pub title: String,
    pub theme: ActTheme,
    pub description: String,
    pub operations: Vec<CampaignOperation>,
    pub act_requirements: ActRequirements,
    pub corporate_response_level: u8, // 1-5, affects AI aggression
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActTheme {
    Infiltration,    // Act 1: Establish base, learn the city
    Expansion,       // Act 2: Spread influence, gain resources  
    Confrontation,   // Act 3: Open corporate warfare
    Liberation,      // Act 4: Final push for city control
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CampaignOperation {
    pub id: usize,
    pub district_id: String,
    pub title: String,
    pub operation_type: OperationType,
    pub story_beat: String,
    pub prerequisites: Vec<OperationPrereq>,
    pub rewards: OperationRewards,
    pub corporate_target: Corporation,
    pub difficulty_tier: u8, // 1-3 within each act
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OperationType {
    Infiltration,    // Stealth-based missions
    Sabotage,        // Infrastructure disruption
    Heist,          // Resource acquisition
    Liberation,      // Population freeing
    Intelligence,    // Information gathering
    Warfare,        // Direct confrontation
}

#[derive(Debug, Clone, Hash, Serialize, Deserialize)]
pub enum OperationPrereq {
    DistrictControl(String),        // Must control specific district
    CorporateWeakness(Corporation), // Target corp must be weakened
    ResourceLevel(u32),             // Minimum credits/equipment
    ResearchUnlock(String),         // Specific tech required
    PopulationSupport(u8),          // Community support level 1-10
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct NeoSingaporeVictory {
    pub min_district_control: usize,      // Must control X districts
    pub corporate_hq_destroyed: Vec<Corporation>, // All corp HQs eliminated
    pub population_liberation: f32,        // % of citizens freed from surveillance
    pub key_infrastructure: Vec<String>,   // Critical systems under control
}

// === SINGAPORE DISTRICTS ===
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SingaporeDistrict {
    pub id: String,
    pub name: String,
    pub district_type: DistrictType,
    pub controlling_corp: Corporation,
    pub population: u32,
    pub corruption_level: u8, // 1-10
    pub strategic_value: StrategyValue,
    pub connections: Vec<String>, // Connected districts
    pub landmarks: Vec<String>,   // Recognizable locations
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub enum DistrictType {
    Corporate,      // CBD, Marina Bay
    Industrial,     // Jurong, Tuas
    Residential,    // HDB estates, private housing
    Commercial,     // Orchard, shopping districts  
    Transport,      // Changi, port areas
    Underground,    // Hidden/informal areas
    Government,     // Administrative centers
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StrategyValue {
    Financial,      // Money laundering, banking
    Intelligence,   // Data centers, surveillance hubs
    Logistics,      // Transport, supply chains
    Manufacturing,  // Production facilities
    Population,     // Large civilian populations
    Symbolic,       // Morale/propaganda value
}

// === CAMPAIGN CREATION ===
pub fn create_neosingapore_campaign() -> NeoSingaporeCampaign {
    NeoSingaporeCampaign {
        acts: vec![
            // ACT 1: INFILTRATION (Operations 1-8)
            CampaignAct {
                id: 1,
                title: "Shadow Arrival".to_string(),
                theme: ActTheme::Infiltration,
                description: "Establish your presence in Neo-Singapore. Learn the corporate power structures and build your underground network.".to_string(),
                corporate_response_level: 1,
                operations: vec![
                    CampaignOperation {
                        id: 1,
                        district_id: "geylang".to_string(),
                        title: "Safe Harbor".to_string(),
                        operation_type: OperationType::Infiltration,
                        story_beat: "Arrive in Neo-Singapore's Geylang district. The corporate experiment has turned Singapore into a surveillance state, but the underground economy still thrives in the shadows.".to_string(),
                        prerequisites: vec![], // Starting operation
                        rewards: OperationRewards { credits: 2000, research_unlocks: vec!["urban_navigation".to_string()], equipment: vec!["basic_scanner".to_string()] },
                        corporate_target: Corporation::Syndicate, // Local criminal networks
                        difficulty_tier: 1,
                    },
                    CampaignOperation {
                        id: 2,
                        district_id: "chinatown".to_string(),
                        title: "Ancient Networks".to_string(),
                        operation_type: OperationType::Intelligence,
                        story_beat: "Chinatown's shophouses hide traditional resistance networks that predate corporate control. Gain their trust and access to underground tunnels.".to_string(),
                        prerequisites: vec![OperationPrereq::DistrictControl("geylang".to_string())],
                        rewards: OperationRewards { credits: 2500, research_unlocks: vec!["tunnel_networks".to_string()], equipment: vec![] },
                        corporate_target: Corporation::Helix, // Cultural preservation vs corporate homogenization
                        difficulty_tier: 1,
                    },
                    CampaignOperation {
                        id: 3,
                        district_id: "clarke_quay".to_string(),
                        title: "River Runners".to_string(),
                        operation_type: OperationType::Sabotage,
                        story_beat: "The Singapore River's entertainment district masks smuggling operations. Disrupt Nexus surveillance boats and establish water transport routes.".to_string(),
                        prerequisites: vec![OperationPrereq::DistrictControl("chinatown".to_string())],
                        rewards: OperationRewards { credits: 3000, research_unlocks: vec!["water_transport".to_string()], equipment: vec!["stealth_boat".to_string()] },
                        corporate_target: Corporation::Nexus,
                        difficulty_tier: 2,
                    },
                    CampaignOperation {
                        id: 4,
                        district_id: "little_india".to_string(),
                        title: "Spice Routes".to_string(),
                        operation_type: OperationType::Liberation,
                        story_beat: "Little India's tight-knit community has resisted corporate gentrification. Help them organize against Omnicorp's 'urban renewal' projects.".to_string(),
                        prerequisites: vec![OperationPrereq::PopulationSupport(3)],
                        rewards: OperationRewards { credits: 3500, research_unlocks: vec!["community_organizing".to_string()], equipment: vec![] },
                        corporate_target: Corporation::Omnicorp,
                        difficulty_tier: 2,
                    },
                    CampaignOperation {
                        id: 5,
                        district_id: "kampong_glam".to_string(),
                        title: "Cultural Resistance".to_string(),
                        operation_type: OperationType::Intelligence,
                        story_beat: "The historic Malay quarter maintains cultural traditions that corporate algorithms can't parse. Establish a secure communication hub.".to_string(),
                        prerequisites: vec![OperationPrereq::DistrictControl("little_india".to_string())],
                        rewards: OperationRewards { credits: 4000, research_unlocks: vec!["encrypted_comms".to_string()], equipment: vec!["comm_array".to_string()] },
                        corporate_target: Corporation::Nexus,
                        difficulty_tier: 2,
                    },
                    CampaignOperation {
                        id: 6,
                        district_id: "tanjong_pagar".to_string(),
                        title: "Port Authority".to_string(),
                        operation_type: OperationType::Heist,
                        story_beat: "Tanjong Pagar's container port is now 'Helix Data Harbor.' Their shipping manifests hide the movement of surveillance equipment across Asia.".to_string(),
                        prerequisites: vec![OperationPrereq::ResearchUnlock("water_transport".to_string())],
                        rewards: OperationRewards { credits: 5000, research_unlocks: vec!["supply_interdiction".to_string()], equipment: vec!["cargo_scanner".to_string()] },
                        corporate_target: Corporation::Helix,
                        difficulty_tier: 3,
                    },
                    CampaignOperation {
                        id: 7,
                        district_id: "orchard".to_string(),
                        title: "Shopping Surveillance".to_string(),
                        operation_type: OperationType::Sabotage,
                        story_beat: "Orchard Road is now 'Nexus Commercial Corridor.' Every purchase, every movement is tracked. Hack their consumer surveillance network.".to_string(),
                        prerequisites: vec![OperationPrereq::ResearchUnlock("encrypted_comms".to_string())],
                        rewards: OperationRewards { credits: 6000, research_unlocks: vec!["surveillance_hacking".to_string()], equipment: vec!["privacy_cloak".to_string()] },
                        corporate_target: Corporation::Nexus,
                        difficulty_tier: 3,
                    },
                    CampaignOperation {
                        id: 8,
                        district_id: "sentosa".to_string(),
                        title: "Corporate Playground".to_string(),
                        operation_type: OperationType::Intelligence,
                        story_beat: "Sentosa Island is now an exclusive corporate resort where executives make deals that affect millions. Infiltrate their private meetings.".to_string(),
                        prerequisites: vec![OperationPrereq::DistrictControl("orchard".to_string())],
                        rewards: OperationRewards { credits: 7000, research_unlocks: vec!["corporate_intelligence".to_string()], equipment: vec!["executive_disguise".to_string()] },
                        corporate_target: Corporation::Nexus,
                        difficulty_tier: 3,
                    },
                ],
                act_requirements: ActRequirements {
                    min_territories: 0,
                    min_daily_income: 0,
                    required_research: vec![],
                    previous_act_completion: false,
                },
            },

            // ACT 2: EXPANSION (Operations 9-16)
            CampaignAct {
                id: 2,
                title: "Network Growth".to_string(),
                theme: ActTheme::Expansion,
                description: "Expand your influence across Singapore's districts. Build the infrastructure needed for open resistance.".to_string(),
                corporate_response_level: 2,
                operations: vec![
                    CampaignOperation {
                        id: 9,
                        district_id: "raffles_place".to_string(),
                        title: "Financial Fortress".to_string(),
                        operation_type: OperationType::Heist,
                        story_beat: "Raffles Place, now 'Aegis Financial District,' processes corporate transactions for all of Southeast Asia. Hack their trading algorithms.".to_string(),
                        prerequisites: vec![OperationPrereq::ResearchUnlock("corporate_intelligence".to_string())],
                        rewards: OperationRewards { credits: 10000, research_unlocks: vec!["financial_warfare".to_string()], equipment: vec!["quantum_processor".to_string()] },
                        corporate_target: Corporation::Aegis,
                        difficulty_tier: 1,
                    },
                    CampaignOperation {
                        id: 10,
                        district_id: "jurong".to_string(),
                        title: "Industrial Sabotage".to_string(),
                        operation_type: OperationType::Sabotage,
                        story_beat: "Jurong is now 'Omnicorp Industrial Complex.' Their automated factories produce surveillance drones for global export. Shut down production.".to_string(),
                        prerequisites: vec![OperationPrereq::ResearchUnlock("supply_interdiction".to_string())],
                        rewards: OperationRewards { credits: 8000, research_unlocks: vec!["industrial_hacking".to_string()], equipment: vec!["factory_virus".to_string()] },
                        corporate_target: Corporation::Omnicorp,
                        difficulty_tier: 2,
                    },
                    // ... continue with remaining operations for Act 2
                ],
                act_requirements: ActRequirements {
                    min_territories: 4,
                    min_daily_income: 15000,
                    required_research: vec!["surveillance_hacking".to_string()],
                    previous_act_completion: true,
                },
            },

            // ACT 3: CONFRONTATION (Operations 17-24)
            CampaignAct {
                id: 3,
                title: "Open War".to_string(),
                theme: ActTheme::Confrontation,
                description: "The corporations know you're here. Engage in direct warfare while protecting liberated populations.".to_string(),
                corporate_response_level: 4,
                operations: vec![
                    CampaignOperation {
                        id: 17,
                        district_id: "marina_bay".to_string(),
                        title: "Corporate Towers".to_string(),
                        operation_type: OperationType::Warfare,
                        story_beat: "Marina Bay's corporate towers coordinate the surveillance state. Time for a direct assault on the symbols of corporate power.".to_string(),
                        prerequisites: vec![OperationPrereq::ResourceLevel(50000)],
                        rewards: OperationRewards { credits: 15000, research_unlocks: vec!["tower_assault".to_string()], equipment: vec!["heavy_weapons".to_string()] },
                        corporate_target: Corporation::Nexus,
                        difficulty_tier: 3,
                    },
                    // ... continue with high-intensity operations
                ],
                act_requirements: ActRequirements {
                    min_territories: 10,
                    min_daily_income: 50000,
                    required_research: vec!["financial_warfare".to_string(), "industrial_hacking".to_string()],
                    previous_act_completion: true,
                },
            },

            // ACT 4: LIBERATION (Operations 25-32)
            CampaignAct {
                id: 4,
                title: "Digital Liberation".to_string(),
                theme: ActTheme::Liberation,
                description: "Coordinate the final push. Liberate Singapore's population and send a message to corporate powers worldwide.".to_string(),
                corporate_response_level: 5,
                operations: vec![
                    CampaignOperation {
                        id: 25,
                        district_id: "changi".to_string(),
                        title: "Wings of Freedom".to_string(),
                        operation_type: OperationType::Liberation,
                        story_beat: "Changi Airport connects Singapore to the world. Broadcast the liberation signal globally and coordinate with resistance cells worldwide.".to_string(),
                        prerequisites: vec![OperationPrereq::CorporateWeakness(Corporation::Nexus)],
                        rewards: OperationRewards { credits: 25000, research_unlocks: vec!["global_coordination".to_string()], equipment: vec!["satellite_array".to_string()] },
                        corporate_target: Corporation::Nexus,
                        difficulty_tier: 3,
                    },
                    // ... final liberation operations
                ],
                act_requirements: ActRequirements {
                    min_territories: 16,
                    min_daily_income: 100000,
                    required_research: vec!["tower_assault".to_string()],
                    previous_act_completion: true,
                },
            },
        ],
        total_operations: 32,
        victory_conditions: NeoSingaporeVictory {
            min_district_control: 20,
            corporate_hq_destroyed: vec![Corporation::Nexus, Corporation::Omnicorp, Corporation::Helix, Corporation::Aegis],
            population_liberation: 0.8, // 80% of population freed from surveillance
            key_infrastructure: vec!["changi_airport".to_string(), "marina_bay_towers".to_string(), "jurong_industrial".to_string()],
        },
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationRewards {
    pub credits: u32,
    pub research_unlocks: Vec<String>,
    pub equipment: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActRequirements {
    pub min_territories: usize,
    pub min_daily_income: u32,
    pub required_research: Vec<String>,
    pub previous_act_completion: bool,
}

// Map real Singapore locations to districts
pub fn create_singapore_districts() -> Vec<SingaporeDistrict> {
    vec![
        SingaporeDistrict {
            id: "geylang".to_string(),
            name: "Geylang Underground".to_string(),
            district_type: DistrictType::Underground,
            controlling_corp: Corporation::Syndicate,
            population: 150000,
            corruption_level: 8,
            strategic_value: StrategyValue::Population,
            connections: vec!["chinatown".to_string(), "marina_bay".to_string()],
            landmarks: vec!["Geylang Market".to_string(), "Red Light District".to_string()],
        },
        SingaporeDistrict {
            id: "marina_bay".to_string(),
            name: "Marina Corporate Plaza".to_string(),
            district_type: DistrictType::Corporate,
            controlling_corp: Corporation::Nexus,
            population: 50000,
            corruption_level: 2,
            strategic_value: StrategyValue::Financial,
            connections: vec!["raffles_place".to_string(), "orchard".to_string()],
            landmarks: vec!["Marina Bay Sands".to_string(), "Corporate Towers".to_string()],
        },
        // ... continue with all Singapore districts
    ]
}