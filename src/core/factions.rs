// src/core/factions.rs - Simple faction system for enemy-vs-enemy combat
use bevy::prelude::*;
use crate::core::*;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Faction {
    Player,      // Agents
    Corporate,   // Standard security
    Syndicate,   // Rival faction
    Police,      // Law enforcement
    Civilian,    // Non-combatants
    Military,
}

impl Faction {
    pub fn is_hostile_to(&self, other: &Faction) -> bool {
        match (self, other) {
            // Everyone is hostile to players
            (_, Faction::Player) | (Faction::Player, _) => true,
            
            // Corporate vs Syndicate rivalry
            (Faction::Corporate, Faction::Syndicate) | (Faction::Syndicate, Faction::Corporate) => true,
            
            // Police are hostile to both Corporate and Syndicate
            (Faction::Police, Faction::Corporate) | (Faction::Corporate, Faction::Police) => true,
            (Faction::Police, Faction::Syndicate) | (Faction::Syndicate, Faction::Police) => true,
            
            // Same faction or civilian relations
            _ => false,
        }
    }
    
    pub fn color(&self) -> Color {
        match self {
            Faction::Player => Color::srgb(0.1, 1.0, 0.1),
            Faction::Corporate => Color::srgb(1.0, 0.1, 0.1),
            Faction::Syndicate => Color::srgb(0.8, 0.2, 0.8),
            Faction::Police => Color::srgb(0.2, 0.2, 1.0),
            Faction::Civilian => Color::srgb(1.0, 1.0, 0.1),
            Faction::Military => Color::srgb(0.5, 0.8, 0.5),
        }
    }
}

pub fn setup_factions_system(
    mut commands: Commands,
    agent_query: Query<Entity, (With<Agent>, Without<Faction>)>,
    enemy_query: Query<Entity, (With<Enemy>, Without<Faction>, Without<Police>)>,
    police_query: Query<Entity, (With<Police>, Without<Faction>)>,
    civilian_query: Query<Entity, (With<Civilian>, Without<Faction>)>,
) {

    let missing_factions = agent_query.iter().count() + enemy_query.iter().count() + 
                          police_query.iter().count() + civilian_query.iter().count();
    
    if missing_factions > 0 {
        warn!("Found {} entities without factions - this shouldn't happen", missing_factions);
    }
    

    // Assign factions to existing entities
    for entity in agent_query.iter() {
        commands.entity(entity).insert(Faction::Player);
    }
    
    for entity in enemy_query.iter() {
        // Randomly assign Corporate or Syndicate to create conflict
        let faction = if rand::random::<bool>() {
            Faction::Corporate
        } else {
            Faction::Syndicate
        };
        commands.entity(entity).insert(faction);
    }
    
    for entity in police_query.iter() {
        commands.entity(entity).insert(Faction::Police);
    }
    
    for entity in civilian_query.iter() {
        commands.entity(entity).insert(Faction::Civilian);
    }

}

// Update sprite colors based on faction
pub fn faction_color_system(
    mut enemy_query: Query<(&mut Sprite, &Faction), (With<Enemy>, Changed<Faction>)>,
) {
    for (mut sprite, faction) in enemy_query.iter_mut() {
        // COLOURS OVERIDE THE SPRITES
        // sprite.color = faction.color();
    }
}
