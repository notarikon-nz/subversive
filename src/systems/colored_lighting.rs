// src/systems/colored_lighting.rs - Enhanced colored lighting system
use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use crate::core::*;
use crate::systems::tilemap::IsometricSettings;
use crate::systems::tile_lighting::{TileLight, TileLightingGrid, ShadowCaster};

// === COLORED LIGHTING COMPONENTS ===

#[derive(Component, Clone)]
pub struct ColoredLight {
    pub base_color: Color,
    pub intensity: f32,
    pub radius: f32,
    pub light_type: LightType,
    pub temperature: f32,     // Kelvin temperature for realistic lighting
    pub saturation: f32,     // 0.0 = white light, 1.0 = full color
    pub flicker: Option<ColorFlicker>,
}

#[derive(Clone)]
pub struct ColorFlicker {
    pub frequency: f32,
    pub color_variance: f32,    // How much the color shifts
    pub intensity_variance: f32, // How much the brightness shifts
}

#[derive(Debug, Clone, Copy)]
pub enum LightType {
    // Infrastructure lighting
    StreetLight,        // Warm white (3000K)
    SecurityLight,      // Cool white (6000K)  
    EmergencyLight,     // Red (emergency situations)
    
    // Facility lighting
    OfficeLight,        // Neutral white (4000K)
    IndustrialLight,    // Cool white, high intensity
    LaboratoryLight,    // Very cool white (7000K)
    
    // Atmospheric lighting
    NeonSign,          // Vibrant colors (customizable)
    FireLight,         // Warm orange/red
    MonitorGlow,       // Blue/cyan
    
    // Special effects
    AlarmLight,        // Flashing red
    HazardLight,       // Yellow/orange warning
    LaserSight,        // Intense red beam
}

// === COLORED LIGHTING GRID ===

#[derive(Resource)]
pub struct ColoredLightingGrid {
    pub width: usize,
    pub height: usize,
    pub red_channel: Vec<f32>,
    pub green_channel: Vec<f32>,
    pub blue_channel: Vec<f32>,
    pub intensity_channel: Vec<f32>,
    pub dirty: bool,
}

impl ColoredLightingGrid {
    pub fn new(width: usize, height: usize) -> Self {
        let size = width * height;
        Self {
            width,
            height,
            red_channel: vec![0.0; size],
            green_channel: vec![0.0; size],
            blue_channel: vec![0.0; size],
            intensity_channel: vec![0.0; size],
            dirty: true,
        }
    }

    pub fn get_color(&self, x: usize, y: usize) -> Color {
        if x < self.width && y < self.height {
            let idx = y * self.width + x;
            Color::srgb(
                self.red_channel[idx],
                self.green_channel[idx],
                self.blue_channel[idx]
            )
        } else {
            Color::BLACK
        }
    }

    pub fn get_intensity(&self, x: usize, y: usize) -> f32 {
        if x < self.width && y < self.height {
            self.intensity_channel[y * self.width + x]
        } else {
            0.0
        }
    }

    pub fn add_light(&mut self, x: usize, y: usize, color: Color, intensity: f32) {
        if x < self.width && y < self.height {
            let idx = y * self.width + x;
            let linear_color = LinearRgba::from(color);
            // Additive color blending with intensity weighting
            self.red_channel[idx] += linear_color.red * intensity;
            self.green_channel[idx] += linear_color.green * intensity;
            self.blue_channel[idx] += linear_color.blue * intensity;
            self.intensity_channel[idx] += intensity;
        }
    }

    pub fn clear(&mut self) {
        self.red_channel.fill(0.0);
        self.green_channel.fill(0.0);
        self.blue_channel.fill(0.0);
        self.intensity_channel.fill(0.0);
        self.dirty = true;
    }

    pub fn normalize(&mut self) {
        // Normalize colors to prevent oversaturation
        for i in 0..self.red_channel.len() {
            let max_component = self.red_channel[i]
                .max(self.green_channel[i])
                .max(self.blue_channel[i]);
            
            if max_component > 1.0 {
                self.red_channel[i] /= max_component;
                self.green_channel[i] /= max_component;
                self.blue_channel[i] /= max_component;
            }
        }
    }
}

// === LIGHT TYPE CONFIGURATIONS ===

impl ColoredLight {
    pub fn new(light_type: LightType) -> Self {
        match light_type {
            LightType::StreetLight => Self {
                base_color: Color::srgb(1.0, 0.9, 0.7),
                intensity: 0.8,
                radius: 120.0,
                light_type,
                temperature: 3000.0,
                saturation: 0.3,
                flicker: None,
            },
            
            LightType::SecurityLight => Self {
                base_color: Color::srgb(0.9, 0.95, 1.0),
                intensity: 1.0,
                radius: 150.0,
                light_type,
                temperature: 6000.0,
                saturation: 0.1,
                flicker: None,
            },
            
            LightType::EmergencyLight => Self {
                base_color: Color::srgb(1.0, 0.1, 0.1),
                intensity: 0.6,
                radius: 80.0,
                light_type,
                temperature: 2000.0,
                saturation: 0.9,
                flicker: Some(ColorFlicker {
                    frequency: 0.5,
                    color_variance: 0.2,
                    intensity_variance: 0.4,
                }),
            },
            
            LightType::OfficeLight => Self {
                base_color: Color::srgb(0.95, 0.95, 0.9),
                intensity: 0.7,
                radius: 100.0,
                light_type,
                temperature: 4000.0,
                saturation: 0.1,
                flicker: None,
            },
            
            LightType::LaboratoryLight => Self {
                base_color: Color::srgb(0.8, 0.9, 1.0),
                intensity: 0.9,
                radius: 140.0,
                light_type,
                temperature: 7000.0,
                saturation: 0.2,
                flicker: None,
            },
            
            LightType::NeonSign => Self {
                base_color: Color::srgb(0.0, 1.0, 1.0), // Cyan default
                intensity: 0.5,
                radius: 60.0,
                light_type,
                temperature: 10000.0,
                saturation: 1.0,
                flicker: Some(ColorFlicker {
                    frequency: 8.0,
                    color_variance: 0.1,
                    intensity_variance: 0.3,
                }),
            },
            
            LightType::FireLight => Self {
                base_color: Color::srgb(1.0, 0.4, 0.1),
                intensity: 0.7,
                radius: 90.0,
                light_type,
                temperature: 1800.0,
                saturation: 0.8,
                flicker: Some(ColorFlicker {
                    frequency: 12.0,
                    color_variance: 0.3,
                    intensity_variance: 0.5,
                }),
            },
            
            LightType::MonitorGlow => Self {
                base_color: Color::srgb(0.2, 0.6, 1.0),
                intensity: 0.3,
                radius: 40.0,
                light_type,
                temperature: 9000.0,
                saturation: 0.6,
                flicker: Some(ColorFlicker {
                    frequency: 60.0,
                    color_variance: 0.05,
                    intensity_variance: 0.1,
                }),
            },
            
            LightType::AlarmLight => Self {
                base_color: Color::srgb(1.0, 0.0, 0.0),
                intensity: 0.8,
                radius: 100.0,
                light_type,
                temperature: 2000.0,
                saturation: 1.0,
                flicker: Some(ColorFlicker {
                    frequency: 2.0,
                    color_variance: 0.0,
                    intensity_variance: 1.0, // Full on/off
                }),
            },
            
            LightType::HazardLight => Self {
                base_color: Color::srgb(1.0, 0.8, 0.0),
                intensity: 0.6,
                radius: 80.0,
                light_type,
                temperature: 2500.0,
                saturation: 0.7,
                flicker: Some(ColorFlicker {
                    frequency: 1.0,
                    color_variance: 0.1,
                    intensity_variance: 0.3,
                }),
            },
            
            LightType::IndustrialLight => Self {
                base_color: Color::srgb(0.9, 0.9, 1.0),
                intensity: 1.2,
                radius: 180.0,
                light_type,
                temperature: 5000.0,
                saturation: 0.1,
                flicker: None,
            },
            
            LightType::LaserSight => Self {
                base_color: Color::srgb(1.0, 0.0, 0.0),
                intensity: 2.0,
                radius: 200.0,
                light_type,
                temperature: 6500.0,
                saturation: 1.0,
                flicker: None,
            },
        }
    }
    
    pub fn with_custom_color(mut self, color: Color) -> Self {
        self.base_color = color;
        self.saturation = 1.0;
        self
    }
    
    pub fn with_intensity(mut self, intensity: f32) -> Self {
        self.intensity = intensity;
        self
    }
    
    pub fn with_radius(mut self, radius: f32) -> Self {
        self.radius = radius;
        self
    }
}

// === COLORED LIGHTING CALCULATION SYSTEM ===

pub fn calculate_colored_lighting(
    mut colored_grid: ResMut<ColoredLightingGrid>,
    light_query: Query<(&Transform, &ColoredLight, &crate::core::DeviceState), Without<MarkedForDespawn>>,
    shadow_query: Query<(&Transform, &ShadowCaster), Without<ColoredLight>>,
    isometric_settings: Res<IsometricSettings>,
    day_night: Res<crate::core::DayNightCycle>,
    time: Res<Time>,
) {
    if !colored_grid.dirty && !day_night.is_changed() {
        return;
    }

    // Clear previous lighting
    colored_grid.clear();

    // Apply ambient lighting based on time of day
    let (ambient_color, ambient_intensity) = match day_night.current_period {
        crate::core::TimeOfDay::Day => (Color::srgb(1.0, 1.0, 0.95), 0.4),
        crate::core::TimeOfDay::Dusk => (Color::srgb(1.0, 0.7, 0.5), 0.3),
        crate::core::TimeOfDay::Night => (Color::srgb(0.2, 0.3, 0.5), 0.1),
        crate::core::TimeOfDay::Dawn => (Color::srgb(0.9, 0.8, 0.7), 0.25),
    };

    // Apply ambient lighting to all tiles
    for y in 0..colored_grid.height {
        for x in 0..colored_grid.width {
            colored_grid.add_light(x, y, ambient_color, ambient_intensity);
        }
    }

    // Calculate colored light from each source
    for (transform, colored_light, device_state) in light_query.iter() {
        if !device_state.powered || !device_state.operational {
            continue;
        }

        let world_pos = transform.translation.truncate();
        let light_tile = isometric_settings.world_to_tile(world_pos);
        
        // Apply flicker effect
        let (current_color, current_intensity) = if let Some(flicker) = &colored_light.flicker {
            let time_factor = time.elapsed_secs() * flicker.frequency;
            
            // Intensity flicker
            let intensity_flicker = (time_factor.sin() * flicker.intensity_variance + 1.0).clamp(0.1, 1.0);
            let final_intensity = colored_light.intensity * intensity_flicker;
            
            // Color flicker
            let color_shift = time_factor.cos() * flicker.color_variance;
            let linear_color = LinearRgba::from(colored_light.base_color);
            let flickered_color = Color::srgb(
                (linear_color.red + color_shift).clamp(0.0, 1.0),
                (linear_color.green + color_shift * 0.5).clamp(0.0, 1.0),
                (linear_color.blue + color_shift * 0.3).clamp(0.0, 1.0),
            );
            
            (flickered_color, final_intensity)
        } else {
            (colored_light.base_color, colored_light.intensity)
        };

        // Propagate colored light
        propagate_colored_light(
            &mut colored_grid,
            light_tile,
            current_color,
            current_intensity,
            colored_light.radius,
            &isometric_settings,
        );
    }

    // Normalize to prevent oversaturation
    colored_grid.normalize();
    colored_grid.dirty = false;
}

fn propagate_colored_light(
    colored_grid: &mut ColoredLightingGrid,
    center: IVec2,
    color: Color,
    intensity: f32,
    radius: f32,
    isometric_settings: &IsometricSettings,
) {
    let radius_tiles = (radius / (isometric_settings.tile_width * 0.5)) as i32;
    
    for dy in -radius_tiles..=radius_tiles {
        for dx in -radius_tiles..=radius_tiles {
            let tile_pos = IVec2::new(center.x + dx, center.y + dy);
            
            if tile_pos.x < 0 || tile_pos.y < 0 || 
               tile_pos.x >= colored_grid.width as i32 || 
               tile_pos.y >= colored_grid.height as i32 {
                continue;
            }

            let distance = (dx * dx + dy * dy) as f32;
            let max_distance = radius_tiles * radius_tiles;
            
            if distance <= max_distance as f32 {
                // Smooth distance falloff
                let falloff = 1.0 - (distance.sqrt() / radius_tiles as f32);
                let falloff_smooth = falloff * falloff; // Quadratic falloff for more realistic lighting
                
                let light_contribution = intensity * falloff_smooth;
                
                let x = tile_pos.x as usize;
                let y = tile_pos.y as usize;
                
                colored_grid.add_light(x, y, color, light_contribution);
            }
        }
    }
}

// === COLORED VISUAL UPDATE SYSTEM ===

pub fn update_visuals_from_colored_lighting(
    colored_grid: Res<ColoredLightingGrid>,
    tilemap_query: Query<&TileStorage, With<crate::systems::tilemap::IsometricMap>>,
    mut tile_query: Query<&mut Sprite, With<TileTextureIndex>>,
    day_night: Res<crate::core::DayNightCycle>,
) {
    if !colored_grid.is_changed() && !day_night.is_changed() {
        return;
    }

    let Ok(tile_storage) = tilemap_query.single() else { return; };
    
    for y in 0..colored_grid.height {
        for x in 0..colored_grid.width {
            let tile_pos = TilePos { x: x as u32, y: y as u32 };
            
            if let Some(tile_entity) = tile_storage.get(&tile_pos) {
                if let Ok(mut sprite) = tile_query.get_mut(tile_entity) {
                    let light_color = colored_grid.get_color(x, y);
                    let light_intensity = colored_grid.get_intensity(x, y);
                    
                    // Combine base tile color with lighting
                    let base_brightness = 0.5; // Base tile brightness
                    let final_brightness = (base_brightness + light_intensity).clamp(0.1, 1.0);
                    let linear_color = LinearRgba::from(light_color);
                    // Apply colored lighting
                    sprite.color = Color::srgb(
                        linear_color.red * final_brightness,
                        linear_color.green * final_brightness,
                        linear_color.blue * final_brightness,
                    );
                }
            }
        }
    }
}

// === ENTITY COLORED LIGHTING ===

pub fn update_entity_colored_lighting(
    colored_grid: Res<ColoredLightingGrid>,
    isometric_settings: Res<IsometricSettings>,
    mut entity_query: Query<(&Transform, &mut Sprite), (Or<(With<Agent>, With<Enemy>, With<Civilian>)>, Without<TileTextureIndex>)>,
    day_night: Res<crate::core::DayNightCycle>,
) {
    if !colored_grid.is_changed() && !day_night.is_changed() {
        return;
    }

    for (transform, mut sprite) in entity_query.iter_mut() {
        let world_pos = transform.translation.truncate();
        let tile_pos = isometric_settings.world_to_tile(world_pos);
        
        if tile_pos.x >= 0 && tile_pos.y >= 0 &&
           tile_pos.x < colored_grid.width as i32 &&
           tile_pos.y < colored_grid.height as i32 {
            
            let x = tile_pos.x as usize;
            let y = tile_pos.y as usize;
            
            let light_color = colored_grid.get_color(x, y);
            let light_intensity = colored_grid.get_intensity(x, y);
            
            // Apply colored lighting to entity
            let base_brightness = 0.7; // Entities should be slightly brighter than tiles
            let final_brightness = (base_brightness + light_intensity).clamp(0.2, 1.0);
            let linear_color = LinearRgba::from(light_color);
            sprite.color = Color::srgb(
                linear_color.red * final_brightness,
                linear_color.green * final_brightness,
                linear_color.blue * final_brightness,
            );
        }
    }
}

// === SETUP SYSTEM ===

pub fn setup_colored_lighting_system(
    mut commands: Commands,
    isometric_settings: Res<IsometricSettings>,
) {
    let colored_grid = ColoredLightingGrid::new(
        isometric_settings.map_width as usize,
        isometric_settings.map_height as usize,
    );
    
    commands.insert_resource(colored_grid);
    info!("Colored lighting system initialized: {}x{}", 
          isometric_settings.map_width, isometric_settings.map_height);
}

// === LIGHTING BEHAVIOR SYSTEM ===

pub fn colored_light_behavior_system(
    mut light_query: Query<(&mut ColoredLight, &mut Sprite, &crate::core::DeviceState, &Health), Without<TileTextureIndex>>,
    mut colored_grid: ResMut<crate::systems::colored_lighting::ColoredLightingGrid>,
) {
    let mut any_changed = false;

    for (mut colored_light, mut sprite, device_state, health) in light_query.iter_mut() {
        let should_be_on = device_state.powered && device_state.operational && health.0 > 0.0;
        
        // Update light based on state
        if should_be_on {
            // Light is working - use full color and intensity
            sprite.color = colored_light.base_color;
            // Keep original intensity
        } else if health.0 <= 0.0 {
            // Light is destroyed - dark
            sprite.color = Color::srgb(0.2, 0.2, 0.2);
            colored_light.intensity = 0.0;
            any_changed = true;
        } else {
            // Light is hacked/unpowered - very dim
            let mut linear_color = LinearRgba::from(sprite.color);
            let dim_color = linear_color * 0.3;
            sprite.color = Color::srgb(dim_color.red, dim_color.green, dim_color.blue);
            colored_light.intensity *= 0.1;
            any_changed = true;
        }
    }

    if any_changed {
        colored_grid.dirty = true;
    }
}