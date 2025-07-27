// Replace health_bars.rs with this enhanced version
use bevy::prelude::*;
use crate::core::*;

#[derive(Component)]
pub struct HealthBar {
    pub max_health: f32,
    pub fill: Entity,
}

#[derive(Component)]
pub struct AgentStatusBar {
    pub agent_index: usize,
    pub health_fill: Entity,
    pub ammo_fill: Entity,
    pub number_text: Entity,
}

const BAR_SIZE: Vec2 = Vec2::new(32.0, 4.0);
const AMMO_BAR_SIZE: Vec2 = Vec2::new(32.0, 2.0);
const HEALTH_OFFSET: Vec3 = Vec3::new(0.0, 25.0, 0.1);
const AMMO_OFFSET: Vec3 = Vec3::new(0.0, 20.0, 0.1);
const NUMBER_OFFSET: Vec3 = Vec3::new(0.0, 32.0, 0.2);

// Spawn status bars for agents
pub fn spawn_agent_status_bars(
    mut commands: Commands,
    query: Query<Entity, (With<Agent>, Without<AgentStatusBar>)>,
    asset_server: Res<AssetServer>,
) {
    for (idx, entity) in query.iter().enumerate() {
        if idx >= 3 { continue; } // Only for first 3 agents
        
        // Health bar background
        let health_bg = commands.spawn((
            Sprite {
                color: Color::srgb(0.2, 0.2, 0.2),
                custom_size: Some(BAR_SIZE),
                ..default()
            },
            Transform::from_translation(HEALTH_OFFSET),
        )).id();
        
        // Health fill
        let health_fill = commands.spawn((
            Sprite {
                color: Color::srgb(0.2, 0.8, 0.2),
                custom_size: Some(BAR_SIZE),
                anchor: bevy::sprite::Anchor::CenterLeft,
                ..default()
            },
            Transform::from_translation(
                HEALTH_OFFSET + Vec3::new(-BAR_SIZE.x * 0.5, 0.0, 0.1)
            ),
        )).id();
        
        // Ammo bar background
        let ammo_bg = commands.spawn((
            Sprite {
                color: Color::srgb(0.15, 0.15, 0.15),
                custom_size: Some(AMMO_BAR_SIZE),
                ..default()
            },
            Transform::from_translation(AMMO_OFFSET),
        )).id();
        
        // Ammo fill
        let ammo_fill = commands.spawn((
            Sprite {
                color: Color::srgb(0.8, 0.8, 0.2),
                custom_size: Some(AMMO_BAR_SIZE),
                anchor: bevy::sprite::Anchor::CenterLeft,
                ..default()
            },
            Transform::from_translation(
                AMMO_OFFSET + Vec3::new(-AMMO_BAR_SIZE.x * 0.5, 0.0, 0.1)
            ),
        )).id();
        
        // Agent number text
        let number_text = commands.spawn((
            Text2d::new(format!("{}", idx + 1)),
            TextFont {
                font: asset_server.load("fonts/monospace.ttf"),
                font_size: 12.0,
                ..default()
            },
            TextColor(Color::WHITE),
            Transform::from_translation(NUMBER_OFFSET),
        )).id();
        
        // Add all to agent entity
        commands.entity(entity)
            .insert(AgentStatusBar {
                agent_index: idx,
                health_fill,
                ammo_fill,
                number_text,
            })
            .add_child(health_bg)
            .add_child(health_fill)
            .add_child(ammo_bg)
            .add_child(ammo_fill)
            .add_child(number_text);
    }
}

// Update agent status bars
pub fn update_agent_status_bars(
    mut sprites: Query<&mut Sprite>,
    query: Query<(&Health, &WeaponState, &AgentStatusBar), With<Agent>>,
) {
    for (health, weapon_state, status_bar) in query.iter() {
        // Update health bar
        if let Ok(mut sprite) = sprites.get_mut(status_bar.health_fill) {
            let health_ratio = (health.0 / 100.0).clamp(0.0, 1.0);
            sprite.custom_size = Some(Vec2::new(BAR_SIZE.x * health_ratio, BAR_SIZE.y));
            sprite.color = health_color(health_ratio);
        }
        
        // Update ammo bar
        if let Ok(mut sprite) = sprites.get_mut(status_bar.ammo_fill) {
            let ammo_ratio = if weapon_state.max_ammo > 0 {
                weapon_state.current_ammo as f32 / weapon_state.max_ammo as f32
            } else {
                1.0
            };
            sprite.custom_size = Some(Vec2::new(AMMO_BAR_SIZE.x * ammo_ratio, AMMO_BAR_SIZE.y));
            sprite.color = ammo_color(ammo_ratio);
        }
    }
}

// Enemy health bars (only when damaged)
pub fn spawn_enemy_health_bars(
    mut commands: Commands,
    query: Query<(Entity, &Health), (Or<(With<Enemy>, With<Vehicle>)>, Without<HealthBar>, Changed<Health>)>,
) {
    for (entity, health) in query.iter() {
        if health.0 < 100.0 && health.0 > 0.0 {
            let ratio = health.0 / 100.0;
            
            let fill = commands.spawn((
                Sprite {
                    color: health_color(ratio),
                    custom_size: Some(Vec2::new(BAR_SIZE.x * ratio, BAR_SIZE.y)),
                    anchor: bevy::sprite::Anchor::CenterLeft,
                    ..default()
                },
                Transform::from_translation(
                    HEALTH_OFFSET + Vec3::new(-BAR_SIZE.x * 0.5, 0.0, 0.1)
                ),
            )).id();
            
            let bg = commands.spawn((
                Sprite {
                    color: Color::srgb(0.2, 0.2, 0.2),
                    custom_size: Some(BAR_SIZE),
                    ..default()
                },
                Transform::from_translation(HEALTH_OFFSET),
            )).id();
            
            commands.entity(entity)
                .insert(HealthBar { max_health: 100.0, fill })
                .add_child(bg)
                .add_child(fill);
        }
    }
}

// Keep existing update and cleanup systems for enemies
pub fn update_enemy_health_bars(
    mut commands: Commands,
    mut sprites: Query<&mut Sprite>,
    query: Query<(Entity, &Health, &HealthBar, &Children), (Without<Agent>, Changed<Health>)>,
) {
    for (entity, health, bar, children) in query.iter() {
        if health.0 <= 0.0 || health.0 >= 100.0 {
            for child in children.iter() {
                commands.entity(child).insert(MarkedForDespawn);
            }
            commands.entity(entity).remove::<HealthBar>();
            continue;
        }
        
        if let Ok(mut sprite) = sprites.get_mut(bar.fill) {
            let ratio = (health.0 / bar.max_health).clamp(0.0, 1.0);
            sprite.custom_size = Some(Vec2::new(BAR_SIZE.x * ratio, BAR_SIZE.y));
            sprite.color = health_color(ratio);
        }
    }
}

#[inline]
fn health_color(ratio: f32) -> Color {
    if ratio > 0.6 { Color::srgb(0.2, 0.8, 0.2) }
    else if ratio > 0.3 { Color::srgb(0.8, 0.8, 0.2) }
    else { Color::srgb(0.8, 0.2, 0.2) }
}

#[inline]
fn ammo_color(ratio: f32) -> Color {
    if ratio > 0.5 { Color::srgb(0.8, 0.8, 0.2) }
    else if ratio > 0.2 { Color::srgb(0.8, 0.5, 0.2) }
    else { Color::srgb(0.8, 0.2, 0.2) }
}
