// src/systems/tile_properties.rs - Core tile functionality system
use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use crate::core::*;
use crate::systems::pathfinding::PathfindingGrid;
use crate::systems::enhanced_pathfinding::*;

// === ENHANCED TILE SYSTEM ===
#[derive(Component, Debug, Clone)]
pub struct TileProperties {
    pub movement_cost: f32,      // 1.0 = normal, 2.0 = slow, 0.5 = fast
    pub provides_cover: f32,     // 0.0 = none, 1.0 = full cover
    pub blocks_vision: bool,     // True if blocks line of sight
    pub destructible: Option<TileHealth>,
    pub interaction: Option<TileInteraction>,
    pub environmental: TileEnvironment,
}

#[derive(Debug, Clone)]
pub struct TileHealth {
    pub current: f32,
    pub max: f32,
    pub debris_type: TileType,   // What tile becomes when destroyed
}

#[derive(Debug, Clone)]
pub enum TileInteraction {
    Hackable { device_type: DeviceType, difficulty: u8 },
    Climbable { height: f32 },
    Door { requires_key: bool, is_open: bool },
    Switch { target_tiles: Vec<(u32, u32)> },
}

#[derive(Debug, Clone)]
pub struct TileEnvironment {
    pub is_flammable: bool,
    pub conducts_electricity: bool,
    pub water_level: f32,        // 0.0 = dry, 1.0 = flooded
    pub temperature: f32,        // For fire spread mechanics
}

#[derive(Debug, Clone, Copy)]
pub enum DeviceType {
    SecurityCamera,
    StreetLight,
    ElectricPanel,
    Terminal,
}

// === ENHANCED TILE TYPES ===
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TileType {
    // Basic terrain
    Grass,
    Concrete,
    Asphalt,
    Water,
    Mud,
    
    // Infrastructure
    Road,
    Sidewalk,
    Parking,
    
    // Buildings and structures
    Wall,
    ReinforcedWall,
    Window,
    Door,
    Rubble,
    Building,
    
    // Cover elements
    LowCover,    // Barriers, cars
    HighCover,   // Walls, buildings
    
    // Special
    Cover,
    Hazardous,   // Electrical, chemical
    Restricted,  // High security areas
    
    // Urban zones
    Residential,
    Commercial,
    Industrial,
}

// === TILE PROPERTIES DATABASE ===
impl TileProperties {
    pub fn for_tile_type(tile_type: TileType) -> Self {
        match tile_type {
            TileType::Grass => Self {
                movement_cost: 1.2,
                provides_cover: 0.0,
                blocks_vision: false,
                destructible: None,
                interaction: None,
                environmental: TileEnvironment {
                    is_flammable: true,
                    conducts_electricity: false,
                    water_level: 0.0,
                    temperature: 20.0,
                },
            },
            
            TileType::Concrete | TileType::Sidewalk => Self {
                movement_cost: 1.0,
                provides_cover: 0.0,
                blocks_vision: false,
                destructible: None,
                interaction: None,
                environmental: TileEnvironment {
                    is_flammable: false,
                    conducts_electricity: false,
                    water_level: 0.0,
                    temperature: 20.0,
                },
            },
            
            TileType::Road | TileType::Asphalt => Self {
                movement_cost: 0.8, // Faster movement on roads
                provides_cover: 0.0,
                blocks_vision: false,
                destructible: None,
                interaction: None,
                environmental: TileEnvironment {
                    is_flammable: false,
                    conducts_electricity: false,
                    water_level: 0.0,
                    temperature: 20.0,
                },
            },
            
            TileType::Wall => Self {
                movement_cost: f32::INFINITY, // Impassable
                provides_cover: 1.0,
                blocks_vision: true,
                destructible: Some(TileHealth {
                    current: 100.0,
                    max: 100.0,
                    debris_type: TileType::Rubble,
                }),
                interaction: None,
                environmental: TileEnvironment {
                    is_flammable: false,
                    conducts_electricity: false,
                    water_level: 0.0,
                    temperature: 20.0,
                },
            },
            
            TileType::ReinforcedWall => Self {
                movement_cost: f32::INFINITY,
                provides_cover: 1.0,
                blocks_vision: true,
                destructible: Some(TileHealth {
                    current: 250.0,
                    max: 250.0,
                    debris_type: TileType::Rubble,
                }),
                interaction: None,
                environmental: TileEnvironment {
                    is_flammable: false,
                    conducts_electricity: false,
                    water_level: 0.0,
                    temperature: 20.0,
                },
            },
            
            TileType::Window => Self {
                movement_cost: f32::INFINITY,
                provides_cover: 0.3, // Partial cover
                blocks_vision: false, // Can see through
                destructible: Some(TileHealth {
                    current: 25.0,
                    max: 25.0,
                    debris_type: TileType::Concrete, // Broken glass on concrete
                }),
                interaction: None,
                environmental: TileEnvironment {
                    is_flammable: false,
                    conducts_electricity: false,
                    water_level: 0.0,
                    temperature: 20.0,
                },
            },
            
            TileType::Door => Self {
                movement_cost: 1.0, // Passable when open
                provides_cover: 0.8,
                blocks_vision: true,
                destructible: Some(TileHealth {
                    current: 75.0,
                    max: 75.0,
                    debris_type: TileType::Rubble,
                }),
                interaction: Some(TileInteraction::Door {
                    requires_key: false,
                    is_open: false,
                }),
                environmental: TileEnvironment {
                    is_flammable: true,
                    conducts_electricity: false,
                    water_level: 0.0,
                    temperature: 20.0,
                },
            },
            
            TileType::LowCover => Self {
                movement_cost: 1.5, // Slows movement
                provides_cover: 0.5,
                blocks_vision: false,
                destructible: Some(TileHealth {
                    current: 50.0,
                    max: 50.0,
                    debris_type: TileType::Rubble,
                }),
                interaction: None,
                environmental: TileEnvironment {
                    is_flammable: false,
                    conducts_electricity: false,
                    water_level: 0.0,
                    temperature: 20.0,
                },
            },
            
            TileType::Water => Self {
                movement_cost: 2.0, // Very slow in water
                provides_cover: 0.0,
                blocks_vision: false,
                destructible: None,
                interaction: None,
                environmental: TileEnvironment {
                    is_flammable: false,
                    conducts_electricity: true, // Dangerous!
                    water_level: 1.0,
                    temperature: 15.0,
                },
            },
            
            TileType::Mud => Self {
                movement_cost: 1.8,
                provides_cover: 0.0,
                blocks_vision: false,
                destructible: None,
                interaction: None,
                environmental: TileEnvironment {
                    is_flammable: false,
                    conducts_electricity: false,
                    water_level: 0.3,
                    temperature: 18.0,
                },
            },
            
            TileType::Rubble => Self {
                movement_cost: 1.6,
                provides_cover: 0.3,
                blocks_vision: false,
                destructible: None,
                interaction: None,
                environmental: TileEnvironment {
                    is_flammable: false,
                    conducts_electricity: false,
                    water_level: 0.0,
                    temperature: 20.0,
                },
            },
            
            TileType::Hazardous => Self {
                movement_cost: 1.0,
                provides_cover: 0.0,
                blocks_vision: false,
                destructible: None,
                interaction: None,
                environmental: TileEnvironment {
                    is_flammable: true,
                    conducts_electricity: true,
                    water_level: 0.0,
                    temperature: 50.0, // Hot surface
                },
            },
            
            _ => Self::default(), // Fallback for other types
        }
    }
    
    pub fn can_move_through(&self) -> bool {
        self.movement_cost < f32::INFINITY
    }
    
    pub fn is_destructible(&self) -> bool {
        self.destructible.is_some()
    }
    
    pub fn apply_damage(&mut self, damage: f32) -> bool {
        if let Some(ref mut health) = self.destructible {
            health.current -= damage;
            health.current <= 0.0
        } else {
            false
        }
    }
}

impl Default for TileProperties {
    fn default() -> Self {
        Self::for_tile_type(TileType::Concrete)
    }
}

// === TILE MANAGEMENT SYSTEMS ===

// System to assign properties to newly created tiles
pub fn assign_tile_properties_system(
    mut commands: Commands,
    new_tiles: Query<(Entity, &TileTextureIndex), (Added<TileTextureIndex>, Without<TileProperties>)>,
) {
    for (entity, texture_index) in new_tiles.iter() {
        let tile_type = texture_index_to_tile_type(texture_index.0);
        let properties = TileProperties::for_tile_type(tile_type);
        
        commands.entity(entity).insert(properties);
    }
}

// Convert texture index back to tile type
pub fn texture_index_to_tile_type(texture_index: u32) -> TileType {
    match texture_index {
        0 => TileType::Grass,
        1 => TileType::Concrete,
        2 => TileType::Asphalt,
        10 => TileType::Industrial,
        11 => TileType::Commercial,
        12 => TileType::Residential,
        20 => TileType::Road,
        21 => TileType::Sidewalk,
        30 => TileType::Wall,
        31 => TileType::Window,
        32 => TileType::Door,
        35 => TileType::LowCover,
        40 => TileType::Water,
        41 => TileType::Mud,
        50 => TileType::Rubble,
        60 => TileType::Hazardous,
        _ => TileType::Concrete, // Default fallback
    }
}

// System to handle tile destruction
pub fn tile_destruction_system(
    mut commands: Commands,
    mut tile_query: Query<(Entity, &mut TileProperties, &mut TileTextureIndex, &TilePos)>,
    mut explosion_events: EventReader<crate::core::GrenadeEvent>,
    mut damage_events: EventReader<TileDamageEvent>,
    tilemap_query: Query<&TileStorage, With<crate::systems::tilemap::IsometricMap>>,
    mut pathfinding_grid: ResMut<PathfindingGrid>,
    isometric_settings: Res<crate::systems::tilemap::IsometricSettings>,
) {
    let Ok(tile_storage) = tilemap_query.single() else { return; };
    
    // Handle explosion damage
    for explosion in explosion_events.read() {
        let explosion_tile_pos = isometric_settings.world_to_tile(explosion.target_pos);
        let damage_radius = (explosion.explosion_radius / isometric_settings.tile_width) as i32;
        
        // Damage tiles in explosion radius
        for y in (explosion_tile_pos.y - damage_radius)..=(explosion_tile_pos.y + damage_radius) {
            for x in (explosion_tile_pos.x - damage_radius)..=(explosion_tile_pos.x + damage_radius) {
                if x >= 0 && y >= 0 && x < isometric_settings.map_width as i32 && y < isometric_settings.map_height as i32 {
                    let tile_pos = TilePos { x: x as u32, y: y as u32 };
                    
                    if let Some(tile_entity) = tile_storage.get(&tile_pos) {
                        if let Ok((entity, mut properties, mut texture_index, _)) = tile_query.get_mut(tile_entity) {
                            let distance = ((explosion_tile_pos.x - x).pow(2) + (explosion_tile_pos.y - y).pow(2)) as f32;
                            let damage_falloff = 1.0 - (distance / (damage_radius as f32).powi(2));
                            let damage = explosion.damage * damage_falloff.max(0.0);
                            
                            if properties.apply_damage(damage) {
                                // Tile destroyed, convert to debris
                                if let Some(ref health) = properties.destructible {
                                    let new_tile_type = health.debris_type;
                                    *properties = TileProperties::for_tile_type(new_tile_type);
                                    texture_index.0 = tile_type_to_texture_index(new_tile_type);
                                    
                                    info!("Tile destroyed at ({}, {}) -> {:?}", x, y, new_tile_type);
                                }
                            }
                        }
                    }
                }
            }
        }
        
        // Update pathfinding grid after destruction
        update_pathfinding_from_tiles(&mut pathfinding_grid, &tile_query, &isometric_settings);
    }
    
    // Handle direct tile damage events
    for damage_event in damage_events.read() {
        if let Ok((entity, mut properties, mut texture_index, tile_pos)) = tile_query.get_mut(damage_event.tile_entity) {
            if properties.apply_damage(damage_event.damage) {
                if let Some(ref health) = properties.destructible {
                    let new_tile_type = health.debris_type;
                    *properties = TileProperties::for_tile_type(new_tile_type);
                    texture_index.0 = tile_type_to_texture_index(new_tile_type);
                    
                    info!("Tile directly damaged at ({}, {})", tile_pos.x, tile_pos.y);
                }
            }
        }
    }
}

pub fn tile_type_to_texture_index(tile_type: TileType) -> u32 {
    match tile_type {
        TileType::Grass => 0,
        TileType::Concrete => 1,
        TileType::Asphalt => 2,
        TileType::Road => 20,
        TileType::Sidewalk => 21,
        TileType::Wall => 30,
        TileType::Window => 31,
        TileType::Door => 32,
        TileType::LowCover => 35,
        TileType::Water => 40,
        TileType::Mud => 41,
        TileType::Rubble => 50,
        TileType::Hazardous => 60,
        _ => 1, // Default to concrete
    }
}

// === MOVEMENT COST INTEGRATION ===
pub fn update_pathfinding_from_tiles(
    pathfinding_grid: &mut PathfindingGrid,
    tile_query: &Query<(Entity, &mut TileProperties, &mut TileTextureIndex, &TilePos)>,
    isometric_settings: &crate::systems::tilemap::IsometricSettings,
) {
    // Resize grid to match tilemap
    pathfinding_grid.width = isometric_settings.map_width as usize;
    pathfinding_grid.height = isometric_settings.map_height as usize;
    pathfinding_grid.tiles.clear();
    pathfinding_grid.tiles.resize(
        pathfinding_grid.width * pathfinding_grid.height, 
        crate::systems::pathfinding::TileType::Walkable
    );
    
    // Update tiles based on properties
    for (_, properties, _, tile_pos) in tile_query.iter() {
        let x = tile_pos.x as usize;
        let y = tile_pos.y as usize;
        
        if x < pathfinding_grid.width && y < pathfinding_grid.height {
            let pathfinding_type = if !properties.can_move_through() {
                crate::systems::pathfinding::TileType::Blocked
            } else if properties.movement_cost > 1.5 {
                crate::systems::pathfinding::TileType::Difficult
            } else {
                crate::systems::pathfinding::TileType::Walkable
            };
            
            pathfinding_grid.set_tile(x, y, pathfinding_type);
        }
    }
    
    pathfinding_grid.dirty = false;
}

// === COVER SYSTEM INTEGRATION ===
#[derive(Component)]
pub struct TileCover {
    pub position: Vec2,
    pub cover_value: f32,
    pub blocks_vision: bool,
}

pub fn tile_cover_system(
    mut commands: Commands,
    tile_query: Query<(&TileProperties, &TilePos), Changed<TileProperties>>,
    isometric_settings: Res<crate::systems::tilemap::IsometricSettings>,
    existing_cover: Query<Entity, With<TileCover>>,
) {
    // Clean up existing tile cover points
    for entity in existing_cover.iter() {
        commands.entity(entity).despawn();
    }
    
    // Create new cover points for tiles that provide cover
    for (properties, tile_pos) in tile_query.iter() {
        if properties.provides_cover > 0.0 {
            let world_pos = isometric_settings.tile_to_world(IVec2::new(tile_pos.x as i32, tile_pos.y as i32));
            
            commands.spawn(TileCover {
                position: world_pos,
                cover_value: properties.provides_cover,
                blocks_vision: properties.blocks_vision,
            });
        }
    }
}

// === EVENTS ===
#[derive(Event)]
pub struct TileDamageEvent {
    pub tile_entity: Entity,
    pub damage: f32,
    pub damage_type: DamageType,
}

#[derive(Debug, Clone, Copy)]
pub enum DamageType {
    Explosive,
    Kinetic,
    Energy,
    Fire,
}

// === TILE INTERACTION SYSTEM ===
/*
pub fn tile_interaction_system(
    mut action_events: EventReader<ActionEvent>,
    mut tile_query: Query<(Entity, &mut TileProperties, &TilePos)>,
    tilemap_query: Query<&TileStorage, With<crate::systems::tilemap::IsometricMap>>,
    isometric_settings: Res<crate::systems::tilemap::IsometricSettings>,
    agent_query: Query<&Transform, With<Agent>>,
) {
    let Ok(tile_storage) = tilemap_query.single() else { return; };
    
    for event in action_events.read() {
        if let Action::InteractWith(target) = event.action {
            // Check if we're interacting with a tile
            if let Ok(agent_transform) = agent_query.get(event.entity) {
                let agent_pos = agent_transform.translation.truncate();
                let tile_pos = isometric_settings.world_to_tile(agent_pos);
                
                if let Some(tile_entity) = tile_storage.get(&TilePos { 
                    x: tile_pos.x as u32, 
                    y: tile_pos.y as u32 
                }) {
                    if let Ok((_, mut properties, _)) = tile_query.get_mut(tile_entity) {
                        handle_tile_interaction(&mut properties, event.entity);
                    }
                }
            }
        }
    }
}
*/
pub fn tile_interaction_system(
    mut action_events: EventReader<ActionEvent>,
    mut tile_query: Query<(Entity, &mut TileProperties, &TilePos)>,
    tilemap_query: Query<&TileStorage, With<crate::systems::tilemap::IsometricMap>>,
    isometric_settings: Res<crate::systems::tilemap::IsometricSettings>,
    agent_query: Query<&Transform, With<Agent>>,
) {
    let Ok(tile_storage) = tilemap_query.get_single() else {
        warn!("tile_interaction_system: Expected one IsometricMap, found zero or more.");
        return;
    };

    for event in action_events.read() {
        if let Action::InteractWith(_) = event.action {
            // Get agent transform safely
            let Ok(agent_transform) = agent_query.get(event.entity) else {
                warn!("tile_interaction_system: Event entity {:?} is not an Agent.", event.entity);
                continue;
            };

            let agent_pos = agent_transform.translation.truncate();
            let tile_pos = isometric_settings.world_to_tile(agent_pos);

            let tile_coords = TilePos {
                x: tile_pos.x as u32,
                y: tile_pos.y as u32,
            };

            if let Some(tile_entity) = tile_storage.get(&tile_coords) {
                match tile_query.get_mut(tile_entity) {
                    Ok((_, mut properties, _)) => {
                        // handle_tile_interaction(&mut properties, event.entity);
                        info!("handle_tile_interaction");
                    }
                    Err(err) => {
                        warn!(
                            "tile_interaction_system: Tile entity {:?} does not match expected query (TileProperties, TilePos): {:?}",
                            tile_entity, err
                        );
                    }
                }
            } else {
                debug!(
                    "tile_interaction_system: No tile found at tile_pos {:?} for agent {:?}",
                    tile_coords, event.entity
                );
            }
        }
    }
}



// === DEBUG SYSTEM ===
#[cfg(debug_assertions)]
pub fn debug_enhanced_pathfinding_system(
    mut gizmos: Gizmos,
    enhanced_grid: Res<EnhancedPathfindingGrid>,
    agents: Query<(&Transform, &crate::systems::pathfinding::PathfindingAgent), With<Agent>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut show_debug: Local<bool>,
) {
    if keyboard.just_pressed(KeyCode::F10) {
        *show_debug = !*show_debug;
        info!("Enhanced pathfinding debug: {}", if *show_debug { "ON" } else { "OFF" });
    }
    
    if !*show_debug { return; }
    
    // Draw grid bounds
    let world_min = enhanced_grid.offset;
    let world_max = enhanced_grid.offset + Vec2::new(
        enhanced_grid.width as f32 * enhanced_grid.tile_size,
        enhanced_grid.height as f32 * enhanced_grid.tile_size
    );
    
    gizmos.rect_2d(
        bevy::math::Isometry2d::from_translation((world_min + world_max) * 0.5),
        world_max - world_min,
        Color::srgb(0.3, 0.3, 0.3)
    );
    
    // Sample and draw tile properties (every 4th tile to avoid performance issues)
    for y in (0..enhanced_grid.height).step_by(4) {
        for x in (0..enhanced_grid.width).step_by(4) {
            let world_pos = enhanced_grid.grid_to_world(IVec2::new(x as i32, y as i32));
            let movement_cost = enhanced_grid.get_movement_cost(x, y);
            let cover_value = enhanced_grid.get_cover_value(x, y);
            let blocks_vision = enhanced_grid.blocks_vision(x, y);
            
            let color = if movement_cost >= f32::INFINITY {
                Color::srgb(1.0, 0.0, 0.0) // Red for blocked
            } else if cover_value > 0.5 {
                Color::srgb(0.0, 0.0, 1.0) // Blue for high cover
            } else if movement_cost > 1.5 {
                Color::srgb(1.0, 1.0, 0.0) // Yellow for slow
            } else {
                Color::srgb(0.0, 1.0, 0.0) // Green for normal
            };
            
            gizmos.circle_2d(world_pos, 4.0, color);
            
            if blocks_vision {
                gizmos.line_2d(
                    world_pos + Vec2::new(-3.0, -3.0),
                    world_pos + Vec2::new(3.0, 3.0),
                    Color::srgb(1.0, 0.0, 1.0)
                );
            }
        }
    }
    
    // Draw active enhanced paths
    for (transform, agent) in agents.iter() {
        if agent.current_path.len() > 1 {
            let agent_pos = transform.translation.truncate();
            
            // Draw path
            for i in 0..agent.current_path.len() - 1 {
                gizmos.line_2d(
                    agent.current_path[i],
                    agent.current_path[i + 1],
                    Color::srgb(0.0, 1.0, 1.0)
                );
            }
            
            // Draw current target
            if agent.path_index < agent.current_path.len() {
                gizmos.circle_2d(
                    agent.current_path[agent.path_index],
                    8.0,
                    Color::srgb(1.0, 0.0, 1.0)
                );
            }
        }
    }
}