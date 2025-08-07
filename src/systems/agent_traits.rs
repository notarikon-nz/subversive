// src/systems/agent_traits.rs - Simple trait application system
use bevy::prelude::*;
use crate::core::*;
use crate::core::agent_upgrades::*;

// Apply trait effects to agent stats during gameplay
pub fn apply_trait_effects_system(
    mut agent_query: Query<(&AgentUpgrades, &mut MovementSpeed, &mut Health), With<Agent>>,
    mut weapon_query: Query<&mut WeaponState, With<Agent>>,
) {
    for (upgrades, mut movement_speed, mut health) in agent_query.iter_mut() {
        let effects = upgrades.calculate_total_effects();
        
        // Apply speed bonuses/penalties
        let mut speed_modifier = 1.0;
        let mut health_modifier = 0.0;
        
        for effect in &effects {
            match effect {
                CyberneticEffect::SpeedBonus(bonus) => speed_modifier += bonus,
                CyberneticEffect::HealthBonus(bonus) => health_modifier += bonus,
                _ => {}
            }
        }
        
        // Apply modifiers (base values would need to be stored)
        movement_speed.0 = 150.0 * speed_modifier; // Base 150.0 speed
        
        // Health bonus (one-time application - would need better tracking)
        if health_modifier != 0.0 && health.0 == 100.0 {
            health.0 += health_modifier;
        }
    }
}

// Apply trait effects to combat
pub fn apply_trait_combat_effects(
    upgrades: &AgentUpgrades,
    base_damage: f32,
    base_accuracy: f32,
    weapon_type: &WeaponType,
) -> (f32, f32) {
    let effects = upgrades.calculate_total_effects();
    let mut damage_modifier = 1.0;
    let mut accuracy_modifier = 1.0;
    
    for effect in &effects {
        match effect {
            CyberneticEffect::DamageBonus(bonus) => {
                // Check for weapon-specific trait bonuses
                if weapon_type == &WeaponType::Flamethrower {
                    // Pyromaniac trait bonus applies here
                    damage_modifier += bonus;
                }
            },
            CyberneticEffect::AccuracyBonus(bonus) => {
                accuracy_modifier += bonus;
            },
            _ => {}
        }
    }
    
    (base_damage * damage_modifier, base_accuracy * accuracy_modifier)
}

// Trait-specific behavior system
pub fn trait_behavior_system(
    agent_query: Query<(Entity, &AgentUpgrades, &Transform), With<Agent>>,
    enemy_query: Query<&Transform, (With<Enemy>, Without<Agent>)>,
    mut action_events: EventWriter<ActionEvent>,
) {
    for (agent_entity, upgrades, agent_transform) in agent_query.iter() {
        for trait_data in &upgrades.traits {
            match trait_data.id.as_str() {
                "berserker" => {option
                    // Berserker agents prefer close combat
                    let agent_pos = agent_transform.translation.truncate();
                    let nearby_enemies = enemy_query.iter()
                        .filter(|enemy_transform| {
                            agent_pos.distance(enemy_transform.translation.truncate()) <= 60.0
                        })
                        .count();
                    
                    if nearby_enemies >= 2 {
                        // Could trigger aggressive behavior here
                        info!("Berserker agent {} engaging multiple enemies!", agent_entity.index());
                    }
                },
                
                "ghost" => {
                    // Ghost agents avoid detection - could influence movement patterns
                    // This would integrate with existing stealth systems
                },
                
                "tech_savant" => {
                    // Tech savants hack terminals faster
                    // This would modify interaction times
                },
                
                _ => {}
            }
        }
    }
}

// Visual trait indicators
pub fn trait_visual_system(
    mut gizmos: Gizmos,
    agent_query: Query<(&Transform, &AgentUpgrades), With<Agent>>,
    selection: Res<SelectionState>,
) {
    for (transform, upgrades) in agent_query.iter() {
        if !upgrades.traits.is_empty() {
            let pos = transform.translation.truncate();
            
            // Draw small trait indicator
            for (i, trait_data) in upgrades.traits.iter().enumerate() {
                let offset = Vec2::new((i as f32 - 1.0) * 8.0, -25.0);
                let color = trait_data.rarity.color();
                gizmos.circle_2d(pos + offset, 3.0, color);
            }
        }
    }
}

// Example trait effect in spawners.rs:
/*
fn spawn_agent_with_trait(
    commands: &mut Commands, 
    position: Vec2, 
    level: u8, 
    agent_idx: usize, 
    global_data: &GlobalData, 
    sprites: &GameSprites
) {
    // ... existing spawn code ...
    
    // Only assign trait on first spawn (not on mission restart)
    let mut upgrades = AgentUpgrades::default();
    
    // Check if this is a new agent (no previous loadout data)
    let loadout = global_data.get_agent_loadout(agent_idx);
    if loadout.weapon_configs.len() == 1 && loadout.cybernetics.is_empty() {
        // New agent - assign random trait
        if let Some(trait_data) = assign_random_trait() {
            upgrades.traits.push(trait_data.clone());
            info!("Agent {} spawned with trait: {} ({})", 
                  agent_idx + 1, trait_data.name, trait_data.description);
        }
    }
    
    commands.entity(entity).insert(upgrades);
}
*/