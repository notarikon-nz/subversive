// src/systems/weather.rs
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use crate::core::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WeatherState {
    ClearSkies,
    LightRain,
    HeavyRain,
    Snow,
}

#[derive(Resource, Serialize, Deserialize)]
pub struct WeatherSystem {
    pub current_weather: WeatherState,
    pub intensity: f32, // 0.0 - 1.0 for particle density
    pub wind_direction: Vec2,
    pub wind_strength: f32,
}

impl Default for WeatherSystem {
    fn default() -> Self {
        Self {
            current_weather: WeatherState::ClearSkies,
            intensity: 0.5,
            wind_direction: Vec2::new(0.1, -0.3),
            wind_strength: 1.0,
        }
    }
}

// Weather particle component
#[derive(Component)]
pub struct WeatherParticle {
    pub velocity: Vec2,
    pub lifetime: f32,
    pub max_lifetime: f32,
    pub particle_type: WeatherParticleType,
    pub layer: WeatherLayer,    
}

#[derive(Debug, Clone, Copy)]
pub enum WeatherLayer {
    Background,
    Foreground,
}


#[derive(Debug, Clone, Copy)]
pub enum WeatherParticleType {
    Raindrop,
    Snowflake,
}

// Weather overlay component for screen effects
#[derive(Component)]
pub struct WeatherOverlay {
    pub opacity: f32,
}

// Weather particle pool for performance
#[derive(Resource)]
pub struct WeatherParticlePool {
    pub rain_particles: Vec<Entity>,
    pub snow_particles: Vec<Entity>,
    pub max_particles: usize,
    pub spawn_timer: Timer,
}

impl Default for WeatherParticlePool {
    fn default() -> Self {
        Self {
            rain_particles: Vec::new(),
            snow_particles: Vec::new(),
            max_particles: 500,
            spawn_timer: Timer::from_seconds(0.005, TimerMode::Repeating),
        }
    }
}

impl WeatherSystem {
    pub fn determine_weather_for_city(city: &City, day_of_year: u32) -> WeatherState {
        use crate::core::cities::CityTrait;
        
        // Base weather on location (latitude) and season
        let season = match day_of_year % 365 {
            0..=90 => Season::Winter,
            91..=180 => Season::Spring,
            181..=270 => Season::Summer,
            _ => Season::Fall,
        };

        // Latitude-based climate zones
        let climate_zone = if city.coordinates.latitude.abs() > 60.0 {
            ClimateZone::Arctic
        } else if city.coordinates.latitude.abs() > 40.0 {
            ClimateZone::Temperate
        } else {
            ClimateZone::Tropical
        };

        // City traits influence weather
        let has_industrial = city.traits.contains(&CityTrait::HeavyIndustry);
        let has_coastal = city.coordinates.longitude.abs() < 10.0; // Simplified coastal check

        // Weather probability based on climate, season, and city traits
        let rain_chance = match (climate_zone, season) {
            (ClimateZone::Tropical, _) => 0.4,
            (ClimateZone::Temperate, Season::Spring) => 0.6,
            (ClimateZone::Temperate, Season::Summer) => 0.3,
            (ClimateZone::Temperate, Season::Fall) => 0.5,
            (ClimateZone::Temperate, Season::Winter) => 0.4,
            (ClimateZone::Arctic, Season::Winter) => 0.7,
            (ClimateZone::Arctic, _) => 0.3,
        };

        let snow_chance = match (climate_zone, season) {
            (ClimateZone::Arctic, Season::Winter) => 0.6,
            (ClimateZone::Arctic, _) => 0.2,
            (ClimateZone::Temperate, Season::Winter) => 0.3,
            _ => 0.0,
        };

        // Industrial cities have more overcast/rainy weather
        let rain_modifier = if has_industrial { 1.3 } else { 1.0 };
        let coastal_modifier = if has_coastal { 1.2 } else { 1.0 };

        // Simple deterministic random based on city ID and day
        let seed = city.id.len() as u32 + day_of_year;
        let weather_roll = ((seed * 7919) % 100) as f32 / 100.0;

        if weather_roll < snow_chance {
            WeatherState::Snow
        } else if weather_roll < rain_chance * rain_modifier * coastal_modifier * 0.3 {
            WeatherState::HeavyRain
        } else if weather_roll < rain_chance * rain_modifier * coastal_modifier {
            WeatherState::LightRain
        } else {
            WeatherState::ClearSkies
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum Season {
    Spring,
    Summer,
    Fall,
    Winter,
}

#[derive(Debug, Clone, Copy)]
enum ClimateZone {
    Tropical,
    Temperate,
    Arctic,
}

// Initialize weather system for mission
pub fn setup_weather_system(
    mut commands: Commands,
    cities_db: Res<CitiesDatabase>,
    cities_progress: Res<CitiesProgress>,
    global_data: Res<GlobalData>,
    launch_data: Option<Res<MissionLaunchData>>,
) {
    let weather = if let Some(launch_data) = launch_data {
        if let Some(city) = cities_db.get_city(&launch_data.city_id) {
            let day_of_year = global_data.current_day % 365;
            WeatherSystem {
                current_weather: WeatherSystem::determine_weather_for_city(city, day_of_year),
                intensity: fastrand::f32() * 0.5 + 0.5, // 0.5 - 1.0
                wind_direction: Vec2::new(
                    fastrand::f32() * 0.4 - 0.2,
                    -0.3 - fastrand::f32() * 0.4
                ),
                wind_strength: fastrand::f32() * 0.5 + 0.5,
            }
        } else {
            WeatherSystem::default()
        }
    } else {
        WeatherSystem::default()
    };

    info!("Mission weather: {:?} (intensity: {:.2})", weather.current_weather, weather.intensity);
    
    commands.insert_resource(weather);
    commands.insert_resource(WeatherParticlePool::default());
}

// Spawn weather overlay based on current weather
pub fn spawn_weather_overlay(
    mut commands: Commands,
    weather: Res<WeatherSystem>,
    camera_query: Query<&Transform, With<Camera>>,
) {
    let overlay_color = match weather.current_weather {
        WeatherState::ClearSkies => return, // No overlay needed
        WeatherState::LightRain => Color::srgba(0.3, 0.4, 0.6, 0.1),
        WeatherState::HeavyRain => Color::srgba(0.2, 0.3, 0.5, 0.2),
        WeatherState::Snow => Color::srgba(0.9, 0.9, 1.0, 0.15),
    };

    if let Ok(camera_transform) = camera_query.single() {
        commands.spawn((
            Sprite {
                color: overlay_color,
                custom_size: Some(Vec2::new(2560.0, 1440.0)), // Large enough to cover screen
                ..default()
            },
            Transform::from_translation(Vec3::new(
                camera_transform.translation.x,
                camera_transform.translation.y,
                100.0, // High Z to be on top
            )),
            WeatherOverlay { opacity: overlay_color.alpha() },
        ));
    }
}

// Main weather particle system
pub fn weather_particle_system(
    mut commands: Commands,
    time: Res<Time>,
    weather: Res<WeatherSystem>,
    mut particle_pool: ResMut<WeatherParticlePool>,
    camera_query: Query<&Transform, With<Camera>>,
    mut particles: Query<(Entity, &mut Transform, &mut WeatherParticle), (Without<Camera>, Without<MarkedForDespawn>)>,
) {
    if weather.current_weather == WeatherState::ClearSkies {
        // Clean up any existing particles
        for (entity, _, _) in particles.iter() {
            commands.entity(entity).insert(MarkedForDespawn);
        }
        return;
    }

    let Ok(camera_transform) = camera_query.single() else { return; };
    
    // Update spawn timer
    particle_pool.spawn_timer.tick(time.delta());
    
    // Spawn new particles
    if particle_pool.spawn_timer.finished() {
        let (fg_count, bg_count) = match weather.current_weather {
            WeatherState::LightRain => (
                ((6.0 * weather.intensity) as usize).max(1), // Foreground
                ((4.0 * weather.intensity) as usize).max(1)  // Background
            ),
            WeatherState::HeavyRain => (
                ((10.0 * weather.intensity) as usize).max(1), // Foreground
                ((8.0 * weather.intensity) as usize).max(1)   // Background
            ),
            WeatherState::Snow => (
                ((2.0 * weather.intensity) as usize).max(1), // Much smaller batches for snow
                ((1.0 * weather.intensity) as usize).max(1)
            ),
            _ => (0, 0),
        };

        let current_count = particles.iter().count();
        if current_count < particle_pool.max_particles {
            let total_to_spawn = (fg_count + bg_count).min(particle_pool.max_particles - current_count);
            
            // Spawn foreground particles
            for _ in 0..(fg_count.min(total_to_spawn)) {
                spawn_weather_particle(&mut commands, &weather, camera_transform, WeatherLayer::Foreground);
            }
            
            // Spawn background particles
            let remaining = total_to_spawn.saturating_sub(fg_count);
            for _ in 0..(bg_count.min(remaining)) {
                spawn_weather_particle(&mut commands, &weather, camera_transform, WeatherLayer::Background);
            }
        }
    }

    // Update existing particles
    for (entity, mut transform, mut particle) in particles.iter_mut() {
        particle.lifetime -= time.delta_secs();
        
        // Check if particle is far below camera before despawning
        let camera_bottom = camera_transform.translation.y - 400.0; // More generous despawn distance
        if particle.lifetime <= 0.0 || transform.translation.y < camera_bottom {
            commands.entity(entity).insert(MarkedForDespawn);
            continue;
        }

        // Apply velocity with wind influence
        let wind_effect = weather.wind_direction * weather.wind_strength * 0.3;
        let final_velocity = particle.velocity + wind_effect;
        
        transform.translation.x += final_velocity.x * time.delta_secs();
        transform.translation.y += final_velocity.y * time.delta_secs();

        // Apply size/alpha fade for snow with gentler transition
        if matches!(particle.particle_type, WeatherParticleType::Snowflake) {
            let life_ratio = particle.lifetime / particle.max_lifetime;
            let scale = 0.8 + life_ratio * 0.2; // Even subtler scaling
            transform.scale = Vec3::splat(scale);
            
            // Add gentle swaying motion for snow based on layer
            let sway_multiplier = match particle.layer {
                WeatherLayer::Foreground => 1.0,
                WeatherLayer::Background => 0.6, // Less sway for background
            };
            let sway_offset = (time.elapsed_secs() * 0.8 + transform.translation.x * 0.008).sin() * 8.0;
            transform.translation.x += sway_offset * time.delta_secs() * 0.4 * sway_multiplier;
        }
    }
}


// Spawn individual weather particles
fn spawn_weather_particle(
    commands: &mut Commands,
    weather: &WeatherSystem,
    camera_transform: &Transform,
    layer: WeatherLayer,
) {
    let screen_width = 1280.0;
    let screen_height = 720.0;
    
    // Spawn in a wider area with random distribution to avoid banding
    let spawn_width = screen_width + 500.0; // Even wider for better distribution
    let spawn_x = camera_transform.translation.x + 
        (fastrand::f32() - 0.5) * spawn_width;
    
    // Spawn higher above screen to give particles more travel time
    let spawn_y = camera_transform.translation.y + screen_height / 2.0 + 
        150.0 + fastrand::f32() * 300.0; // More height variation

    let (particle_type, velocity, lifetime, color, size, z_layer) = match weather.current_weather {
        WeatherState::LightRain => {
            let base_velocity = Vec2::new(0.0, -350.0) + weather.wind_direction * 80.0;
            let base_lifetime = 5.0;
            
            match layer {
                WeatherLayer::Foreground => (
                    WeatherParticleType::Raindrop,
                    base_velocity, // Full speed
                    base_lifetime,
                    Color::srgba(0.6, 0.7, 1.0, 0.6), // Brighter
                    Vec2::new(1.2, 8.0), // Slightly larger
                    60.0, // Higher Z
                ),
                WeatherLayer::Background => (
                    WeatherParticleType::Raindrop,
                    base_velocity * 0.7, // 70% speed
                    base_lifetime * 1.2, // Longer lifetime to compensate
                    Color::srgba(0.4, 0.5, 0.8, 0.3), // Darker, more transparent
                    Vec2::new(0.8, 6.0), // Smaller
                    40.0, // Lower Z
                ),
            }
        },
        WeatherState::HeavyRain => {
            let base_velocity = Vec2::new(0.0, -500.0) + weather.wind_direction * 120.0;
            let base_lifetime = 4.0;
            
            match layer {
                WeatherLayer::Foreground => (
                    WeatherParticleType::Raindrop,
                    base_velocity, // Full speed
                    base_lifetime,
                    Color::srgba(0.5, 0.6, 0.9, 0.8), // Bright
                    Vec2::new(1.5, 12.0), // Larger drops
                    60.0,
                ),
                WeatherLayer::Background => (
                    WeatherParticleType::Raindrop,
                    base_velocity * 0.6, // 60% speed
                    base_lifetime * 1.4,
                    Color::srgba(0.3, 0.4, 0.7, 0.4), // Much darker
                    Vec2::new(1.0, 8.0), // Smaller
                    40.0,
                ),
            }
        },
        WeatherState::Snow => {
            let base_velocity = Vec2::new(0.0, -40.0) + weather.wind_direction * 20.0 + 
                Vec2::new(fastrand::f32() * 15.0 - 7.5, 0.0); // More random drift
            let base_lifetime = 30.0; // Much longer for snow
            
            match layer {
                WeatherLayer::Foreground => (
                    WeatherParticleType::Snowflake,
                    base_velocity,
                    base_lifetime,
                    Color::srgba(1.0, 1.0, 1.0, 0.9), // Bright white
                    Vec2::new(3.0, 3.0),
                    60.0,
                ),
                WeatherLayer::Background => (
                    WeatherParticleType::Snowflake,
                    base_velocity * 0.5, // Much slower
                    base_lifetime * 1.5, // Even longer lifetime
                    Color::srgba(0.8, 0.8, 0.9, 0.4), // Dimmer, slight blue tint
                    Vec2::new(2.0, 2.0), // Smaller
                    40.0,
                ),
            }
        },
        _ => return,
    };

    commands.spawn((
        Sprite {
            color,
            custom_size: Some(size),
            ..default()
        },
        Transform::from_translation(Vec3::new(spawn_x, spawn_y, z_layer)),
        WeatherParticle {
            velocity,
            lifetime,
            max_lifetime: lifetime,
            particle_type,
            layer,
        },
    ));
}

// Update weather overlay to follow camera
pub fn update_weather_overlay(
    mut overlay_query: Query<&mut Transform, (With<WeatherOverlay>, Without<Camera>)>,
    camera_query: Query<&Transform, (With<Camera>, Without<WeatherOverlay>)>,
) {
    let Ok(camera_transform) = camera_query.single() else { return; };
    
    for mut overlay_transform in overlay_query.iter_mut() {
        overlay_transform.translation.x = camera_transform.translation.x;
        overlay_transform.translation.y = camera_transform.translation.y;
    }
}

// Clean up weather system when leaving mission
pub fn cleanup_weather_system(
    mut commands: Commands,
    particles: Query<Entity, With<WeatherParticle>>,
    overlays: Query<Entity, With<WeatherOverlay>>,
) {
    for entity in particles.iter() {
        commands.entity(entity).insert(MarkedForDespawn);
    }
    
    for entity in overlays.iter() {
        commands.entity(entity).insert(MarkedForDespawn);
    }
}

// Optional: Weather effects on gameplay
pub fn weather_gameplay_effects(
    weather: Res<WeatherSystem>,
    mut visibility_query: Query<&mut Visibility, Or<(With<Agent>, With<Enemy>)>>,
) {
    // Reduce visibility in heavy weather
    let visibility_modifier = match weather.current_weather {
        WeatherState::ClearSkies => 1.0,
        WeatherState::LightRain => 0.9,
        WeatherState::HeavyRain => 0.7,
        WeatherState::Snow => 0.8,
    };

    // This is a simple example - you could expand this to affect:
    // - Movement speed (slower in snow/heavy rain)
    // - Sound detection range
    // - Weapon accuracy
    // - AI behavior patterns
}

// Debug system for testing weather
pub fn weather_debug_system(
    input: Res<ButtonInput<KeyCode>>,
    mut weather: ResMut<WeatherSystem>,
) {
    if input.just_pressed(KeyCode::F5) {
        weather.current_weather = match weather.current_weather {
            WeatherState::ClearSkies => WeatherState::LightRain,
            WeatherState::LightRain => WeatherState::HeavyRain,
            WeatherState::HeavyRain => WeatherState::Snow,
            WeatherState::Snow => WeatherState::ClearSkies,
        };
        info!("Weather changed to: {:?}", weather.current_weather);
    }
}