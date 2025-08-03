// src/systems/colored_lighting.rs - Enhanced colored lighting system
use bevy::prelude::*;
use bevy_light_2d::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use crate::core::*;

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

// === LIGHTING BEHAVIOR SYSTEM ===

pub fn colored_light_behavior_system(
    mut light_query: Query<(&mut ColoredLight, &mut Sprite, &crate::core::DeviceState, &Health), Without<TileTextureIndex>>,
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
}


// === SINGLE UNIFIED SPAWN FUNCTION ===

pub fn spawn_colored_light(
    commands: &mut Commands,
    position: Vec2,
    light_type: LightType,
    network_id: Option<String>,
    power_grid: Option<&mut ResMut<crate::core::PowerGrid>>,
) -> Entity {
    info!("Spawning {:?} light at {:?}", light_type, position); // ADD THIS

    let colored_light = ColoredLight::new(light_type);
    let z_height = match light_type {
        LightType::LaserSight | LightType::MonitorGlow => 1.0,
        LightType::FireLight => 3.0,
        LightType::NeonSign | LightType::EmergencyLight => 5.0,
        LightType::HazardLight => 6.0,
        LightType::StreetLight | LightType::SecurityLight => 10.0,
        LightType::AlarmLight => 12.0,
        LightType::OfficeLight | LightType::LaboratoryLight | LightType::IndustrialLight => 15.0,
    };

    let sprite_size = match light_type {
        LightType::LaserSight => Vec2::new(200.0, 2.0),
        LightType::MonitorGlow => Vec2::new(12.0, 8.0),
        LightType::EmergencyLight | LightType::HazardLight => Vec2::new(6.0, 6.0),
        LightType::StreetLight => Vec2::new(8.0, 24.0),
        LightType::SecurityLight | LightType::AlarmLight => Vec2::new(12.0, 8.0),
        LightType::OfficeLight => Vec2::new(16.0, 4.0),
        LightType::LaboratoryLight => Vec2::new(20.0, 6.0),
        LightType::IndustrialLight => Vec2::new(24.0, 12.0),
        LightType::NeonSign => Vec2::new(32.0, 16.0), // Default size
        LightType::FireLight => Vec2::new(8.0, 12.0),
    };

    let health = match light_type {
        LightType::MonitorGlow => 15.0,
        LightType::NeonSign | LightType::EmergencyLight => 20.0,
        LightType::FireLight => 0.0, // Fire doesn't have health
        LightType::OfficeLight => 30.0,
        LightType::HazardLight | LightType::AlarmLight => 35.0,
        LightType::LaboratoryLight => 40.0,
        LightType::StreetLight => 50.0,
        LightType::SecurityLight => 75.0,
        LightType::IndustrialLight => 100.0,
        LightType::LaserSight => 0.0, // Attached to weapons
    };

    let device_type = match light_type {
        LightType::SecurityLight => DeviceType::Camera,
        LightType::AlarmLight => DeviceType::AlarmPanel,
        LightType::NeonSign => DeviceType::Billboard,
        _ => DeviceType::StreetLight,
    };

    let mut entity_commands = commands.spawn((
        Transform::from_translation(position.extend(z_height)),
        colored_light.clone(),
        Sprite {
            color: colored_light.base_color,
            custom_size: Some(sprite_size),
            ..default()
        },
        PointLight2d {
            intensity: z_height / 3.0,
            radius: z_height * 10.0,
            falloff: 1.0,
            cast_shadows: true,
            color: colored_light.base_color,
        },
    ));

    // Add physics only if it has health
    if health > 0.0 {
        entity_commands.insert((
            Health(health),
            bevy_rapier2d::prelude::RigidBody::Fixed,
            bevy_rapier2d::prelude::Collider::cuboid(sprite_size.x * 0.5, sprite_size.y * 0.5),
        ));
    }

    // Add shadow caster for tall lights
    if matches!(light_type, LightType::StreetLight | LightType::SecurityLight | LightType::IndustrialLight) {
        let shadow_height = match light_type {
            LightType::IndustrialLight => 12.0,
            LightType::StreetLight => 8.0,
            LightType::SecurityLight => 6.0,
            _ => 4.0,
        };
        // entity_commands.insert(ShadowCaster { height: shadow_height, opacity: 0.3 });
    }

    // Add hackable components if networked
    if let Some(network_id) = network_id {
        entity_commands.insert((
            crate::core::Hackable::new(device_type).with_network(network_id.clone()),
            crate::core::DeviceState::new(device_type),
        ));

        // Add to power network
        if let Some(power_grid) = power_grid {
            power_grid.networks.entry(network_id.clone())
                .or_insert_with(|| crate::core::PowerNetwork::new(network_id.clone()));
            
            if let Some(network) = power_grid.networks.get_mut(&network_id) {
                network.connected_devices.insert(entity_commands.id());
            }
        }
    } else if health > 0.0 && !matches!(light_type, LightType::FireLight | LightType::LaserSight) {
        // Non-networked but still hackable (battery powered)
        entity_commands.insert((
            crate::core::Hackable::new(device_type),
            crate::core::DeviceState::new(device_type),
        ));
    }

    entity_commands.id()
}

