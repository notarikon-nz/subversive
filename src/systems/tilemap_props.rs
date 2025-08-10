// src/systems/tilemap_props.rs - Multi-tile prop support for tilemap

use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use crate::systems::tilemap::IsometricSettings;

// === COMPONENTS ===

/// Component for props that span multiple tiles
#[derive(Component)]
pub struct MultiTileProp {
    pub base_pos: IVec2,
    pub tile_size: IVec3,  // width, height, vertical_tiles
    pub sprite_size: Vec2,
}

/// Component for isometric depth sorting
#[derive(Component)]
pub struct IsometricDepth {
    pub base_depth: f32,
    pub height_offset: f32,
    pub dynamic: bool,
}

/// Component marking tiles as occupied
#[derive(Component)]
pub struct TileOccupant {
    pub tile_pos: IVec2,
    pub occupying_entity: Entity,
    pub blocks_movement: bool,
}

// === PROP DEFINITIONS ===

#[derive(Clone, Copy, Debug)]
pub struct PropDefinition {
    pub size: IVec3,
    pub sprite_size: Vec2,
    pub anchor: Vec2,
    pub needs_depth_sort: bool,
}

impl PropDefinition {
    pub const STREETLIGHT: Self = Self {
        size: IVec3::new(1, 1, 3),  // 1x1 base, 3 tiles tall
        sprite_size: Vec2::new(32.0, 96.0),
        anchor: Vec2::new(0.5, 0.0),
        needs_depth_sort: true,
    };
    
    pub const TREE: Self = Self {
        size: IVec3::new(2, 2, 4),  // 2x2 base, 4 tiles tall
        sprite_size: Vec2::new(128.0, 160.0),
        anchor: Vec2::new(0.5, 0.0),
        needs_depth_sort: true,
    };
    
    pub const TRASH_CAN: Self = Self {
        size: IVec3::new(1, 1, 1),
        sprite_size: Vec2::new(64.0, 48.0),
        anchor: Vec2::new(0.5, 0.0),
        needs_depth_sort: false,
    };
}

// === SPAWNING FUNCTIONS ===

/// Spawn a multi-tile prop
pub fn spawn_multi_tile_prop(
    commands: &mut Commands,
    prop_def: PropDefinition,
    world_pos: Vec2,
    settings: &IsometricSettings,
    asset_server: &AssetServer,
    texture_path: &str,
) -> Entity {
    let tile_pos = settings.world_to_tile(world_pos);
    let render_pos = settings.tile_to_world(tile_pos);
    let depth = calculate_depth(tile_pos, prop_def.size.z as f32);
    
    let texture: Handle<Image> = asset_server.load(texture_path);
    
    let entity = commands.spawn((
        // texture,
        Sprite {
            custom_size: Some(prop_def.sprite_size),
            anchor: bevy::sprite::Anchor::BottomCenter,
            ..default()
        },
        Transform::from_translation(render_pos.extend(depth)),
        MultiTileProp {
            base_pos: tile_pos,
            tile_size: prop_def.size,
            sprite_size: prop_def.sprite_size,
        },
        IsometricDepth {
            base_depth: depth,
            height_offset: prop_def.size.z as f32 * 0.1,
            dynamic: prop_def.needs_depth_sort,
        },
    )).id();
    
    // Mark tiles as occupied
    if prop_def.size.x > 1 || prop_def.size.y > 1 {
        mark_tiles_occupied(commands, tile_pos, prop_def.size.truncate(), entity);
    }
    
    entity
}

fn calculate_depth(tile_pos: IVec2, height_tiles: f32) -> f32 {
    let base = -(tile_pos.y as f32 + tile_pos.x as f32) * 0.01;
    let height_adjustment = height_tiles * 0.001;
    base + height_adjustment
}

fn mark_tiles_occupied(
    commands: &mut Commands,
    base_pos: IVec2,
    size: IVec2,
    prop_entity: Entity,
) {
    for dy in 0..size.y {
        for dx in 0..size.x {
            let tile_pos = base_pos + IVec2::new(dx, dy);
            commands.spawn(TileOccupant {
                tile_pos,
                occupying_entity: prop_entity,
                blocks_movement: true,
            });
        }
    }
}

// === SYSTEMS ===

/// Dynamic depth sorting for isometric props
pub fn isometric_depth_sorting(
    mut query: Query<(&mut Transform, &IsometricDepth), Changed<Transform>>,
    settings: Res<IsometricSettings>,
) {
    for (mut transform, depth) in query.iter_mut() {
        if !depth.dynamic {
            continue;
        }
        
        let world_pos = transform.translation.truncate();
        let tile_pos = settings.world_to_tile(world_pos);
        let new_depth = calculate_depth(tile_pos, depth.height_offset * 10.0);
        transform.translation.z = new_depth;
    }
}

/// Check collision with multi-tile props
pub fn check_prop_collision(
    world_pos: Vec2,
    settings: &IsometricSettings,
    occupants: &Query<&TileOccupant>,
) -> bool {
    let tile_pos = settings.world_to_tile(world_pos);
    
    for occupant in occupants.iter() {
        if occupant.tile_pos == tile_pos && occupant.blocks_movement {
            return true;
        }
    }
    
    false
}

/// Culling system for props
pub fn prop_culling_system(
    mut props: Query<(&Transform, &mut Visibility), With<MultiTileProp>>,
    camera: Query<&Transform, (With<Camera>, Without<MultiTileProp>)>,
) {
    let Ok(cam_transform) = camera.get_single() else { return; };
    let cam_pos = cam_transform.translation.truncate();
    
    const CULL_DISTANCE: f32 = 1000.0;
    
    for (transform, mut visibility) in props.iter_mut() {
        let distance = cam_pos.distance(transform.translation.truncate());
        *visibility = if distance > CULL_DISTANCE {
            Visibility::Hidden
        } else {
            Visibility::Visible
        };
    }
}