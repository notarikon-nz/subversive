// src/core/goap.rs
use bevy::prelude::*;
use std::collections::{HashMap, VecDeque};

// Macro for easier HashMap creation
macro_rules! hashmap {
    ($($key:expr => $value:expr),* $(,)?) => {
        {
            let mut map = HashMap::new();
            $(map.insert($key, $value);)*
            map
        }
    };
}

// === WORLD STATE ===
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WorldKey {
    // Position states
    AtPatrolPoint,
    AtLastKnownPosition,
    AtTarget,
    
    // Knowledge states
    HasTarget,
    TargetVisible,
    HeardSound,
    
    // Equipment states
    HasWeapon,
    WeaponLoaded,
    
    // Alert states
    IsAlert,
    IsInvestigating,

    // Cover states
    InCover,
    CoverAvailable,
    UnderFire,
}

pub type WorldState = HashMap<WorldKey, bool>;

// === ACTIONS ===
#[derive(Debug, Clone)]
pub struct GoapAction {
    pub name: &'static str,
    pub cost: f32,
    pub preconditions: WorldState,
    pub effects: WorldState,
    pub action_type: ActionType,
}

#[derive(Debug, Clone)]
pub enum ActionType {
    Patrol,
    MoveTo { target: Vec2 },
    Attack { target: Entity },
    Investigate { location: Vec2 },
    Search { area: Vec2 },
    Reload,
    CallForHelp,
    TakeCover,
}

// === GOALS ===
#[derive(Debug, Clone)]
pub struct Goal {
    pub name: &'static str,
    pub priority: f32,
    pub desired_state: WorldState,
}

// === PLANNER ===
#[derive(Component)]
pub struct GoapAgent {
    pub current_plan: VecDeque<GoapAction>,
    pub current_goal: Option<Goal>,
    pub world_state: WorldState,
    pub available_actions: Vec<GoapAction>,
    pub goals: Vec<Goal>,
    pub planning_cooldown: f32,
}

impl Default for GoapAgent {
    fn default() -> Self {
        let mut agent = Self {
            current_plan: VecDeque::new(),
            current_goal: None,
            world_state: WorldState::new(),
            available_actions: Vec::new(),
            goals: Vec::new(),
            planning_cooldown: 0.0,
        };
        
        agent.setup_default_actions();
        agent.setup_default_goals();
        agent.setup_initial_world_state();
        
        agent
    }
}

impl GoapAgent {
    fn setup_default_actions(&mut self) {
        self.available_actions = vec![
            // Patrol action
            GoapAction {
                name: "patrol",
                cost: 1.0,
                preconditions: hashmap![
                    WorldKey::IsAlert => false,
                    WorldKey::HasTarget => false
                ],
                effects: hashmap![
                    WorldKey::AtPatrolPoint => true
                ],
                action_type: ActionType::Patrol,
            },
            
            // Return to patrol after losing target
            GoapAction {
                name: "return_to_patrol",
                cost: 1.5,
                preconditions: hashmap![
                    WorldKey::HasTarget => false,
                    WorldKey::AtPatrolPoint => false
                ],
                effects: hashmap![
                    WorldKey::AtPatrolPoint => true,
                    WorldKey::IsAlert => false
                ],
                action_type: ActionType::Patrol,
            },
            
            // Take cover action
            GoapAction {
                name: "take_cover",
                cost: 2.0,
                preconditions: hashmap![
                    WorldKey::HasTarget => true,
                    WorldKey::InCover => false,
                    WorldKey::CoverAvailable => true
                ],
                effects: hashmap![
                    WorldKey::InCover => true
                ],
                action_type: ActionType::TakeCover,
            },

            // Calm down action - simple way to become unalert
            GoapAction {
                name: "calm_down",
                cost: 0.5,
                preconditions: hashmap![
                    WorldKey::HasTarget => false,
                    WorldKey::TargetVisible => false
                ],
                effects: hashmap![
                    WorldKey::IsAlert => false
                ],
                action_type: ActionType::Patrol, // Just patrol to calm down
            },
            
            // Move to investigate
            GoapAction {
                name: "investigate",
                cost: 2.0,
                preconditions: hashmap![
                    WorldKey::HeardSound => true,
                    WorldKey::IsAlert => false // Only investigate when not already in combat
                ],
                effects: hashmap![
                    WorldKey::AtLastKnownPosition => true,
                    WorldKey::IsInvestigating => true,
                    WorldKey::HeardSound => false
                ],
                action_type: ActionType::Investigate { location: Vec2::ZERO }, // Will be updated
            },
            
            // Attack target
            GoapAction {
                name: "attack",
                cost: 1.0,
                preconditions: hashmap![
                    WorldKey::HasTarget => true,
                    WorldKey::TargetVisible => true,
                    WorldKey::HasWeapon => true
                    // Removed WeaponLoaded requirement for now
                ],
                effects: hashmap![
                    WorldKey::HasTarget => false // Assume target eliminated
                ],
                action_type: ActionType::Attack { target: Entity::PLACEHOLDER },
            },
            
            // Move to target
            GoapAction {
                name: "move_to_target",
                cost: 3.0,
                preconditions: hashmap![
                    WorldKey::HasTarget => true
                    // Removed TargetVisible requirement - we can move to last known position
                ],
                effects: hashmap![
                    WorldKey::AtTarget => true
                    // Removed TargetVisible effect - moving doesn't guarantee visibility
                ],
                action_type: ActionType::MoveTo { target: Vec2::ZERO },
            },
            
            // Reload weapon
            GoapAction {
                name: "reload",
                cost: 2.0,
                preconditions: hashmap![
                    WorldKey::HasWeapon => true,
                    WorldKey::WeaponLoaded => false
                ],
                effects: hashmap![
                    WorldKey::WeaponLoaded => true
                ],
                action_type: ActionType::Reload,
            },
        ];
    }
    
    fn setup_default_goals(&mut self) {
        self.goals = vec![
            Goal {
                name: "eliminate_threat",
                priority: 10.0,
                desired_state: hashmap![
                    WorldKey::HasTarget => false  // Just eliminate the target, don't worry about alert state
                ],
            },
            
            Goal {
                name: "investigate_disturbance",
                priority: 5.0,
                desired_state: hashmap![
                    WorldKey::HeardSound => false  // Just stop the sound investigation
                ],
            },
            
            Goal {
                name: "patrol_area",
                priority: 1.0,
                desired_state: hashmap![
                    WorldKey::IsAlert => false  // Just be calm - don't require being at patrol point
                ],
            },
        ];
    }
    
    fn setup_initial_world_state(&mut self) {
        self.world_state.insert(WorldKey::HasWeapon, true);
        self.world_state.insert(WorldKey::WeaponLoaded, true);
        self.world_state.insert(WorldKey::IsAlert, false);
        self.world_state.insert(WorldKey::HasTarget, false);
        self.world_state.insert(WorldKey::TargetVisible, false);
        self.world_state.insert(WorldKey::HeardSound, false);
        self.world_state.insert(WorldKey::AtPatrolPoint, true);
        self.world_state.insert(WorldKey::AtLastKnownPosition, false);
        self.world_state.insert(WorldKey::AtTarget, false);
        self.world_state.insert(WorldKey::IsInvestigating, false);
    }
    
    pub fn update_world_state(&mut self, key: WorldKey, value: bool) {
        self.world_state.insert(key, value);
    }
    
    pub fn plan(&mut self) -> bool {
        // Find the highest priority goal that isn't already satisfied
        let goal = self.goals.iter()
            .filter(|g| !self.is_goal_satisfied(&g.desired_state))
            .max_by(|a, b| a.priority.partial_cmp(&b.priority).unwrap_or(std::cmp::Ordering::Equal));
        
        if let Some(goal) = goal {
            let old_goal = self.current_goal.as_ref().map(|g| g.name);
            self.current_goal = Some(goal.clone());
            
            // Debug output when goal changes
            if old_goal != Some(goal.name) {
                info!("GOAP: New goal selected: {} (priority: {})", goal.name, goal.priority);
                info!("GOAP: Goal desired state: {:?}", goal.desired_state);
            }
            
            self.current_plan = self.find_plan(&goal.desired_state);
            let success = !self.current_plan.is_empty();
            
            if success {
                info!("GOAP: Plan found with {} actions", self.current_plan.len());
                for (i, action) in self.current_plan.iter().enumerate() {
                    info!("  {}: {} (cost: {})", i, action.name, action.cost);
                }
            } else {
                info!("GOAP: No plan found for goal: {}", goal.name);
                info!("GOAP: Current world state: {:?}", self.world_state);
                info!("GOAP: Available actions:");
                for action in &self.available_actions {
                    info!("  - {} (cost: {}, preconditions: {:?}, effects: {:?})", 
                          action.name, action.cost, action.preconditions, action.effects);
                }
            }
            
            success
        } else {
            info!("GOAP: All goals satisfied");
            false
        }
    }
    
    fn is_goal_satisfied(&self, desired_state: &WorldState) -> bool {
        desired_state.iter().all(|(key, &desired_value)| {
            self.world_state.get(key).unwrap_or(&false) == &desired_value
        })
    }
    
    fn find_plan(&self, goal_state: &WorldState) -> VecDeque<GoapAction> {
        // Simple backward chaining planner
        let mut plan = VecDeque::new();
        let mut current_state = self.world_state.clone();
        let mut remaining_goals = goal_state.clone();
        
        // Maximum planning depth to prevent infinite loops
        let max_depth = 10;
        let mut depth = 0;
        
        while !remaining_goals.is_empty() && depth < max_depth {
            depth += 1;
            
            // Find an action that satisfies at least one remaining goal
            if let Some(action) = self.find_satisfying_action(&remaining_goals, &current_state) {
                // Check if we can execute this action (preconditions met)
                if self.can_execute_action(&action, &current_state) {
                    // Apply the action's effects
                    self.apply_effects(&action.effects, &mut current_state);
                    
                    // Remove satisfied goals
                    for (key, &value) in &action.effects {
                        if remaining_goals.get(key) == Some(&value) {
                            remaining_goals.remove(key);
                        }
                    }
                    
                    plan.push_front(action);
                } else {
                    // We need to satisfy the action's preconditions first
                    // Add them as sub-goals
                    for (&key, &value) in &action.preconditions {
                        if current_state.get(&key) != Some(&value) {
                            remaining_goals.insert(key, value);
                        }
                    }
                }
            } else {
                // No action can satisfy the remaining goals
                break;
            }
        }
        
        if remaining_goals.is_empty() {
            plan
        } else {
            VecDeque::new() // Failed to find complete plan
        }
    }
    
    fn find_satisfying_action(&self, goals: &WorldState, _current_state: &WorldState) -> Option<GoapAction> {
        self.available_actions.iter()
            .find(|action| {
                // Check if this action satisfies at least one goal
                action.effects.iter().any(|(key, &value)| {
                    goals.get(key) == Some(&value)
                })
            })
            .cloned()
    }
    
    fn can_execute_action(&self, action: &GoapAction, current_state: &WorldState) -> bool {
        action.preconditions.iter().all(|(key, &required_value)| {
            current_state.get(key).unwrap_or(&false) == &required_value
        })
    }
    
    fn apply_effects(&self, effects: &WorldState, current_state: &mut WorldState) {
        for (&key, &value) in effects {
            current_state.insert(key, value);
        }
    }
    
    pub fn get_next_action(&mut self) -> Option<GoapAction> {
        self.current_plan.pop_front()
    }
    
    pub fn abort_plan(&mut self) {
        self.current_plan.clear();
        self.current_goal = None;
    }
}



// === INTEGRATION WITH EXISTING AI ===
use crate::core::*;
use crate::systems::ai::AIState;

pub fn goap_ai_system(
    mut commands: Commands,  // Add Commands for spawning components
    mut enemy_query: Query<(
        Entity, 
        &Transform, 
        &mut AIState, 
        &mut GoapAgent,
        &mut Vision,
        &Patrol
    ), (With<Enemy>, Without<Dead>)>,
    agent_query: Query<(Entity, &Transform), With<Agent>>,
    cover_query: Query<(Entity, &Transform, &CoverPoint), Without<Enemy>>,  // Add cover query
    mut action_events: EventWriter<ActionEvent>,
    mut audio_events: EventWriter<AudioEvent>,
    time: Res<Time>,
    game_mode: Res<GameMode>,
) {
    if game_mode.paused { return; }

    for (enemy_entity, enemy_transform, mut ai_state, mut goap_agent, mut vision, patrol) in enemy_query.iter_mut() {
        // Update planning cooldown
        goap_agent.planning_cooldown -= time.delta_seconds();
        
        // Update world state based on current situation
        update_world_state_from_perception(
            &mut goap_agent,
            enemy_transform,
            &vision,
            &agent_query,
            &mut ai_state,
            patrol,
            &cover_query  // Pass cover query
        );
        
        // Check if current plan is still valid or if we need to replan
        let should_replan = goap_agent.current_plan.is_empty() || 
                          goap_agent.planning_cooldown <= 0.0 ||
                          plan_invalidated(&goap_agent, &ai_state);
        
        // Also replan more frequently in combat situations
        let in_combat = *goap_agent.world_state.get(&WorldKey::HasTarget).unwrap_or(&false) ||
                        *goap_agent.world_state.get(&WorldKey::IsAlert).unwrap_or(&false);
        
        if should_replan {
            goap_agent.plan();
            // Use shorter planning interval in combat
            goap_agent.planning_cooldown = if in_combat { 0.5 } else { 2.0 };
        }
        
        // Execute the next action in the plan
        if let Some(action) = goap_agent.get_next_action() {
            execute_goap_action(
                &action,
                enemy_entity,
                enemy_transform,
                &mut ai_state,
                &mut action_events,
                &mut audio_events,
                patrol,
                &agent_query,
                &vision,
                &cover_query,  // Pass cover query
                &mut commands, // Pass commands
            );
        }
    }
}

fn update_world_state_from_perception(
    goap_agent: &mut GoapAgent,
    enemy_transform: &Transform,
    vision: &Vision,
    agent_query: &Query<(Entity, &Transform), With<Agent>>,
    ai_state: &mut AIState,
    patrol: &Patrol,
    cover_query: &Query<(Entity, &Transform, &CoverPoint), Without<Enemy>>,  // Add cover query
) {
    let enemy_pos = enemy_transform.translation.truncate();
    
    // Check for visible agents and get their position
    let visible_agent = check_line_of_sight_goap(enemy_transform, vision, agent_query);
    let has_target = visible_agent.is_some();
    let target_visible = has_target;
    
    // Update last known target position if we can see a target
    if let Some(agent_entity) = visible_agent {
        if let Ok((_, agent_transform)) = agent_query.get(agent_entity) {
            let target_pos = agent_transform.translation.truncate();
            ai_state.last_known_target = Some(target_pos);
            info!("GOAP: Updated last known target position to {:?}", target_pos);
        }
    }
    
    // Check if we're at a patrol point
    let at_patrol_point = if let Some(patrol_target) = patrol.current_target() {
        enemy_pos.distance(patrol_target) < 20.0 // Within 20 units of patrol point
    } else {
        true // If no patrol points, consider ourselves "at patrol point"
    };
    
    // Check if cover is available
    let cover_available = find_nearest_cover(enemy_pos, cover_query).is_some();
    
    // Update world state
    goap_agent.update_world_state(WorldKey::TargetVisible, target_visible);
    goap_agent.update_world_state(WorldKey::HasTarget, has_target);
    goap_agent.update_world_state(WorldKey::AtPatrolPoint, at_patrol_point);
    goap_agent.update_world_state(WorldKey::CoverAvailable, cover_available);
    
    // Debug output when target status changes
    let old_has_target = goap_agent.world_state.get(&WorldKey::HasTarget).copied().unwrap_or(false);
    if has_target != old_has_target {
        if has_target {
            info!("GOAP: Enemy acquired target! Visible: {}", target_visible);
            // Force immediate replanning when we get a target
            goap_agent.abort_plan();
        } else {
            info!("GOAP: Enemy lost target, returning to patrol");
            // Clear last known target after a delay or when we reach the investigation point
            goap_agent.abort_plan(); // Force replanning
        }
    }
    
    // Update based on AI state
    match &ai_state.mode {
        crate::systems::ai::AIMode::Patrol => {
            goap_agent.update_world_state(WorldKey::IsAlert, false);
            goap_agent.update_world_state(WorldKey::IsInvestigating, false);
        },
        crate::systems::ai::AIMode::Combat { .. } => {
            goap_agent.update_world_state(WorldKey::IsAlert, true);
            goap_agent.update_world_state(WorldKey::IsInvestigating, false);
        },
        crate::systems::ai::AIMode::Investigate { .. } => {
            goap_agent.update_world_state(WorldKey::IsAlert, true);
            goap_agent.update_world_state(WorldKey::IsInvestigating, true);
        },
        crate::systems::ai::AIMode::Search { .. } => {
            goap_agent.update_world_state(WorldKey::IsAlert, true);
            goap_agent.update_world_state(WorldKey::IsInvestigating, true);
        },
    }
}

fn plan_invalidated(goap_agent: &GoapAgent, ai_state: &AIState) -> bool {
    // Check if the world state has changed significantly
    match &ai_state.mode {
        crate::systems::ai::AIMode::Combat { .. } => {
            // In combat, prioritize immediate threats
            !goap_agent.world_state.get(&WorldKey::HasTarget).unwrap_or(&false)
        },
        _ => false,
    }
}

fn execute_goap_action(
    action: &GoapAction,
    enemy_entity: Entity,
    enemy_transform: &Transform,
    ai_state: &mut AIState,
    action_events: &mut EventWriter<ActionEvent>,
    audio_events: &mut EventWriter<AudioEvent>,
    patrol: &Patrol,
    agent_query: &Query<(Entity, &Transform), With<Agent>>,
    vision: &Vision,
    cover_query: &Query<(Entity, &Transform, &CoverPoint), Without<Enemy>>,  // Add cover query
    commands: &mut Commands,  // Add commands
) {
    match &action.action_type {
        ActionType::Patrol => {
            if let Some(target) = patrol.current_target() {
                ai_state.mode = crate::systems::ai::AIMode::Patrol;
                action_events.send(ActionEvent {
                    entity: enemy_entity,
                    action: Action::MoveTo(target),
                });
                info!("GOAP: Enemy {} patrolling to {:?}", enemy_entity.index(), target);
            } else {
                // No patrol points defined, just stay in patrol mode
                ai_state.mode = crate::systems::ai::AIMode::Patrol;
                info!("GOAP: Enemy {} in patrol mode (no patrol points)", enemy_entity.index());
            }
        },
        
        ActionType::MoveTo { target } => {
            action_events.send(ActionEvent {
                entity: enemy_entity,
                action: Action::MoveTo(*target),
            });
        },
        
        ActionType::Attack { target: _ } => {
            info!("GOAP: Executing attack action...");
            
            // First, try to find a visible target using the same vision system
            if let Some(agent_entity) = check_line_of_sight_goap(
                &Transform::from_translation(enemy_transform.translation),
                vision, // Now we have access to the vision component
                agent_query
            ) {
                // Get target position for distance check
                if let Ok((_, agent_transform)) = agent_query.get(agent_entity) {
                    let distance = enemy_transform.translation.truncate()
                        .distance(agent_transform.translation.truncate());
                    
                    info!("GOAP: Target visible at distance {:.1}", distance);
                    
                    // Update AI state to combat mode
                    ai_state.mode = crate::systems::ai::AIMode::Combat { target: agent_entity };
                    
                    // Check if we're in attack range
                    if distance <= 150.0 {
                        action_events.send(ActionEvent {
                            entity: enemy_entity,
                            action: Action::Attack(agent_entity),
                        });
                        info!("GOAP: Enemy {} attacking target {} at range {:.1}", 
                              enemy_entity.index(), agent_entity.index(), distance);
                    } else {
                        // Too far, move closer
                        action_events.send(ActionEvent {
                            entity: enemy_entity,
                            action: Action::MoveTo(agent_transform.translation.truncate()),
                        });
                        info!("GOAP: Enemy {} moving closer to target (distance: {:.1})", 
                              enemy_entity.index(), distance);
                    }
                }
            } else {
                info!("GOAP: No target visible during attack execution");
                
                // No visible target, try to move to the last known position if we have one
                if let Some(last_pos) = ai_state.last_known_target {
                    let distance_to_last_known = enemy_transform.translation.truncate().distance(last_pos);
                    
                    action_events.send(ActionEvent {
                        entity: enemy_entity,
                        action: Action::MoveTo(last_pos),
                    });
                    
                    ai_state.mode = crate::systems::ai::AIMode::Investigate { location: last_pos };
                    info!("GOAP: Enemy {} moving to last known target position {:?} (distance: {:.1})", 
                          enemy_entity.index(), last_pos, distance_to_last_known);
                } else {
                    info!("GOAP: Attack action but no target visible and no last known position!");
                }
            }
        },
        
        ActionType::Investigate { location } => {
            let investigation_target = if let Some(last_pos) = ai_state.last_known_target {
                last_pos
            } else {
                *location
            };
            
            ai_state.mode = crate::systems::ai::AIMode::Investigate { 
                location: investigation_target 
            };
            action_events.send(ActionEvent {
                entity: enemy_entity,
                action: Action::MoveTo(investigation_target),
            });
        },
        
        ActionType::Search { area } => {
            ai_state.mode = crate::systems::ai::AIMode::Search { area: *area };
        },
        
        ActionType::Reload => {
            // Handle reload logic - for now just a placeholder
            info!("Enemy {} reloading weapon", enemy_entity.index());
        },
        
        ActionType::TakeCover => {
            info!("GOAP: Executing take cover action...");
            
            if let Some((cover_entity, cover_pos)) = find_nearest_cover(enemy_transform.translation.truncate(), cover_query) {
                // Move to cover position
                action_events.send(ActionEvent {
                    entity: enemy_entity,
                    action: Action::MoveTo(cover_pos),
                });
                
                // Add InCover component when we reach the cover
                commands.entity(enemy_entity).insert(InCover { cover_entity });
                
                info!("GOAP: Enemy {} taking cover at {:?}", enemy_entity.index(), cover_pos);
            } else {
                info!("GOAP: No cover available!");
            }
        },
        
        ActionType::CallForHelp => {
            audio_events.send(AudioEvent {
                sound: AudioType::Alert,
                volume: 1.0,
            });
        },
    }
}

fn check_line_of_sight_goap(
    enemy_transform: &Transform,
    vision: &Vision,
    agent_query: &Query<(Entity, &Transform), With<Agent>>,
) -> Option<Entity> {
    let enemy_pos = enemy_transform.translation.truncate();
    
    for (agent_entity, agent_transform) in agent_query.iter() {
        let agent_pos = agent_transform.translation.truncate();
        let to_agent = agent_pos - enemy_pos;
        let distance = to_agent.length();
        
        if distance <= vision.range && distance > 1.0 {
            let agent_direction = to_agent.normalize();
            let dot_product = vision.direction.dot(agent_direction);
            let angle_cos = (vision.angle / 2.0).cos();
            
            if dot_product >= angle_cos {
                return Some(agent_entity);
            }
        }
    }
    
    None
}

// === DEBUG AND CONFIGURATION SYSTEMS ===

#[derive(Resource)]
pub struct GoapConfig {
    pub debug_enabled: bool,
    pub planning_interval: f32,
    pub max_plan_depth: usize,
}

impl Default for GoapConfig {
    fn default() -> Self {
        Self {
            debug_enabled: false,
            planning_interval: 2.0,
            max_plan_depth: 10,
        }
    }
}

// Debug system to visualize GOAP state
pub fn goap_debug_system(
    mut gizmos: Gizmos,
    config: Res<GoapConfig>,
    goap_query: Query<(Entity, &Transform, &GoapAgent), With<Enemy>>,
) {
    if !config.debug_enabled { return; }
    
    for (entity, transform, goap_agent) in goap_query.iter() {
        let pos = transform.translation.truncate();
        
        // Draw GOAP indicator (blue circle above enemy)
        gizmos.circle_2d(pos + Vec2::new(0.0, 40.0), 8.0, Color::srgb(0.3, 0.8, 1.0));
        
        // Draw current goal indicator
        if let Some(goal) = &goap_agent.current_goal {
            let goal_color = match goal.name {
                "eliminate_threat" => Color::srgb(1.0, 0.2, 0.2),
                "investigate_disturbance" => Color::srgb(1.0, 0.8, 0.2),
                "patrol_area" => Color::srgb(0.2, 1.0, 0.2),
                _ => Color::WHITE,
            };
            
            gizmos.circle_2d(pos + Vec2::new(0.0, 35.0), 6.0, goal_color);
        }
        
        // Draw plan length indicator (line showing how many actions in plan)
        let plan_length = goap_agent.current_plan.len() as f32;
        if plan_length > 0.0 {
            gizmos.line_2d(
                pos + Vec2::new(-15.0, -35.0),
                pos + Vec2::new(-15.0 + (plan_length * 6.0), -35.0),
                Color::srgb(0.3, 0.8, 0.8),
            );
        }
        
        // Draw world state indicators
        let mut y_offset = -45.0;
        let indicator_size = 3.0;
        
        // Show key world states as small colored squares
        if *goap_agent.world_state.get(&WorldKey::HasTarget).unwrap_or(&false) {
            gizmos.rect_2d(pos + Vec2::new(-20.0, y_offset), 0.0, Vec2::splat(indicator_size), Color::srgb(1.0, 0.0, 0.0));
        }
        if *goap_agent.world_state.get(&WorldKey::TargetVisible).unwrap_or(&false) {
            gizmos.rect_2d(pos + Vec2::new(-15.0, y_offset), 0.0, Vec2::splat(indicator_size), Color::srgb(1.0, 0.5, 0.0));
        }
        if *goap_agent.world_state.get(&WorldKey::IsAlert).unwrap_or(&false) {
            gizmos.rect_2d(pos + Vec2::new(-10.0, y_offset), 0.0, Vec2::splat(indicator_size), Color::srgb(1.0, 1.0, 0.0));
        }
        if *goap_agent.world_state.get(&WorldKey::HeardSound).unwrap_or(&false) {
            gizmos.rect_2d(pos + Vec2::new(-5.0, y_offset), 0.0, Vec2::splat(indicator_size), Color::srgb(0.0, 1.0, 1.0));
        }
    }
}

// Configuration system for runtime tuning
pub fn goap_config_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut config: ResMut<GoapConfig>,
) {
    // Toggle debug with F4
    if keyboard.just_pressed(KeyCode::F4) {
        config.debug_enabled = !config.debug_enabled;
        info!("GOAP Debug: {}", if config.debug_enabled { "ON" } else { "OFF" });
        
        if config.debug_enabled {
            info!("GOAP Debug Legend:");
            info!("  Blue circle = GOAP enemy");
            info!("  Red = eliminate_threat goal");
            info!("  Yellow = investigate_disturbance goal"); 
            info!("  Green = patrol_area goal");
            info!("  Cyan line = current plan length");
            info!("  Small squares = world state (Red=HasTarget, Orange=TargetVisible, Yellow=IsAlert, Cyan=HeardSound)");
        }
    }
    
    // Adjust planning interval with +/-
    if keyboard.pressed(KeyCode::Equal) {
        config.planning_interval = (config.planning_interval + 0.1).min(10.0);
        info!("GOAP Planning Interval: {:.1}s", config.planning_interval);
    }
    
    if keyboard.pressed(KeyCode::Minus) && config.planning_interval > 0.1 {
        config.planning_interval = (config.planning_interval - 0.1).max(0.1);
        info!("GOAP Planning Interval: {:.1}s", config.planning_interval);
    }
}

// System to apply config changes to existing agents
pub fn apply_goap_config_system(
    config: Res<GoapConfig>,
    mut goap_query: Query<&mut GoapAgent, With<Enemy>>,
) {
    if !config.is_changed() { return; }
    
    for mut goap_agent in goap_query.iter_mut() {
        // Force replanning with new interval (planning_cooldown will be updated in main AI system)
        if config.debug_enabled {
            goap_agent.abort_plan(); // Force immediate replanning when debug is enabled
        }
    }
}

#[derive(Component)]
pub struct CoverPoint {
    pub capacity: u8,      // How many enemies can use this cover
    pub current_users: u8, // Currently occupied spots
    pub cover_direction: Vec2, // Direction this cover protects from
}

#[derive(Component)]
pub struct InCover {
    pub cover_entity: Entity, // Which cover point we're using
}

// Cover utility function
fn find_nearest_cover(
    enemy_pos: Vec2, 
    cover_query: &Query<(Entity, &Transform, &CoverPoint), Without<Enemy>>
) -> Option<(Entity, Vec2)> {
    let mut nearest_cover = None;
    let mut nearest_distance = f32::INFINITY;
    
    for (cover_entity, cover_transform, cover_point) in cover_query.iter() {
        // Only consider cover that has available capacity
        if cover_point.current_users >= cover_point.capacity {
            continue;
        }
        
        let cover_pos = cover_transform.translation.truncate();
        let distance = enemy_pos.distance(cover_pos);
        
        if distance < nearest_distance {
            nearest_distance = distance;
            nearest_cover = Some((cover_entity, cover_pos));
        }
    }
    
    nearest_cover
}