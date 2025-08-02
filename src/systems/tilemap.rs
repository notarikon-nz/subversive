// src/systems/tilemap.rs - Isometric tilemap system for Subversive
use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use crate::core::*;
use crate::systems::urban_simulation::{UrbanAreas, UrbanZone};
use crate::systems::scenes::{SceneData};

// === TILEMAP COMPONENTS ===
#[derive(Component)]
pub struct IsometricMap;

#[derive(Component)]
pub struct TilePosition {
    pub x: i32,
    pub y: i32,
}

use crate::systems::tile_properties::{TileType};

// === ISOMETRIC CONVERSION ===
#[derive(Resource)]
pub struct IsometricSettings {
    pub tile_width: f32,
    pub tile_height: f32,
    pub map_width: u32,
    pub map_height: u32,
}

impl Default for IsometricSettings {
    fn default() -> Self {
        Self {
            tile_width: 64.0,   // Width of isometric tile
            tile_height: 32.0,  // Height of isometric tile
            map_width: 50,      // Tiles wide
            map_height: 40,     // Tiles tall
        }
    }
}

// === COORDINATE CONVERSION ===
impl IsometricSettings {
    /// Convert world coordinates to tile coordinates
    pub fn world_to_tile(&self, world_pos: Vec2) -> IVec2 {
        // Reverse the isometric projection
        let screen_x = world_pos.x;
        let screen_y = world_pos.y;

        // Isometric to tile conversion
        let tile_x = ((screen_x / (self.tile_width * 0.5)) + (screen_y / (self.tile_height * 0.5))) * 0.5;
        let tile_y = ((screen_y / (self.tile_height * 0.5)) - (screen_x / (self.tile_width * 0.5))) * 0.5;

        IVec2::new(tile_x.floor() as i32, tile_y.floor() as i32)
    }

    /// Convert tile coordinates to world coordinates (center of tile)
    pub fn tile_to_world(&self, tile_pos: IVec2) -> Vec2 {
        let x = (tile_pos.x - tile_pos.y) as f32 * (self.tile_width * 0.5);
        let y = (tile_pos.x + tile_pos.y) as f32 * (self.tile_height * 0.5);
        Vec2::new(x, y)
    }

    /// Convert screen coordinates to world coordinates for isometric camera
    pub fn screen_to_world(&self, screen_pos: Vec2, camera_transform: &Transform) -> Vec2 {
        // Basic screen to world conversion - camera transform handles the rest
        let world_pos = screen_pos + camera_transform.translation.truncate();
        world_pos
    }
}

// === TILEMAP SETUP ===
pub fn setup_isometric_tilemap(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    let settings = IsometricSettings::default();

    // Load tilemap texture
    let texture_handle: Handle<Image> = asset_server.load("tilemaps/iso_tiles.png");

    // Create tilemap entity
    let map_size = TilemapSize {
        x: settings.map_width,
        y: settings.map_height
    };

    let tilemap_entity = commands.spawn_empty().id();
    let mut tile_storage = TileStorage::empty(map_size);

    // Fill map with basic grass tiles initially
    for x in 0..map_size.x {
        for y in 0..map_size.y {
            let tile_pos = TilePos { x, y };
            let tile_entity = commands.spawn((
                TileBundle {
                    position: tile_pos,
                    tilemap_id: TilemapId(tilemap_entity),
                    texture_index: TileTextureIndex(0), // Grass tile
                    ..Default::default()
                },
                TilePosition { x: x as i32, y: y as i32 },
            )).id();

            tile_storage.set(&tile_pos, tile_entity);
        }
    }

    // Configure tilemap
    let tile_size = TilemapTileSize {
        x: settings.tile_width,
        y: settings.tile_height
    };

    let grid_size = tile_size.into();
    let map_type = bevy_ecs_tilemap::map::TilemapType::Isometric(IsoCoordSystem::Diamond);

    let center_transform = get_tilemap_center_transform(
        &map_size,
        &grid_size,
        &map_type,         // &bevy_ecs_tilemap::map::TilemapType - THIS IS WRONG
        0.0
    );


    commands.entity(tilemap_entity).insert((
        TilemapBundle {
            grid_size,
            map_type,        //bevy_ecs_tilemap::map::TilemapType - THIS IS WRONG
            size: map_size,
            storage: tile_storage,
            texture: TilemapTexture::Single(texture_handle),
            tile_size,
            transform: center_transform,
            ..Default::default()
        },
        IsometricMap,
    ));

    commands.insert_resource(settings);
    info!("Isometric tilemap initialized: {}x{} tiles", map_size.x, map_size.y);
}

// === SCENE TO TILEMAP CONVERSION ===
pub fn generate_tilemap_from_scene(
    mut commands: Commands,
    scene_data: Res<SceneData>,
    urban_areas: Res<UrbanAreas>,
    settings: Res<IsometricSettings>,
    tilemap_query: Query<(Entity, &TileStorage), With<IsometricMap>>,
) {
    let Ok((tilemap_entity, tile_storage)) = tilemap_query.single() else { return; };

    // Generate base terrain
    generate_base_terrain(&mut commands, &settings, tilemap_entity, tile_storage);

    // Add urban zones
    apply_urban_zones(&mut commands, &urban_areas, &settings, tilemap_entity, tile_storage);

    // Add roads and infrastructure
    generate_road_network(&mut commands, &settings, tilemap_entity, tile_storage);

    // Add buildings for enemy/terminal positions
    apply_scene_structures(&mut commands, &scene_data, &settings, tilemap_entity, tile_storage);
}

fn generate_base_terrain(
    commands: &mut Commands,
    settings: &IsometricSettings,
    tilemap_entity: Entity,
    tile_storage: &TileStorage,
) {
    // Create varied terrain base
    for y in 0..settings.map_height {
        for x in 0..settings.map_width {
            let tile_pos = TilePos { x, y };

            if let Some(tile_entity) = tile_storage.get(&tile_pos) {
                // Vary terrain based on position
                let texture_index = match (x + y) % 4 {
                    0 => 0, // Grass
                    1 => 1, // Dirt
                    2 => 2, // Concrete
                    _ => 0, // Default grass
                };

                commands.entity(tile_entity).insert(TileTextureIndex(texture_index));
            }
        }
    }
}

fn apply_urban_zones(
    commands: &mut Commands,
    urban_areas: &UrbanAreas,
    settings: &IsometricSettings,
    tilemap_entity: Entity,
    tile_storage: &TileStorage,
) {
    // Apply work zones (industrial/commercial tiles)
    for zone in &urban_areas.work_zones {
        apply_zone_to_tiles(commands, zone, 10, settings, tile_storage); // Industrial texture
    }

    // Apply shopping zones (commercial tiles)
    for zone in &urban_areas.shopping_zones {
        apply_zone_to_tiles(commands, zone, 11, settings, tile_storage); // Commercial texture
    }

    // Apply residential zones
    for zone in &urban_areas.residential_zones {
        apply_zone_to_tiles(commands, zone, 12, settings, tile_storage); // Residential texture
    }
}

fn apply_zone_to_tiles(
    commands: &mut Commands,
    zone: &UrbanZone,
    texture_index: u32,
    settings: &IsometricSettings,
    tile_storage: &TileStorage,
) {
    let center_tile = settings.world_to_tile(zone.center);
    let radius_tiles = (zone.radius / (settings.tile_width * 0.5)) as i32;

    for y in (center_tile.y - radius_tiles)..=(center_tile.y + radius_tiles) {
        for x in (center_tile.x - radius_tiles)..=(center_tile.x + radius_tiles) {
            if x >= 0 && y >= 0 && x < settings.map_width as i32 && y < settings.map_height as i32 {
                let tile_world_pos = settings.tile_to_world(IVec2::new(x, y));
                if zone.center.distance(tile_world_pos) <= zone.radius {
                    let tile_pos = TilePos { x: x as u32, y: y as u32 };
                    if let Some(tile_entity) = tile_storage.get(&tile_pos) {
                        commands.entity(tile_entity).insert(TileTextureIndex(texture_index));
                    }
                }
            }
        }
    }
}

fn generate_road_network(
    commands: &mut Commands,
    settings: &IsometricSettings,
    tilemap_entity: Entity,
    tile_storage: &TileStorage,
) {
    // Create main roads (horizontal and vertical)
    let road_texture = 20; // Road tile index

    // Horizontal road through middle
    let mid_y = settings.map_height / 2;
    for x in 0..settings.map_width {
        let tile_pos = TilePos { x, y: mid_y };
        if let Some(tile_entity) = tile_storage.get(&tile_pos) {
            commands.entity(tile_entity).insert(TileTextureIndex(road_texture));
        }
    }

    // Vertical road through middle
    let mid_x = settings.map_width / 2;
    for y in 0..settings.map_height {
        let tile_pos = TilePos { x: mid_x, y };
        if let Some(tile_entity) = tile_storage.get(&tile_pos) {
            commands.entity(tile_entity).insert(TileTextureIndex(road_texture));
        }
    }
}

fn apply_scene_structures(
    commands: &mut Commands,
    scene_data: &SceneData,
    settings: &IsometricSettings,
    tilemap_entity: Entity,
    tile_storage: &TileStorage,
) {
    let building_texture = 30; // Building tile index

    // Add buildings around enemy positions
    for enemy in &scene_data.enemies {
        let world_pos = Vec2::from(enemy.position);
        let tile_pos = settings.world_to_tile(world_pos);

        // Create small building cluster
        for dy in -1..=1 {
            for dx in -1..=1 {
                let check_pos = IVec2::new(tile_pos.x + dx, tile_pos.y + dy);
                if check_pos.x >= 0 && check_pos.y >= 0 &&
                   check_pos.x < settings.map_width as i32 && check_pos.y < settings.map_height as i32 {
                    let tile_pos = TilePos { x: check_pos.x as u32, y: check_pos.y as u32 };
                    if let Some(tile_entity) = tile_storage.get(&tile_pos) {
                        commands.entity(tile_entity).insert(TileTextureIndex(building_texture));
                    }
                }
            }
        }
    }

    // Add special tiles for terminals
    let terminal_texture = 31;
    for terminal in &scene_data.terminals {
        let world_pos = Vec2::from(terminal.position);
        let tile_pos = settings.world_to_tile(world_pos);

        if tile_pos.x >= 0 && tile_pos.y >= 0 &&
           tile_pos.x < settings.map_width as i32 && tile_pos.y < settings.map_height as i32 {
            let tile_pos = TilePos { x: tile_pos.x as u32, y: tile_pos.y as u32 };
            if let Some(tile_entity) = tile_storage.get(&tile_pos) {
                commands.entity(tile_entity).insert(TileTextureIndex(terminal_texture));
            }
        }
    }
}

// === PATHFINDING INTEGRATION ===
pub fn update_pathfinding_from_tilemap(
    mut pathfinding_grid: ResMut<crate::systems::pathfinding::PathfindingGrid>,
    settings: Res<IsometricSettings>,
    tilemap_query: Query<&TileStorage, With<IsometricMap>>,
    tile_query: Query<&TileTextureIndex>,
) {
    let Ok(tile_storage) = tilemap_query.single() else { return; };

    let grid_width = settings.map_width as usize;
    let grid_height = settings.map_height as usize;

    // Clear and resize pathfinding grid to match tilemap
    pathfinding_grid.width = grid_width;
    pathfinding_grid.height = grid_height;

    pathfinding_grid.tile_size = (settings.tile_width + settings.tile_height) * 0.5; // Average for pathfinding
    pathfinding_grid.offset = -(Vec2::new(settings.map_width as f32, settings.map_height as f32) * pathfinding_grid.tile_size * 0.5);
    pathfinding_grid.tiles.clear();
    pathfinding_grid.tiles.resize(grid_width * grid_height, crate::systems::pathfinding::TileType::Walkable);

    // Update pathfinding grid based on tile types
    for y in 0..settings.map_height {
        for x in 0..settings.map_width {
            let tile_pos = TilePos { x, y };
            if let Some(tile_entity) = tile_storage.get(&tile_pos) {
                if let Ok(texture_index) = tile_query.get(tile_entity) {
                    let pathfinding_type = match texture_index.0 {
                        20 => crate::systems::pathfinding::TileType::Walkable, // Roads
                        30..=39 => crate::systems::pathfinding::TileType::Blocked, // Buildings
                        _ => crate::systems::pathfinding::TileType::Walkable, // Default walkable
                    };

                    pathfinding_grid.set_tile(x as usize, y as usize, pathfinding_type);
                }
            }
        }
    }

    pathfinding_grid.dirty = false;
    info!("Updated pathfinding grid from tilemap");
}

// === MOUSE INPUT FOR ISOMETRIC ===
pub fn handle_isometric_mouse_input(
    windows: Query<&Window>,
    cameras: Query<(&Camera, &GlobalTransform)>,
    settings: Res<IsometricSettings>,
    mut action_events: EventWriter<ActionEvent>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    selection: Res<SelectionState>,
) {
    if !mouse_button.just_pressed(MouseButton::Right) { return; }
    if selection.selected.is_empty() { return; }

    let Ok(window) = windows.single() else { return; };
    let Ok((camera, camera_transform)) = cameras.single() else { return; };
    let Some(cursor_pos) = window.cursor_position() else { return; };

    // Convert screen to world coordinates for isometric
    if let Ok(world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_pos) {
        // Send move command to selected units
        for &selected_entity in &selection.selected {
            action_events.write(ActionEvent {
                entity: selected_entity,
                action: Action::MoveTo(world_pos),
            });
        }
    }
}

// === UTILITIES ===
pub fn get_tile_type_from_texture(texture_index: u32) -> TileType {
    match texture_index {
        0 => TileType::Grass,
        1 => TileType::Concrete,
        20 => TileType::Road,
        21 => TileType::Sidewalk,
        30..=39 => TileType::Building,
        _ => TileType::Grass,
    }
}

pub fn get_texture_from_tile_type(tile_type: TileType) -> u32 {
    match tile_type {
        TileType::Grass => 0,
        TileType::Concrete => 1,
        TileType::Asphalt => 2,
        TileType::Road => 20,
        TileType::Sidewalk => 21,
        TileType::Building => 30,
        TileType::Wall => 31,
        TileType::Residential => 12,
        TileType::Commercial => 11,
        TileType::Industrial => 10,
        _ => 0,
    }
}



// Add this new function for isometric-specific mouse handling:
pub fn get_isometric_world_mouse_position(
    windows: &Query<&Window>,
    cameras: &Query<(&Camera, &GlobalTransform, &crate::systems::isometric_camera::IsometricCamera)>,
) -> Option<Vec2> {
    let window = windows.single().ok()?;
    let (camera, camera_transform, _) = cameras.single().ok()?;
    let cursor_pos = window.cursor_position()?;

    // Convert cursor position to world coordinates for isometric camera
    camera.viewport_to_world_2d(camera_transform, cursor_pos).ok()
}

// Alternative: Create a unified function that tries both camera types
pub fn get_unified_world_mouse_position(
    windows: &Query<&Window>,
    regular_cameras: &Query<(&Camera, &GlobalTransform), Without<crate::systems::isometric_camera::IsometricCamera>>,
    isometric_cameras: &Query<(&Camera, &GlobalTransform, &crate::systems::isometric_camera::IsometricCamera)>,
) -> Option<Vec2> {
    let window = windows.single().ok()?;
    let cursor_pos = window.cursor_position()?;

    // Try isometric camera first
    if let Ok((camera, camera_transform, _)) = isometric_cameras.single() {
        return camera.viewport_to_world_2d(camera_transform, cursor_pos).ok();
    }

    // Fallback to regular camera
    if let Ok((camera, camera_transform)) = regular_cameras.single() {
        return camera.viewport_to_world_2d(camera_transform, cursor_pos).ok();
    }

    None
}
