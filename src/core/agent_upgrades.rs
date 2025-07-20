// src/core/agent_upgrades.rs - Cybernetics, performance tracking, and traits
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

// === CYBERNETIC UPGRADES ===
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CyberneticUpgrade {
    pub id: String,
    pub name: String,
    pub description: String,
    pub cost: u32,
    pub category: CyberneticCategory,
    pub effects: Vec<CyberneticEffect>,
    pub prerequisites: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CyberneticCategory {
    Combat,      // Damage, accuracy, speed
    Stealth,     // Detection, noise reduction
    Utility,     // Hacking, interaction
    Survival,    // Health, recovery
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CyberneticEffect {
    DamageBonus(f32),           // +15% damage
    AccuracyBonus(f32),         // +10% accuracy  
    SpeedBonus(f32),            // +20% movement speed
    HealthBonus(f32),           // +25 max health
    StealthBonus(f32),          // -20% detection range
    HackingBonus(f32),          // +50% hack speed
    ExperienceBonus(f32),       // +25% XP gain
    RecoveryReduction(u32),     // -1 day recovery time
    NoiseReduction(f32),        // -30% weapon noise
    VisionBonus(f32),           // +25% vision range
}

// === AGENT PERFORMANCE TRACKING ===
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AgentPerformance {
    pub missions_completed: u32,
    pub missions_survived: u32,
    pub enemies_killed: u32,
    pub terminals_hacked: u32,
    pub stealth_missions: u32,        // Missions completed without alerts
    pub total_damage_dealt: f32,
    pub total_damage_taken: f32,
    pub fastest_mission_time: f32,
    pub longest_survival_streak: u32,
    pub current_survival_streak: u32,
    pub veteran_bonuses: Vec<VeteranBonus>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VeteranBonus {
    pub name: String,
    pub description: String,
    pub unlock_condition: String,
    pub effect: CyberneticEffect,
}

// === AGENT TRAITS ===
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TraitRarity {
    Common,
    Uncommon, 
    Rare,
    Legendary,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AgentTrait {
    pub id: String,
    pub name: String,
    pub description: String,
    pub positive_effects: Vec<CyberneticEffect>,
    pub negative_effects: Vec<CyberneticEffect>,
    pub rarity: TraitRarity,
}

#[derive(Resource, Serialize, Deserialize)]
pub struct TraitsDatabase {
    pub traits: Vec<AgentTrait>,
}

impl TraitRarity {
    pub fn color(&self) -> Color {
        match self {
            TraitRarity::Common => Color::srgb(0.8, 0.8, 0.8),
            TraitRarity::Uncommon => Color::srgb(0.2, 0.8, 0.2),
            TraitRarity::Rare => Color::srgb(0.2, 0.6, 1.0),
            TraitRarity::Legendary => Color::srgb(1.0, 0.6, 0.2),
        }
    }
}

impl TraitsDatabase {
    pub fn load() -> Self {
        match std::fs::read_to_string("data/traits.json") {
            Ok(content) => {
                match serde_json::from_str::<TraitsDatabase>(&content) {  // Add type annotation
                    Ok(data) => {
                        info!("Loaded {} traits from data/traits.json", data.traits.len());
                        data
                    },
                    Err(e) => {
                        error!("Failed to parse traits.json: {}", e);
                        Self { traits: Vec::new() }
                    }
                }
            },
            Err(e) => {
                error!("Failed to load data/traits.json: {}", e);
                Self { traits: Vec::new() }
            }
        }
    }
    
    pub fn get_trait(&self, id: &str) -> Option<&AgentTrait> {
        self.traits.iter().find(|t| t.id == id)
    }
    
    pub fn get_traits_by_rarity(&self, rarity: TraitRarity) -> Vec<&AgentTrait> {
        self.traits.iter().filter(|t| t.rarity == rarity).collect()
    }
}

// === EXTENDED AGENT COMPONENT ===
#[derive(Component, Default, Clone, Serialize, Deserialize)]
pub struct AgentUpgrades {
    pub installed_cybernetics: Vec<CyberneticUpgrade>,
    pub performance: AgentPerformance,
    pub traits: Vec<AgentTrait>,
    pub total_upgrade_cost: u32,
}

impl AgentUpgrades {
    pub fn calculate_total_effects(&self) -> Vec<CyberneticEffect> {
        let mut effects = Vec::new();
        
        // Cybernetic effects
        for cybernetic in &self.installed_cybernetics {
            effects.extend(cybernetic.effects.clone());
        }
        
        // Trait effects (both positive and negative)
        for trait_data in &self.traits {
            effects.extend(trait_data.positive_effects.clone());
            effects.extend(trait_data.negative_effects.clone());
        }
        
        // Veteran bonuses
        for bonus in &self.performance.veteran_bonuses {
            effects.push(bonus.effect.clone());
        }
        
        effects
    }
    
    pub fn get_effective_stat(&self, base_value: f32, effect_type: fn(f32) -> CyberneticEffect) -> f32 {
        let effects = self.calculate_total_effects();
        let mut modifier = 1.0;
        
        for effect in effects {
            match effect {
                CyberneticEffect::DamageBonus(bonus) if matches!(effect_type(0.0), CyberneticEffect::DamageBonus(_)) => {
                    modifier += bonus;
                },
                CyberneticEffect::AccuracyBonus(bonus) if matches!(effect_type(0.0), CyberneticEffect::AccuracyBonus(_)) => {
                    modifier += bonus;
                },
                CyberneticEffect::SpeedBonus(bonus) if matches!(effect_type(0.0), CyberneticEffect::SpeedBonus(_)) => {
                    modifier += bonus;
                },
                _ => {}
            }
        }
        
        base_value * modifier
    }
}

// === VETERAN BONUS SYSTEM ===
pub fn check_veteran_bonuses(performance: &mut AgentPerformance) {
    let mut new_bonuses = Vec::new();
    
    // Survival bonuses
    if performance.missions_survived >= 10 && !has_bonus(performance, "veteran_survivor") {
        new_bonuses.push(VeteranBonus {
            name: "Veteran Survivor".to_string(),
            description: "Survived 10+ missions".to_string(),
            unlock_condition: "Survive 10 missions".to_string(),
            effect: CyberneticEffect::HealthBonus(20.0),
        });
    }
    
    // Combat bonuses
    if performance.enemies_killed >= 50 && !has_bonus(performance, "combat_veteran") {
        new_bonuses.push(VeteranBonus {
            name: "Combat Veteran".to_string(),
            description: "Eliminated 50+ enemies".to_string(),
            unlock_condition: "Kill 50 enemies".to_string(),
            effect: CyberneticEffect::DamageBonus(0.15),
        });
    }
    
    // Stealth bonuses
    if performance.stealth_missions >= 5 && !has_bonus(performance, "shadow_operative") {
        new_bonuses.push(VeteranBonus {
            name: "Shadow Operative".to_string(),
            description: "Completed 5+ stealth missions".to_string(),
            unlock_condition: "Complete 5 stealth missions".to_string(),
            effect: CyberneticEffect::StealthBonus(0.20),
        });
    }
    
    // Speed bonuses
    if performance.fastest_mission_time > 0.0 && performance.fastest_mission_time < 120.0 && !has_bonus(performance, "speed_demon") {
        new_bonuses.push(VeteranBonus {
            name: "Speed Demon".to_string(),
            description: "Completed mission in under 2 minutes".to_string(),
            unlock_condition: "Complete mission under 2 minutes".to_string(),
            effect: CyberneticEffect::SpeedBonus(0.15),
        });
    }
    
    performance.veteran_bonuses.extend(new_bonuses);
}

fn has_bonus(performance: &AgentPerformance, bonus_name: &str) -> bool {
    performance.veteran_bonuses.iter().any(|b| b.name.contains(bonus_name))
}

pub fn assign_random_trait_from_db(traits_db: &TraitsDatabase) -> Option<AgentTrait> {
    if traits_db.traits.is_empty() {
        warn!("No traits loaded from database");
        return None;
    }
    
    let roll = rand::random::<f32>();
    
    // 30% chance of no trait
    if roll < 0.3 {
        return None;
    }
    
    // Weighted rarity selection
    let rarity_roll = rand::random::<f32>();
    let target_rarity = if rarity_roll < 0.50 {
        TraitRarity::Common
    } else if rarity_roll < 0.80 {
        TraitRarity::Uncommon  
    } else if rarity_roll < 0.95 {
        TraitRarity::Rare
    } else {
        TraitRarity::Legendary
    };
    
    let filtered_traits = traits_db.get_traits_by_rarity(target_rarity);
    if filtered_traits.is_empty() {
        // Fallback to any trait if none of target rarity
        let random_index = rand::random::<usize>() % traits_db.traits.len();
        Some(traits_db.traits[random_index].clone())
    } else {
        let random_index = rand::random::<usize>() % filtered_traits.len();
        Some(filtered_traits[random_index].clone())
    }
}

// === PERFORMANCE TRACKING ===
pub fn update_agent_performance(
    agent_upgrades: &mut AgentUpgrades,
    mission_success: bool,
    enemies_killed: u32,
    terminals_accessed: u32,
    mission_time: f32,
    damage_dealt: f32,
    damage_taken: f32,
    stealth_mission: bool,
) {
    let perf = &mut agent_upgrades.performance;
    
    perf.missions_completed += 1;
    if mission_success {
        perf.missions_survived += 1;
        perf.current_survival_streak += 1;
        perf.longest_survival_streak = perf.longest_survival_streak.max(perf.current_survival_streak);
    } else {
        perf.current_survival_streak = 0;
    }
    
    perf.enemies_killed += enemies_killed;
    perf.terminals_hacked += terminals_accessed;
    perf.total_damage_dealt += damage_dealt;
    perf.total_damage_taken += damage_taken;
    
    if stealth_mission {
        perf.stealth_missions += 1;
    }
    
    if perf.fastest_mission_time == 0.0 || mission_time < perf.fastest_mission_time {
        perf.fastest_mission_time = mission_time;
    }
    
    check_veteran_bonuses(perf);
}