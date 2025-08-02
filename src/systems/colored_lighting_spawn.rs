// src/systems/colored_lighting_spawn.rs - Clean, efficient colored light spawning
use bevy::prelude::*;
use crate::core::*;
use crate::systems::colored_lighting::*;
use crate::systems::tile_lighting::ShadowCaster;

// === SINGLE UNIFIED SPAWN FUNCTION ===

pub fn spawn_colored_light(
    commands: &mut Commands,
    position: Vec2,
    light_type: LightType,
    network_id: Option<String>,
    power_grid: Option<&mut ResMut<crate::core::PowerGrid>>,
) -> Entity {
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
        entity_commands.insert(ShadowCaster { height: shadow_height, opacity: 0.3 });
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

// === CONVENIENCE FUNCTIONS ===

pub fn spawn_neon_sign(
    commands: &mut Commands,
    position: Vec2,
    color: Color,
    text: &str,
) -> Entity {
    let size = Vec2::new(text.len() as f32 * 8.0, 16.0);
    let mut light = ColoredLight::new(LightType::NeonSign).with_custom_color(color);
    
    commands.spawn((
        Transform::from_translation(position.extend(5.0)),
        light,
        Sprite {
            color,
            custom_size: Some(size),
            ..default()
        },
        Health(20.0),
        bevy_rapier2d::prelude::RigidBody::Fixed,
        bevy_rapier2d::prelude::Collider::cuboid(size.x * 0.5, size.y * 0.5),
        crate::core::Hackable::new(crate::core::DeviceType::Billboard),
        crate::core::DeviceState::new(crate::core::DeviceType::Billboard),
    )).id()
}

pub fn spawn_laser_sight(
    commands: &mut Commands,
    position: Vec2,
    direction: Vec2,
) -> Entity {
    commands.spawn((
        Transform::from_translation(position.extend(1.0))
            .with_rotation(Quat::from_rotation_z(direction.y.atan2(direction.x))),
        ColoredLight::new(LightType::LaserSight),
        Sprite {
            color: Color::srgb(1.0, 0.0, 0.0),
            custom_size: Some(Vec2::new(200.0, 2.0)),
            ..default()
        },
    )).id()
}

// === AREA SPAWN FUNCTIONS ===

pub fn spawn_area_lighting(
    commands: &mut Commands,
    center: Vec2,
    area_type: AreaType,
    network_id: String,
    power_grid: &mut ResMut<crate::core::PowerGrid>,
) -> Vec<Entity> {
    match area_type {
        AreaType::Street => spawn_street_grid(commands, center, network_id, power_grid),
        AreaType::Corporate => spawn_corporate_grid(commands, center, network_id, power_grid),
        AreaType::Industrial => spawn_industrial_grid(commands, center, network_id, power_grid),
        AreaType::Laboratory => spawn_lab_grid(commands, center, network_id, power_grid),
        AreaType::Entertainment => spawn_entertainment_grid(commands, center),
    }
}

#[derive(Debug, Clone, Copy)]
pub enum AreaType {
    Street,
    Corporate,
    Industrial,
    Laboratory,
    Entertainment,
}

fn spawn_street_grid(
    commands: &mut Commands,
    center: Vec2,
    network_id: String,
    power_grid: &mut ResMut<crate::core::PowerGrid>,
) -> Vec<Entity> {
    let positions = vec![
        Vec2::new(-150.0, 0.0), Vec2::new(-50.0, 100.0), Vec2::new(50.0, 100.0),
        Vec2::new(150.0, 0.0), Vec2::new(0.0, -150.0),
    ];
    
    positions.into_iter()
        .map(|offset| {
            let pos = center + offset;
            spawn_colored_light(commands, pos, LightType::StreetLight, Some(network_id.clone()), Some(power_grid))
        })
        .collect()
}

fn spawn_corporate_grid(
    commands: &mut Commands,
    center: Vec2,
    network_id: String,
    power_grid: &mut ResMut<crate::core::PowerGrid>,
) -> Vec<Entity> {
    let mut lights = Vec::new();
    
    // Office lighting grid
    for x in -1..=1 {
        for y in -1..=1 {
            let pos = center + Vec2::new(x as f32 * 80.0, y as f32 * 80.0);
            lights.push(spawn_colored_light(commands, pos, LightType::OfficeLight, Some(network_id.clone()), Some(power_grid)));
        }
    }
    
    // Security perimeter
    for i in 0..4 {
        let angle = i as f32 * std::f32::consts::PI * 0.5;
        let pos = center + Vec2::new(angle.cos(), angle.sin()) * 120.0;
        lights.push(spawn_colored_light(commands, pos, LightType::SecurityLight, Some(network_id.clone()), Some(power_grid)));
    }
    
    lights
}

fn spawn_industrial_grid(
    commands: &mut Commands,
    center: Vec2,
    network_id: String,
    power_grid: &mut ResMut<crate::core::PowerGrid>,
) -> Vec<Entity> {
    let mut lights = Vec::new();
    
    // High-intensity industrial lights
    for x in -1..=1 {
        for y in -1..=1 {
            if x == 0 && y == 0 { continue; } // Skip center
            let pos = center + Vec2::new(x as f32 * 150.0, y as f32 * 150.0);
            lights.push(spawn_colored_light(commands, pos, LightType::IndustrialLight, Some(network_id.clone()), Some(power_grid)));
        }
    }
    
    // Hazard lights around perimeter
    for i in 0..6 {
        let angle = i as f32 * std::f32::consts::TAU / 6.0;
        let pos = center + Vec2::new(angle.cos(), angle.sin()) * 200.0;
        lights.push(spawn_colored_light(commands, pos, LightType::HazardLight, None, None));
    }
    
    lights
}

fn spawn_lab_grid(
    commands: &mut Commands,
    center: Vec2,
    network_id: String,
    power_grid: &mut ResMut<crate::core::PowerGrid>,
) -> Vec<Entity> {
    let mut lights = Vec::new();
    
    // Lab lighting
    for i in 0..4 {
        let angle = i as f32 * std::f32::consts::PI * 0.5;
        let pos = center + Vec2::new(angle.cos(), angle.sin()) * 60.0;
        lights.push(spawn_colored_light(commands, pos, LightType::LaboratoryLight, Some(network_id.clone()), Some(power_grid)));
    }
    
    // Emergency lighting
    for i in 0..4 {
        let angle = i as f32 * std::f32::consts::PI * 0.5 + std::f32::consts::PI * 0.25;
        let pos = center + Vec2::new(angle.cos(), angle.sin()) * 80.0;
        lights.push(spawn_colored_light(commands, pos, LightType::EmergencyLight, None, None));
    }
    
    lights
}

fn spawn_entertainment_grid(
    commands: &mut Commands,
    center: Vec2,
) -> Vec<Entity> {
    let colors = vec![
        Color::srgb(1.0, 0.0, 1.0), // Magenta
        Color::srgb(0.0, 1.0, 1.0), // Cyan  
        Color::srgb(1.0, 1.0, 0.0), // Yellow
        Color::srgb(1.0, 0.5, 0.0), // Orange
    ];
    
    colors.into_iter().enumerate()
        .map(|(i, color)| {
            let angle = i as f32 * std::f32::consts::TAU / 4.0;
            let pos = center + Vec2::new(angle.cos(), angle.sin()) * 80.0;
            spawn_neon_sign(commands, pos, color, "NEON")
        })
        .collect()
}

// === SCENE INTEGRATION ===

pub fn spawn_enhanced_colored_scene_lighting(
    commands: &mut Commands,
    scene: &crate::systems::scenes::SceneData,
    mut power_grid: ResMut<crate::core::PowerGrid>,
) {
    // Main street lighting
    let street_lights = spawn_area_lighting(
        commands, 
        Vec2::ZERO, 
        AreaType::Street, 
        "main_grid".to_string(), 
        &mut power_grid
    );
    
    // Security lighting near terminals
    for terminal in &scene.terminals {
        let pos = Vec2::from(terminal.position) + Vec2::new(0.0, 30.0);
        spawn_colored_light(commands, pos, LightType::SecurityLight, Some("security_grid".to_string()), Some(&mut power_grid));
    }
    
    // Emergency beacons near enemies
    for enemy in &scene.enemies {
        if fastrand::f32() < 0.3 {
            let pos = Vec2::from(enemy.position) + Vec2::new((fastrand::f32() - 0.5) * 60.0, (fastrand::f32() - 0.5) * 60.0);
            spawn_colored_light(commands, pos, LightType::EmergencyLight, None, None);
        }
    }
    
    // Atmospheric neon
    let neon_configs = vec![
        (Vec2::new(-200.0, 80.0), Color::srgb(1.0, 0.0, 1.0)),
        (Vec2::new(180.0, -80.0), Color::srgb(0.0, 1.0, 1.0)),
        (Vec2::new(-80.0, -120.0), Color::srgb(1.0, 1.0, 0.0)),
    ];
    
    for (pos, color) in neon_configs {
        spawn_neon_sign(commands, pos, color, "BAR");
    }
}
