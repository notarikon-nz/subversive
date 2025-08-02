// src/systems/enhanced_pathfinding.rs - Pathfinding with tile properties integration
use bevy::prelude::*;
use crate::core::*;
use crate::systems::pathfinding::*;
use crate::systems::tile_properties::*;
use std::collections::{BinaryHeap, HashMap, HashSet};

// === ENHANCED PATHFINDING TYPES ===
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum EnhancedTileType {
    Walkable(MovementCost),
    Blocked,
    Difficult(MovementCost),
    Hazardous(MovementCost), // Walkable but dangerous
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct MovementCost {
    pub base_cost: u32, // Multiplied by 100 for integer math (1.5 = 150)
    pub environmental_cost: u32, // Additional cost from weather, etc.
}

// === ENHANCED A* NODE ===
#[derive(Clone, Debug)]
struct EnhancedNode {
    pos: (usize, usize),
    g_cost: f32,
    h_cost: f32,
    movement_cost: f32,     // Actual movement cost to reach this tile
    safety_cost: f32,       // Additional cost for dangerous tiles
    parent: Option<(usize, usize)>,
}

impl EnhancedNode {
    fn f_cost(&self) -> f32 {
        self.g_cost + self.h_cost + self.safety_cost
    }
}

impl PartialEq for EnhancedNode {
    fn eq(&self, other: &Self) -> bool {
        self.pos == other.pos
    }
}

impl Eq for EnhancedNode {}

impl PartialOrd for EnhancedNode {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for EnhancedNode {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other.f_cost().partial_cmp(&self.f_cost()).unwrap_or(std::cmp::Ordering::Equal)
    }
}

// === ENHANCED PATHFINDING GRID ===
#[derive(Resource, Default)]
pub struct EnhancedPathfindingGrid {
    pub width: usize,
    pub height: usize,
    pub tile_size: f32,
    pub offset: Vec2,
    pub tiles: Vec<EnhancedTileType>,
    pub movement_costs: Vec<f32>,      // Per-tile movement multipliers
    pub safety_costs: Vec<f32>,        // Per-tile danger penalties
    pub cover_values: Vec<f32>,        // Per-tile cover protection
    pub vision_blocking: Vec<bool>,    // Per-tile vision occlusion
    pub dirty: bool,
}

impl EnhancedPathfindingGrid {
    pub fn new(world_size: Vec2, tile_size: f32) -> Self {
        let width = (world_size.x / tile_size) as usize;
        let height = (world_size.y / tile_size) as usize;
        let tile_count = width * height;
        
        Self {
            width,
            height,
            tile_size,
            offset: -world_size * 0.5,
            tiles: vec![EnhancedTileType::Walkable(MovementCost { base_cost: 100, environmental_cost: 0 }); tile_count],
            movement_costs: vec![1.0; tile_count],
            safety_costs: vec![0.0; tile_count],
            cover_values: vec![0.0; tile_count],
            vision_blocking: vec![false; tile_count],
            dirty: true,
        }
    }
    
    pub fn get_tile_index(&self, x: usize, y: usize) -> Option<usize> {
        if x < self.width && y < self.height {
            Some(y * self.width + x)
        } else {
            None
        }
    }
    
    pub fn set_tile_properties(&mut self, x: usize, y: usize, properties: &TileProperties) {
        if let Some(index) = self.get_tile_index(x, y) {
            // Set movement cost
            self.movement_costs[index] = properties.movement_cost;
            
            // Set tile type based on movement cost
            self.tiles[index] = if !properties.can_move_through() {
                EnhancedTileType::Blocked
            } else if properties.movement_cost > 1.5 {
                EnhancedTileType::Difficult(MovementCost {
                    base_cost: (properties.movement_cost * 100.0) as u32,
                    environmental_cost: 0,
                })
            } else if properties.environmental.is_flammable || properties.environmental.conducts_electricity {
                EnhancedTileType::Hazardous(MovementCost {
                    base_cost: (properties.movement_cost * 100.0) as u32,
                    environmental_cost: 50, // Extra cost for hazards
                })
            } else {
                EnhancedTileType::Walkable(MovementCost {
                    base_cost: (properties.movement_cost * 100.0) as u32,
                    environmental_cost: 0,
                })
            };
            
            // Set safety cost (higher for dangerous areas)
            self.safety_costs[index] = if properties.environmental.conducts_electricity && properties.environmental.water_level > 0.0 {
                100.0 // Very dangerous: electrified water
            } else if properties.environmental.temperature > 60.0 {
                50.0  // Hot surfaces
            } else if properties.environmental.is_flammable {
                25.0  // Flammable areas
            } else {
                0.0   // Safe
            };
            
            // Set cover and vision
            self.cover_values[index] = properties.provides_cover;
            self.vision_blocking[index] = properties.blocks_vision;
        }
    }
    
    pub fn get_movement_cost(&self, x: usize, y: usize) -> f32 {
        self.get_tile_index(x, y)
            .map(|index| self.movement_costs[index])
            .unwrap_or(f32::INFINITY)
    }
    
    pub fn get_cover_value(&self, x: usize, y: usize) -> f32 {
        self.get_tile_index(x, y)
            .map(|index| self.cover_values[index])
            .unwrap_or(0.0)
    }
    
    pub fn blocks_vision(&self, x: usize, y: usize) -> bool {
        self.get_tile_index(x, y)
            .map(|index| self.vision_blocking[index])
            .unwrap_or(true)
    }

    pub fn world_to_tile(&self, world_pos: Vec2) -> Option<IVec2> {
        let local_pos = world_pos - self.offset;
        let x = (local_pos.x / self.tile_size) as i32;
        let y = (local_pos.y / self.tile_size) as i32;
        
        if x >= 0 && y >= 0 && x < self.width as i32 && y < self.height as i32 {
            Some(IVec2::new(x, y))
        } else {
            None
        }
    }
    
    pub fn grid_to_world(&self, grid_pos: IVec2) -> Vec2 {
        let x = grid_pos.x as f32 * self.tile_size + self.tile_size * 0.5;
        let y = grid_pos.y as f32 * self.tile_size + self.tile_size * 0.5;
        self.offset + Vec2::new(x, y)
    }    
}

// === ENHANCED PATHFINDING ALGORITHM ===
pub fn find_enhanced_path(
    grid: &EnhancedPathfindingGrid, 
    start: Vec2, 
    goal: Vec2,
    prefer_cover: bool,
    avoid_hazards: bool,
) -> Option<Vec<Vec2>> {
    let start_grid = grid.world_to_tile(start)?;
    let goal_grid = grid.world_to_tile(goal)?;
    
    if !is_tile_walkable(grid, goal_grid.x as usize, goal_grid.y as usize) {
        return None;
    }
    
    let mut open_set = BinaryHeap::new();
    let mut closed_set = HashSet::new();
    let mut came_from = HashMap::new();
    
    let start_node = EnhancedNode {
        pos: (start_grid.x as usize, start_grid.y as usize),
        g_cost: 0.0,
        h_cost: enhanced_heuristic((start_grid.x as usize, start_grid.y as usize), (goal_grid.x as usize, goal_grid.y as usize)),
        movement_cost: 0.0,
        safety_cost: 0.0,
        parent: None,
    };
    
    open_set.push(start_node);
    
    while let Some(current) = open_set.pop() {
        if current.pos == (goal_grid.x as usize, goal_grid.y as usize) {
            return Some(reconstruct_enhanced_path(grid, came_from, current.pos, start, goal));
        }
        
        closed_set.insert(current.pos);
        
        for neighbor_pos in get_enhanced_neighbors(grid, current.pos) {
            if closed_set.contains(&neighbor_pos) {
                continue;
            }
            
            if !is_tile_walkable(grid, neighbor_pos.0, neighbor_pos.1) {
                continue;
            }
            
            let movement_cost = get_enhanced_movement_cost(grid, current.pos, neighbor_pos, prefer_cover, avoid_hazards);
            let tentative_g = current.g_cost + movement_cost;
            
            let neighbor_node = EnhancedNode {
                pos: neighbor_pos,
                g_cost: tentative_g,
                h_cost: enhanced_heuristic(neighbor_pos, (goal_grid.x as usize, goal_grid.y as usize)),
                movement_cost: grid.get_movement_cost(neighbor_pos.0, neighbor_pos.1),
                safety_cost: if avoid_hazards { 
                    grid.get_tile_index(neighbor_pos.0, neighbor_pos.1)
                        .map(|idx| grid.safety_costs[idx])
                        .unwrap_or(0.0)
                } else { 
                    0.0 
                },
                parent: Some(current.pos),
            };
            
            let mut should_add = true;
            for existing in &open_set {
                if existing.pos == neighbor_pos && existing.g_cost <= tentative_g {
                    should_add = false;
                    break;
                }
            }
            
            if should_add {
                came_from.insert(neighbor_pos, current.pos);
                open_set.push(neighbor_node);
            }
        }
    }
    
    None
}

fn get_enhanced_neighbors(grid: &EnhancedPathfindingGrid, pos: (usize, usize)) -> Vec<(usize, usize)> {
    let mut neighbors = Vec::new();
    let (x, y) = pos;
    
    for dx in -1i32..=1 {
        for dy in -1i32..=1 {
            if dx == 0 && dy == 0 { continue; }
            
            let nx = x as i32 + dx;
            let ny = y as i32 + dy;
            
            if nx >= 0 && ny >= 0 && 
               (nx as usize) < grid.width && (ny as usize) < grid.height {
                neighbors.push((nx as usize, ny as usize));
            }
        }
    }
    neighbors
}

fn is_tile_walkable(grid: &EnhancedPathfindingGrid, x: usize, y: usize) -> bool {
    if let Some(index) = grid.get_tile_index(x, y) {
        !matches!(grid.tiles[index], EnhancedTileType::Blocked)
    } else {
        false
    }
}

fn get_enhanced_movement_cost(
    grid: &EnhancedPathfindingGrid,
    from: (usize, usize),
    to: (usize, usize),
    prefer_cover: bool,
    avoid_hazards: bool,
) -> f32 {
    let base_cost = if from.0 != to.0 && from.1 != to.1 { 
        1.41421356
    } else { 
        1.0
    };
    
    let movement_multiplier = grid.get_movement_cost(to.0, to.1);
    let mut total_cost = base_cost * movement_multiplier;
    
    if prefer_cover {
        let cover_value = grid.get_cover_value(to.0, to.1);
        total_cost -= cover_value * 0.5;
    }
    
    if avoid_hazards {
        if let Some(index) = grid.get_tile_index(to.0, to.1) {
            total_cost += grid.safety_costs[index] * 0.1;
        }
    }
    
    total_cost.max(0.1)
}

fn enhanced_heuristic(a: (usize, usize), b: (usize, usize)) -> f32 {
    let dx = (a.0 as f32 - b.0 as f32).abs();
    let dy = (a.1 as f32 - b.1 as f32).abs();
    
    let diagonal = dx.min(dy);
    let straight = (dx - diagonal) + (dy - diagonal);
    
    diagonal * 1.41421356 + straight
}

fn reconstruct_enhanced_path(
    grid: &EnhancedPathfindingGrid,
    came_from: HashMap<(usize, usize), (usize, usize)>,
    goal: (usize, usize),
    start_world: Vec2,
    goal_world: Vec2
) -> Vec<Vec2> {
    let mut path = Vec::new();
    let mut current = goal;
    
    while let Some(&parent) = came_from.get(&current) {
        path.push(grid.grid_to_world(IVec2::new(current.0 as i32, current.1 as i32)));
        current = parent;
    }
    
    path.push(start_world);
    path.reverse();
    
    smooth_enhanced_path(&mut path, grid);
    
    path
}

fn smooth_enhanced_path(path: &mut Vec<Vec2>, grid: &EnhancedPathfindingGrid) {
    if path.len() < 3 { return; }
    
    let mut i = 0;
    while i + 2 < path.len() {
        let p1 = path[i];
        let p3 = path[i + 2];
        
        if can_move_directly(grid, p1, p3) {
            path.remove(i + 1);
        } else {
            i += 1;
        }
    }
}

fn can_move_directly(grid: &EnhancedPathfindingGrid, from: Vec2, to: Vec2) -> bool {
    let from_tile = grid.world_to_tile(from);
    let to_tile = grid.world_to_tile(to);
    
    if from_tile.is_none() || to_tile.is_none() { return false; }
    
    let from_tile = from_tile.unwrap();
    let to_tile = to_tile.unwrap();
    
    let dx = (to_tile.x - from_tile.x).abs();
    let dy = (to_tile.y - from_tile.y).abs();
    let steps = dx.max(dy) as usize;
    
    if steps == 0 { return true; }
    
    for step in 0..=steps {
        let t = step as f32 / steps as f32;
        let x = (from_tile.x as f32 + t * (to_tile.x - from_tile.x) as f32) as usize;
        let y = (from_tile.y as f32 + t * (to_tile.y - from_tile.y) as f32) as usize;
        
        if !is_tile_walkable(grid, x, y) {
            return false;
        }
    }
    
    true
}

// === SYSTEM FUNCTIONS ===

pub fn update_enhanced_pathfinding_system(
    mut enhanced_grid: ResMut<EnhancedPathfindingGrid>,
    tile_query: Query<(&TileProperties, &bevy_ecs_tilemap::tiles::TilePos), Changed<TileProperties>>,
    isometric_settings: Res<crate::systems::tilemap::IsometricSettings>,
) {
    if tile_query.is_empty() { return; }
    
    if enhanced_grid.width != isometric_settings.map_width as usize ||
       enhanced_grid.height != isometric_settings.map_height as usize {
        *enhanced_grid = EnhancedPathfindingGrid::new(
            Vec2::new(
                isometric_settings.map_width as f32 * isometric_settings.tile_width,
                isometric_settings.map_height as f32 * isometric_settings.tile_height,
            ),
            (isometric_settings.tile_width + isometric_settings.tile_height) * 0.5,
        );
    }
    
    for (properties, tile_pos) in tile_query.iter() {
        enhanced_grid.set_tile_properties(tile_pos.x as usize, tile_pos.y as usize, properties);
    }
    
    enhanced_grid.dirty = false;
}

// System to provide enhanced pathfinding capability to agents
pub fn add_enhanced_pathfinding_to_agents(
    mut commands: Commands,
    agents: Query<Entity, (With<Agent>, Without<crate::systems::pathfinding::PathfindingAgent>)>,
) {
    for entity in agents.iter() {
        commands.entity(entity).insert(crate::systems::pathfinding::PathfindingAgent {
            current_path: Vec::new(),
            path_index: 0,
            recalculate: false,
        });
    }
}

// Enhanced movement system that uses tile properties
pub fn enhanced_movement_system(
    mut commands: Commands,
    mut action_events: EventReader<ActionEvent>,
    mut agents: Query<(Entity, &mut Transform, &MovementSpeed, &mut crate::systems::pathfinding::PathfindingAgent), With<Agent>>,
    enhanced_grid: Res<EnhancedPathfindingGrid>,
    time: Res<Time>,
    game_mode: Res<GameMode>,
) {
    if game_mode.paused { return; }
    
    // Handle new movement commands with enhanced pathfinding
    for event in action_events.read() {
        if let Action::MoveTo(target_pos) = event.action {
            if let Ok((entity, transform, _, mut agent)) = agents.get_mut(event.entity) {
                let start_pos = transform.translation.truncate();
                
                // Use enhanced pathfinding with preferences
                if let Some(path) = find_enhanced_path(
                    &enhanced_grid, 
                    start_pos, 
                    target_pos,
                    true,  // prefer_cover
                    true,  // avoid_hazards
                ) {
                    agent.current_path = path;
                    agent.path_index = 0;
                    agent.recalculate = false;
                    info!("Enhanced path found with {} waypoints", agent.current_path.len());
                } else {
                    warn!("No enhanced path found from {:?} to {:?}", start_pos, target_pos);
                    agent.current_path.clear();
                }
            }
        }
    }
    
    // Execute pathfinding movement with tile-based speed modifications
    for (entity, mut transform, speed, mut agent) in agents.iter_mut() {
        if agent.current_path.is_empty() { continue; }
        
        let current_pos = transform.translation.truncate();
        
        if agent.path_index >= agent.current_path.len() {
            agent.current_path.clear();
            continue;
        }
        
        let target = agent.current_path[agent.path_index];
        let direction = (target - current_pos).normalize_or_zero();
        let distance = current_pos.distance(target);
        
        if distance < 8.0 { // Close enough to waypoint
            agent.path_index += 1;
            if agent.path_index >= agent.current_path.len() {
                agent.current_path.clear();
                continue;
            }
        }
        
        // Get movement cost from current tile
        let tile_pos = enhanced_grid.world_to_tile(current_pos);
        let movement_multiplier = if let Some(tile_pos) = tile_pos {
            1.0 / enhanced_grid.get_movement_cost(tile_pos.x as usize, tile_pos.y as usize).max(0.1)
        } else {
            1.0
        };
        
        // Apply movement with tile-based speed modification
        let effective_speed = speed.0 * movement_multiplier;
        let movement = direction * effective_speed * time.delta_secs();
        transform.translation += movement.extend(0.0);
    }
}

// System to check line of sight using tile properties
pub fn enhanced_vision_system(
    mut vision_query: Query<(&mut Vision, &Transform), With<Agent>>,
    enhanced_grid: Res<EnhancedPathfindingGrid>,
) {
    for (mut vision, transform) in vision_query.iter_mut() {
        let observer_pos = transform.translation.truncate();
        
        // Update vision based on tile properties
        if let Some(tile_pos) = enhanced_grid.world_to_tile(observer_pos) {
            // Check if current tile blocks vision (agent is behind cover)
            if enhanced_grid.blocks_vision(tile_pos.x as usize, tile_pos.y as usize) {
                // Reduce vision when behind vision-blocking tiles
                vision.range *= 0.7;
            }
            
            // Could add weather effects, lighting conditions, etc. here
            let cover_value = enhanced_grid.get_cover_value(tile_pos.x as usize, tile_pos.y as usize);
            if cover_value > 0.5 {
                // Heavy cover also reduces detection range
                vision.range *= 0.8;
            }
        }
    }
}

// System to handle cover mechanics
pub fn enhanced_cover_system(
    mut agents: Query<(Entity, &Transform, &mut Health), With<Agent>>,
    enhanced_grid: Res<EnhancedPathfindingGrid>,
    mut damage_events: EventReader<CombatEvent>,
) {
    for damage_event in damage_events.read() {
        if let Ok((entity, transform, mut health)) = agents.get_mut(damage_event.target) {
            let target_pos = transform.translation.truncate();
            
            if let Some(tile_pos) = enhanced_grid.world_to_tile(target_pos) {
                let cover_value = enhanced_grid.get_cover_value(tile_pos.x as usize, tile_pos.y as usize);
                
                // Reduce damage based on cover
                let damage_reduction = cover_value * 0.8; // Up to 80% damage reduction
                let final_damage = damage_event.damage * (1.0 - damage_reduction);
                
                // Apply the reduced damage
                health.0 -= final_damage;
                
                if cover_value > 0.0 {
                    info!("Cover reduced damage from {:.1} to {:.1} ({:.0}% reduction)", 
                          damage_event.damage, final_damage, damage_reduction * 100.0);
                }
            } else {
                // No cover, apply full damage
                health.0 -= damage_event.damage;
            }
        }
    }
}

// === UTILITY FUNCTIONS ===

// Line of sight check between two world positions
pub fn has_line_of_sight(
    enhanced_grid: &EnhancedPathfindingGrid,
    from: Vec2,
    to: Vec2,
) -> bool {
    let from_tile = enhanced_grid.world_to_tile(from);
    let to_tile = enhanced_grid.world_to_tile(to);
    
    if from_tile.is_none() || to_tile.is_none() { 
        return false; 
    }
    
    let from_tile = from_tile.unwrap();
    let to_tile = to_tile.unwrap();
    
    // Bresenham line algorithm to check vision blocking tiles
    let dx = (to_tile.x - from_tile.x).abs();
    let dy = (to_tile.y - from_tile.y).abs();
    let mut x = from_tile.x;
    let mut y = from_tile.y;
    
    let x_inc = if to_tile.x > from_tile.x { 1 } else { -1 };
    let y_inc = if to_tile.y > from_tile.y { 1 } else { -1 };
    
    let mut error = dx - dy;
    
    loop {
        // Check current tile for vision blocking
        if enhanced_grid.blocks_vision(x as usize, y as usize) {
            return false;
        }
        
        // Reached target
        if x == to_tile.x && y == to_tile.y {
            break;
        }
        
        let error2 = 2 * error;
        if error2 > -dy {
            error -= dy;
            x += x_inc;
        }
        if error2 < dx {
            error += dx;
            y += y_inc;
        }
    }
    
    true
}

// Get the best cover position near a target location
pub fn find_best_cover_position(
    enhanced_grid: &EnhancedPathfindingGrid,
    near_position: Vec2,
    search_radius: f32,
) -> Option<Vec2> {
    let center_tile = enhanced_grid.world_to_tile(near_position)?;
    let search_tiles = (search_radius / enhanced_grid.tile_size) as i32;
    
    let mut best_position = None;
    let mut best_cover_value = 0.0;
    
    for dy in -search_tiles..=search_tiles {
        for dx in -search_tiles..=search_tiles {
            let check_x = center_tile.x + dx;
            let check_y = center_tile.y + dy;
            
            if check_x >= 0 && check_y >= 0 && 
               (check_x as usize) < enhanced_grid.width && 
               (check_y as usize) < enhanced_grid.height {
                
                let cover_value = enhanced_grid.get_cover_value(check_x as usize, check_y as usize);
                
                if cover_value > best_cover_value && 
                   is_tile_walkable(enhanced_grid, check_x as usize, check_y as usize) {
                    best_cover_value = cover_value;
                    best_position = Some(enhanced_grid.grid_to_world(IVec2::new(check_x, check_y)));
                }
            }
        }
    }
    
    best_position
}

// Check if a tile position is safe (no hazards)
pub fn is_tile_safe(
    enhanced_grid: &EnhancedPathfindingGrid,
    tile_x: usize,
    tile_y: usize,
) -> bool {
    if let Some(index) = enhanced_grid.get_tile_index(tile_x, tile_y) {
        enhanced_grid.safety_costs[index] < 10.0 // Threshold for "safe"
    } else {
        false
    }
}

// Get movement cost multiplier for current tile
pub fn get_tile_movement_multiplier(
    enhanced_grid: &EnhancedPathfindingGrid,
    world_pos: Vec2,
) -> f32 {
    if let Some(tile_pos) = enhanced_grid.world_to_tile(world_pos) {
        let movement_cost = enhanced_grid.get_movement_cost(tile_pos.x as usize, tile_pos.y as usize);
        if movement_cost < f32::INFINITY {
            1.0 / movement_cost.max(0.1)
        } else {
            0.0 // Blocked tile
        }
    } else {
        1.0 // Default multiplier
    }
}

