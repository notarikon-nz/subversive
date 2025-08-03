// src/core/neosingapore_campaign_database.rs - Campaign Database for Neo-Singapore
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::core::*;

// === NEO-SINGAPORE CAMPAIGN DATABASE ===
#[derive(Debug, Resource, Deserialize, Serialize)]
pub struct NeoSingaporeCampaignDatabase {
    pub campaign: NeoSingaporeCampaign,
    pub districts: HashMap<String, SingaporeDistrict>,
    pub district_connections: HashMap<String, Vec<String>>,
}

impl NeoSingaporeCampaignDatabase {
    pub fn load() -> Self {
        // Try to load from file, fallback to initialization
        match std::fs::read_to_string("data/neosingapore_campaign.json") {
            Ok(content) => serde_json::from_str(&content).unwrap_or_else(|_| Self::initialize()),
            Err(_) => Self::initialize()
        }
    }

    pub fn initialize() -> Self {
        Self {
            campaign: create_neosingapore_campaign(),
            districts: create_singapore_districts_map(),
            district_connections: create_district_connections(),
        }
    }

    // === COMPATIBILITY METHODS (replacing ExtendedCampaignDatabase) ===
    
    pub fn get_operation(&self, operation_id: usize) -> Option<&CampaignOperation> {
        self.campaign.acts.iter()
            .flat_map(|act| &act.operations)
            .find(|operation| operation.id == operation_id)
    }
    
    pub fn get_operation_by_district(&self, district_id: &str) -> Option<&CampaignOperation> {
        self.campaign.acts.iter()
            .flat_map(|act| &act.operations)
            .find(|operation| operation.district_id == district_id)
    }
    
    pub fn is_operation_available(&self, operation_id: usize, controlled_districts: &std::collections::HashSet<String>) -> bool {
        if let Some(operation) = self.get_operation(operation_id) {
            operation.prerequisites.iter().all(|prereq| {
                match prereq {
                    OperationPrereq::DistrictControl(district_id) => controlled_districts.contains(district_id),
                    OperationPrereq::ResourceLevel(_) => true, // Would check actual resources in real implementation
                    OperationPrereq::ResearchUnlock(_) => true, // Would check research progress
                    OperationPrereq::PopulationSupport(_) => true, // Would check actual support levels
                    OperationPrereq::CorporateWeakness(_) => true, // Would check corporate strength
                }
            })
        } else {
            false
        }
    }

    pub fn get_available_targets(&self, controlled_districts: &std::collections::HashSet<String>) -> Vec<String> {
        let mut targets = Vec::new();

        // Always include connected districts (districts adjacent to controlled ones)
        for district_id in controlled_districts {
            if let Some(connections) = self.get_district_connections(district_id) {
                for connection in connections {
                    if !controlled_districts.contains(connection) {
                        targets.push(connection.clone());
                    }
                }
            }
        }

        // Add operations that have their prerequisites met
        for operation in self.campaign.acts.iter().flat_map(|act| &act.operations) {
            if !controlled_districts.contains(&operation.district_id) &&
               self.is_operation_available(operation.id, controlled_districts) {
                targets.push(operation.district_id.clone());
            }
        }

        // Add districts that are strategically important
        for (district_id, district) in &self.districts {
            if !controlled_districts.contains(district_id) {
                match district.strategic_value {
                    StrategyValue::Financial | StrategyValue::Intelligence => {
                        // High-value targets are always available if connected
                        if self.is_district_accessible(district_id, controlled_districts) {
                            targets.push(district_id.clone());
                        }
                    },
                    _ => {}
                }
            }
        }

        targets.sort();
        targets.dedup();
        targets
    }

    pub fn get_current_act(&self, operation_progress: usize) -> Option<&CampaignAct> {
        for act in &self.campaign.acts {
            let act_start = act.operations.first()?.id;
            let act_end = act.operations.last()?.id;
            if operation_progress >= act_start && operation_progress <= act_end {
                return Some(act);
            }
        }
        None
    }

    // === DISTRICT-SPECIFIC METHODS ===
    
    pub fn get_district(&self, district_id: &str) -> Option<&SingaporeDistrict> {
        self.districts.get(district_id)
    }

    pub fn get_district_connections(&self, district_id: &str) -> Option<&Vec<String>> {
        self.district_connections.get(district_id)
    }

    pub fn is_district_accessible(&self, district_id: &str, controlled_districts: &std::collections::HashSet<String>) -> bool {
        // Check if the district is connected to any controlled district
        if let Some(connections) = self.get_district_connections(district_id) {
            connections.iter().any(|connected_id| controlled_districts.contains(connected_id))
        } else {
            false
        }
    }

    pub fn get_districts_by_type(&self, district_type: DistrictType) -> Vec<&SingaporeDistrict> {
        self.districts.values()
            .filter(|district| district.district_type == district_type)
            .collect()
    }

    pub fn get_districts_by_corporation(&self, corporation: Corporation) -> Vec<&SingaporeDistrict> {
        self.districts.values()
            .filter(|district| district.controlling_corp == corporation)
            .collect()
    }

    pub fn get_operations_in_act(&self, act_id: usize) -> Option<&Vec<CampaignOperation>> {
        self.campaign.acts.iter()
            .find(|act| act.id == act_id)
            .map(|act| &act.operations)
    }

    pub fn get_next_available_operations(&self, controlled_districts: &std::collections::HashSet<String>, completed_operations: &std::collections::HashSet<usize>) -> Vec<&CampaignOperation> {
        self.campaign.acts.iter()
            .flat_map(|act| &act.operations)
            .filter(|operation| {
                !completed_operations.contains(&operation.id) &&
                self.is_operation_available(operation.id, controlled_districts)
            })
            .collect()
    }

    pub fn calculate_liberation_progress(&self, controlled_districts: &std::collections::HashSet<String>) -> f32 {
        let total_population: u32 = self.districts.values().map(|d| d.population).sum();
        let liberated_population: u32 = controlled_districts.iter()
            .filter_map(|id| self.get_district(id))
            .map(|d| d.population)
            .sum();
        
        if total_population > 0 {
            liberated_population as f32 / total_population as f32
        } else {
            0.0
        }
    }

    pub fn get_corporate_strength(&self, corporation: Corporation, controlled_districts: &std::collections::HashSet<String>) -> f32 {
        let corporate_districts: Vec<_> = self.get_districts_by_corporation(corporation);
        let total_corporate_population: u32 = corporate_districts.iter().map(|d| d.population).sum();
        
        let controlled_corporate_population: u32 = corporate_districts.iter()
            .filter(|d| controlled_districts.contains(&d.id))
            .map(|d| d.population)
            .sum();
        
        if total_corporate_population > 0 {
            1.0 - (controlled_corporate_population as f32 / total_corporate_population as f32)
        } else {
            0.0
        }
    }

    // === LEGACY COMPATIBILITY ===
    
    // For backwards compatibility with existing code that expects chapters
    pub fn get_chapter(&self, chapter_id: usize) -> Option<LegacyChapter> {
        self.get_operation(chapter_id).map(|op| LegacyChapter {
            id: op.id,
            city_id: op.district_id.clone(),
            title: op.title.clone(),
            theme: operation_type_to_chapter_theme(op.operation_type.clone()),
            story_beat: op.story_beat.clone(),
            prerequisites: extract_district_prerequisites(&op.prerequisites),
            rewards: operation_rewards_to_chapter_rewards(&op.rewards),
        })
    }
    
    pub fn get_chapter_by_city(&self, city_id: &str) -> Option<LegacyChapter> {
        self.get_operation_by_district(city_id).map(|op| LegacyChapter {
            id: op.id,
            city_id: op.district_id.clone(),
            title: op.title.clone(),
            theme: operation_type_to_chapter_theme(op.operation_type.clone()),
            story_beat: op.story_beat.clone(),
            prerequisites: extract_district_prerequisites(&op.prerequisites),
            rewards: operation_rewards_to_chapter_rewards(&op.rewards),
        })
    }
    
    pub fn is_chapter_available(&self, chapter_id: usize, completed_cities: &std::collections::HashSet<String>) -> bool {
        self.is_operation_available(chapter_id, completed_cities)
    }
}

// === CAMPAIGN CREATION FUNCTIONS ===
fn create_neosingapore_campaign() -> NeoSingaporeCampaign {
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
                    create_operation(1, "geylang", "Safe Harbor", OperationType::Infiltration, 
                        "Arrive in Neo-Singapore's Geylang district. The corporate experiment has turned Singapore into a surveillance state, but the underground economy still thrives in the shadows.",
                        vec![], Corporation::Syndicate, 1),
                    create_operation(2, "chinatown", "Ancient Networks", OperationType::Intelligence,
                        "Chinatown's shophouses hide traditional resistance networks that predate corporate control. Gain their trust and access to underground tunnels.",
                        vec![OperationPrereq::DistrictControl("geylang".to_string())], Corporation::Helix, 1),
                    create_operation(3, "clarke_quay", "River Runners", OperationType::Sabotage,
                        "The Singapore River's entertainment district masks smuggling operations. Disrupt Nexus surveillance boats and establish water transport routes.",
                        vec![OperationPrereq::DistrictControl("chinatown".to_string())], Corporation::Nexus, 2),
                    create_operation(4, "little_india", "Spice Routes", OperationType::Liberation,
                        "Little India's tight-knit community has resisted corporate gentrification. Help them organize against Omnicorp's 'urban renewal' projects.",
                        vec![OperationPrereq::PopulationSupport(3)], Corporation::Omnicorp, 2),
                    create_operation(5, "kampong_glam", "Cultural Resistance", OperationType::Intelligence,
                        "The historic Malay quarter maintains cultural traditions that corporate algorithms can't parse. Establish a secure communication hub.",
                        vec![OperationPrereq::DistrictControl("little_india".to_string())], Corporation::Nexus, 2),
                    create_operation(6, "tanjong_pagar", "Port Authority", OperationType::Heist,
                        "Tanjong Pagar's container port is now 'Helix Data Harbor.' Their shipping manifests hide the movement of surveillance equipment across Asia.",
                        vec![OperationPrereq::ResearchUnlock("water_transport".to_string())], Corporation::Helix, 3),
                    create_operation(7, "orchard", "Shopping Surveillance", OperationType::Sabotage,
                        "Orchard Road is now 'Nexus Commercial Corridor.' Every purchase, every movement is tracked. Hack their consumer surveillance network.",
                        vec![OperationPrereq::ResearchUnlock("encrypted_comms".to_string())], Corporation::Nexus, 3),
                    create_operation(8, "sentosa", "Corporate Playground", OperationType::Intelligence,
                        "Sentosa Island is now an exclusive corporate resort where executives make deals that affect millions. Infiltrate their private meetings.",
                        vec![OperationPrereq::DistrictControl("orchard".to_string())], Corporation::Nexus, 3),
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
                    create_operation(9, "raffles_place", "Financial Fortress", OperationType::Heist,
                        "Raffles Place, now 'Aegis Financial District,' processes corporate transactions for all of Southeast Asia. Hack their trading algorithms.",
                        vec![OperationPrereq::ResearchUnlock("corporate_intelligence".to_string())], Corporation::Aegis, 1),
                    create_operation(10, "jurong", "Industrial Sabotage", OperationType::Sabotage,
                        "Jurong is now 'Omnicorp Industrial Complex.' Their automated factories produce surveillance drones for global export. Shut down production.",
                        vec![OperationPrereq::ResearchUnlock("supply_interdiction".to_string())], Corporation::Omnicorp, 2),
                    create_operation(11, "bedok", "Housing Revolution", OperationType::Liberation,
                        "Bedok's HDB estates house hundreds of thousands of workers. Organize them against corporate exploitation and surveillance.",
                        vec![OperationPrereq::PopulationSupport(5)], Corporation::Nexus, 2),
                    create_operation(12, "woodlands", "Northern Gateway", OperationType::Intelligence,
                        "Woodlands controls the causeway to Malaysia. Corporate smuggling operations hide among legitimate trade.",
                        vec![OperationPrereq::DistrictControl("jurong".to_string())], Corporation::Helix, 2),
                    create_operation(13, "tampines", "Suburban Networks", OperationType::Liberation,
                        "Tampines represents middle-class Singapore under corporate control. Break their illusion of safety and prosperity.",
                        vec![OperationPrereq::DistrictControl("bedok".to_string())], Corporation::Omnicorp, 3),
                    create_operation(14, "serangoon", "Transport Hub", OperationType::Sabotage,
                        "Serangoon's transport interchange is a surveillance chokepoint. Hack the tracking systems and create blind spots.",
                        vec![OperationPrereq::ResearchUnlock("surveillance_hacking".to_string())], Corporation::Nexus, 3),
                    create_operation(15, "bukit_timah", "Elite Enclaves", OperationType::Intelligence,
                        "Bukit Timah's wealthy residents include corporate executives and government officials. Extract intelligence from their secure networks.",
                        vec![OperationPrereq::ResourceLevel(25000)], Corporation::Aegis, 3),
                    create_operation(16, "ang_mo_kio", "Central Command", OperationType::Warfare,
                        "Ang Mo Kio houses a major corporate data center. This operation marks the transition from covert to open resistance.",
                        vec![OperationPrereq::DistrictControl("serangoon".to_string())], Corporation::Nexus, 3),
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
                    create_operation(17, "marina_bay", "Corporate Towers", OperationType::Warfare,
                        "Marina Bay's corporate towers coordinate the surveillance state. Time for a direct assault on the symbols of corporate power.",
                        vec![OperationPrereq::ResourceLevel(50000)], Corporation::Nexus, 3),
                    create_operation(18, "punggol", "Smart City Nightmare", OperationType::Sabotage,
                        "Punggol is the 'smart city' prototype - every device is connected, monitored, controlled. Break their perfect system.",
                        vec![OperationPrereq::ResearchUnlock("network_warfare".to_string())], Corporation::Omnicorp, 2),
                    create_operation(19, "changi_business", "Airport Approaches", OperationType::Intelligence,
                        "Changi Business Park controls approaches to the airport. Secure this before the final push on Singapore's gateway to the world.",
                        vec![OperationPrereq::DistrictControl("punggol".to_string())], Corporation::Helix, 2),
                    create_operation(20, "tuas", "Industrial Fortress", OperationType::Warfare,
                        "Tuas industrial area is Omnicorp's manufacturing stronghold. Heavy fighting expected - bring everything you have.",
                        vec![OperationPrereq::CorporateWeakness(Corporation::Omnicorp)], Corporation::Omnicorp, 3),
                    create_operation(21, "clementi", "University Rebellion", OperationType::Liberation,
                        "NUS and NTU campuses in Clementi have become corporate research facilities. Liberate Singapore's academic future.",
                        vec![OperationPrereq::PopulationSupport(7)], Corporation::Helix, 2),
                    create_operation(22, "pasir_ris", "Eastern Stronghold", OperationType::Warfare,
                        "Pasir Ris controls the eastern approaches. Corporate forces are massing here for a counterattack.",
                        vec![OperationPrereq::DistrictControl("tampines".to_string())], Corporation::Aegis, 3),
                    create_operation(23, "yishun", "Northern Defense", OperationType::Warfare,
                        "Yishun forms the northern defensive line. Break through here to threaten corporate supply lines from Malaysia.",
                        vec![OperationPrereq::DistrictControl("woodlands".to_string())], Corporation::Aegis, 3),
                    create_operation(24, "holland_village", "Media War", OperationType::Liberation,
                        "Holland Village's expat community controls international media narratives. Win the propaganda war to gain global support.",
                        vec![OperationPrereq::ResearchUnlock("media_warfare".to_string())], Corporation::Helix, 2),
                ],
                act_requirements: ActRequirements {
                    min_territories: 10,
                    min_daily_income: 50000,
                    required_research: vec!["network_warfare".to_string(), "media_warfare".to_string()],
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
                    create_operation(25, "changi_airport", "Wings of Freedom", OperationType::Liberation,
                        "Changi Airport connects Singapore to the world. Broadcast the liberation signal globally and coordinate with resistance cells worldwide.",
                        vec![OperationPrereq::DistrictControl("changi_business".to_string())], Corporation::Nexus, 3),
                    create_operation(26, "downtown_core", "Heart of Power", OperationType::Warfare,
                        "The Downtown Core houses government buildings now controlled by corporate boards. Restore democratic governance.",
                        vec![OperationPrereq::CorporateWeakness(Corporation::Aegis)], Corporation::Aegis, 3),
                    create_operation(27, "jurong_island", "Chemical Liberation", OperationType::Sabotage,
                        "Jurong Island's chemical plants produce the drugs used for population control. Destroy their mind-control infrastructure.",
                        vec![OperationPrereq::DistrictControl("tuas".to_string())], Corporation::Omnicorp, 3),
                    create_operation(28, "sentosa_cove", "Elite Exodus", OperationType::Intelligence,
                        "Sentosa Cove's luxury residences house fleeing corporate executives. Extract their secrets before they escape.",
                        vec![OperationPrereq::DistrictControl("sentosa".to_string())], Corporation::Nexus, 2),
                    create_operation(29, "upper_thomson", "Final Resistance", OperationType::Warfare,
                        "Upper Thomson is the last corporate stronghold. Heavy defenses, but victory here means total liberation.",
                        vec![OperationPrereq::ResourceLevel(100000)], Corporation::Helix, 3),
                    create_operation(30, "kallang", "Sports and Surveillance", OperationType::Sabotage,
                        "Kallang's Sports Hub was converted into a mass surveillance center. Destroy it to free Singapore's collective consciousness.",
                        vec![OperationPrereq::DistrictControl("downtown_core".to_string())], Corporation::Nexus, 2),
                    create_operation(31, "novena", "Medical Freedom", OperationType::Liberation,
                        "Novena's medical district has been experimenting on Singapore's population. End their human trials and heal the city.",
                        vec![OperationPrereq::CorporateWeakness(Corporation::Helix)], Corporation::Helix, 2),
                    create_operation(32, "fort_canning", "Digital Declaration", OperationType::Liberation,
                        "From Fort Canning, Singapore's historic heart, broadcast the final liberation declaration. The corporate age is over.",
                        vec![OperationPrereq::DistrictControl("kallang".to_string()), OperationPrereq::DistrictControl("novena".to_string())], Corporation::Nexus, 1),
                ],
                act_requirements: ActRequirements {
                    min_territories: 16,
                    min_daily_income: 100000,
                    required_research: vec!["global_coordination".to_string()],
                    previous_act_completion: true,
                },
            },
        ],
        total_operations: 32,
        victory_conditions: NeoSingaporeVictory {
            min_district_control: 24,
            corporate_hq_destroyed: vec![],
            population_liberation: 0.8,
            key_infrastructure: vec!["changi_airport".to_string(), "marina_bay".to_string(), "downtown_core".to_string()],
        },
    }
}

fn create_singapore_districts_map() -> HashMap<String, SingaporeDistrict> {
    let districts = vec![
        // Central Districts
        SingaporeDistrict {
            id: "marina_bay".to_string(),
            name: "Marina Corporate Plaza".to_string(),
            district_type: DistrictType::Corporate,
            controlling_corp: Corporation::Nexus,
            population: 15000,
            corruption_level: 2,
            strategic_value: StrategyValue::Financial,
            connections: vec!["downtown_core".to_string(), "raffles_place".to_string()],
            landmarks: vec!["Marina Bay Sands".to_string(), "Corporate Towers".to_string()],
        },
        SingaporeDistrict {
            id: "downtown_core".to_string(),
            name: "Government Complex".to_string(),
            district_type: DistrictType::Government,
            controlling_corp: Corporation::Aegis,
            population: 8000,
            corruption_level: 3,
            strategic_value: StrategyValue::Symbolic,
            connections: vec!["marina_bay".to_string(), "fort_canning".to_string(), "kallang".to_string()],
            landmarks: vec!["Parliament House".to_string(), "Supreme Court".to_string()],
        },
        SingaporeDistrict {
            id: "raffles_place".to_string(),
            name: "Aegis Financial District".to_string(),
            district_type: DistrictType::Corporate,
            controlling_corp: Corporation::Aegis,
            population: 12000,
            corruption_level: 2,
            strategic_value: StrategyValue::Financial,
            connections: vec!["marina_bay".to_string(), "chinatown".to_string(), "tanjong_pagar".to_string()],
            landmarks: vec!["Financial Towers".to_string(), "Stock Exchange".to_string()],
        },
        
        // Underground/Traditional Districts
        SingaporeDistrict {
            id: "geylang".to_string(),
            name: "Geylang Underground".to_string(),
            district_type: DistrictType::Underground,
            controlling_corp: Corporation::Syndicate,
            population: 80000,
            corruption_level: 8,
            strategic_value: StrategyValue::Population,
            connections: vec!["kallang".to_string(), "bedok".to_string()],
            landmarks: vec!["Night Markets".to_string(), "Underground Networks".to_string()],
        },
        SingaporeDistrict {
            id: "chinatown".to_string(),
            name: "Heritage District".to_string(),
            district_type: DistrictType::Residential,
            controlling_corp: Corporation::Helix,
            population: 45000,
            corruption_level: 5,
            strategic_value: StrategyValue::Population,
            connections: vec!["raffles_place".to_string(), "tanjong_pagar".to_string(), "clarke_quay".to_string()],
            landmarks: vec!["Traditional Shophouses".to_string(), "Heritage Center".to_string()],
        },
        SingaporeDistrict {
            id: "little_india".to_string(),
            name: "Cultural Quarter".to_string(),
            district_type: DistrictType::Residential,
            controlling_corp: Corporation::Omnicorp,
            population: 35000,
            corruption_level: 6,
            strategic_value: StrategyValue::Population,
            connections: vec!["kampong_glam".to_string(), "serangoon".to_string()],
            landmarks: vec!["Tekka Market".to_string(), "Cultural Centers".to_string()],
        },
        SingaporeDistrict {
            id: "kampong_glam".to_string(),
            name: "Historic Malay Quarter".to_string(),
            district_type: DistrictType::Residential,
            controlling_corp: Corporation::Nexus,
            population: 25000,
            corruption_level: 4,
            strategic_value: StrategyValue::Intelligence,
            connections: vec!["little_india".to_string(), "fort_canning".to_string()],
            landmarks: vec!["Sultan Mosque".to_string(), "Traditional Streets".to_string()],
        },
        
        // Commercial Districts  
        SingaporeDistrict {
            id: "orchard".to_string(),
            name: "Nexus Commercial Corridor".to_string(),
            district_type: DistrictType::Commercial,
            controlling_corp: Corporation::Nexus,
            population: 20000,
            corruption_level: 3,
            strategic_value: StrategyValue::Intelligence,
            connections: vec!["clarke_quay".to_string(), "novena".to_string()],
            landmarks: vec!["Shopping Centers".to_string(), "Surveillance Network".to_string()],
        },
        SingaporeDistrict {
            id: "clarke_quay".to_string(),
            name: "Entertainment Hub".to_string(),
            district_type: DistrictType::Commercial,
            controlling_corp: Corporation::Nexus,
            population: 18000,
            corruption_level: 6,
            strategic_value: StrategyValue::Intelligence,
            connections: vec!["chinatown".to_string(), "orchard".to_string(), "fort_canning".to_string()],
            landmarks: vec!["River Entertainment".to_string(), "Surveillance Boats".to_string()],
        },
        
        // Transport and Industrial
        SingaporeDistrict {
            id: "tanjong_pagar".to_string(),
            name: "Helix Data Harbor".to_string(),
            district_type: DistrictType::Transport,
            controlling_corp: Corporation::Helix,
            population: 30000,
            corruption_level: 4,
            strategic_value: StrategyValue::Logistics,
            connections: vec!["changi_business".to_string(), "pasir_ris".to_string()],
            landmarks: vec!["Airport Terminals".to_string(), "Control Tower".to_string()],
        },
        
        // Residential Areas
        SingaporeDistrict {
            id: "bedok".to_string(),
            name: "Eastern Housing Estates".to_string(),
            district_type: DistrictType::Residential,
            controlling_corp: Corporation::Nexus,
            population: 200000,
            corruption_level: 4,
            strategic_value: StrategyValue::Population,
            connections: vec!["geylang".to_string(), "tampines".to_string()],
            landmarks: vec!["HDB Blocks".to_string(), "Shopping Centers".to_string()],
        },
        SingaporeDistrict {
            id: "tampines".to_string(),
            name: "Model Suburb".to_string(),
            district_type: DistrictType::Residential,
            controlling_corp: Corporation::Omnicorp,
            population: 180000,
            corruption_level: 3,
            strategic_value: StrategyValue::Population,
            connections: vec!["bedok".to_string(), "pasir_ris".to_string(), "punggol".to_string()],
            landmarks: vec!["Mall Complex".to_string(), "Surveillance Grid".to_string()],
        },
        SingaporeDistrict {
            id: "ang_mo_kio".to_string(),
            name: "Central Data Node".to_string(),
            district_type: DistrictType::Residential,
            controlling_corp: Corporation::Nexus,
            population: 160000,
            corruption_level: 3,
            strategic_value: StrategyValue::Intelligence,
            connections: vec!["serangoon".to_string(), "yishun".to_string()],
            landmarks: vec!["Data Center".to_string(), "Residential Towers".to_string()],
        },
        SingaporeDistrict {
            id: "serangoon".to_string(),
            name: "Transport Nexus".to_string(),
            district_type: DistrictType::Transport,
            controlling_corp: Corporation::Nexus,
            population: 140000,
            corruption_level: 4,
            strategic_value: StrategyValue::Logistics,
            connections: vec!["little_india".to_string(), "ang_mo_kio".to_string(), "punggol".to_string()],
            landmarks: vec!["MRT Interchange".to_string(), "Monitoring Station".to_string()],
        },
        
        // Northern Districts
        SingaporeDistrict {
            id: "woodlands".to_string(),
            name: "Northern Gateway".to_string(),
            district_type: DistrictType::Transport,
            controlling_corp: Corporation::Helix,
            population: 90000,
            corruption_level: 5,
            strategic_value: StrategyValue::Logistics,
            connections: vec!["jurong".to_string(), "yishun".to_string()],
            landmarks: vec!["Causeway Checkpoint".to_string(), "Border Control".to_string()],
        },
        SingaporeDistrict {
            id: "yishun".to_string(),
            name: "Northern Defense Line".to_string(),
            district_type: DistrictType::Residential,
            controlling_corp: Corporation::Aegis,
            population: 170000,
            corruption_level: 4,
            strategic_value: StrategyValue::Population,
            connections: vec!["ang_mo_kio".to_string(), "woodlands".to_string()],
            landmarks: vec!["Defense Installations".to_string(), "Housing Estates".to_string()],
        },
        
        // Eastern Districts
        SingaporeDistrict {
            id: "punggol".to_string(),
            name: "Smart City Prototype".to_string(),
            district_type: DistrictType::Residential,
            controlling_corp: Corporation::Omnicorp,
            population: 150000,
            corruption_level: 2,
            strategic_value: StrategyValue::Intelligence,
            connections: vec!["tampines".to_string(), "serangoon".to_string()],
            landmarks: vec!["Smart Home Grid".to_string(), "AI Control Center".to_string()],
        },
        SingaporeDistrict {
            id: "pasir_ris".to_string(),
            name: "Eastern Stronghold".to_string(),
            district_type: DistrictType::Residential,
            controlling_corp: Corporation::Aegis,
            population: 130000,
            corruption_level: 3,
            strategic_value: StrategyValue::Population,
            connections: vec!["tampines".to_string(), "changi_airport".to_string()],
            landmarks: vec!["Coastal Defense".to_string(), "Military Housing".to_string()],
        },
        SingaporeDistrict {
            id: "changi_business".to_string(),
            name: "Business Park".to_string(),
            district_type: DistrictType::Corporate,
            controlling_corp: Corporation::Helix,
            population: 40000,
            corruption_level: 2,
            strategic_value: StrategyValue::Intelligence,
            connections: vec!["changi_airport".to_string(), "pasir_ris".to_string()],
            landmarks: vec!["Corporate Offices".to_string(), "Research Labs".to_string()],
        },
        
        // Western Districts
        SingaporeDistrict {
            id: "tuas".to_string(),
            name: "Industrial Fortress".to_string(),
            district_type: DistrictType::Industrial,
            controlling_corp: Corporation::Omnicorp,
            population: 60000,
            corruption_level: 4,
            strategic_value: StrategyValue::Manufacturing,
            connections: vec!["jurong".to_string(), "jurong_island".to_string()],
            landmarks: vec!["Heavy Industry".to_string(), "Chemical Plants".to_string()],
        },
        SingaporeDistrict {
            id: "jurong_island".to_string(),
            name: "Chemical Control".to_string(),
            district_type: DistrictType::Industrial,
            controlling_corp: Corporation::Omnicorp,
            population: 5000,
            corruption_level: 3,
            strategic_value: StrategyValue::Manufacturing,
            connections: vec!["tuas".to_string()],
            landmarks: vec!["Chemical Synthesis".to_string(), "Population Control Labs".to_string()],
        },
        SingaporeDistrict {
            id: "clementi".to_string(),
            name: "Academic Complex".to_string(),
            district_type: DistrictType::Government,
            controlling_corp: Corporation::Helix,
            population: 110000,
            corruption_level: 3,
            strategic_value: StrategyValue::Intelligence,
            connections: vec!["jurong".to_string(), "bukit_timah".to_string(), "holland_village".to_string()],
            landmarks: vec!["Universities".to_string(), "Research Facilities".to_string()],
        },
        
        // Central/Elite Districts
        SingaporeDistrict {
            id: "bukit_timah".to_string(),
            name: "Elite Residential".to_string(),
            district_type: DistrictType::Residential,
            controlling_corp: Corporation::Aegis,
            population: 50000,
            corruption_level: 1,
            strategic_value: StrategyValue::Intelligence,
            connections: vec!["clementi".to_string(), "novena".to_string(), "holland_village".to_string()],
            landmarks: vec!["Luxury Estates".to_string(), "Private Security".to_string()],
        },
        SingaporeDistrict {
            id: "holland_village".to_string(),
            name: "International Media Hub".to_string(),
            district_type: DistrictType::Commercial,
            controlling_corp: Corporation::Helix,
            population: 35000,
            corruption_level: 2,
            strategic_value: StrategyValue::Intelligence,
            connections: vec!["clementi".to_string(), "bukit_timah".to_string()],
            landmarks: vec!["Media Centers".to_string(), "Expat Compounds".to_string()],
        },
        SingaporeDistrict {
            id: "novena".to_string(),
            name: "Medical District".to_string(),
            district_type: DistrictType::Government,
            controlling_corp: Corporation::Helix,
            population: 70000,
            corruption_level: 3,
            strategic_value: StrategyValue::Population,
            connections: vec!["orchard".to_string(), "bukit_timah".to_string(), "upper_thomson".to_string()],
            landmarks: vec!["Medical Complex".to_string(), "Research Hospitals".to_string()],
        },
        SingaporeDistrict {
            id: "upper_thomson".to_string(),
            name: "Corporate Stronghold".to_string(),
            district_type: DistrictType::Corporate,
            controlling_corp: Corporation::Helix,
            population: 45000,
            corruption_level: 2,
            strategic_value: StrategyValue::Intelligence,
            connections: vec!["novena".to_string()],
            landmarks: vec!["Helix Headquarters".to_string(), "Security Complex".to_string()],
        },
        
        // Historic and Symbolic
        SingaporeDistrict {
            id: "fort_canning".to_string(),
            name: "Historic Command".to_string(),
            district_type: DistrictType::Government,
            controlling_corp: Corporation::Nexus,
            population: 2000,
            corruption_level: 1,
            strategic_value: StrategyValue::Symbolic,
            connections: vec!["clarke_quay".to_string(), "kampong_glam".to_string(), "downtown_core".to_string()],
            landmarks: vec!["Historic Fort".to_string(), "Communications Tower".to_string()],
        },
        SingaporeDistrict {
            id: "kallang".to_string(),
            name: "Sports and Surveillance".to_string(),
            district_type: DistrictType::Government,
            controlling_corp: Corporation::Nexus,
            population: 80000,
            corruption_level: 4,
            strategic_value: StrategyValue::Intelligence,
            connections: vec!["downtown_core".to_string(), "geylang".to_string()],
            landmarks: vec!["Sports Hub".to_string(), "Mass Surveillance Center".to_string()],
        },
        
        // Island Districts
        SingaporeDistrict {
            id: "sentosa".to_string(),
            name: "Corporate Resort".to_string(),
            district_type: DistrictType::Commercial,
            controlling_corp: Corporation::Nexus,
            population: 8000,
            corruption_level: 2,
            strategic_value: StrategyValue::Intelligence,
            connections: vec!["sentosa_cove".to_string()],
            landmarks: vec!["Luxury Resort".to_string(), "Executive Meetings".to_string()],
        },
        SingaporeDistrict {
            id: "sentosa_cove".to_string(),
            name: "Elite Haven".to_string(),
            district_type: DistrictType::Residential,
            controlling_corp: Corporation::Nexus,
            population: 3000,
            corruption_level: 1,
            strategic_value: StrategyValue::Intelligence,
            connections: vec!["sentosa".to_string()],
            landmarks: vec!["Luxury Residences".to_string(), "Private Marina".to_string()],
        },
    ];

    districts.into_iter().map(|d| (d.id.clone(), d)).collect()
}

fn create_district_connections() -> HashMap<String, Vec<String>> {

    // Define all connections (will be made bidirectional)
    let connection_pairs = vec![
        ("marina_bay", vec!["downtown_core", "raffles_place"]),
        ("downtown_core", vec!["fort_canning", "kallang"]),
        ("raffles_place", vec!["chinatown", "tanjong_pagar"]),
        ("geylang", vec!["kallang", "bedok"]),
        ("chinatown", vec!["tanjong_pagar", "clarke_quay"]),
        ("little_india", vec!["kampong_glam", "serangoon"]),
        ("kampong_glam", vec!["fort_canning"]),
        ("orchard", vec!["clarke_quay", "novena"]),
        ("clarke_quay", vec!["fort_canning"]),
        ("jurong", vec!["tuas", "clementi", "woodlands"]),
        ("changi_airport", vec!["changi_business", "pasir_ris"]),
        ("bedok", vec!["tampines"]),
        ("tampines", vec!["pasir_ris", "punggol"]),
        ("ang_mo_kio", vec!["serangoon", "yishun"]),
        ("serangoon", vec!["punggol"]),
        ("woodlands", vec!["yishun"]),
        ("punggol", vec![]),  // Only connected through serangoon/tampines
        ("pasir_ris", vec![]),  // Only connected through tampines/changi_airport
        ("changi_business", vec![]),  // Only connected through changi_airport
        ("tuas", vec!["jurong_island"]),
        ("jurong_island", vec![]),  // Only connected through tuas
        ("clementi", vec!["bukit_timah", "holland_village"]),
        ("bukit_timah", vec!["novena", "holland_village"]),
        ("holland_village", vec![]),  // Only connected through clementi/bukit_timah
        ("novena", vec!["upper_thomson"]),
        ("upper_thomson", vec![]),  // Only connected through novena
        ("fort_canning", vec![]),  // Only connected through clarke_quay/kampong_glam/downtown_core
        ("kallang", vec![]),  // Only connected through downtown_core/geylang
        ("sentosa", vec!["sentosa_cove"]),
        ("sentosa_cove", vec![]),  // Only connected through sentosa
    ];

    // This creates a bidirectional connection map
    let mut connections: HashMap<String, Vec<String>> = HashMap::new();

    // Create bidirectional connections
    for (district, connected_districts) in connection_pairs {
        let district_id = district.to_string();
        let mut district_connections = connected_districts.iter().map(|s| s.to_string()).collect::<Vec<_>>();
        
        // Add existing connections if any
        if let Some(existing) = connections.get(&district_id) {
            district_connections.extend(existing.clone());
        }
        
        connections.insert(district_id.clone(), district_connections);
        
        // Add reverse connections
        for connected in connected_districts {
            let connected_id = connected.to_string();
            let entry = connections.entry(connected_id).or_insert_with(Vec::new);
            if !entry.contains(&district_id) {
                entry.push(district_id.clone());
            }
        }
    }

    connections
}

fn create_operation(id: usize, district_id: &str, title: &str, operation_type: OperationType, 
                   story_beat: &str, prerequisites: Vec<OperationPrereq>, 
                   corporate_target: Corporation, difficulty_tier: u8) -> CampaignOperation {
    CampaignOperation {
        id,
        district_id: district_id.to_string(),
        title: title.to_string(),
        operation_type,
        story_beat: story_beat.to_string(),
        prerequisites,
        rewards: OperationRewards {
            credits: 2000 + (id as u32 * 500),
            research_unlocks: vec![],
            equipment: vec![],
        },
        corporate_target,
        difficulty_tier,
    }
}

// === LEGACY COMPATIBILITY STRUCTURES ===
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LegacyChapter {
    pub id: usize,
    pub city_id: String,
    pub title: String,
    pub theme: ChapterTheme,
    pub story_beat: String,
    pub prerequisites: Vec<String>,
    pub rewards: ChapterRewards,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChapterRewards {
    pub credits: u32,
    pub research_unlocks: Vec<String>,
    pub special_equipment: Vec<String>,
}

// === CONVERSION FUNCTIONS ===
fn operation_type_to_chapter_theme(operation_type: OperationType) -> ChapterTheme {
    match operation_type {
        OperationType::Infiltration => ChapterTheme::Underground,
        OperationType::Sabotage => ChapterTheme::Technology,
        OperationType::Heist => ChapterTheme::Corporate,
        OperationType::Liberation => ChapterTheme::Revolution,
        OperationType::Intelligence => ChapterTheme::Surveillance,
        OperationType::Warfare => ChapterTheme::Liberation,
    }
}

fn extract_district_prerequisites(prereqs: &[OperationPrereq]) -> Vec<String> {
    prereqs.iter()
        .filter_map(|prereq| match prereq {
            OperationPrereq::DistrictControl(district_id) => Some(district_id.clone()),
            _ => None,
        })
        .collect()
}


fn operation_rewards_to_chapter_rewards(rewards: &OperationRewards) -> ChapterRewards {
    ChapterRewards {
        credits: rewards.credits,
        research_unlocks: rewards.research_unlocks.clone(),
        special_equipment: rewards.equipment.clone(),
    }
}

