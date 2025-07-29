// src/systems/roads.rs - Efficient road network and flow field pathfinding
use bevy::prelude::*;
use crate::core::*;
use crate::systems::traffic::*;
use crate::systems::death::*;

// === ROAD TILE SYSTEM ===

#[derive(Default, Resource)]
pub struct RoadGrid {
    pub width: usize,
    pub height: usize,
    pub tile_size: f32,
    pub offset: Vec2,
    pub tiles: Vec<RoadTileData>,
    pub dirty: bool,
}

#[derive(Clone)]
pub struct RoadTileData {
    pub tile_type: RoadTileType,
    pub direction: Vec2,
    pub speed_limit: f32,
    pub blocked: bool,
    pub flow_cost: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RoadTileType {
    Empty,
    Road,
    Intersection,
    Sidewalk,
    Building,
}

impl Default for RoadTileData {
    fn default() -> Self {
        Self {
            tile_type: RoadTileType::Empty,
            direction: Vec2::ZERO,
            speed_limit: 100.0,
            blocked: false,
            flow_cost: 1.0,
        }
    }
}

impl RoadGrid {
    pub fn new(world_size: Vec2, tile_size: f32) -> Self {
        let width = (world_size.x / tile_size) as usize;
        let height = (world_size.y / tile_size) as usize;
        let offset = -world_size * 0.5;
        
        let mut grid = Self {
            width,
            height,
            tile_size,
            offset,
            tiles: vec![RoadTileData::default(); width * height],
            dirty: true,
        };
        
        grid.create_default_roads();
        grid
    }
    
    pub fn world_to_grid(&self, world_pos: Vec2) -> Option<(usize, usize)> {
        let local_pos = world_pos - self.offset;
        let x = (local_pos.x / self.tile_size) as usize;
        let y = (local_pos.y / self.tile_size) as usize;
        
        if x < self.width && y < self.height {
            Some((x, y))
        } else {
            None
        }
    }
    
    pub fn grid_to_world(&self, grid_pos: (usize, usize)) -> Vec2 {
        let x = grid_pos.0 as f32 * self.tile_size + self.tile_size * 0.5;
        let y = grid_pos.1 as f32 * self.tile_size + self.tile_size * 0.5;
        self.offset + Vec2::new(x, y)
    }
    
    pub fn get_tile(&self, x: usize, y: usize) -> &RoadTileData {
        if x < self.width && y < self.height {
            &self.tiles[y * self.width + x]
        } else {
            // Return a reference to a static default tile
            static DEFAULT_TILE: RoadTileData = RoadTileData {
                tile_type: RoadTileType::Empty,
                direction: Vec2::ZERO,
                speed_limit: 100.0,
                blocked: false,
                flow_cost: 1.0,
            };
            &DEFAULT_TILE
        }
    }
    
    pub fn set_tile(&mut self, x: usize, y: usize, tile_data: RoadTileData) {
        if x < self.width && y < self.height {
            self.tiles[y * self.width + x] = tile_data;
            self.dirty = true;
        }
    }
    
    fn create_default_roads(&mut self) {
        let center_x = self.width / 2;
        let center_y = self.height / 2;
        
        // Main horizontal road
        for x in 0..self.width {
            self.set_tile(x, center_y, RoadTileData {
                tile_type: RoadTileType::Road,
                direction: if x < center_x { Vec2::NEG_X } else { Vec2::X },
                speed_limit: 120.0,
                blocked: false,
                flow_cost: 1.0,
            });
            
            // Add lanes above and below
            if center_y > 0 {
                self.set_tile(x, center_y - 1, RoadTileData {
                    tile_type: RoadTileType::Road,
                    direction: if x < center_x { Vec2::X } else { Vec2::NEG_X },
                    speed_limit: 120.0,
                    blocked: false,
                    flow_cost: 1.0,
                });
            }
        }
        
        // Main vertical road
        for y in 0..self.height {
            self.set_tile(center_x, y, RoadTileData {
                tile_type: RoadTileType::Road,
                direction: if y < center_y { Vec2::NEG_Y } else { Vec2::Y },
                speed_limit: 120.0,
                blocked: false,
                flow_cost: 1.0,
            });
            
            // Add lanes left and right
            if center_x > 0 {
                self.set_tile(center_x - 1, y, RoadTileData {
                    tile_type: RoadTileType::Road,
                    direction: if y < center_y { Vec2::Y } else { Vec2::NEG_Y },
                    speed_limit: 120.0,
                    blocked: false,
                    flow_cost: 1.0,
                });
            }
        }
        
        // Create intersections
        let intersections = [
            (center_x, center_y),
            (center_x - 1, center_y),
            (center_x, center_y - 1),
            (center_x - 1, center_y - 1),
        ];
        
        for &(x, y) in &intersections {
            self.set_tile(x, y, RoadTileData {
                tile_type: RoadTileType::Intersection,
                direction: Vec2::ZERO, // Multi-directional
                speed_limit: 60.0,
                blocked: false,
                flow_cost: 2.0, // Higher cost for pathfinding
            });
        }
        
        // Add some side streets
        self.create_side_streets();
    }
    
    fn create_side_streets(&mut self) {
        let quarter_x = self.width / 4;
        let three_quarter_x = 3 * self.width / 4;
        let center_y = self.height / 2;
        
        // Left side street
        for y in (center_y - 10)..(center_y + 10) {
            if y < self.height {
                self.set_tile(quarter_x, y, RoadTileData {
                    tile_type: RoadTileType::Road,
                    direction: if y < center_y { Vec2::NEG_Y } else { Vec2::Y },
                    speed_limit: 80.0,
                    blocked: false,
                    flow_cost: 1.2,
                });
            }
        }
        
        // Right side street
        for y in (center_y - 10)..(center_y + 10) {
            if y < self.height {
                self.set_tile(three_quarter_x, y, RoadTileData {
                    tile_type: RoadTileType::Road,
                    direction: if y < center_y { Vec2::NEG_Y } else { Vec2::Y },
                    speed_limit: 80.0,
                    blocked: false,
                    flow_cost: 1.2,
                });
            }
        }
    }
}

// === FLOW FIELD PATHFINDING ===

pub fn update_flow_field_system(
    mut road_grid: ResMut<RoadGrid>,
    mut traffic_system: ResMut<TrafficSystem>,
) {
    if !road_grid.dirty { return; }
    
    // Rebuild flow field from road data
    let flow_field = &mut traffic_system.road_network.flow_field;
    
    for y in 0..flow_field.height {
        for x in 0..flow_field.width {
            let index = y * flow_field.width + x;
            
            // Convert flow field position to road grid position
            let world_pos = Vec2::new(
                (x as f32 - flow_field.width as f32 * 0.5) * flow_field.grid_size,
                (y as f32 - flow_field.height as f32 * 0.5) * flow_field.grid_size,
            );
            
            if let Some((road_x, road_y)) = road_grid.world_to_grid(world_pos) {
                let road_tile = road_grid.get_tile(road_x, road_y);
                
                match road_tile.tile_type {
                    RoadTileType::Road => {
                        flow_field.flow_vectors[index] = road_tile.direction;
                        flow_field.costs[index] = road_tile.flow_cost;
                    },
                    RoadTileType::Intersection => {
                        // Intersection: calculate best direction based on nearby roads
                        flow_field.flow_vectors[index] = calculate_intersection_flow(
                            &road_grid, road_x, road_y
                        );
                        flow_field.costs[index] = road_tile.flow_cost;
                    },
                    _ => {
                        flow_field.flow_vectors[index] = Vec2::ZERO;
                        flow_field.costs[index] = 10.0; // High cost for non-roads
                    }
                }
                
                // Block roads that are marked as blocked
                if road_tile.blocked {
                    flow_field.costs[index] = 100.0;
                }
            }
        }
    }
    
    road_grid.dirty = false;
}

fn calculate_intersection_flow(road_grid: &RoadGrid, x: usize, y: usize) -> Vec2 {
    let mut flow_sum = Vec2::ZERO;
    let mut count = 0;
    
    // Check adjacent tiles for road directions
    let offsets = [(-1, 0), (1, 0), (0, -1), (0, 1)];
    
    for (dx, dy) in offsets {
        let nx = (x as i32 + dx) as usize;
        let ny = (y as i32 + dy) as usize;
        
        if nx < road_grid.width && ny < road_grid.height {
            let neighbor = road_grid.get_tile(nx, ny);
            if neighbor.tile_type == RoadTileType::Road {
                flow_sum += neighbor.direction;
                count += 1;
            }
        }
    }
    
    if count > 0 {
        flow_sum / count as f32
    } else {
        Vec2::ZERO
    }
}

// === EFFICIENT VEHICLE PATHFINDING ===

pub fn flow_field_pathfinding_system(
    mut vehicle_query: Query<(&Transform, &mut TrafficFlow, &TrafficVehicle)>,
    traffic_system: Res<TrafficSystem>,
    road_grid: Res<RoadGrid>,
) {
    let flow_field = &traffic_system.road_network.flow_field;
    
    for (transform, mut flow, vehicle) in vehicle_query.iter_mut() {
        let current_pos = transform.translation.truncate();
        
        // Get flow direction from flow field
        if let Some(flow_direction) = get_flow_at_position(current_pos, flow_field) {
            // Update vehicle's path based on flow field
            if flow.path.is_empty() || flow.path_index >= flow.path.len() {
                update_flow_path(&mut flow, current_pos, flow_direction, vehicle);
            }
        }
    }
}

fn get_flow_at_position(world_pos: Vec2, flow_field: &FlowField) -> Option<Vec2> {
    let grid_x = ((world_pos.x / flow_field.grid_size) + flow_field.width as f32 * 0.5) as usize;
    let grid_y = ((world_pos.y / flow_field.grid_size) + flow_field.height as f32 * 0.5) as usize;
    
    if grid_x < flow_field.width && grid_y < flow_field.height {
        let index = grid_y * flow_field.width + grid_x;
        Some(flow_field.flow_vectors[index])
    } else {
        None
    }
}

fn update_flow_path(flow: &mut TrafficFlow, current_pos: Vec2, flow_direction: Vec2, vehicle: &TrafficVehicle) {
    flow.path.clear();
    
    if flow_direction.length() > 0.1 {
        // Create path following flow direction
        let path_length = match vehicle.vehicle_type {
            TrafficVehicleType::EmergencyAmbulance | TrafficVehicleType::PoliceCar => 300.0,
            TrafficVehicleType::Bus | TrafficVehicleType::Truck => 200.0,
            _ => 150.0,
        };
        
        for i in 1..=4 {
            let waypoint = current_pos + flow_direction * (i as f32 * path_length * 0.25);
            flow.path.push(waypoint);
        }
        
        flow.path_index = 0;
    }
}

// === ROAD BLOCKING SYSTEM ===

pub fn road_blocking_system(
    mut road_grid: ResMut<RoadGrid>,
    explosion_query: Query<&Transform, (With<VehicleExplosion>, Added<VehicleExplosion>)>,
    debris_query: Query<&Transform, With<Corpse>>,
) {
    // Block roads around explosions
    for explosion_transform in explosion_query.iter() {
        let explosion_pos = explosion_transform.translation.truncate();
        block_roads_around_position(&mut road_grid, explosion_pos, 80.0);
    }
    
    // Block roads with vehicle debris (less severe)
    for debris_transform in debris_query.iter() {
        let debris_pos = debris_transform.translation.truncate();
        
        if let Some((x, y)) = road_grid.world_to_grid(debris_pos) {
            let mut tile = road_grid.get_tile(x, y).clone();
            if tile.tile_type == RoadTileType::Road {
                tile.flow_cost = 3.0; // Higher cost but not fully blocked
                road_grid.set_tile(x, y, tile);
            }
        }
    }
}

fn block_roads_around_position(road_grid: &mut RoadGrid, position: Vec2, radius: f32) {
    let tile_radius = (radius / road_grid.tile_size) as usize;
    
    if let Some((center_x, center_y)) = road_grid.world_to_grid(position) {
        for dx in 0..=tile_radius {
            for dy in 0..=tile_radius {
                if (dx * dx + dy * dy) as f32 <= (tile_radius * tile_radius) as f32 {
                    // Block in all four quadrants
                    let positions = [
                        (center_x + dx, center_y + dy),
                        (center_x - dx, center_y + dy),
                        (center_x + dx, center_y - dy),
                        (center_x - dx, center_y - dy),
                    ];
                    
                    for (x, y) in positions {
                        if x < road_grid.width && y < road_grid.height {
                            let mut tile = road_grid.get_tile(x, y).clone();
                            if tile.tile_type == RoadTileType::Road {
                                tile.blocked = true;
                                road_grid.set_tile(x, y, tile);
                            }
                        }
                    }
                }
            }
        }
    }
}

// === ROAD CLEARING SYSTEM ===

pub fn road_clearing_system(
    mut road_grid: ResMut<RoadGrid>,
    mut clear_timer: Local<f32>,
    time: Res<Time>,
) {
    *clear_timer -= time.delta_secs();
    
    if *clear_timer <= 0.0 {
        // Gradually clear blocked roads (emergency services clearing debris)
        let mut dirty = false;
        
        for tile in &mut road_grid.tiles {
            if tile.blocked && rand::random::<f32>() < 0.01 { // 1% chance per tile
                tile.blocked = false;
                dirty = true;
            }
            
            // Reduce flow cost over time
            if tile.flow_cost > 1.0 {
                tile.flow_cost = (tile.flow_cost - 0.1).max(1.0);
                dirty = true;
            }
        }
        
        if dirty {
            road_grid.dirty = true;
        }
        
        *clear_timer = 5.0; // Check every 5 seconds
    }
}

// === SETUP AND DEBUG ===

pub fn setup_road_system(mut commands: Commands) {
    let world_size = Vec2::new(1000.0, 1000.0);
    let tile_size = 20.0; // Larger tiles for performance
    
    commands.insert_resource(RoadGrid::new(world_size, tile_size));
    info!("Road grid initialized: {}x{} tiles", 
          (world_size.x / tile_size) as usize, 
          (world_size.y / tile_size) as usize);
}

#[cfg(debug_assertions)]
pub fn debug_road_system(
    mut gizmos: Gizmos,
    road_grid: Res<RoadGrid>,
    input: Res<ButtonInput<KeyCode>>,
    mut show_roads: Local<bool>,
) {
    if input.just_pressed(KeyCode::KeyR) {
        *show_roads = !*show_roads;
        info!("Road debug: {}", if *show_roads { "ON" } else { "OFF" });
    }
    
    if !*show_roads { return; }
    
    // Draw road tiles
    for y in 0..road_grid.height {
        for x in 0..road_grid.width {
            let tile = road_grid.get_tile(x, y);
            
            if tile.tile_type != RoadTileType::Empty {
                let world_pos = road_grid.grid_to_world((x, y));
                let size = Vec2::splat(road_grid.tile_size * 0.8);
                
                let color = match tile.tile_type {
                    RoadTileType::Road => {
                        if tile.blocked {
                            Color::srgb(0.8, 0.2, 0.2) // Red for blocked
                        } else {
                            Color::srgb(0.3, 0.3, 0.3) // Dark gray for road
                        }
                    },
                    RoadTileType::Intersection => Color::srgb(0.5, 0.5, 0.2), // Yellow
                    RoadTileType::Sidewalk => Color::srgb(0.6, 0.6, 0.6), // Light gray
                    RoadTileType::Building => Color::srgb(0.4, 0.4, 0.4), // Medium gray
                    RoadTileType::Empty => continue,
                };
                
                gizmos.rect_2d(
                    Isometry2d::from_translation(world_pos),
                    size,
                    color
                );
                
                // Draw flow direction arrows
                if tile.tile_type == RoadTileType::Road && tile.direction.length() > 0.1 {
                    let arrow_start = world_pos;
                    let arrow_end = world_pos + tile.direction * road_grid.tile_size * 0.3;
                    gizmos.arrow_2d(arrow_start, arrow_end, Color::srgb(0.0, 1.0, 0.0));
                }
            }
        }
    }
}

