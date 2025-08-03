// src/systems/cursor.rs
use bevy::prelude::*;
use crate::core::*;

#[derive(Resource)]
pub struct CursorSprites {
    pub arrow: Handle<Image>,
    pub crosshair: Handle<Image>,
    pub hand: Handle<Image>,
    pub hacker: Handle<Image>,
    pub examine: Handle<Image>,
    pub move_cursor: Handle<Image>,
}

#[derive(Component)]
pub struct CursorEntity;

#[derive(Resource, Default)]
pub struct LastCursorState {
    pub cursor_type: CursorType,
    pub position: Vec2,
}

#[derive(PartialEq, Clone, Copy)]
pub enum CursorType {
    Arrow,
    Crosshair,
    Hand,
    Hacker,
    Examine,
    Move,
}

impl Default for CursorType {
    fn default() -> Self {
        CursorType::Arrow
    }
}

pub fn load_cursor_sprites(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    let cursor_sprites = CursorSprites {
        arrow: asset_server.load("sprites/cursors/arrow.png"),
        crosshair: asset_server.load("sprites/cursors/crosshair.png"),
        hand: asset_server.load("sprites/cursors/hand.png"),
        hacker: asset_server.load("sprites/cursors/hacker.png"),
        examine: asset_server.load("sprites/cursors/examine.png"),
        move_cursor: asset_server.load("sprites/cursors/move.png"),
    };
    
    commands.insert_resource(cursor_sprites);
    commands.insert_resource(LastCursorState::default());
}

pub fn cursor_system(
    mut commands: Commands,
    cursor_sprites: Res<CursorSprites>,
    mut last_state: ResMut<LastCursorState>,
    windows: Query<&Window>,
    cameras: Query<(&Camera, &GlobalTransform)>,
    selection: Res<SelectionState>,
    game_mode: Res<GameMode>,
    
    // Queries for different interactable types
    enemy_query: Query<(Entity, &Transform, &Health), (With<Enemy>, Without<MarkedForDespawn>)>,
    vehicle_query: Query<(Entity, &Transform, &Health), (With<Vehicle>, Without<MarkedForDespawn>)>,
    terminal_query: Query<(Entity, &Transform, &Terminal, Option<&LoreSource>)>,
    hackable_query: Query<(Entity, &Transform, &Hackable, &DeviceState)>,
    agent_query: Query<(&Transform, &Inventory), With<Agent>>,
    
    // Cursor entity management
    cursor_entity_query: Query<Entity, With<CursorEntity>>,
    mut cursor_transform_query: Query<&mut Transform, (With<CursorEntity>, Without<Camera>)>,
) {
    if game_mode.paused { return; }

    let mouse_pos = if let Some(pos) = get_world_mouse_position(&windows, &cameras) {
        pos
    } else {
        return;
    };

    // Determine what cursor should be displayed
    let cursor_type = determine_cursor_type(
        mouse_pos,
        &selection,
        &game_mode,
        &enemy_query,
        &vehicle_query,
        &terminal_query,
        &hackable_query,
        &agent_query,
    );

    // Only update if cursor type or position changed significantly
    if cursor_type != last_state.cursor_type || mouse_pos.distance(last_state.position) > 1.0 {
        update_cursor_sprite(
            &mut commands,
            &cursor_sprites,
            cursor_type,
            mouse_pos,
            &cursor_entity_query,
            &mut cursor_transform_query,
        );
        
        last_state.cursor_type = cursor_type;
        last_state.position = mouse_pos;
    }
}

pub fn determine_cursor_type(
    mouse_pos: Vec2,
    selection: &SelectionState,
    game_mode: &GameMode,
    enemy_query: &Query<(Entity, &Transform, &Health), (With<Enemy>, Without<MarkedForDespawn>)>,
    vehicle_query: &Query<(Entity, &Transform, &Health), (With<Vehicle>, Without<MarkedForDespawn>)>,
    terminal_query: &Query<(Entity, &Transform, &Terminal, Option<&LoreSource>)>,
    hackable_query: &Query<(Entity, &Transform, &Hackable, &DeviceState)>,
    agent_query: &Query<(&Transform, &Inventory), With<Agent>>,
) -> CursorType {
    // Check for special modes first
    match &game_mode.targeting {
        Some(TargetingMode::Neurovector { .. }) => return CursorType::Examine,
        Some(TargetingMode::Scanning { .. }) => return CursorType::Examine,
        _ => {}
    }

    // Get primary agent for range checks
    let primary_agent = selection.selected.first();
    let (agent_pos, agent_inventory) = if let Some(&agent) = primary_agent {
        if let Ok((transform, inventory)) = agent_query.get(agent) {
            (transform.translation.truncate(), Some(inventory))
        } else {
            (Vec2::ZERO, None)
        }
    } else {
        (Vec2::ZERO, None)
    };

    const INTERACTION_THRESHOLD: f32 = 35.0;

    // Check for attackable targets (highest priority in combat mode)
    if let Some(inventory) = agent_inventory {
        let weapon_range = get_weapon_range_simple(inventory);
        
        // Check enemies
        for (_, transform, health) in enemy_query.iter() {
            if health.0 <= 0.0 { continue; }
            
            let target_pos = transform.translation.truncate();
            if mouse_pos.distance(target_pos) <= INTERACTION_THRESHOLD &&
               agent_pos.distance(target_pos) <= weapon_range {
                return CursorType::Crosshair;
            }
        }
        
        // Check vehicles  
        for (_, transform, health) in vehicle_query.iter() {
            if health.0 <= 0.0 { continue; }
            
            let target_pos = transform.translation.truncate();
            if mouse_pos.distance(target_pos) <= INTERACTION_THRESHOLD &&
               agent_pos.distance(target_pos) <= weapon_range {
                return CursorType::Crosshair;
            }
        }
    }

    // Check for hackable devices
    for (_, transform, hackable, device_state) in hackable_query.iter() {
        if hackable.is_hacked { continue; }
        
        let device_pos = transform.translation.truncate();
        if mouse_pos.distance(device_pos) <= INTERACTION_THRESHOLD &&
           agent_pos.distance(device_pos) <= 40.0 {
            
            // Check if agent has hacking tool
            if let Some(inventory) = agent_inventory {
                if check_hack_tool_available(inventory, hackable) && 
                   device_state.powered && device_state.operational {
                    return CursorType::Hacker;
                }
            }
            return CursorType::Hand; // Can interact but missing tool
        }
    }

    // Check for terminals
    for (_, transform, terminal, lore_source) in terminal_query.iter() {
        if terminal.accessed && lore_source.map_or(true, |ls| ls.accessed) {
            continue;
        }
        
        let terminal_pos = transform.translation.truncate();
        if mouse_pos.distance(terminal_pos) <= INTERACTION_THRESHOLD &&
           agent_pos.distance(terminal_pos) <= terminal.range {
            return CursorType::Hand;
        }
    }

    CursorType::Arrow
}

fn update_cursor_sprite(
    commands: &mut Commands,
    cursor_sprites: &CursorSprites,
    cursor_type: CursorType,
    position: Vec2,
    cursor_entity_query: &Query<Entity, With<CursorEntity>>,
    cursor_transform_query: &mut Query<&mut Transform, (With<CursorEntity>, Without<Camera>)>,
) {
    let sprite_handle = match cursor_type {
        CursorType::Arrow => &cursor_sprites.arrow,
        CursorType::Crosshair => &cursor_sprites.crosshair,
        CursorType::Hand => &cursor_sprites.hand,
        CursorType::Hacker => &cursor_sprites.hacker,
        CursorType::Examine => &cursor_sprites.examine,
        CursorType::Move => &cursor_sprites.move_cursor,
    };

    // Try to update existing cursor entity
    if let Ok(entity) = cursor_entity_query.single() {
        if let Ok(mut transform) = cursor_transform_query.get_mut(entity) {
            transform.translation = position.extend(1000.0); // High Z to render on top
            
            // Update sprite component
            commands.entity(entity).insert(Sprite {
                image: sprite_handle.clone(),
                ..default()
            });
            return;
        }
    }

    // Create new cursor entity if none exists
    commands.spawn((
        CursorEntity,
        Sprite {
            image: sprite_handle.clone(),
            ..default()
        },
        Transform::from_translation(position.extend(1000.0)),
        GlobalTransform::default(),
    ));
}

pub fn get_weapon_range_simple(inventory: &Inventory) -> f32 {
    let base_range = 150.0;
    if let Some(weapon_config) = &inventory.equipped_weapon {
        let stats = weapon_config.stats();
        (base_range * (1.0 + stats.range as f32 * 0.1)).max(50.0)
    } else {
        base_range
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

// Hide system cursor when using custom sprites
pub fn hide_system_cursor(mut windows: Query<&mut Window>) {
    for mut window in windows.iter_mut() {
        window.cursor_options.visible = false;
    }
}


