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
    HasMedKit,
    HasGrenade,
    // Alert states
    IsAlert,
    IsInvestigating,
    // Cover states
    InCover,
    CoverAvailable,
    UnderFire,
    // Communication states
    BackupCalled,
    NearbyAlliesAvailable,    
    // Flanking
    FlankingPosition,
    TacticalAdvantage,
    // Search
    AreaSearched,
    // Retreat
    IsRetreating,
    AtSafeDistance,
    Outnumbered,
    IsInjured,
    // NEW: Advanced tactical states
    TargetGrouped,
    SafeThrowDistance,
    NearAlarmPanel,
    FacilityAlert,
    AllEnemiesAlerted,
    // NEW: Tactical movement states
    BetterCoverAvailable,
    InBetterCover,
    SafetyImproved,
    AlliesAdvancing,
    EnemySuppressed,
    AlliesAdvantage,
    RetreatPathClear,
    SafelyWithdrawing,
    TacticalRetreat,
    // Parity
    IsPanicked,
    HasBetterWeapon,
    InWeaponRange,
    TooClose,
    TooFar,
    // NEW
    ControllingArea,
    SuppressingTarget,
    AgentsGroupedInRange,    
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
    FlankTarget { target_pos: Vec2, flank_pos: Vec2 },
    SearchArea { center: Vec2, radius: f32 },
    Retreat { retreat_point: Vec2 },
    UseMedKit,
    ThrowGrenade { target_pos: Vec2 },
    ActivateAlarm { panel_pos: Vec2 },
    FindBetterCover { new_cover_pos: Vec2 },
    SuppressingFire { target_area: Vec2 },
    FightingWithdrawal { retreat_path: Vec2 },    
    MaintainDistance,
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
            // === BASIC ACTIONS ===
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
                action_type: ActionType::Patrol,
            },
            
            // === INVESTIGATION ACTIONS ===
            GoapAction {
                name: "investigate",
                cost: 2.0,
                preconditions: hashmap![
                    WorldKey::HeardSound => true,
                    WorldKey::IsAlert => false
                ],
                effects: hashmap![
                    WorldKey::AtLastKnownPosition => true,
                    WorldKey::IsInvestigating => true,
                    WorldKey::HeardSound => false
                ],
                action_type: ActionType::Investigate { location: Vec2::ZERO },
            },
            
            // NEW: Search area systematically
            GoapAction {
                name: "search_area",
                cost: 2.5,
                preconditions: hashmap![
                    WorldKey::HeardSound => true,
                    WorldKey::AtLastKnownPosition => true,
                    WorldKey::AreaSearched => false
                ],
                effects: hashmap![
                    WorldKey::AreaSearched => true,
                    WorldKey::IsInvestigating => false,
                    WorldKey::HeardSound => false
                ],
                action_type: ActionType::SearchArea { center: Vec2::ZERO, radius: 50.0 },
            },
            
            // === COMBAT ACTIONS ===
            GoapAction {
                name: "attack",
                cost: 1.0,
                preconditions: hashmap![
                    WorldKey::HasTarget => true,
                    WorldKey::TargetVisible => true,
                    WorldKey::HasWeapon => true
                ],
                effects: hashmap![
                    WorldKey::HasTarget => false
                ],
                action_type: ActionType::Attack { target: Entity::PLACEHOLDER },
            },
            
            GoapAction {
                name: "move_to_target",
                cost: 3.0,
                preconditions: hashmap![
                    WorldKey::HasTarget => true
                ],
                effects: hashmap![
                    WorldKey::AtTarget => true
                ],
                action_type: ActionType::MoveTo { target: Vec2::ZERO },
            },
            
            // NEW: Flank target for tactical advantage
            GoapAction {
                name: "flank_target",
                cost: 3.0,
                preconditions: hashmap![
                    WorldKey::HasTarget => true,
                    WorldKey::TargetVisible => true,
                    WorldKey::FlankingPosition => false
                ],
                effects: hashmap![
                    WorldKey::FlankingPosition => true,
                    WorldKey::TacticalAdvantage => true,
                    WorldKey::AtTarget => true
                ],
                action_type: ActionType::FlankTarget { target_pos: Vec2::ZERO, flank_pos: Vec2::ZERO },
            },
            
            // === DEFENSIVE ACTIONS ===
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
            
            // NEW: Retreat when outmatched
            GoapAction {
                name: "retreat",
                cost: 1.5,
                preconditions: hashmap![
                    WorldKey::IsInjured => true,
                    WorldKey::Outnumbered => true,
                    WorldKey::IsRetreating => false
                ],
                effects: hashmap![
                    WorldKey::AtSafeDistance => true,
                    WorldKey::IsRetreating => true,
                    WorldKey::IsAlert => false
                ],
                action_type: ActionType::Retreat { retreat_point: Vec2::ZERO },
            },
            
            // === SUPPORT ACTIONS ===
            GoapAction {
                name: "call_for_help",
                cost: 1.5,
                preconditions: hashmap![
                    WorldKey::HasTarget => true,
                    WorldKey::BackupCalled => false,
                    WorldKey::NearbyAlliesAvailable => true
                ],
                effects: hashmap![
                    WorldKey::BackupCalled => true
                ],
                action_type: ActionType::CallForHelp,
            },
            
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
            
            // Tactical reload (reload when low on ammo, not empty)
            GoapAction {
                name: "tactical_reload",
                cost: 1.5,
                preconditions: hashmap![
                    WorldKey::HasWeapon => true,
                    WorldKey::WeaponLoaded => true, // Has some ammo but low
                    WorldKey::HasTarget => false // Safe to reload
                ],
                effects: hashmap![
                    WorldKey::WeaponLoaded => true
                ],
                action_type: ActionType::Reload,
            },

            // === NEW: ADVANCED TACTICAL ACTIONS ===
            GoapAction {
                name: "use_medkit",
                cost: 2.5,
                preconditions: hashmap![
                    WorldKey::IsInjured => true,
                    WorldKey::HasMedKit => true,
                    WorldKey::InCover => true // Safe to heal
                ],
                effects: hashmap![
                    WorldKey::IsInjured => false,
                    WorldKey::HasMedKit => false
                ],
                action_type: ActionType::UseMedKit,
            },
            
            GoapAction {
                name: "throw_grenade",
                cost: 3.0,
                preconditions: hashmap![
                    WorldKey::HasGrenade => true,
                    WorldKey::TargetGrouped => true,
                    WorldKey::SafeThrowDistance => true
                ],
                effects: hashmap![
                    WorldKey::HasGrenade => false,
                    WorldKey::TargetGrouped => false // Disrupts enemy formation
                ],
                action_type: ActionType::ThrowGrenade { target_pos: Vec2::ZERO },
            },
            
            GoapAction {
                name: "activate_alarm",
                cost: 2.0,
                preconditions: hashmap![
                    WorldKey::HasTarget => true,
                    WorldKey::NearAlarmPanel => true,
                    WorldKey::FacilityAlert => false
                ],
                effects: hashmap![
                    WorldKey::FacilityAlert => true,
                    WorldKey::AllEnemiesAlerted => true,
                    WorldKey::BackupCalled => true // Implicit backup call
                ],
                action_type: ActionType::ActivateAlarm { panel_pos: Vec2::ZERO },
            },

            // === NEW: TACTICAL MOVEMENT ACTIONS ===
            GoapAction {
                name: "find_better_cover",
                cost: 2.0,
                preconditions: hashmap![
                    WorldKey::InCover => true,
                    WorldKey::UnderFire => true,
                    WorldKey::BetterCoverAvailable => true
                ],
                effects: hashmap![
                    WorldKey::InBetterCover => true,
                    WorldKey::SafetyImproved => true,
                    WorldKey::UnderFire => false
                ],
                action_type: ActionType::FindBetterCover { new_cover_pos: Vec2::ZERO },
            },
            
            GoapAction {
                name: "suppressing_fire",
                cost: 1.5,
                preconditions: hashmap![
                    WorldKey::HasTarget => true,
                    WorldKey::HasWeapon => true,
                    WorldKey::AlliesAdvancing => true
                ],
                effects: hashmap![
                    WorldKey::EnemySuppressed => true,
                    WorldKey::AlliesAdvantage => true
                ],
                action_type: ActionType::SuppressingFire { target_area: Vec2::ZERO },
            },
            
            GoapAction {
                name: "fighting_withdrawal",
                cost: 2.5,
                preconditions: hashmap![
                    WorldKey::Outnumbered => true,
                    WorldKey::IsInjured => true,
                    WorldKey::RetreatPathClear => true
                ],
                effects: hashmap![
                    WorldKey::SafelyWithdrawing => true,
                    WorldKey::TacticalRetreat => true,
                    WorldKey::AtSafeDistance => true
                ],
                action_type: ActionType::FightingWithdrawal { retreat_path: Vec2::ZERO },
            },

            GoapAction {
                name: "pickup_better_weapon",
                cost: 1.0,
                preconditions: hashmap![
                    WorldKey::HasBetterWeapon => true,
                    WorldKey::IsPanicked => false
                ],
                effects: hashmap![
                    WorldKey::HasBetterWeapon => false
                ],
                action_type: ActionType::MoveTo { target: Vec2::ZERO },
            },
            
            GoapAction {
                name: "panic_flee",
                cost: 0.5,
                preconditions: hashmap![
                    WorldKey::IsPanicked => true
                ],
                effects: hashmap![
                    WorldKey::AtSafeDistance => true
                ],
                action_type: ActionType::Retreat { retreat_point: Vec2::ZERO },
            },
            
            GoapAction {
                name: "maintain_weapon_range",
                cost: 1.5,
                preconditions: hashmap![
                    WorldKey::HasTarget => true,
                    WorldKey::TooClose => true
                ],
                effects: hashmap![
                    WorldKey::InWeaponRange => true,
                    WorldKey::TooClose => false
                ],
                action_type: ActionType::MaintainDistance,
            },
            
            GoapAction {
                name: "close_distance",
                cost: 2.0,
                preconditions: hashmap![
                    WorldKey::HasTarget => true,
                    WorldKey::TooFar => true
                ],
                effects: hashmap![
                    WorldKey::InWeaponRange => true,
                    WorldKey::TooFar => false
                ],
                action_type: ActionType::MoveTo { target: Vec2::ZERO },
            },

            GoapAction {
                name: "flamethrower_area_denial",
                cost: 2.0,
                preconditions: hashmap![
                    WorldKey::HasTarget => true,
                    WorldKey::AgentsGroupedInRange => true,
                    WorldKey::InWeaponRange => true
                ],
                effects: hashmap![
                    WorldKey::ControllingArea => true,
                    WorldKey::TacticalAdvantage => true
                ],
                action_type: ActionType::Attack { target: Entity::PLACEHOLDER },
            },
            
            GoapAction {
                name: "minigun_suppression",
                cost: 1.5,
                preconditions: hashmap![
                    WorldKey::HasTarget => true,
                    WorldKey::InWeaponRange => true,
                    WorldKey::InCover => true
                ],
                effects: hashmap![
                    WorldKey::SuppressingTarget => true,
                    WorldKey::EnemySuppressed => true
                ],
                action_type: ActionType::Attack { target: Entity::PLACEHOLDER },
            },
        ];
    }

    fn setup_default_goals(&mut self) {
        self.goals = vec![
            Goal {
                name: "survival",
                priority: 12.0, // Highest - life preservation
                desired_state: hashmap![
                    WorldKey::AtSafeDistance => true,
                    WorldKey::IsInjured => false
                ],
            },
            
            Goal {
                name: "eliminate_threat",
                priority: 10.0, // High - combat effectiveness
                desired_state: hashmap![
                    WorldKey::HasTarget => false
                ],
            },
            
            Goal {
                name: "area_control",
                priority: 7.0,
                desired_state: hashmap![
                    WorldKey::ControllingArea => true,
                    WorldKey::SuppressingTarget => true
                ],
            },

            Goal {
                name: "coordinate_defense",
                priority: 9.0, // High - team tactics
                desired_state: hashmap![
                    WorldKey::FacilityAlert => true,
                    WorldKey::AllEnemiesAlerted => true
                ],
            },
            
            Goal {
                name: "tactical_advantage",
                priority: 8.0, // Medium-high - smart combat
                desired_state: hashmap![
                    WorldKey::TacticalAdvantage => true,
                    WorldKey::FlankingPosition => true
                ],
            },
            
            Goal {
                name: "thorough_search",
                priority: 6.0, // Medium - complete investigation
                desired_state: hashmap![
                    WorldKey::AreaSearched => true,
                    WorldKey::HeardSound => false
                ],
            },
            
            Goal {
                name: "investigate_disturbance",
                priority: 5.0, // Medium-low - basic investigation
                desired_state: hashmap![
                    WorldKey::HeardSound => false
                ],
            },
            
            Goal {
                name: "patrol_area",
                priority: 1.0, // Lowest - default behavior
                desired_state: hashmap![
                    WorldKey::IsAlert => false
                ],
            },

            Goal {
                name: "panic_survival",
                priority: 15.0,
                desired_state: hashmap![
                    WorldKey::IsPanicked => false,
                    WorldKey::AtSafeDistance => true
                ],
            },
            
            Goal {
                name: "weapon_upgrade",
                priority: 4.0,
                desired_state: hashmap![
                    WorldKey::HasBetterWeapon => false
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
        self.world_state.insert(WorldKey::InCover, false);
        self.world_state.insert(WorldKey::CoverAvailable, false);
        self.world_state.insert(WorldKey::UnderFire, false);
        self.world_state.insert(WorldKey::BackupCalled, false);
        self.world_state.insert(WorldKey::NearbyAlliesAvailable, false);
        // Tactical states
        self.world_state.insert(WorldKey::FlankingPosition, false);
        self.world_state.insert(WorldKey::TacticalAdvantage, false);
        self.world_state.insert(WorldKey::AreaSearched, false);
        self.world_state.insert(WorldKey::IsRetreating, false);
        self.world_state.insert(WorldKey::AtSafeDistance, false);
        self.world_state.insert(WorldKey::Outnumbered, false);
        self.world_state.insert(WorldKey::IsInjured, false);
        // NEW: Advanced tactical states
        self.world_state.insert(WorldKey::HasMedKit, false); // Will be set based on inventory
        self.world_state.insert(WorldKey::HasGrenade, false); // Will be set based on inventory
        self.world_state.insert(WorldKey::TargetGrouped, false);
        self.world_state.insert(WorldKey::SafeThrowDistance, false);
        self.world_state.insert(WorldKey::NearAlarmPanel, false);
        self.world_state.insert(WorldKey::FacilityAlert, false);
        self.world_state.insert(WorldKey::AllEnemiesAlerted, false);
        // NEW: Tactical movement states
        self.world_state.insert(WorldKey::BetterCoverAvailable, false);
        self.world_state.insert(WorldKey::InBetterCover, false);
        self.world_state.insert(WorldKey::SafetyImproved, false);
        self.world_state.insert(WorldKey::AlliesAdvancing, false);
        self.world_state.insert(WorldKey::EnemySuppressed, false);
        self.world_state.insert(WorldKey::AlliesAdvantage, false);
        self.world_state.insert(WorldKey::RetreatPathClear, false);
        self.world_state.insert(WorldKey::SafelyWithdrawing, false);
        self.world_state.insert(WorldKey::TacticalRetreat, false);
        // NEW: Parity states        
        self.world_state.insert(WorldKey::IsPanicked, false);
        self.world_state.insert(WorldKey::HasBetterWeapon, false);
        self.world_state.insert(WorldKey::InWeaponRange, false);
        self.world_state.insert(WorldKey::TooClose, false);
        self.world_state.insert(WorldKey::TooFar, false);

        self.world_state.insert(WorldKey::ControllingArea, false);
        self.world_state.insert(WorldKey::SuppressingTarget, false);
        self.world_state.insert(WorldKey::AgentsGroupedInRange, false);        


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
            let _old_goal = self.current_goal.as_ref().map(|g| g.name);
            self.current_goal = Some(goal.clone());
            
            self.current_plan = self.find_plan(&goal.desired_state);
            let success = !self.current_plan.is_empty();
            
            success
        } else {
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
    mut commands: Commands,
    mut enemy_query: Query<(
        Entity, 
        &Transform, 
        &mut AIState, 
        &mut GoapAgent,
        &mut Vision,
        &Patrol,
        &Health,
        Option<&WeaponState> // Add optional weapon state
    ), (With<Enemy>, Without<Dead>)>,
    agent_query: Query<(Entity, &Transform), With<Agent>>,
    cover_query: Query<(Entity, &Transform, &CoverPoint), Without<Enemy>>,
    all_enemy_query: Query<(Entity, &Transform), (With<Enemy>, Without<Dead>)>,
    mut action_events: EventWriter<ActionEvent>,
    mut audio_events: EventWriter<AudioEvent>,
    mut alert_events: EventWriter<AlertEvent>,
    time: Res<Time>,
    game_mode: Res<GameMode>,
) {
    if game_mode.paused { return; }

    for (enemy_entity, enemy_transform, mut ai_state, mut goap_agent, mut vision, patrol, health, weapon_state) in enemy_query.iter_mut() {
        goap_agent.planning_cooldown -= time.delta_secs();
        
        // Enhanced world state updates with weapon state
        update_world_state_from_perception(
            &mut goap_agent,
            enemy_transform,
            &mut vision,
            &agent_query,
            &mut ai_state,
            patrol,
            &cover_query,
            &all_enemy_query,
            enemy_entity,
            health,
            weapon_state, // Pass weapon state
        );
        
        let should_replan = goap_agent.current_plan.is_empty() || 
                          goap_agent.planning_cooldown <= 0.0 ||
                          plan_invalidated(&goap_agent, &ai_state, health);
        
        let in_danger = *goap_agent.world_state.get(&WorldKey::IsInjured).unwrap_or(&false) ||
                       *goap_agent.world_state.get(&WorldKey::Outnumbered).unwrap_or(&false);
        
        let in_combat = *goap_agent.world_state.get(&WorldKey::HasTarget).unwrap_or(&false) ||
                        *goap_agent.world_state.get(&WorldKey::IsAlert).unwrap_or(&false);
        
        if should_replan {
            goap_agent.plan();
            goap_agent.planning_cooldown = if in_danger { 
                0.3
            } else if in_combat { 
                0.5
            } else { 
                2.0
            };
        }
        
        if let Some(action) = goap_agent.get_next_action() {
            execute_goap_action(
                &action,
                enemy_entity,
                enemy_transform,
                &mut ai_state,
                &mut action_events,
                &mut audio_events,
                &mut alert_events,
                patrol,
                &agent_query,
                &vision,
                &cover_query,
                &mut commands,
            );
        }
    }
}


fn update_world_state_from_perception(
    goap_agent: &mut GoapAgent,
    enemy_transform: &Transform,
    vision: &mut Vision,
    agent_query: &Query<(Entity, &Transform), With<Agent>>,
    ai_state: &mut AIState,
    patrol: &Patrol,
    cover_query: &Query<(Entity, &Transform, &CoverPoint), Without<Enemy>>,
    all_enemy_query: &Query<(Entity, &Transform), (With<Enemy>, Without<Dead>)>,
    current_enemy: Entity,
    health: &Health,
    weapon_state: Option<&WeaponState>, // Add weapon state parameter
) {
    let enemy_pos = enemy_transform.translation.truncate();
    
    // === VISION DIRECTION UPDATE ===
    match &ai_state.mode {
        crate::systems::ai::AIMode::Patrol => {
            if let Some(target) = patrol.current_target() {
                let direction = (target - enemy_pos).normalize_or_zero();
                if direction != Vec2::ZERO {
                    vision.direction = direction;
                }
            }
        },
        crate::systems::ai::AIMode::Combat { target } => {
            if let Ok((_, target_transform)) = agent_query.get(*target) {
                let direction = (target_transform.translation.truncate() - enemy_pos).normalize_or_zero();
                if direction != Vec2::ZERO {
                    vision.direction = direction;
                }
            }
        },
        crate::systems::ai::AIMode::Investigate { location } => {
            let direction = (*location - enemy_pos).normalize_or_zero();
            if direction != Vec2::ZERO {
                vision.direction = direction;
            }
        },
        crate::systems::ai::AIMode::Search { area } => {
            let direction = (*area - enemy_pos).normalize_or_zero();
            if direction != Vec2::ZERO {
                vision.direction = direction;
            }
        },
        crate::systems::ai::AIMode::Panic => {
            goap_agent.update_world_state(WorldKey::IsAlert, true);
            goap_agent.update_world_state(WorldKey::IsInvestigating, false);
            goap_agent.update_world_state(WorldKey::IsPanicked, true);
            goap_agent.update_world_state(WorldKey::IsRetreating, true);
        },        
    }
    
    // === BASIC PERCEPTION ===
    let visible_agent = check_line_of_sight_goap(enemy_transform, vision, agent_query);
    let has_target = visible_agent.is_some();
    let target_visible = has_target;
    
    if let Some(agent_entity) = visible_agent {
        if let Ok((_, agent_transform)) = agent_query.get(agent_entity) {
            let target_pos = agent_transform.translation.truncate();
            ai_state.last_known_target = Some(target_pos);
        }
    }
    
    let at_patrol_point = if let Some(patrol_target) = patrol.current_target() {
        enemy_pos.distance(patrol_target) < 20.0
    } else {
        true
    };
    
    let cover_available = find_nearest_cover(enemy_pos, cover_query).is_some();
    
    let nearby_allies = all_enemy_query.iter()
        .filter(|(entity, transform)| {
            *entity != current_enemy &&
            enemy_pos.distance(transform.translation.truncate()) <= 200.0
        })
        .count() > 0;
    
    // === ENHANCED TACTICAL ASSESSMENT ===
    let is_injured = health.0 < 50.0;
    
    let agent_count = agent_query.iter()
        .filter(|(_, transform)| {
            enemy_pos.distance(transform.translation.truncate()) <= 200.0
        })
        .count();
    
    let nearby_enemy_count = all_enemy_query.iter()
        .filter(|(entity, transform)| {
            *entity != current_enemy &&
            enemy_pos.distance(transform.translation.truncate()) <= 200.0
        })
        .count();
    
    let outnumbered = agent_count > nearby_enemy_count + 1;
    
    let at_safe_distance = !agent_query.iter()
        .any(|(_, transform)| {
            enemy_pos.distance(transform.translation.truncate()) <= 150.0
        });
    
    // === NEW: ADVANCED TACTICAL PERCEPTION ===
    // Check if multiple agents are grouped together (grenade opportunity)
    let agents_in_area: Vec<_> = agent_query.iter()
        .filter(|(_, transform)| {
            enemy_pos.distance(transform.translation.truncate()) <= 300.0
        })
        .collect();
    
    let target_grouped = if agents_in_area.len() >= 2 {
        // Check if agents are close to each other
        let positions: Vec<Vec2> = agents_in_area.iter()
            .map(|(_, t)| t.translation.truncate())
            .collect();
        
        positions.iter().any(|&pos1| {
            positions.iter().filter(|&&pos2| pos1.distance(pos2) <= 80.0).count() >= 2
        })
    } else {
        false
    };
    
    // Safe throw distance - not too close, not too far
    let safe_throw_distance = if let Some((_, agent_transform)) = agents_in_area.first() {
        let distance = enemy_pos.distance(agent_transform.translation.truncate());
        distance >= 100.0 && distance <= 250.0
    } else {
        false
    };
    
    // Simple alarm panel detection (would be improved with actual panel entities)
    let near_alarm_panel = false; // TODO: Implement actual alarm panel detection
    
    // Equipment states (simplified - would check actual inventory)
    let has_medkit = is_injured && rand::random::<f32>() < 0.3; // 30% chance to have medkit when injured
    let has_grenade = target_grouped && rand::random::<f32>() < 0.2; // 20% chance to have grenade when opportunity exists
    
    // === NEW: TACTICAL MOVEMENT PERCEPTION ===
    // Check if better cover is available when under fire
    let under_fire = agent_count > 0 && enemy_pos.distance(
        agent_query.iter().next().map(|(_, t)| t.translation.truncate()).unwrap_or(Vec2::ZERO)
    ) <= 120.0; // Within close combat range
    
    let better_cover_available = if under_fire && cover_available {
        // Simple check: if current cover exists, assume better cover might be available
        cover_query.iter().count() > 1
    } else {
        false
    };
    
    // Check if allies are advancing (other enemies moving toward agents)
    let allies_advancing = nearby_enemy_count > 0 && agent_count > 0;
    
    // Simple retreat path check - clear if no agents between enemy and patrol point
    let retreat_path_clear = if let Some(patrol_point) = patrol.current_target() {
        let to_patrol = (patrol_point - enemy_pos).normalize_or_zero();
        let retreat_blocked = agent_query.iter().any(|(_, agent_transform)| {
            let agent_pos = agent_transform.translation.truncate();
            let to_agent = (agent_pos - enemy_pos).normalize_or_zero();
            to_patrol.dot(to_agent) > 0.7 // Agent is in retreat direction
        });
        !retreat_blocked
    } else {
        true
    };
    
    // === WEAPON/AMMO STATE ===
    if let Some(weapon_state) = weapon_state {
        let weapon_loaded = weapon_state.current_ammo > 0;
        let _needs_reload = weapon_state.needs_reload();
        
        goap_agent.update_world_state(WorldKey::WeaponLoaded, weapon_loaded);
        goap_agent.update_world_state(WorldKey::HasWeapon, true);
        
    } else {
        goap_agent.update_world_state(WorldKey::WeaponLoaded, true);
        goap_agent.update_world_state(WorldKey::HasWeapon, true);
    }
    
    // Update all world states
    goap_agent.update_world_state(WorldKey::TargetVisible, target_visible);
    goap_agent.update_world_state(WorldKey::HasTarget, has_target);
    goap_agent.update_world_state(WorldKey::AtPatrolPoint, at_patrol_point);
    goap_agent.update_world_state(WorldKey::CoverAvailable, cover_available);
    goap_agent.update_world_state(WorldKey::NearbyAlliesAvailable, nearby_allies);
    goap_agent.update_world_state(WorldKey::IsInjured, is_injured);
    goap_agent.update_world_state(WorldKey::Outnumbered, outnumbered);
    goap_agent.update_world_state(WorldKey::AtSafeDistance, at_safe_distance);
    
    // NEW: Advanced tactical states
    goap_agent.update_world_state(WorldKey::TargetGrouped, target_grouped);
    goap_agent.update_world_state(WorldKey::SafeThrowDistance, safe_throw_distance);
    goap_agent.update_world_state(WorldKey::NearAlarmPanel, near_alarm_panel);
    goap_agent.update_world_state(WorldKey::HasMedKit, has_medkit);
    goap_agent.update_world_state(WorldKey::HasGrenade, has_grenade);
    // NEW: Tactical movement states
    goap_agent.update_world_state(WorldKey::UnderFire, under_fire);
    goap_agent.update_world_state(WorldKey::BetterCoverAvailable, better_cover_available);
    goap_agent.update_world_state(WorldKey::AlliesAdvancing, allies_advancing);
    goap_agent.update_world_state(WorldKey::RetreatPathClear, retreat_path_clear);
    
    if !has_target {
        goap_agent.update_world_state(WorldKey::FlankingPosition, false);
        goap_agent.update_world_state(WorldKey::TacticalAdvantage, false);
    }
    
    // Update based on AI state with immediate mode switch for target detection
    match &ai_state.mode {
        crate::systems::ai::AIMode::Patrol => {
            goap_agent.update_world_state(WorldKey::IsAlert, false);
            goap_agent.update_world_state(WorldKey::IsInvestigating, false);
            goap_agent.update_world_state(WorldKey::IsRetreating, false);
            
            if has_target && visible_agent.is_some() {
                ai_state.mode = crate::systems::ai::AIMode::Combat { target: visible_agent.unwrap() };
                goap_agent.update_world_state(WorldKey::IsAlert, true);
                goap_agent.abort_plan();
            }
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
        crate::systems::ai::AIMode::Panic => {
            goap_agent.update_world_state(WorldKey::IsAlert, true);
            goap_agent.update_world_state(WorldKey::IsInvestigating, false);
            goap_agent.update_world_state(WorldKey::IsPanicked, true);
            goap_agent.update_world_state(WorldKey::IsRetreating, true);
        },         
    }
}

fn plan_invalidated(goap_agent: &GoapAgent, ai_state: &AIState, health: &Health) -> bool {
    let critically_injured = health.0 < 30.0;
    let planning_survival = goap_agent.current_goal.as_ref()
        .map(|g| g.name == "survival" || g.name == "panic_survival")
        .unwrap_or(false);
    
    if critically_injured && !planning_survival {
        return true;
    }
    
    let outnumbered = *goap_agent.world_state.get(&WorldKey::Outnumbered).unwrap_or(&false);
    let has_tactical_goal = goap_agent.current_goal.as_ref()
        .map(|g| g.name == "tactical_advantage" || g.name == "survival" || g.name == "panic_survival")
        .unwrap_or(false);
    
    if outnumbered && !has_tactical_goal {
        return true;
    }
    
    // Check for panic state changes
    let is_panicked = *goap_agent.world_state.get(&WorldKey::IsPanicked).unwrap_or(&false);
    if is_panicked && !planning_survival {
        return true;
    }
    
    // Check combat state changes
    match &ai_state.mode {
        crate::systems::ai::AIMode::Combat { .. } => {
            !goap_agent.world_state.get(&WorldKey::HasTarget).unwrap_or(&false)
        },
        crate::systems::ai::AIMode::Panic => {
            // Always replan when in panic to find escape routes
            true
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
    alert_events: &mut EventWriter<AlertEvent>,
    patrol: &Patrol,
    agent_query: &Query<(Entity, &Transform), With<Agent>>,
    vision: &Vision,
    cover_query: &Query<(Entity, &Transform, &CoverPoint), Without<Enemy>>,
    commands: &mut Commands,
) {
    match &action.action_type {
        // === BASIC ACTIONS ===
        ActionType::Patrol => {
            if let Some(target) = patrol.current_target() {
                ai_state.mode = crate::systems::ai::AIMode::Patrol;
                action_events.write(ActionEvent {
                    entity: enemy_entity,
                    action: Action::MoveTo(target),
                });
            } else {
                ai_state.mode = crate::systems::ai::AIMode::Patrol;
            }
        },
        
        ActionType::MoveTo { target } => {
            action_events.write(ActionEvent {
                entity: enemy_entity,
                action: Action::MoveTo(*target),
            });
        },
        
        // === ENHANCED COMBAT ACTIONS ===
        ActionType::Attack { target: _ } => {
            if let Some(agent_entity) = check_line_of_sight_goap(
                &Transform::from_translation(enemy_transform.translation),
                vision,
                agent_query
            ) {
                if let Ok((_, agent_transform)) = agent_query.get(agent_entity) {
                    let distance = enemy_transform.translation.truncate()
                        .distance(agent_transform.translation.truncate());
                    
                    ai_state.mode = crate::systems::ai::AIMode::Combat { target: agent_entity };
                    
                    if distance <= 150.0 {
                        action_events.write(ActionEvent {
                            entity: enemy_entity,
                            action: Action::Attack(agent_entity),
                        });
                    } else {
                        action_events.write(ActionEvent {
                            entity: enemy_entity,
                            action: Action::MoveTo(agent_transform.translation.truncate()),
                        });
                    }
                }
            } else if let Some(last_pos) = ai_state.last_known_target {
                action_events.write(ActionEvent {
                    entity: enemy_entity,
                    action: Action::MoveTo(last_pos),
                });
                ai_state.mode = crate::systems::ai::AIMode::Investigate { location: last_pos };
            }
        },
        
        // NEW: Flanking maneuver
        ActionType::FlankTarget { target_pos: _, flank_pos: _ } => {
            if let Some(agent_entity) = find_closest_agent(enemy_transform, agent_query) {
                if let Ok((_, agent_transform)) = agent_query.get(agent_entity) {
                    let agent_pos = agent_transform.translation.truncate();
                    let enemy_pos = enemy_transform.translation.truncate();
                    
                    // Calculate flanking position (90 degrees from current approach)
                    let to_agent = (agent_pos - enemy_pos).normalize_or_zero();
                    let flank_offset = Vec2::new(-to_agent.y, to_agent.x) * 80.0; // Perpendicular
                    let flank_position = agent_pos + flank_offset;
                    
                    action_events.write(ActionEvent {
                        entity: enemy_entity,
                        action: Action::MoveTo(flank_position),
                    });
                    
                    ai_state.mode = crate::systems::ai::AIMode::Combat { target: agent_entity };
                }
            }
        },
        
        // === INVESTIGATION ACTIONS ===
        ActionType::Investigate { location } => {
            let investigation_target = if let Some(last_pos) = ai_state.last_known_target {
                last_pos
            } else {
                *location
            };
            
            ai_state.mode = crate::systems::ai::AIMode::Investigate { 
                location: investigation_target 
            };
            action_events.write(ActionEvent {
                entity: enemy_entity,
                action: Action::MoveTo(investigation_target),
            });
        },
        
        // NEW: Systematic area search
        ActionType::SearchArea { center, radius } => {
            let search_center = if let Some(last_pos) = ai_state.last_known_target {
                last_pos
            } else {
                *center
            };
            
            // Calculate search pattern point using simple spiral
            let enemy_pos = enemy_transform.translation.truncate();
            let angle = (enemy_pos.x + enemy_pos.y) * 0.1; // Pseudo-random angle
            let search_offset = Vec2::new(angle.cos(), angle.sin()) * radius;
            let search_point = search_center + search_offset;
            
            action_events.write(ActionEvent {
                entity: enemy_entity,
                action: Action::MoveTo(search_point),
            });
            
            ai_state.mode = crate::systems::ai::AIMode::Search { area: search_center };
        },
        
        ActionType::Search { area } => {
            ai_state.mode = crate::systems::ai::AIMode::Search { area: *area };
        },
        
        // === DEFENSIVE ACTIONS ===
        ActionType::TakeCover => {
            if let Some((cover_entity, cover_pos)) = find_nearest_cover(enemy_transform.translation.truncate(), cover_query) {
                action_events.write(ActionEvent {
                    entity: enemy_entity,
                    action: Action::MoveTo(cover_pos),
                });
                commands.entity(enemy_entity).insert(InCover { cover_entity });
            }
        },
        
        // NEW: Tactical retreat
        ActionType::Retreat { retreat_point: _ } => {
            let enemy_pos = enemy_transform.translation.truncate();
            
            // Calculate retreat direction - away from nearest agent
            let retreat_direction = if let Some(agent_entity) = find_closest_agent(enemy_transform, agent_query) {
                if let Ok((_, agent_transform)) = agent_query.get(agent_entity) {
                    let to_agent = agent_transform.translation.truncate() - enemy_pos;
                    -to_agent.normalize_or_zero() // Opposite direction
                } else {
                    Vec2::new(1.0, 0.0) // Fallback
                }
            } else {
                // Fallback to patrol point or default direction
                if let Some(patrol_point) = patrol.current_target() {
                    (patrol_point - enemy_pos).normalize_or_zero()
                } else {
                    Vec2::new(1.0, 0.0)
                }
            };
            
            let retreat_distance = 120.0;
            let retreat_point = enemy_pos + retreat_direction * retreat_distance;
            
            action_events.write(ActionEvent {
                entity: enemy_entity,
                action: Action::MoveTo(retreat_point),
            });
            
            ai_state.mode = crate::systems::ai::AIMode::Patrol;
            
        },
        
        ActionType::CallForHelp => {
            audio_events.write(AudioEvent {
                sound: AudioType::Alert,
                volume: 1.0,
            });
            
            alert_events.write(AlertEvent {
                alerter: enemy_entity,
                position: enemy_transform.translation.truncate(),
                alert_type: AlertType::CallForHelp,
            });
            
        },

        ActionType::Reload => {
            action_events.write(ActionEvent {
                entity: enemy_entity,
                action: Action::Reload,
            });
        },
        
        // === NEW: ADVANCED ACTION EXECUTION ===
        ActionType::UseMedKit => {
            // Self-heal when safe
            audio_events.write(AudioEvent {
                sound: AudioType::Alert, // Reuse existing sound
                volume: 0.3,
            });
            
            // Add healing to event system later, for now just log
            info!("Enemy {} using medkit to heal", enemy_entity.index());
        },
        
        ActionType::ThrowGrenade { target_pos: _ } => {
            // Calculate grenade throw position
            let throw_target = if let Some(agent_entity) = find_closest_agent(enemy_transform, agent_query) {
                if let Ok((_, agent_transform)) = agent_query.get(agent_entity) {
                    // Predict agent movement slightly
                    agent_transform.translation.truncate() + Vec2::new(20.0, 0.0)
                } else {
                    enemy_transform.translation.truncate() + Vec2::new(50.0, 0.0)
                }
            } else {
                enemy_transform.translation.truncate() + Vec2::new(50.0, 0.0)
            };
            
            // Trigger explosion effect and damage
            audio_events.write(AudioEvent {
                sound: AudioType::Alert, // Loud explosion sound
                volume: 1.0,
            });
            
            // Add grenade explosion to event system later
            info!("Enemy {} throwing grenade at {:?}", enemy_entity.index(), throw_target);
        },
        
        ActionType::ActivateAlarm { panel_pos: _ } => {
            // Trigger facility-wide alert
            audio_events.write(AudioEvent {
                sound: AudioType::Alert,
                volume: 1.0,
            });
            
            // Send alert to all enemies
            alert_events.write(AlertEvent {
                alerter: enemy_entity,
                position: enemy_transform.translation.truncate(),
                alert_type: AlertType::EnemySpotted, // Reuse existing type
            });
            
            info!("Enemy {} activated facility alarm!", enemy_entity.index());
        },
        
        // === NEW: TACTICAL MOVEMENT EXECUTION ===
        ActionType::FindBetterCover { new_cover_pos: _ } => {
            // Find the best available cover point (furthest from agents)
            if let Some((cover_entity, cover_pos)) = find_best_cover(
                enemy_transform.translation.truncate(), 
                cover_query, 
                agent_query
            ) {
                action_events.write(ActionEvent {
                    entity: enemy_entity,
                    action: Action::MoveTo(cover_pos),
                });
                commands.entity(enemy_entity).insert(InCover { cover_entity });
                info!("Enemy {} moving to better cover", enemy_entity.index());
            }
        },
        
        ActionType::SuppressingFire { target_area: _ } => {
            // Sustained fire toward enemy position to pin them down
            if let Some(agent_entity) = find_closest_agent(enemy_transform, agent_query) {
                if let Ok((_, _agent_transform)) = agent_query.get(agent_entity) {
                    // Fire repeatedly at agent area
                    action_events.write(ActionEvent {
                        entity: enemy_entity,
                        action: Action::Attack(agent_entity),
                    });
                    
                    // Audio cue for suppressing fire
                    audio_events.write(AudioEvent {
                        sound: AudioType::Gunshot,
                        volume: 0.8,
                    });
                    
                    info!("Enemy {} providing suppressing fire", enemy_entity.index());
                }
            }
        },
        
        ActionType::MaintainDistance => {
            if let Some(agent_entity) = find_closest_agent(enemy_transform, agent_query) {
                if let Ok((_, agent_transform)) = agent_query.get(agent_entity) {
                    let agent_pos = agent_transform.translation.truncate();
                    let enemy_pos = enemy_transform.translation.truncate();
                    let away_direction = (enemy_pos - agent_pos).normalize_or_zero();
                    let retreat_pos = enemy_pos + away_direction * 80.0;
                    
                    action_events.write(ActionEvent {
                        entity: enemy_entity,
                        action: Action::MoveTo(retreat_pos),
                    });
                }
            }
        },

        ActionType::FightingWithdrawal { retreat_path: _ } => {
            // Retreat while maintaining defensive fire
            let enemy_pos = enemy_transform.translation.truncate();
            
            // Calculate tactical retreat position
            let retreat_target = if let Some(patrol_point) = patrol.current_target() {
                // Retreat toward patrol point
                let to_patrol = (patrol_point - enemy_pos).normalize_or_zero();
                enemy_pos + to_patrol * 100.0
            } else {
                // Fallback: move away from nearest agent
                if let Some(agent_entity) = find_closest_agent(enemy_transform, agent_query) {
                    if let Ok((_, agent_transform)) = agent_query.get(agent_entity) {
                        let away_from_agent = (enemy_pos - agent_transform.translation.truncate()).normalize_or_zero();
                        enemy_pos + away_from_agent * 100.0
                    } else {
                        enemy_pos + Vec2::new(100.0, 0.0)
                    }
                } else {
                    enemy_pos + Vec2::new(100.0, 0.0)
                }
            };
            
            // Move to retreat position
            action_events.write(ActionEvent {
                entity: enemy_entity,
                action: Action::MoveTo(retreat_target),
            });
            
            // Provide covering fire while retreating
            if let Some(agent_entity) = find_closest_agent(enemy_transform, agent_query) {
                action_events.write(ActionEvent {
                    entity: enemy_entity,
                    action: Action::Attack(agent_entity),
                });
            }
            
            ai_state.mode = crate::systems::ai::AIMode::Patrol;
            info!("Enemy {} executing fighting withdrawal", enemy_entity.index());
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
    
    for (_entity, transform, goap_agent) in goap_query.iter() {
        let pos = transform.translation.truncate();
        
        // Draw GOAP indicator (blue circle above enemy)
        gizmos.circle_2d(pos + Vec2::new(0.0, 40.0), 8.0, Color::srgb(0.3, 0.8, 1.0));
        
        // Draw current goal indicator
        if let Some(goal) = &goap_agent.current_goal {
            let goal_color = match goal.name {
                "eliminate_threat" => Color::srgb(1.0, 0.2, 0.2),
                "investigate_disturbance" => Color::srgb(1.0, 0.8, 0.2),
                "patrol_area" => Color::srgb(0.2, 1.0, 0.2),
                "coordinate_defense" => Color::srgb(0.8, 0.2, 0.8),
                "survival" => Color::srgb(1.0, 0.4, 0.0),
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
            gizmos.rect_2d(
                pos + Vec2::new(-20.0, y_offset), 
                Vec2::splat(indicator_size), 
                Color::srgb(1.0, 0.0, 0.0)
            );
        }
        if *goap_agent.world_state.get(&WorldKey::TargetVisible).unwrap_or(&false) {
            gizmos.rect_2d(
                pos + Vec2::new(-15.0, y_offset), 
                Vec2::splat(indicator_size), 
                Color::srgb(1.0, 0.5, 0.0)
            );
        }
        if *goap_agent.world_state.get(&WorldKey::IsAlert).unwrap_or(&false) {
            gizmos.rect_2d(
                pos + Vec2::new(-10.0, y_offset), 
                Vec2::splat(indicator_size), 
                Color::srgb(1.0, 1.0, 0.0)
            );
        }
        if *goap_agent.world_state.get(&WorldKey::HeardSound).unwrap_or(&false) {
            gizmos.rect_2d(
                pos + Vec2::new(-5.0, y_offset), 
                Vec2::splat(indicator_size), 
                Color::srgb(0.0, 1.0, 1.0)
            );
        }
        
        // NEW: Advanced state indicators (second row)
        y_offset = -55.0;
        if *goap_agent.world_state.get(&WorldKey::IsInjured).unwrap_or(&false) {
            gizmos.rect_2d(
                pos + Vec2::new(-20.0, y_offset), 
                Vec2::splat(indicator_size), 
                Color::srgb(0.8, 0.2, 0.8) // Purple for injured
            );
        }
        if *goap_agent.world_state.get(&WorldKey::HasMedKit).unwrap_or(&false) {
            gizmos.rect_2d(
                pos + Vec2::new(-15.0, y_offset), 
                Vec2::splat(indicator_size), 
                Color::srgb(0.2, 0.8, 0.2) // Green for medkit
            );
        }
        if *goap_agent.world_state.get(&WorldKey::HasGrenade).unwrap_or(&false) {
            gizmos.rect_2d(
                pos + Vec2::new(-10.0, y_offset), 
                Vec2::splat(indicator_size), 
                Color::srgb(0.8, 0.8, 0.0) // Yellow for grenade
            );
        }
        if *goap_agent.world_state.get(&WorldKey::TargetGrouped).unwrap_or(&false) {
            gizmos.rect_2d(
                pos + Vec2::new(-5.0, y_offset), 
                Vec2::splat(indicator_size), 
                Color::srgb(1.0, 0.4, 0.0) // Orange for grouped targets
            );
        }
        
        // NEW: Third row - tactical movement states
        y_offset = -65.0;
        if *goap_agent.world_state.get(&WorldKey::UnderFire).unwrap_or(&false) {
            gizmos.rect_2d(
                pos + Vec2::new(-20.0, y_offset), 
                Vec2::splat(indicator_size), 
                Color::srgb(1.0, 0.0, 0.0) // Red for under fire
            );
        }
        if *goap_agent.world_state.get(&WorldKey::BetterCoverAvailable).unwrap_or(&false) {
            gizmos.rect_2d(
                pos + Vec2::new(-15.0, y_offset), 
                Vec2::splat(indicator_size), 
                Color::srgb(0.0, 0.8, 1.0) // Cyan for better cover
            );
        }
        if *goap_agent.world_state.get(&WorldKey::AlliesAdvancing).unwrap_or(&false) {
            gizmos.rect_2d(
                pos + Vec2::new(-10.0, y_offset), 
                Vec2::splat(indicator_size), 
                Color::srgb(0.0, 1.0, 0.0) // Green for allies advancing
            );
        }
        if *goap_agent.world_state.get(&WorldKey::TacticalRetreat).unwrap_or(&false) {
            gizmos.rect_2d(
                pos + Vec2::new(-5.0, y_offset), 
                Vec2::splat(indicator_size), 
                Color::srgb(0.5, 0.5, 1.0) // Light blue for tactical retreat
            );
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
            info!("  Purple = coordinate_defense goal");
            info!("  Orange = survival goal");
            info!("  Cyan line = current plan length");
            info!("  Row 1: Red=HasTarget, Orange=TargetVisible, Yellow=IsAlert, Cyan=HeardSound");
            info!("  Row 2: Purple=IsInjured, Green=HasMedKit, Yellow=HasGrenade, Orange=TargetGrouped");
            info!("  Row 3: Red=UnderFire, Cyan=BetterCover, Green=AlliesAdvancing, LightBlue=TacticalRetreat");
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

#[derive(Component)]
pub struct AlarmPanel {
    pub activated: bool,
    pub range: f32,
}

#[derive(Component)]
pub struct Equipment {
    pub medkits: u8,
    pub grenades: u8,
    pub tools: Vec<String>,
}

impl Default for Equipment {
    fn default() -> Self {
        Self {
            medkits: 1,
            grenades: 0, // Start without grenades
            tools: Vec::new(),
        }
    }
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

fn find_best_cover(
    enemy_pos: Vec2,
    cover_query: &Query<(Entity, &Transform, &CoverPoint), Without<Enemy>>,
    agent_query: &Query<(Entity, &Transform), With<Agent>>,
) -> Option<(Entity, Vec2)> {
    let mut best_cover = None;
    let mut best_score = f32::NEG_INFINITY;
    
    for (cover_entity, cover_transform, cover_point) in cover_query.iter() {
        if cover_point.current_users >= cover_point.capacity {
            continue;
        }
        
        let cover_pos = cover_transform.translation.truncate();
        let distance_to_cover = enemy_pos.distance(cover_pos);
        
        if distance_to_cover > 150.0 { continue; } // Too far
        
        // Score based on: close to enemy, far from agents
        let mut score = 100.0 - distance_to_cover; // Prefer closer cover
        
        // Bonus for being far from agents
        for (_, agent_transform) in agent_query.iter() {
            let distance_to_agent = cover_pos.distance(agent_transform.translation.truncate());
            score += distance_to_agent * 0.5; // Prefer cover far from agents
        }
        
        if score > best_score {
            best_score = score;
            best_cover = Some((cover_entity, cover_pos));
        }
    }
    
    best_cover
}


// Helper function to find closest agent
fn find_closest_agent(
    enemy_transform: &Transform,
    agent_query: &Query<(Entity, &Transform), With<Agent>>,
) -> Option<Entity> {
    let enemy_pos = enemy_transform.translation.truncate();
    
    agent_query.iter()
        .min_by(|(_, a_transform), (_, b_transform)| {
            let a_distance = enemy_pos.distance(a_transform.translation.truncate());
            let b_distance = enemy_pos.distance(b_transform.translation.truncate());
            a_distance.partial_cmp(&b_distance).unwrap_or(std::cmp::Ordering::Equal)
        })
        .map(|(entity, _)| entity)
}