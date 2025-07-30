// src/systems/advanced_prompts.rs
use bevy::prelude::*;
use crate::core::*;
use crate::systems::interaction_prompts::*;

#[derive(Resource)]
pub struct PromptSettings {
    pub max_distance: f32,
    pub fade_distance: f32,
    pub animation_enabled: bool,
    pub show_tooltips: bool,
    pub stack_prompts: bool,
}

impl Default for PromptSettings {
    fn default() -> Self {
        Self {
            max_distance: 100.0,
            fade_distance: 80.0,
            animation_enabled: true,
            show_tooltips: true,
            stack_prompts: true,
        }
    }
}

#[derive(Component)]
pub struct PromptTooltip {
    pub text: String,
    pub background_color: Color,
    pub text_color: Color,
}

#[derive(Component)]
pub struct DistanceFade {
    pub agent_pos: Vec2,
    pub max_distance: f32,
    pub fade_distance: f32,
}

#[derive(Component)]
pub struct PromptStack {
    pub prompts: Vec<InteractionType>,
    pub vertical_spacing: f32,
}

// Enhanced prompt system with distance-based fading and tooltips
pub fn advanced_prompt_system(
    mut commands: Commands,
    interaction_sprites: Res<InteractionSprites>,
    prompt_settings: Res<PromptSettings>,
    selection: Res<SelectionState>,
    inventory_query: Query<(&Transform, &Inventory), With<Agent>>,
    terminal_query: Query<(Entity, &Transform, &Terminal, Option<&LoreSource>)>,
    hackable_query: Query<(Entity, &Transform, &Hackable, &DeviceState)>,
    weapon_query: Query<(Entity, &Transform, &Health), (With<Enemy>, Without<MarkedForDespawn>)>,
    
    // Cleanup
    existing_prompts: Query<Entity, Or<(With<InteractionPrompt>, With<PromptTooltip>, With<PromptStack>)>>,
    game_mode: Res<GameMode>,
) {
    if game_mode.paused { return; }

    // Clean up existing prompts
    for entity in existing_prompts.iter() {
        commands.entity(entity).insert(MarkedForDespawn);
    }

    for &selected_agent in &selection.selected {
        if let Ok((agent_transform, inventory)) = inventory_query.get(selected_agent) {
            let agent_pos = agent_transform.translation.truncate();

            // Collect all nearby interactions
            let mut interactions = collect_nearby_interactions(
                agent_pos,
                inventory,
                &terminal_query,
                &hackable_query,
                &weapon_query,
                &prompt_settings,
            );

            // Sort by distance for proper stacking
            interactions.sort_by(|a, b| a.distance.partial_cmp(&b.distance).unwrap_or(std::cmp::Ordering::Equal));

            if prompt_settings.stack_prompts && interactions.len() > 1 {
                spawn_stacked_prompts(&mut commands, &interaction_sprites, &interactions, &prompt_settings);
            } else {
                spawn_individual_prompts(&mut commands, &interaction_sprites, &interactions, &prompt_settings, agent_pos);
            }
        }
    }
}

#[derive(Debug)]
struct InteractionData {
    entity: Entity,
    position: Vec2,
    distance: f32,
    interaction_type: InteractionType,
    prompt_text: String,
    color: Color,
    tooltip: Option<String>,
}

fn collect_nearby_interactions(
    agent_pos: Vec2,
    inventory: &Inventory,
    terminal_query: &Query<(Entity, &Transform, &Terminal, Option<&LoreSource>)>,
    hackable_query: &Query<(Entity, &Transform, &Hackable, &DeviceState)>,
    weapon_query: &Query<(Entity, &Transform, &Health), (With<Enemy>, Without<MarkedForDespawn>)>,
    settings: &PromptSettings,
) -> Vec<InteractionData> {
    let mut interactions = Vec::new();

    // Collect terminal interactions
    for (entity, transform, terminal, lore_source) in terminal_query.iter() {
        if terminal.accessed && lore_source.map_or(true, |ls| ls.accessed) {
            continue;
        }

        let pos = transform.translation.truncate();
        let distance = agent_pos.distance(pos);

        if distance <= terminal.range && distance <= settings.max_distance {
            interactions.push(InteractionData {
                entity,
                position: pos,
                distance,
                interaction_type: InteractionType::Interact,
                prompt_text: "Interact".to_string(),
                color: get_terminal_color(&terminal.terminal_type),
                tooltip: Some(get_terminal_tooltip(&terminal.terminal_type)),
            });
        }
    }

    // Collect hackable interactions
    for (entity, transform, hackable, device_state) in hackable_query.iter() {
        if hackable.is_hacked { continue; }

        let pos = transform.translation.truncate();
        let distance = agent_pos.distance(pos);

        if distance <= 40.0 && distance <= settings.max_distance {
            let has_tool = check_hack_tool_available(inventory, hackable);
            let is_operational = device_state.powered && device_state.operational;

            let (interaction_type, color, tooltip) = if has_tool && is_operational {
                (
                    InteractionType::Interact,
                    Color::srgb(0.2, 0.8, 0.8),
                    format!("Hack Device (Security: {})", hackable.security_level),
                )
            } else if !has_tool {
                (
                    InteractionType::Unavailable,
                    Color::srgb(0.8, 0.2, 0.2),
                    "Requires Hacker Tool".to_string(),
                )
            } else {
                (
                    InteractionType::Unavailable,
                    Color::srgb(0.6, 0.6, 0.2),
                    "Device Offline".to_string(),
                )
            };

            interactions.push(InteractionData {
                entity,
                position: pos,
                distance,
                interaction_type,
                prompt_text: "Hack".to_string(),
                color,
                tooltip: Some(tooltip),
            });
        }
    }

    // Collect combat targets
    let weapon_range = get_weapon_range_simple(inventory);
    for (entity, transform, health) in weapon_query.iter() {
        if health.0 <= 0.0 { continue; }

        let pos = transform.translation.truncate();
        let distance = agent_pos.distance(pos);

        if distance <= weapon_range && distance <= settings.max_distance {
            interactions.push(InteractionData {
                entity,
                position: pos,
                distance,
                interaction_type: InteractionType::Attack,
                prompt_text: "Attack".to_string(),
                color: Color::srgb(1.0, 0.2, 0.2),
                tooltip: Some(format!("Attack Enemy (Range: {:.0}m)", distance)),
            });
        }
    }

    interactions
}

fn spawn_stacked_prompts(
    commands: &mut Commands,
    sprites: &InteractionSprites,
    interactions: &[InteractionData],
    settings: &PromptSettings,
) {
    if interactions.is_empty() { return; }

    // Use the closest interaction as the base position
    let base_pos = interactions[0].position;
    let stack_height = 25.0;

    for (i, interaction) in interactions.iter().enumerate() {
        let prompt_pos = base_pos + Vec2::new(0.0, 40.0 + i as f32 * stack_height);
        
        spawn_enhanced_prompt(
            commands,
            sprites,
            prompt_pos,
            interaction.interaction_type,
            interaction.entity,
            interaction.color,
            &interaction.tooltip,
            Some(interaction.distance),
            settings,
        );
    }

    // Add stack indicator
    if interactions.len() > 3 {
        let indicator_pos = base_pos + Vec2::new(15.0, 40.0);
        spawn_stack_counter(commands, indicator_pos, interactions.len());
    }
}

fn spawn_individual_prompts(
    commands: &mut Commands,
    sprites: &InteractionSprites,
    interactions: &[InteractionData],
    settings: &PromptSettings,
    agent_pos: Vec2,
) {
    for interaction in interactions.iter() {
        let prompt_pos = interaction.position + Vec2::new(0.0, 30.0);
        
        spawn_enhanced_prompt(
            commands,
            sprites,
            prompt_pos,
            interaction.interaction_type,
            interaction.entity,
            interaction.color,
            &interaction.tooltip,
            Some(interaction.distance),
            settings,
        );
    }
}

fn spawn_enhanced_prompt(
    commands: &mut Commands,
    sprites: &InteractionSprites,
    position: Vec2,
    prompt_type: InteractionType,
    target_entity: Entity,
    tint_color: Color,
    tooltip: &Option<String>,
    distance: Option<f32>,
    settings: &PromptSettings,
) {
    let sprite_handle = match prompt_type {
        InteractionType::Interact => &sprites.key_e,
        InteractionType::Attack => &sprites.key_f,
        InteractionType::Reload => &sprites.key_r,
        InteractionType::Unavailable => &sprites.key_question,
    };

    // Background with distance-based alpha
    let base_alpha = if let Some(dist) = distance {
        calculate_distance_alpha(dist, settings.fade_distance, settings.max_distance)
    } else {
        0.7
    };

    // Background circle
    let bg_entity = commands.spawn((
        Sprite {
            color: Color::srgba(0.0, 0.0, 0.0, base_alpha),
            custom_size: Some(Vec2::new(24.0, 24.0)),
            ..default()
        },
        Transform::from_translation(position.extend(100.0)),
        GlobalTransform::default(),
        InteractionPrompt {
            target_entity,
            prompt_type,
        },
    )).id();

    // Key sprite
    let key_entity = commands.spawn((
        Sprite {
            image: sprite_handle.clone(),
            color: tint_color.with_alpha(base_alpha + 0.3),
            custom_size: Some(Vec2::new(32.0, 32.0)),
            ..default()
        },
        Transform::from_translation(position.extend(101.0)),
        GlobalTransform::default(),
        InteractionPrompt {
            target_entity,
            prompt_type,
        },
    )).id();

    // Add distance fade component
    if let Some(dist) = distance {
        commands.entity(bg_entity).insert(DistanceFade {
            agent_pos: position - Vec2::new(0.0, 30.0), // Approximate agent position
            max_distance: settings.max_distance,
            fade_distance: settings.fade_distance,
        });
        commands.entity(key_entity).insert(DistanceFade {
            agent_pos: position - Vec2::new(0.0, 30.0),
            max_distance: settings.max_distance,
            fade_distance: settings.fade_distance,
        });
    }

    // Add tooltip if enabled
    if settings.show_tooltips {
        if let Some(tooltip_text) = tooltip {
            spawn_tooltip(commands, position + Vec2::new(0.0, 20.0), tooltip_text, tint_color);
        }
    }
}

fn spawn_tooltip(
    commands: &mut Commands,
    position: Vec2,
    text: &str,
    color: Color,
) {
    let tooltip_width = text.len() as f32 * 6.0 + 10.0;
    let tooltip_height = 16.0;

    // Tooltip background
    commands.spawn((
        Sprite {
            color: Color::srgba(0.0, 0.0, 0.0, 0.8),
            custom_size: Some(Vec2::new(tooltip_width, tooltip_height)),
            ..default()
        },
        Transform::from_translation(position.extend(102.0)),
        GlobalTransform::default(),
        PromptTooltip {
            text: text.to_string(),
            background_color: Color::srgba(0.0, 0.0, 0.0, 0.8),
            text_color: color,
        },
    ));

    commands.spawn((
        Text2d::new(text.clone()),
        TextFont {
            font_size: 10.0,
            ..default()
        },
        TextColor(Color::WHITE),
        Transform::from_translation(position.extend(103.0)),
        GlobalTransform::default(),
        PromptTooltip {
            text: text.to_string(),
            background_color: color.with_alpha(0.6),
            text_color: color,
        },
    ));
}

fn spawn_stack_counter(
    commands: &mut Commands,
    position: Vec2,
    count: usize,
) {
    // Simple counter visualization (could be enhanced with number sprites)
    commands.spawn((
        Sprite {
            color: Color::srgb(1.0, 1.0, 0.0),
            custom_size: Some(Vec2::new(8.0, 8.0)),
            ..default()
        },
        Transform::from_translation(position.extend(103.0)),
        GlobalTransform::default(),
    ));
}

fn calculate_distance_alpha(distance: f32, fade_distance: f32, max_distance: f32) -> f32 {
    if distance <= fade_distance {
        0.8
    } else {
        let fade_range = max_distance - fade_distance;
        let fade_progress = (distance - fade_distance) / fade_range;
        (0.8 * (1.0 - fade_progress)).max(0.2)
    }
}

// Update system for distance-based fading
pub fn distance_fade_system(
    selection: Res<SelectionState>,
    agent_query: Query<&Transform, With<Agent>>,
    mut fade_query: Query<(&DistanceFade, &mut Sprite)>,
) {
    if let Some(&agent) = selection.selected.first() {
        if let Ok(agent_transform) = agent_query.get(agent) {
            let agent_pos = agent_transform.translation.truncate();
            
            for (fade, mut sprite) in fade_query.iter_mut() {
                let distance = agent_pos.distance(fade.agent_pos);
                let alpha = calculate_distance_alpha(distance, fade.fade_distance, fade.max_distance);
                sprite.color.set_alpha(alpha);
            }
        }
    }
}

fn get_terminal_tooltip(terminal_type: &TerminalType) -> String {
    match terminal_type {
        TerminalType::Objective => "Mission Objective - Complete for rewards".to_string(),
        TerminalType::Equipment => "Equipment Cache - Weapons and tools".to_string(),
        TerminalType::Intel => "Intel Terminal - Valuable information".to_string(),
    }
}

fn get_terminal_color(terminal_type: &TerminalType) -> Color {
    match terminal_type {
        TerminalType::Objective => Color::srgb(0.9, 0.2, 0.2),
        TerminalType::Equipment => Color::srgb(0.2, 0.5, 0.9),
        TerminalType::Intel => Color::srgb(0.2, 0.8, 0.3),
    }
}

fn check_hack_tool_available(inventory: &Inventory, hackable: &Hackable) -> bool {
    match &hackable.requires_tool {
        Some(required_tool) => {
            inventory.equipped_tools.iter().any(|tool| {
                matches!((tool, required_tool), 
                    (ToolType::Hacker, HackTool::BasicHacker) |
                    (ToolType::Hacker, HackTool::AdvancedHacker)
                )
            })
        },
        None => true,
    }
}

fn get_weapon_range_simple(inventory: &Inventory) -> f32 {
    let base_range = 150.0;
    if let Some(weapon_config) = &inventory.equipped_weapon {
        let stats = weapon_config.stats();
        (base_range * (1.0 + stats.range as f32 * 0.1)).max(50.0)
    } else {
        base_range
    }
}