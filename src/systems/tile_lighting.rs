// src/systems/tile_lighting.rs - Efficient tile-based lighting system
use bevy::prelude::*;
use bevy_light_2d::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use crate::core::*;
use crate::systems::tilemap::{IsometricSettings};
use crate::systems::tile_properties::{TileType, TileProperties};

// === LIGHTING COMPONENTS ===

#[derive(Component)]
pub struct TileLight {
    pub intensity: f32,
    pub radius: f32,
    pub color: Color,
    pub flicker: Option<FlickerSettings>,
    pub powered: bool, // Links to power grid
}

#[derive(Clone)]
pub struct FlickerSettings {
    pub frequency: f32,
    pub intensity_variance: f32,
}

#[derive(Component)]
pub struct ShadowCaster {
    pub height: f32, // How tall the obstacle is (affects shadow length)
    pub opacity: f32, // How much light it blocks (0.0 - 1.0)
}

// === LIGHTING RESOURCES ===

#[derive(Default, Resource)]
pub struct TileLightingGrid {
    pub width: usize,
    pub height: usize,
    pub light_levels: Vec<f32>, // 0.0 - 1.0 per tile
    pub shadow_levels: Vec<f32>, // 0.0 - 1.0 shadow intensity
    pub dirty: bool,
}

impl TileLightingGrid {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            light_levels: vec![0.0; width * height],
            shadow_levels: vec![0.0; width * height],
            dirty: true,
        }
    }

    pub fn get_light_level(&self, x: usize, y: usize) -> f32 {
        if x < self.width && y < self.height {
            self.light_levels[y * self.width + x]
        } else {
            0.0
        }
    }

    pub fn set_light_level(&mut self, x: usize, y: usize, level: f32) {
        if x < self.width && y < self.height {
            self.light_levels[y * self.width + x] = level.clamp(0.0, 1.0);
        }
    }

    pub fn get_shadow_level(&self, x: usize, y: usize) -> f32 {
        if x < self.width && y < self.height {
            self.shadow_levels[y * self.width + x]
        } else {
            0.0
        }
    }

    pub fn set_shadow_level(&mut self, x: usize, y: usize, level: f32) {
        if x < self.width && y < self.height {
            self.shadow_levels[y * self.width + x] = level.clamp(0.0, 1.0);
        }
    }

    pub fn clear(&mut self) {
        self.light_levels.fill(0.0);
        self.shadow_levels.fill(0.0);
        self.dirty = true;
    }
}

// === SETUP SYSTEM ===

pub fn setup_tile_lighting_system(
    mut commands: Commands,
    isometric_settings: Res<IsometricSettings>,
) {
    let lighting_grid = TileLightingGrid::new(
        isometric_settings.map_width as usize,
        isometric_settings.map_height as usize,
    );
    
    commands.insert_resource(lighting_grid);
    info!("Tile lighting system initialized: {}x{}", 
          isometric_settings.map_width, isometric_settings.map_height);
}

// === LIGHT CALCULATION SYSTEM ===

pub fn calculate_tile_lighting(
    mut lighting_grid: ResMut<TileLightingGrid>,
    light_query: Query<(&Transform, &TileLight), Without<MarkedForDespawn>>,
    shadow_query: Query<(&Transform, &ShadowCaster), Without<TileLight>>,
    isometric_settings: Res<IsometricSettings>,
    day_night: Res<crate::core::DayNightCycle>,
    time: Res<Time>,
) {
    if !lighting_grid.dirty && !day_night.is_changed() {
        return; // Skip expensive calculation if nothing changed
    }

    // Clear previous lighting
    lighting_grid.clear();

    // Apply ambient lighting based on time of day
    let ambient_level = day_night.get_visibility_modifier() * 0.3; // Base ambient
    for level in lighting_grid.light_levels.iter_mut() {
        *level = ambient_level;
    }

    // Calculate shadows first (they affect light propagation)
    calculate_shadows(&mut lighting_grid, &shadow_query, &isometric_settings, &day_night);

    // Calculate light from each source
    for (transform, tile_light) in light_query.iter() {
        if !tile_light.powered { continue; }

        let world_pos = transform.translation.truncate();
        let light_tile = isometric_settings.world_to_tile(world_pos);
        
        // Apply flicker effect
        let current_intensity = if let Some(flicker) = &tile_light.flicker {
            let flicker_factor = (time.elapsed_secs() * flicker.frequency).sin() * 
                                flicker.intensity_variance + 1.0;
            tile_light.intensity * flicker_factor.clamp(0.1, 1.0)
        } else {
            tile_light.intensity
        };

        // Light propagation using simple distance falloff
        propagate_light(
            &mut lighting_grid,
            light_tile,
            current_intensity,
            tile_light.radius,
            &isometric_settings,
        );
    }

    lighting_grid.dirty = false;
}

fn calculate_shadows(
    lighting_grid: &mut TileLightingGrid,
    shadow_query: &Query<(&Transform, &ShadowCaster), Without<TileLight>>,
    isometric_settings: &IsometricSettings,
    day_night: &crate::core::DayNightCycle,
) {
    // Only cast shadows during day (when sun is strong enough)
    if !matches!(day_night.current_period, crate::core::TimeOfDay::Day) {
        return;
    }

    // Simple directional shadows from "sun"
    let sun_direction = Vec2::new(0.5, -0.8).normalize(); // Northwest to southeast
    
    for (transform, shadow_caster) in shadow_query.iter() {
        let world_pos = transform.translation.truncate();
        let caster_tile = isometric_settings.world_to_tile(world_pos);
        
        // Cast shadow in sun direction
        let shadow_length = shadow_caster.height * 2.0; // Taller objects = longer shadows
        cast_shadow(
            lighting_grid,
            caster_tile,
            sun_direction,
            shadow_length,
            shadow_caster.opacity,
            isometric_settings,
        );
    }
}

fn propagate_light(
    lighting_grid: &mut TileLightingGrid,
    center: IVec2,
    intensity: f32,
    radius: f32,
    isometric_settings: &IsometricSettings,
) {
    let radius_tiles = (radius / (isometric_settings.tile_width * 0.5)) as i32;
    
    for dy in -radius_tiles..=radius_tiles {
        for dx in -radius_tiles..=radius_tiles {
            let tile_pos = IVec2::new(center.x + dx, center.y + dy);
            
            if tile_pos.x < 0 || tile_pos.y < 0 || 
               tile_pos.x >= lighting_grid.width as i32 || 
               tile_pos.y >= lighting_grid.height as i32 {
                continue;
            }

            let distance = (dx * dx + dy * dy) as f32;
            let max_distance = radius_tiles * radius_tiles;
            
            if distance <= max_distance as f32 {
                // Simple distance falloff
                let falloff = 1.0 - (distance / max_distance as f32);
                let light_contribution = intensity * falloff;
                
                let x = tile_pos.x as usize;
                let y = tile_pos.y as usize;
                
                // Apply shadow reduction
                let shadow_factor = 1.0 - lighting_grid.get_shadow_level(x, y);
                let final_light = light_contribution * shadow_factor;
                
                // Additive lighting (multiple sources combine)
                let current_light = lighting_grid.get_light_level(x, y);
                lighting_grid.set_light_level(x, y, (current_light + final_light).min(1.0));
            }
        }
    }
}

fn cast_shadow(
    lighting_grid: &mut TileLightingGrid,
    caster_pos: IVec2,
    direction: Vec2,
    length: f32,
    opacity: f32,
    isometric_settings: &IsometricSettings,
) {
    let length_tiles = (length / (isometric_settings.tile_width * 0.5)) as i32;
    
    for step in 1..=length_tiles {
        let shadow_pos = caster_pos + IVec2::new(
            (direction.x * step as f32) as i32,
            (direction.y * step as f32) as i32,
        );
        
        if shadow_pos.x < 0 || shadow_pos.y < 0 || 
           shadow_pos.x >= lighting_grid.width as i32 || 
           shadow_pos.y >= lighting_grid.height as i32 {
            break;
        }

        // Shadow intensity falls off with distance
        let distance_factor = 1.0 - (step as f32 / length_tiles as f32);
        let shadow_intensity = opacity * distance_factor;
        
        let x = shadow_pos.x as usize;
        let y = shadow_pos.y as usize;
        
        let current_shadow = lighting_grid.get_shadow_level(x, y);
        lighting_grid.set_shadow_level(x, y, (current_shadow + shadow_intensity).min(1.0));
    }
}

// === VISUAL UPDATE SYSTEM ===

pub fn update_tile_visuals_from_lighting(
    lighting_grid: Res<TileLightingGrid>,
    tilemap_query: Query<&TileStorage, With<crate::systems::tilemap::IsometricMap>>,
    mut tile_query: Query<&mut Sprite, With<TileTextureIndex>>,
    day_night: Res<crate::core::DayNightCycle>,
) {
    if !lighting_grid.is_changed() && !day_night.is_changed() {
        return;
    }

    let Ok(tile_storage) = tilemap_query.single() else { return; };
    
    // Calculate base darkness level based on time of day
    let base_darkness = match day_night.current_period {
        crate::core::TimeOfDay::Day => 1.0,     // Full brightness during day
        crate::core::TimeOfDay::Dusk => 0.7,    // Slight dimming at dusk
        crate::core::TimeOfDay::Night => 0.2,   // Very dark at night
        crate::core::TimeOfDay::Dawn => 0.6,    // Dawn lighting
    };
    
    for y in 0..lighting_grid.height {
        for x in 0..lighting_grid.width {
            let tile_pos = TilePos { x: x as u32, y: y as u32 };
            
            if let Some(tile_entity) = tile_storage.get(&tile_pos) {
                if let Ok(mut sprite) = tile_query.get_mut(tile_entity) {
                    let light_level = lighting_grid.get_light_level(x, y);
                    let shadow_level = lighting_grid.get_shadow_level(x, y);
                    
                    // Combine base darkness + artificial lighting - shadows
                    let final_brightness = (base_darkness + light_level - shadow_level).clamp(0.1, 1.0);
                    
                    // Apply brightness to tile color
                    sprite.color = Color::srgb(final_brightness, final_brightness, final_brightness);
                }
            }
        }
    }
}

// === LIGHT ENTITY SPAWNING ===

pub fn spawn_street_light(
    commands: &mut Commands,
    position: Vec2,
    network_id: String,
    power_grid: &mut ResMut<crate::core::PowerGrid>,
) -> Entity {
    // Create the network if it doesn't exist
    power_grid.networks.entry(network_id.clone())
        .or_insert_with(|| crate::core::PowerNetwork::new(network_id.clone()));

    let entity = commands.spawn((
        Transform::from_translation(position.extend(10.0)),
        TileLight {
            intensity: 0.8,
            radius: 120.0,
            color: Color::srgb(1.0, 0.9, 0.7), // Warm white
            flicker: None,
            powered: true,
        },
        ShadowCaster {
            height: 8.0,
            opacity: 0.3,
        },
        // Visual sprite
        Sprite {
            color: Color::srgb(0.8, 0.8, 0.6),
            custom_size: Some(Vec2::new(8.0, 24.0)),
            ..default()
        },
        // Make it hackable and networked
        crate::core::Hackable::new(crate::core::DeviceType::StreetLight)
            .with_network(network_id.clone()),
        crate::core::DeviceState::new(crate::core::DeviceType::StreetLight),
        // Add health for destruction
        Health(50.0),
        // Add physics collider for interactions
        bevy_rapier2d::prelude::RigidBody::Fixed,
        bevy_rapier2d::prelude::Collider::cuboid(4.0, 12.0),
    )).id();

    // Add to power network
    if let Some(network) = power_grid.networks.get_mut(&network_id) {
        network.connected_devices.insert(entity);
    }

    entity
}

pub fn spawn_flickering_lamp(
    commands: &mut Commands,
    position: Vec2,
) -> Entity {
    commands.spawn((
        Transform::from_translation(position.extend(10.0)),
        TileLight {
            intensity: 0.6,
            radius: 80.0,
            color: Color::srgb(1.0, 0.8, 0.6),
            flicker: Some(FlickerSettings {
                frequency: 3.0,
                intensity_variance: 0.3,
            }),
            powered: true,
        },
        Sprite {
            color: Color::srgb(0.9, 0.7, 0.5),
            custom_size: Some(Vec2::new(6.0, 18.0)),
            ..default()
        },
    )).id()
}

// === EMERGENCY LIGHTING (NON-NETWORKED) ===
pub fn spawn_emergency_light(
    commands: &mut Commands,
    position: Vec2,
) -> Entity {
    commands.spawn((
        Transform::from_translation(position.extend(8.0)),
        TileLight {
            intensity: 0.4,
            radius: 60.0,
            color: Color::srgb(1.0, 0.6, 0.6), // Reddish emergency light
            flicker: Some(FlickerSettings {
                frequency: 2.0,
                intensity_variance: 0.4,
            }),
            powered: true, // Always on (battery powered)
        },
        Sprite {
            color: Color::srgb(0.8, 0.4, 0.4),
            custom_size: Some(Vec2::new(6.0, 6.0)),
            ..default()
        },
        Health(25.0), // Easier to destroy
        bevy_rapier2d::prelude::RigidBody::Fixed,
        bevy_rapier2d::prelude::Collider::ball(3.0),
        // Not hackable - physical destruction only
    )).id()
}

// === STREET LIGHT BEHAVIOR SYSTEM ===
pub fn street_light_behavior_system(
    mut light_query: Query<(&mut TileLight, &mut Sprite, &crate::core::DeviceState, &Health), Without<TileTextureIndex>>,
    mut lighting_grid: ResMut<TileLightingGrid>,
) {
    let mut any_changed = false;

    for (mut tile_light, mut sprite, device_state, health) in light_query.iter_mut() {
        let was_powered = tile_light.powered;
        
        // Light is only on if powered, operational, and not destroyed
        let should_be_on = device_state.powered && device_state.operational && health.0 > 0.0;
        
        if tile_light.powered != should_be_on {
            tile_light.powered = should_be_on;
            any_changed = true;
        }

        // Update visual appearance based on state
        if should_be_on {
            // Bright yellow when working
            sprite.color = Color::srgb(1.0, 1.0, 0.7);
            tile_light.intensity = 0.8;
        } else if health.0 <= 0.0 {
            // Dark gray when destroyed
            sprite.color = Color::srgb(0.2, 0.2, 0.2);
            tile_light.intensity = 0.0;
        } else {
            // Dim when hacked/unpowered but not destroyed
            sprite.color = Color::srgb(0.4, 0.4, 0.3);
            tile_light.intensity = 0.1; // Very dim emergency lighting
        }
    }

    if any_changed {
        lighting_grid.dirty = true;
    }
}

// === POWER INTEGRATION ===

pub fn update_lights_from_power_grid(
    mut light_query: Query<(&mut TileLight, &Hackable)>,
    power_grid: Res<crate::core::PowerGrid>,
    mut lighting_grid: ResMut<TileLightingGrid>,
) {
    let mut any_changed = false;
    
    for (mut tile_light, hackable) in light_query.iter_mut() {
        let should_be_powered = if let Some(network_id) = &hackable.network_id {
            // Check if the network exists and is powered
            power_grid.networks.get(network_id)
                .map(|network| network.powered)
                .unwrap_or(false)
        } else {
            // No network connection - assume always powered
            true
        };
        
        if tile_light.powered != should_be_powered {
            tile_light.powered = should_be_powered;
            any_changed = true;
        }
    }
    
    if any_changed {
        lighting_grid.dirty = true;
    }
}

// === WEATHER LIGHTING EFFECTS ===

pub fn apply_weather_lighting_effects(
    weather: Res<crate::systems::weather::WeatherSystem>,
    mut lighting_grid: ResMut<TileLightingGrid>,
) {
    if !weather.is_changed() { return; }
    
    // Weather reduces overall visibility
    let weather_modifier = match weather.current_weather {
        crate::systems::weather::WeatherState::ClearSkies => 1.0,
        crate::systems::weather::WeatherState::LightRain => 0.85,
        crate::systems::weather::WeatherState::HeavyRain => 0.7,
        crate::systems::weather::WeatherState::Snow => 0.8,
    };
    
    // Apply weather dimming to all light levels
    for level in lighting_grid.light_levels.iter_mut() {
        *level *= weather_modifier;
    }
    
    lighting_grid.dirty = true;
}


// === ENTITY LIGHTING SYSTEM ===
// This applies lighting to non-tile entities (agents, enemies, etc.)
pub fn update_entity_lighting_from_grid(
    lighting_grid: Res<TileLightingGrid>,
    isometric_settings: Res<crate::systems::tilemap::IsometricSettings>,
    mut entity_query: Query<(&Transform, &mut Sprite), (Or<(With<Agent>, With<Enemy>, With<Civilian>)>, Without<TileTextureIndex>)>,
    day_night: Res<crate::core::DayNightCycle>,
) {
    if !lighting_grid.is_changed() && !day_night.is_changed() {
        return;
    }

    let base_darkness = match day_night.current_period {
        crate::core::TimeOfDay::Day => 1.0,
        crate::core::TimeOfDay::Dusk => 0.7,
        crate::core::TimeOfDay::Night => 0.2,
        crate::core::TimeOfDay::Dawn => 0.6,
    };

    for (transform, mut sprite) in entity_query.iter_mut() {
        let world_pos = transform.translation.truncate();
        let tile_pos = isometric_settings.world_to_tile(world_pos);
        
        if tile_pos.x >= 0 && tile_pos.y >= 0 &&
           tile_pos.x < lighting_grid.width as i32 &&
           tile_pos.y < lighting_grid.height as i32 {
            
            let x = tile_pos.x as usize;
            let y = tile_pos.y as usize;
            
            let light_level = lighting_grid.get_light_level(x, y);
            let shadow_level = lighting_grid.get_shadow_level(x, y);
            
            let final_brightness = (base_darkness + light_level - shadow_level).clamp(0.1, 1.0);
            
            // Apply lighting to entity
            sprite.color = Color::srgb(final_brightness, final_brightness, final_brightness);
        }
    }
}


// === LIGHT DESTRUCTION SYSTEM ===
pub fn light_destruction_system(
    mut commands: Commands,
    mut light_query: Query<(Entity, &mut Health, &mut TileLight), With<TileLight>>,
    mut combat_events: EventReader<crate::core::CombatEvent>,
    mut explosion_events: EventReader<crate::core::GrenadeEvent>,
    mut lighting_grid: ResMut<TileLightingGrid>,
    mut audio_events: EventWriter<crate::core::AudioEvent>,
) {
    let mut lights_changed = false;

    // Handle direct combat damage
    for event in combat_events.read() {
        if let Ok((entity, mut health, mut light)) = light_query.get_mut(event.target) {
            if event.hit {
                health.0 -= event.damage;
                
                if health.0 <= 0.0 {
                    light.powered = false;
                    light.intensity = 0.0;
                    lights_changed = true;
                    
                    // Sparks and glass breaking sound
                    audio_events.write(crate::core::AudioEvent {
                        sound: crate::core::AudioType::GlassBreak,
                        volume: 0.6,
                    });
                    
                    info!("Street light destroyed by gunfire!");
                }
            }
        }
    }

    // Handle explosion damage
    for explosion in explosion_events.read() {
        for (entity, mut health, mut light) in light_query.iter_mut() {
            // Calculate distance from explosion
            let light_pos = Vec2::ZERO; // You'd get this from Transform component
            let distance = explosion.target_pos.distance(light_pos);
            
            if distance <= explosion.explosion_radius {
                let damage_falloff = 1.0 - (distance / explosion.explosion_radius);
                let damage = explosion.damage * damage_falloff;
                
                health.0 -= damage;
                
                if health.0 <= 0.0 {
                    light.powered = false;
                    light.intensity = 0.0;
                    lights_changed = true;
                    
                    audio_events.send(crate::core::AudioEvent {
                        sound: crate::core::AudioType::Explosion,
                        volume: 0.4,
                    });
                }
            }
        }
    }

    if lights_changed {
        lighting_grid.dirty = true;
    }
}