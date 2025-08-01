// src/systems/pathfinding.rs - Tile-based A* pathfinding for Bevy 0.16.1
use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use std::collections::{BinaryHeap, HashMap, HashSet};
use std::cmp::Ordering;
use crate::core::components::{MovementSpeed};

// ============================================================================
// CORE PATHFINDING COMPONENTS
// ============================================================================

#[derive(Resource, Default)]
pub struct PathfindingGrid {
    pub width: usize,
    pub height: usize,
    pub tile_size: f32,
    pub offset: Vec2, // World position of grid origin
    pub tiles: Vec<TileType>,
    pub dirty: bool, // Flag to rebuild grid when objects change
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum TileType {
    Walkable,
    Blocked,
    Difficult,  // Slower movement, higher cost
                // PLACEHOLDER - Need to add this to spills, etc.
}

#[derive(Component)]
pub struct PathfindingAgent {
    pub current_path: Vec<Vec2>,
    pub path_index: usize,
    pub recalculate: bool,
}

#[derive(Component)]
pub struct PathfindingObstacle {
    pub radius: f32,
    pub blocks_movement: bool,
}

// A* node for pathfinding
#[derive(Clone, Debug)]
struct Node {
    pos: (usize, usize),
    g_cost: f32,  // Distance from start
    h_cost: f32,  // Heuristic distance to goal
    parent: Option<(usize, usize)>,
}

impl Node {
    fn f_cost(&self) -> f32 {
        self.g_cost + self.h_cost
    }
}

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        self.pos == other.pos
    }
}

impl Eq for Node {}

impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Node {
    fn cmp(&self, other: &Self) -> Ordering {
        // Reverse for min-heap behavior
        other.f_cost().partial_cmp(&self.f_cost()).unwrap_or(Ordering::Equal)
    }
}

// ============================================================================
// GRID MANAGEMENT
// ============================================================================

impl PathfindingGrid {
    pub fn new(world_size: Vec2, tile_size: f32) -> Self {
        let width = (world_size.x / tile_size) as usize;
        let height = (world_size.y / tile_size) as usize;
        let offset = -world_size * 0.5; // Center the grid

        Self {
            width,
            height,
            tile_size,
            offset,
            tiles: vec![TileType::Walkable; width * height],
            dirty: true,
        }
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

    pub fn get_tile(&self, x: usize, y: usize) -> TileType {
        if x < self.width && y < self.height {
            self.tiles[y * self.width + x]
        } else {
            TileType::Blocked
        }
    }

    pub fn set_tile(&mut self, x: usize, y: usize, tile_type: TileType) {
        if x < self.width && y < self.height {
            self.tiles[y * self.width + x] = tile_type;
        }
    }

    pub fn clear(&mut self) {
        self.tiles.fill(TileType::Walkable);
        self.dirty = true;
    }

    fn get_neighbors(&self, pos: (usize, usize)) -> Vec<(usize, usize)> {
        let mut neighbors = Vec::new();
        let (x, y) = pos;

        // 8-directional movement
        for dx in -1i32..=1 {
            for dy in -1i32..=1 {
                if dx == 0 && dy == 0 { continue; }

                let nx = x as i32 + dx;
                let ny = y as i32 + dy;

                if nx >= 0 && ny >= 0 &&
                   (nx as usize) < self.width && (ny as usize) < self.height {
                    neighbors.push((nx as usize, ny as usize));
                }
            }
        }
        neighbors
    }
}

// ============================================================================
// A* PATHFINDING ALGORITHM
// ============================================================================

pub fn find_path(grid: &PathfindingGrid, start: Vec2, goal: Vec2) -> Option<Vec<Vec2>> {
    let start_grid = grid.world_to_grid(start)?;
    let goal_grid = grid.world_to_grid(goal)?;

    if grid.get_tile(goal_grid.0, goal_grid.1) == TileType::Blocked {
        return None;
    }

    let mut open_set = BinaryHeap::new();
    let mut closed_set = HashSet::new();
    let mut came_from = HashMap::new();

    let start_node = Node {
        pos: start_grid,
        g_cost: 0.0,
        h_cost: heuristic(start_grid, goal_grid),
        parent: None,
    };

    open_set.push(start_node);

    while let Some(current) = open_set.pop() {
        if current.pos == goal_grid {
            return Some(reconstruct_path(grid, came_from, current.pos, start, goal));
        }

        closed_set.insert(current.pos);

        for neighbor_pos in grid.get_neighbors(current.pos) {
            if closed_set.contains(&neighbor_pos) {
                continue;
            }

            let tile_type = grid.get_tile(neighbor_pos.0, neighbor_pos.1);
            if tile_type == TileType::Blocked {
                continue;
            }

            let movement_cost = get_movement_cost(current.pos, neighbor_pos, tile_type);
            let tentative_g = current.g_cost + movement_cost;

            let neighbor_node = Node {
                pos: neighbor_pos,
                g_cost: tentative_g,
                h_cost: heuristic(neighbor_pos, goal_grid),
                parent: Some(current.pos),
            };

            // Check if this path is better
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

    None // No path found
}

fn heuristic(a: (usize, usize), b: (usize, usize)) -> f32 {
    // Diagonal distance heuristic
    let dx = (a.0 as f32 - b.0 as f32).abs();
    let dy = (a.1 as f32 - b.1 as f32).abs();

    let diagonal = dx.min(dy);
    let straight = (dx - diagonal) + (dy - diagonal);

    diagonal * 1.41421356 + straight // sqrt(2) for diagonal movement
}

fn get_movement_cost(from: (usize, usize), to: (usize, usize), tile_type: TileType) -> f32 {
    let base_cost = if from.0 != to.0 && from.1 != to.1 {
        1.41421356 // Diagonal movement
    } else {
        1.0 // Straight movement
    };

    match tile_type {
        TileType::Walkable => base_cost,
        TileType::Difficult => base_cost * 2.0,
        TileType::Blocked => f32::INFINITY,
    }
}

fn reconstruct_path(
    grid: &PathfindingGrid,
    came_from: HashMap<(usize, usize), (usize, usize)>,
    goal: (usize, usize),
    start_world: Vec2,
    goal_world: Vec2
) -> Vec<Vec2> {
    let mut path = Vec::new();
    let mut current = goal;

    while let Some(&parent) = came_from.get(&current) {
        path.push(grid.grid_to_world(current));
        current = parent;
    }

    path.push(start_world);
    path.reverse();

    // Smooth the path by removing unnecessary waypoints
    smooth_path(&mut path);

    path
}

fn smooth_path(path: &mut Vec<Vec2>) {
    if path.len() < 3 { return; }

    let mut i = 0;
    while i + 2 < path.len() {
        let p1 = path[i];
        let p2 = path[i + 1];
        let p3 = path[i + 2];

        // If the three points are roughly collinear, remove the middle one
        let v1 = (p2 - p1).normalize_or_zero();
        let v2 = (p3 - p2).normalize_or_zero();

        if v1.dot(v2) > 0.95 { // Nearly same direction
            path.remove(i + 1);
        } else {
            i += 1;
        }
    }
}

// ============================================================================
// BEVY SYSTEMS
// ============================================================================

// Initialize the pathfinding grid
pub fn setup_pathfinding_grid(mut commands: Commands) {
    let world_size = Vec2::new(2000.0, 2000.0); // Adjust to your world size
    let tile_size = 20.0; // Balance between accuracy and performance

    let grid = PathfindingGrid::new(world_size, tile_size);
    commands.insert_resource(grid);

    info!("Pathfinding grid initialized: ");// {}x{} tiles", grid.width, grid.height);
}

// Update grid when static objects change
pub fn update_pathfinding_grid(
    mut grid: ResMut<PathfindingGrid>,
    obstacles: Query<(&Transform, &PathfindingObstacle), (Without<Velocity>, Changed<Transform>)>,
    static_obstacles: Query<(&Transform, &Collider), (Without<Velocity>, Without<PathfindingObstacle>)>,
) {
    if !grid.dirty && obstacles.is_empty() && static_obstacles.is_empty() {
        return;
    }

    // Clear grid first
    grid.clear();

    // Mark obstacles from PathfindingObstacle components
    for (transform, obstacle) in obstacles.iter() {
        mark_circle_obstacle(&mut grid, transform.translation.truncate(), obstacle.radius, obstacle.blocks_movement);
    }

    // Mark obstacles from static colliders (cover, terminals, etc.)
    for (transform, collider) in static_obstacles.iter() {
        if let Some(ball) = collider.as_ball() {
            mark_circle_obstacle(&mut grid, transform.translation.truncate(), ball.radius(), true);
        } else if let Some(cuboid) = collider.as_cuboid() {
            mark_rect_obstacle(&mut grid, transform.translation.truncate(), cuboid.half_extents() * 2.0);
        }
    }

    grid.dirty = false;
}

fn mark_circle_obstacle(grid: &mut PathfindingGrid, center: Vec2, radius: f32, blocks: bool) {
    let tile_type = if blocks { TileType::Blocked } else { TileType::Difficult };

    let min_x = ((center.x - radius - grid.offset.x) / grid.tile_size).floor() as i32;
    let max_x = ((center.x + radius - grid.offset.x) / grid.tile_size).ceil() as i32;
    let min_y = ((center.y - radius - grid.offset.y) / grid.tile_size).floor() as i32;
    let max_y = ((center.y + radius - grid.offset.y) / grid.tile_size).ceil() as i32;

    for x in min_x..=max_x {
        for y in min_y..=max_y {
            if x >= 0 && y >= 0 && (x as usize) < grid.width && (y as usize) < grid.height {
                let tile_center = grid.grid_to_world((x as usize, y as usize));
                if center.distance(tile_center) <= radius {
                    grid.set_tile(x as usize, y as usize, tile_type);
                }
            }
        }
    }
}

fn mark_rect_obstacle(grid: &mut PathfindingGrid, center: Vec2, size: Vec2) {
    let half_size = size * 0.5;
    let min = center - half_size;
    let max = center + half_size;

    if let (Some(min_grid), Some(max_grid)) = (grid.world_to_grid(min), grid.world_to_grid(max)) {
        for x in min_grid.0..=max_grid.0 {
            for y in min_grid.1..=max_grid.1 {
                grid.set_tile(x, y, TileType::Blocked);
            }
        }
    }
}

pub fn find_adjacent_position(grid: &PathfindingGrid, target: Vec2, approach_from: Vec2) -> Vec2 {
    if let Some(target_grid) = grid.world_to_grid(target) {
        // Try positions around the target
        let directions = [
            (-1, 0), (1, 0), (0, -1), (0, 1),  // Cardinal directions first
            (-1, -1), (-1, 1), (1, -1), (1, 1) // Then diagonals
        ];

        for (dx, dy) in directions {
            let check_x = target_grid.0 as i32 + dx;
            let check_y = target_grid.1 as i32 + dy;

            if check_x >= 0 && check_y >= 0 &&
               (check_x as usize) < grid.width && (check_y as usize) < grid.height {

                let check_pos = (check_x as usize, check_y as usize);
                if grid.get_tile(check_pos.0, check_pos.1) == TileType::Walkable {
                    let world_pos = grid.grid_to_world(check_pos);

                    // Prefer positions that are closer to the approach direction
                    let to_approach = (approach_from - world_pos).normalize_or_zero();
                    let to_target = (target - world_pos).normalize_or_zero();

                    // If this position allows good approach and target access, use it
                    if to_approach.dot(to_target) > -0.5 { // Not opposing directions
                        return world_pos;
                    }
                }
            }
        }
    }

    // Fallback: return target position (will likely fail pathfinding)
    target
}

// Enhanced pathfinding that handles "move near" vs "move to" targets
pub fn find_path_smart(grid: &PathfindingGrid, start: Vec2, goal: Vec2, allow_adjacent: bool) -> Option<Vec<Vec2>> {
    // First try direct path
    if let Some(path) = find_path(grid, start, goal) {
        return Some(path);
    }

    // If direct path fails and we allow adjacent positioning
    if allow_adjacent {
        let adjacent_goal = find_adjacent_position(grid, goal, start);
        if adjacent_goal != goal {
            if let Some(path) = find_path(grid, start, adjacent_goal) {
                return Some(path);
            }
        }
    }

    None
}

// Pathfinding movement system - replaces your current movement system
pub fn pathfinding_movement_system(
    commands: Commands,
    mut action_events: EventReader<crate::core::ActionEvent>,
    mut agents: Query<(Entity, &mut Transform, &MovementSpeed, &mut PathfindingAgent)>,
    grid: Res<PathfindingGrid>,
    time: Res<Time>,
    game_mode: Res<crate::core::GameMode>,
    // cover_points: Query<&Transform, (With<crate::core::CoverPoint>, Without<crate::core::Agent>)>,
) {
    if game_mode.paused { return; }

    // Handle new movement commands
    for event in action_events.read() {
        if let crate::core::Action::MoveTo(target_pos) = event.action {
            if let Ok((entity, transform, _, mut agent)) = agents.get_mut(event.entity) {
                let start_pos = transform.translation.truncate();

                // Check if target is near a cover point - if so, allow adjacent positioning
                //let near_cover = cover_points.iter().any(|cover_transform| {
                //    cover_transform.translation.truncate().distance(target_pos) < 30.0
                //});

                // if let Some(path) = find_path_smart(&grid, start_pos, target_pos, near_cover) {
                if let Some(path) = find_path(&grid, start_pos, target_pos) {
                    agent.current_path = path;
                    agent.path_index = 0;
                    agent.recalculate = false;
                } else {
                    // warn!("No path found from {:?} to {:?} (near_cover: {})", start_pos, target_pos, near_cover);
                    agent.current_path.clear();
                }
            }
        }
    }

    // Execute pathfinding movement (same as before)
    for (entity, mut transform, speed, mut agent) in agents.iter_mut() {
        if agent.current_path.is_empty() { continue; }

        let current_pos = transform.translation.truncate();

        // Check if we need to recalculate due to dynamic obstacles
        if agent.recalculate && agent.current_path.len() > agent.path_index {
            let goal = *agent.current_path.last().unwrap();
            if let Some(new_path) = find_path(&grid, current_pos, goal) {
                agent.current_path = new_path;
                agent.path_index = 0;
                agent.recalculate = false;
            }
        }

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
                // Reached final destination
                agent.current_path.clear();
                continue;
            }
        }

        // Move towards current waypoint
        let movement = direction * speed.0 * time.delta_secs();
        transform.translation += movement.extend(0.0);
    }
}

// Add pathfinding capability to existing entities
pub fn add_pathfinding_to_agents(
    mut commands: Commands,
    agents: Query<Entity, (With<crate::core::Agent>, Without<PathfindingAgent>)>,
) {
    for entity in agents.iter() {
        commands.entity(entity).insert(PathfindingAgent {
            current_path: Vec::new(),
            path_index: 0,
            recalculate: false,
        });
    }
}

// Debug visualization (optional)
#[cfg(debug_assertions)]
pub fn debug_pathfinding_grid(
    mut gizmos: Gizmos,
    grid: Res<PathfindingGrid>,
    agents: Query<&PathfindingAgent>,
) {
    // Draw grid bounds
    let world_min = grid.offset;
    let world_max = grid.offset + Vec2::new(grid.width as f32 * grid.tile_size, grid.height as f32 * grid.tile_size);

    gizmos.rect_2d(
        Isometry2d::from_translation((world_min + world_max) * 0.5),
        world_max - world_min,
        Color::srgb(0.5, 0.5, 0.5)
    );

    // Draw blocked tiles (sample to avoid performance issues)
    let sample_rate = (grid.width / 50).max(1); // Sample every N tiles
    for x in (0..grid.width).step_by(sample_rate) {
        for y in (0..grid.height).step_by(sample_rate) {
            let tile_type = grid.get_tile(x, y);
            if tile_type != TileType::Walkable {
                let world_pos = grid.grid_to_world((x, y));
                let color = match tile_type {
                    TileType::Blocked => Color::srgb(1.0, 0.0, 0.0),
                    TileType::Difficult => Color::srgb(1.0, 1.0, 0.0),
                    TileType::Walkable => Color::srgb(0.0, 1.0, 0.0),
                };

                gizmos.rect_2d(
                    Isometry2d::from_translation(world_pos),
                    Vec2::splat(grid.tile_size * 0.8),
                    color
                );
            }
        }
    }

    // Draw active paths
    for agent in agents.iter() {
        if agent.current_path.len() > 1 {
            for i in 0..agent.current_path.len() - 1 {
                gizmos.line_2d(
                    agent.current_path[i],
                    agent.current_path[i + 1],
                    Color::srgba(0.0, 0.0, 1.0, 0.4)
                );
            }
        }
    }
}