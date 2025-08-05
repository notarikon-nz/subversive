// src/core/agent_upgrades.rs - Cybernetics, performance tracking, and traits
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

// === VETERAN BONUS CONFIG DATA ===
struct VeteranBonusConfig {
    id: &'static str,
    name: &'static str,
    description: &'static str,
    unlock_condition: &'static str,
    threshold: u32,
    effect_value: f32,
    effect_type: fn(f32) -> CyberneticEffect,
    stat_getter: fn(&AgentPerformance) -> u32,
}

const VETERAN_BONUS_CONFIGS: &[VeteranBonusConfig] = &[
    VeteranBonusConfig {
        id: "veteran_survivor",
        name: "Veteran Survivor",
        description: "Survived 10+ missions",
        unlock_condition: "Survive 10 missions",
        threshold: 10,
        effect_value: 20.0,
        effect_type: CyberneticEffect::HealthBonus,
        stat_getter: |p| p.missions_survived,
    },
    VeteranBonusConfig {
        id: "combat_veteran",
        name: "Combat Veteran",
        description: "Eliminated 50+ enemies",
        unlock_condition: "Kill 50 enemies",
        threshold: 50,
        effect_value: 0.15,
        effect_type: CyberneticEffect::DamageBonus,
        stat_getter: |p| p.enemies_killed,
    },
    VeteranBonusConfig {
        id: "shadow_operative",
        name: "Shadow Operative",
        description: "Completed 5+ stealth missions",
        unlock_condition: "Complete 5 stealth missions",
        threshold: 5,
        effect_value: 0.20,
        effect_type: CyberneticEffect::StealthBonus,
        stat_getter: |p| p.stealth_missions,
    },
];

const TRAIT_RARITY_COLORS: [Color; 4] = [
    Color::srgb(0.8, 0.8, 0.8), // Common
    Color::srgb(0.2, 0.8, 0.2), // Uncommon
    Color::srgb(0.2, 0.6, 1.0), // Rare
    Color::srgb(1.0, 0.6, 0.2), // Legendary
];

const TRAIT_RARITY_THRESHOLDS: [f32; 4] = [0.50, 0.80, 0.95, 1.0];

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

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum CyberneticCategory {
    Combat, Stealth, Utility, Survival,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CyberneticEffect {
    DamageBonus(f32),
    AccuracyBonus(f32),
    SpeedBonus(f32),
    HealthBonus(f32),
    StealthBonus(f32),
    HackingBonus(f32),
    ExperienceBonus(f32),
    RecoveryReduction(u32),
    NoiseReduction(f32),
    VisionBonus(f32),
}

// === AGENT PERFORMANCE TRACKING ===
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AgentPerformance {
    pub missions_completed: u32,
    pub missions_survived: u32,
    pub enemies_killed: u32,
    pub terminals_hacked: u32,
    pub stealth_missions: u32,
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
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum TraitRarity {
    Common = 0,
    Uncommon = 1,
    Rare = 2,
    Legendary = 3,
}

impl TraitRarity {
    pub fn color(self) -> Color {
        TRAIT_RARITY_COLORS[self as usize]
    }
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

impl TraitsDatabase {
    pub fn load() -> Self {
        std::fs::read_to_string("data/traits.json")
            .map_err(|e| error!("Failed to load data/traits.json: {}", e))
            .and_then(|content| {
                serde_json::from_str::<TraitsDatabase>(&content)
                    .map_err(|e| error!("Failed to parse traits.json: {}", e))
            })
            .map(|data| {
                data
            })
            .unwrap_or_else(|_| Self { traits: Vec::new() })
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
        self.installed_cybernetics.iter()
            .flat_map(|c| &c.effects)
            .chain(self.traits.iter().flat_map(|t| t.positive_effects.iter().chain(&t.negative_effects)))
            .chain(self.performance.veteran_bonuses.iter().map(|b| &b.effect))
            .cloned()
            .collect()
    }
    
    pub fn get_effective_stat(&self, base_value: f32, effect_matcher: impl Fn(&CyberneticEffect) -> Option<f32>) -> f32 {
        let modifier = self.calculate_total_effects()
            .iter()
            .filter_map(|effect| effect_matcher(effect))
            .sum::<f32>();
        
        base_value * (1.0 + modifier)
    }
}

// === VETERAN BONUS SYSTEM ===
pub fn check_veteran_bonuses(performance: &mut AgentPerformance) {
    let mut new_bonuses = Vec::new();
    
    for config in VETERAN_BONUS_CONFIGS {
        let current_stat = (config.stat_getter)(performance);
        let has_bonus = performance.veteran_bonuses.iter().any(|b| b.name.contains(config.id));
        
        if current_stat >= config.threshold && !has_bonus {
            new_bonuses.push(VeteranBonus {
                name: config.name.into(),
                description: config.description.into(),
                unlock_condition: config.unlock_condition.into(),
                effect: (config.effect_type)(config.effect_value),
            });
        }
    }
    
    // Special case for speed bonus (uses f32 comparison)
    let has_speed_bonus = performance.veteran_bonuses.iter().any(|b| b.name.contains("speed_demon"));
    if performance.fastest_mission_time > 0.0 && performance.fastest_mission_time < 120.0 && !has_speed_bonus {
        new_bonuses.push(VeteranBonus {
            name: "Speed Demon".into(),
            description: "Completed mission in under 2 minutes".into(),
            unlock_condition: "Complete mission under 2 minutes".into(),
            effect: CyberneticEffect::SpeedBonus(0.15),
        });
    }
    
    performance.veteran_bonuses.extend(new_bonuses);
}

pub fn assign_random_trait_from_db(traits_db: &TraitsDatabase) -> Option<AgentTrait> {
    if traits_db.traits.is_empty() {
        warn!("No traits loaded from database");
        return None;
    }
    
    // 30% chance of no trait
    if rand::random::<f32>() < 0.3 {
        return None;
    }
    
    // Weighted rarity selection
    let rarity_roll = rand::random::<f32>();
    let target_rarity = TRAIT_RARITY_THRESHOLDS
        .iter()
        .position(|&threshold| rarity_roll < threshold)
        .map(|i| unsafe { std::mem::transmute(i as u8) }) // Safe: we know the values are valid
        .unwrap_or(TraitRarity::Common);
    
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
    perf.enemies_killed += enemies_killed;
    perf.terminals_hacked += terminals_accessed;
    perf.total_damage_dealt += damage_dealt;
    perf.total_damage_taken += damage_taken;
    
    if mission_success {
        perf.missions_survived += 1;
        perf.current_survival_streak += 1;
        perf.longest_survival_streak = perf.longest_survival_streak.max(perf.current_survival_streak);
    } else {
        perf.current_survival_streak = 0;
    }
    
    if stealth_mission {
        perf.stealth_missions += 1;
    }
    
    if perf.fastest_mission_time == 0.0 || mission_time < perf.fastest_mission_time {
        perf.fastest_mission_time = mission_time;
    }
    
    check_veteran_bonuses(perf);
}