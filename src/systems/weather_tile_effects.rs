// src/systems/weather_tile_effects.rs - Weather effects on tile properties
use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use crate::core::*;
use crate::systems::tilemap::{IsometricSettings};
use crate::systems::tile_properties::{TileType, TileProperties, texture_index_to_tile_type};
use crate::systems::weather::{WeatherSystem, WeatherState};

// === WEATHER TILE COMPONENTS ===

#[derive(Component)]
pub struct WeatherAffectedTile {
    pub base_properties: TileProperties,
    pub current_properties: TileProperties,
    pub wet_level: f32,      // 0.0 - 1.0 how wet the tile is
    pub snow_level: f32,     // 0.0 - 1.0 how much snow accumulated
    pub dry_timer: f32,      // Time until tile starts drying
}

impl WeatherAffectedTile {
    pub fn new(base_properties: TileProperties) -> Self {
        let base_copy = base_properties.clone();
        Self {
            base_properties,
            current_properties: base_copy,
            wet_level: 0.0,
            snow_level: 0.0,
            dry_timer: 0.0,
        }
    }
}

// === WEATHER GRID RESOURCE ===

#[derive(Resource)]
pub struct WeatherTileGrid {
    pub width: usize,
    pub height: usize,
    pub wetness: Vec<f32>,        // Per-tile wetness level
    pub snow_coverage: Vec<f32>,  // Per-tile snow accumulation
    pub update_timer: Timer,
}

impl WeatherTileGrid {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            wetness: vec![0.0; width * height],
            snow_coverage: vec![0.0; width * height],
            update_timer: Timer::from_seconds(0.5, TimerMode::Repeating), // Update every 0.5s
        }
    }

    pub fn get_wetness(&self, x: usize, y: usize) -> f32 {
        if x < self.width && y < self.height {
            self.wetness[y * self.width + x]
        } else {
            0.0
        }
    }

    pub fn set_wetness(&mut self, x: usize, y: usize, level: f32) {
        if x < self.width && y < self.height {
            self.wetness[y * self.width + x] = level.clamp(0.0, 1.0);
        }
    }

    pub fn get_snow(&self, x: usize, y: usize) -> f32 {
        if x < self.width && y < self.height {
            self.snow_coverage[y * self.width + x]
        } else {
            0.0
        }
    }

    pub fn set_snow(&mut self, x: usize, y: usize, level: f32) {
        if x < self.width && y < self.height {
            self.snow_coverage[y * self.width + x] = level.clamp(0.0, 1.0);
        }
    }
}

pub fn apply_weather_movement_effects(
    mut movement_query: Query<(&Transform, &mut crate::core::MovementSpeed), Or<(With<Agent>, With<Enemy>, With<Civilian>)>>,
    weather_grid: Res<WeatherTileGrid>,
    tile_query: Query<&TileProperties>,
    tilemap_query: Query<&TileStorage, With<crate::systems::tilemap::IsometricMap>>,
    isometric_settings: Res<crate::systems::tilemap::IsometricSettings>,
) {
    let Ok(tile_storage) = tilemap_query.single() else { return; };

    for (transform, mut movement_speed) in movement_query.iter_mut() {
        let world_pos = transform.translation.truncate();
        let tile_pos = isometric_settings.world_to_tile(world_pos);
        
        if tile_pos.x >= 0 && tile_pos.y >= 0 &&
           tile_pos.x < weather_grid.width as i32 &&
           tile_pos.y < weather_grid.height as i32 {
            
            let tile_coords = TilePos { x: tile_pos.x as u32, y: tile_pos.y as u32 };
            
            if let Some(tile_entity) = tile_storage.get(&tile_coords) {
                if let Ok(tile_properties) = tile_query.get(tile_entity) {
                    // Apply weather-modified movement cost
                    // Note: movement_cost is inverse - higher cost = slower movement
                    let weather_modifier = 1.0 / tile_properties.movement_cost;
                    movement_speed.0 *= weather_modifier;
                }
            }
        }
    }
}


pub fn weather_tile_audio_system(
    movement_query: Query<(&Transform, &crate::core::MovementSpeed, Option<&Agent>), Changed<Transform>>,
    weather_grid: Res<WeatherTileGrid>,
    isometric_settings: Res<IsometricSettings>,
    mut audio_events: EventWriter<crate::core::AudioEvent>,
) {
    for (transform, _speed, agent) in movement_query.iter() {
        // Only play audio for player agents
        if agent.is_none() { continue; }
        
        let world_pos = transform.translation.truncate();
        let tile_pos = isometric_settings.world_to_tile(world_pos);
        
        if tile_pos.x >= 0 && tile_pos.y >= 0 &&
           tile_pos.x < weather_grid.width as i32 &&
           tile_pos.y < weather_grid.height as i32 {
            
            let x = tile_pos.x as usize;
            let y = tile_pos.y as usize;
            
            let wetness = weather_grid.get_wetness(x, y);
            let snow = weather_grid.get_snow(x, y);
            
            // Play appropriate footstep sounds based on weather conditions
            if snow > 0.4 {
                // Crunchy snow footsteps (less frequent to avoid spam)
                if fastrand::f32() < 0.1 { // 10% chance per movement
                    audio_events.write(crate::core::AudioEvent {
                        sound: crate::core::AudioType::FootstepSnow,
                        volume: 0.3,
                    });
                }
            } else if wetness > 0.5 {
                // Splashing in puddles
                if fastrand::f32() < 0.15 { // 15% chance
                    audio_events.write(crate::core::AudioEvent {
                        sound: crate::core::AudioType::FootstepWet,
                        volume: 0.4,
                    });
                }
            }
        }
    }
}

// === SETUP SYSTEM ===

pub fn setup_weather_tile_system(
    mut commands: Commands,
    isometric_settings: Res<IsometricSettings>,
) {
    let weather_grid = WeatherTileGrid::new(
        isometric_settings.map_width as usize,
        isometric_settings.map_height as usize,
    );
    
    commands.insert_resource(weather_grid);
    info!("Weather tile effects system initialized");
}

// === WEATHER ACCUMULATION SYSTEM ===

pub fn update_weather_tile_accumulation(
    mut weather_grid: ResMut<WeatherTileGrid>,
    weather: Res<WeatherSystem>,
    time: Res<Time>,
) {
    weather_grid.update_timer.tick(time.delta());
    
    if !weather_grid.update_timer.finished() {
        return;
    }

    let accumulation_rate = match weather.current_weather {
        WeatherState::LightRain => 0.02 * weather.intensity,
        WeatherState::HeavyRain => 0.05 * weather.intensity,
        WeatherState::Snow => 0.03 * weather.intensity,
        WeatherState::ClearSkies => 0.0,
    };

    let drying_rate = match weather.current_weather {
        WeatherState::ClearSkies => 0.01, // Faster drying in clear weather
        WeatherState::Snow => 0.001,      // Very slow drying in snow
        _ => 0.005,                       // Normal drying rate
    };

    // Update wetness and snow for all tiles
    for y in 0..weather_grid.height {
        for x in 0..weather_grid.width {
            match weather.current_weather {
                WeatherState::LightRain | WeatherState::HeavyRain => {
                    // Increase wetness
                    let current_wet = weather_grid.get_wetness(x, y);
                    weather_grid.set_wetness(x, y, current_wet + accumulation_rate);
                    
                    // Rain melts snow
                    let current_snow = weather_grid.get_snow(x, y);
                    weather_grid.set_snow(x, y, (current_snow - accumulation_rate * 2.0).max(0.0));
                },
                WeatherState::Snow => {
                    // Increase snow accumulation
                    let current_snow = weather_grid.get_snow(x, y);
                    weather_grid.set_snow(x, y, current_snow + accumulation_rate);
                },
                WeatherState::ClearSkies => {
                    // Gradual drying and snow melting
                    let current_wet = weather_grid.get_wetness(x, y);
                    weather_grid.set_wetness(x, y, (current_wet - drying_rate).max(0.0));
                    
                    let current_snow = weather_grid.get_snow(x, y);
                    weather_grid.set_snow(x, y, (current_snow - drying_rate * 0.5).max(0.0));
                },
            }
        }
    }
}

// === TILE PROPERTY MODIFICATION SYSTEM ===

pub fn apply_weather_effects_to_tiles(
    mut tile_query: Query<(&mut TileProperties, &TileTextureIndex, &TilePos)>,
    weather_grid: Res<WeatherTileGrid>,
) {
    if !weather_grid.is_changed() {
        return;
    }

    for (mut properties, texture_index, tile_pos) in tile_query.iter_mut() {
        let x = tile_pos.x as usize;
        let y = tile_pos.y as usize;
        
        let wetness = weather_grid.get_wetness(x, y);
        let snow = weather_grid.get_snow(x, y);
        
        // Calculate movement cost modifier based on weather
        let base_cost = match texture_index_to_tile_type(texture_index.0) {
            TileType::Grass => 1.2,
            TileType::Road | TileType::Asphalt => 0.8,
            TileType::Concrete | TileType::Sidewalk => 1.0,
            TileType::Mud => 1.8,
            _ => 1.0,
        };
        
        let mut final_cost = base_cost;
        
        // Wet tiles become slippery (lower movement cost = faster movement)
        if wetness > 0.3 {
            final_cost *= 0.9 - (wetness * 0.2); // Up to 30% faster (more slippery)
        }
        
        // Snow slows movement significantly
        if snow > 0.2 {
            final_cost *= 1.0 + (snow * 0.5); // Up to 50% slower
        }
        
        // Certain tile types are more affected by weather
        let tile_type = texture_index_to_tile_type(texture_index.0);
        match tile_type {
            TileType::Road | TileType::Asphalt => {
                // Roads become very slippery when wet
                if wetness > 0.4 {
                    final_cost *= 0.7; // Even more slippery on wet roads
                }
            },
            TileType::Grass => {
                // Grass becomes muddy when wet
                if wetness > 0.5 {
                    final_cost *= 1.4; // Muddy grass is much slower
                }
            },
            _ => {},
        }
        
        properties.movement_cost = final_cost.clamp(0.1, 5.0);
        
        // Update cover effectiveness - wet cover is less effective
        if wetness > 0.3 {
            let cover_reduction = 1.0 - (wetness * 0.2); // Up to 20% less cover
            properties.provides_cover *= cover_reduction;
        }
        
        // Snow provides slight cover bonus
        if snow > 0.4 {
            let cover_bonus = 1.0 + (snow * 0.1); // Up to 10% more cover
            properties.provides_cover = (properties.provides_cover * cover_bonus).min(1.0);
        }
    }
}


// === VISUAL WEATHER EFFECTS ===

pub fn update_tile_visuals_for_weather(
    weather_grid: Res<WeatherTileGrid>,
    tilemap_query: Query<&TileStorage, With<crate::systems::tilemap::IsometricMap>>,
    mut tile_query: Query<&mut Sprite, With<TileTextureIndex>>,
    mut audio_events: EventWriter<crate::core::AudioEvent>,
) {
    if !weather_grid.is_changed() {
        return;
    }

    let Ok(tile_storage) = tilemap_query.single() else { return; };
    
    for y in 0..weather_grid.height {
        for x in 0..weather_grid.width {
            let tile_pos = TilePos { x: x as u32, y: y as u32 };
            
            if let Some(tile_entity) = tile_storage.get(&tile_pos) {
                if let Ok(mut sprite) = tile_query.get_mut(tile_entity) {
                    let wetness = weather_grid.get_wetness(x, y);
                    let snow = weather_grid.get_snow(x, y);
            
            // Play appropriate footstep sounds based on weather conditions
            if snow > 0.4 {
                // Crunchy snow footsteps (less frequent to avoid spam)
                if fastrand::f32() < 0.1 { // 10% chance per movement
                    audio_events.write(crate::core::AudioEvent { sound: crate::core::AudioType::FootstepSnow,volume: 0.3,});
                }
            } else if wetness > 0.5 {
                // Splashing in puddles
                if fastrand::f32() < 0.15 { // 15% chance
                    audio_events.write(crate::core::AudioEvent {sound: crate::core::AudioType::FootstepWet, volume: 0.4,});
                }
            }
        }
    }
}
    }
}


// === CLEANUP SYSTEM ===

pub fn cleanup_weather_tile_effects(
    mut commands: Commands,
    weather_affected_query: Query<Entity, With<WeatherAffectedTile>>,
) {
    for entity in weather_affected_query.iter() {
        commands.entity(entity).remove::<WeatherAffectedTile>();
    }
}

