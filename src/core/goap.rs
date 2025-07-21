// src/core/goap.rs
use bevy::prelude::*;
use std::collections::{HashMap, VecDeque};
use crate::systems::ai::AIMode;
use crate::core::factions::Faction;

macro_rules! world_state {
    ( $( $key:expr => $value:expr ),* $(,)? ) => {{
        let mut map = std::collections::HashMap::new();
        $(
            map.insert($key, $value);
        )*
        map
    }};
}


// === CORE TYPES ===
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WorldKey {
    // Position & Movement
    AtPatrolPoint, AtLastKnownPosition, AtTarget,
    // Knowledge & Awareness  
    HasTarget, TargetVisible, HeardSound, IsAlert, IsInvestigating,
    // Equipment & Resources
    HasWeapon, WeaponLoaded, HasMedKit, HasGrenade,
    // Combat & Safety
    InCover, CoverAvailable, UnderFire, IsInjured, Outnumbered,
    // Communication & Support
    BackupCalled, NearbyAlliesAvailable,
    // Advanced Tactics
    FlankingPosition, TacticalAdvantage, TargetGrouped, SafeThrowDistance,
    NearAlarmPanel, FacilityAlert, AllEnemiesAlerted,
    // Movement & Positioning
    BetterCoverAvailable, InBetterCover, SafetyImproved, AlliesAdvancing,
    EnemySuppressed, AlliesAdvantage, RetreatPathClear, SafelyWithdrawing,
    TacticalRetreat, AreaSearched, IsRetreating, AtSafeDistance,
    // Weapon Specific
    IsPanicked, HasBetterWeapon, InWeaponRange, TooClose, TooFar,
    ControllingArea, SuppressingTarget, AgentsGroupedInRange,
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
        
        agent.setup_actions_and_goals();
        agent.setup_initial_world_state();
        agent
    }
}

impl GoapAgent {
    // === SETUP ===
    fn setup_actions_and_goals(&mut self) {
        self.available_actions = create_action_library();
        self.goals = create_goal_library();
    }
    
    fn setup_initial_world_state(&mut self) {
        self.world_state = world_state![
            WorldKey::HasWeapon => true,
            WorldKey::WeaponLoaded => true,
            WorldKey::IsAlert => false,
            WorldKey::HasTarget => false,
            WorldKey::TargetVisible => false,
            WorldKey::HeardSound => false,
            WorldKey::AtPatrolPoint => true,
            WorldKey::AtLastKnownPosition => false,
            WorldKey::AtTarget => false,
            WorldKey::IsInvestigating => false,
            WorldKey::InCover => false,
            WorldKey::CoverAvailable => false,
            WorldKey::UnderFire => false,
            WorldKey::BackupCalled => false,
            WorldKey::NearbyAlliesAvailable => false,
            WorldKey::FlankingPosition => false,
            WorldKey::TacticalAdvantage => false,
            WorldKey::AreaSearched => false,
            WorldKey::IsRetreating => false,
            WorldKey::AtSafeDistance => false,
            WorldKey::Outnumbered => false,
            WorldKey::IsInjured => false,
            WorldKey::HasMedKit => false,
            WorldKey::HasGrenade => false,
            WorldKey::TargetGrouped => false,
            WorldKey::SafeThrowDistance => false,
            WorldKey::NearAlarmPanel => false,
            WorldKey::FacilityAlert => false,
            WorldKey::AllEnemiesAlerted => false,
            WorldKey::BetterCoverAvailable => false,
            WorldKey::InBetterCover => false,
            WorldKey::SafetyImproved => false,
            WorldKey::AlliesAdvancing => false,
            WorldKey::EnemySuppressed => false,
            WorldKey::AlliesAdvantage => false,
            WorldKey::RetreatPathClear => false,
            WorldKey::SafelyWithdrawing => false,
            WorldKey::TacticalRetreat => false,
            WorldKey::IsPanicked => false,
            WorldKey::HasBetterWeapon => false,
            WorldKey::InWeaponRange => false,
            WorldKey::TooClose => false,
            WorldKey::TooFar => false,
            WorldKey::ControllingArea => false,
            WorldKey::SuppressingTarget => false,
            WorldKey::AgentsGroupedInRange => false,
        ];
    }
    
    // === STATE MANAGEMENT ===
    pub fn update_world_state(&mut self, key: WorldKey, value: bool) {
        self.world_state.insert(key, value);
    }

    pub fn update_multiple(&mut self, updates: impl IntoIterator<Item = (WorldKey, bool)>) {
        for (key, value) in updates {
            self.update_world_state(key, value);
        }
    }    

    // === PLANNING ===
    pub fn plan(&mut self) -> bool {
        let goal = self.goals.iter()
            .filter(|g| !self.is_goal_satisfied(&g.desired_state))
            .max_by(|a, b| a.priority.partial_cmp(&b.priority).unwrap_or(std::cmp::Ordering::Equal));
        
        if let Some(goal) = goal {
            self.current_goal = Some(goal.clone());
            self.current_plan = self.find_plan(&goal.desired_state);
            !self.current_plan.is_empty()
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
        let mut plan = VecDeque::new();
        let mut current_state = self.world_state.clone();
        let mut remaining_goals = goal_state.clone();
        let max_depth = 10;
        let mut depth = 0;
        
        while !remaining_goals.is_empty() && depth < max_depth {
            depth += 1;
            
            if let Some(action) = self.find_satisfying_action(&remaining_goals, &current_state) {
                if self.can_execute_action(&action, &current_state) {
                    self.apply_effects(&action.effects, &mut current_state);
                    
                    for (key, &value) in &action.effects {
                        if remaining_goals.get(key) == Some(&value) {
                            remaining_goals.remove(key);
                        }
                    }
                    
                    plan.push_front(action);
                } else {
                    for (&key, &value) in &action.preconditions {
                        if current_state.get(&key) != Some(&value) {
                            remaining_goals.insert(key, value);
                        }
                    }
                }
            } else {
                break;
            }
        }
        
        if remaining_goals.is_empty() { plan } else { VecDeque::new() }
    }
    
    fn find_satisfying_action(&self, goals: &WorldState, _current_state: &WorldState) -> Option<GoapAction> {
        self.available_actions.iter()
            .find(|action| {
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

// === ACTION LIBRARY ===
fn create_action_library() -> Vec<GoapAction> {
    vec![
        // Basic Actions
        GoapAction {
            name: "patrol",
            cost: 1.0,
            preconditions: world_state![WorldKey::IsAlert => false, WorldKey::HasTarget => false],
            effects: world_state![WorldKey::AtPatrolPoint => true],
            action_type: ActionType::Patrol,
        },
        GoapAction {
            name: "return_to_patrol",
            cost: 1.5,
            preconditions: world_state![WorldKey::HasTarget => false, WorldKey::AtPatrolPoint => false],
            effects: world_state![WorldKey::AtPatrolPoint => true, WorldKey::IsAlert => false],
            action_type: ActionType::Patrol,
        },
        GoapAction {
            name: "calm_down",
            cost: 0.5,
            preconditions: world_state![WorldKey::HasTarget => false, WorldKey::TargetVisible => false],
            effects: world_state![WorldKey::IsAlert => false],
            action_type: ActionType::Patrol,
        },
        
        // Investigation
        GoapAction {
            name: "investigate",
            cost: 2.0,
            preconditions: world_state![WorldKey::HeardSound => true, WorldKey::IsAlert => false],
            effects: world_state![WorldKey::AtLastKnownPosition => true, WorldKey::IsInvestigating => true, WorldKey::HeardSound => false],
            action_type: ActionType::Investigate { location: Vec2::ZERO },
        },
        GoapAction {
            name: "search_area",
            cost: 2.5,
            preconditions: world_state![WorldKey::HeardSound => true, WorldKey::AtLastKnownPosition => true, WorldKey::AreaSearched => false],
            effects: world_state![WorldKey::AreaSearched => true, WorldKey::IsInvestigating => false, WorldKey::HeardSound => false],
            action_type: ActionType::SearchArea { center: Vec2::ZERO, radius: 50.0 },
        },
        
        // Combat
        GoapAction {
            name: "attack",
            cost: 1.0,
            preconditions: world_state![WorldKey::HasTarget => true, WorldKey::TargetVisible => true, WorldKey::HasWeapon => true],
            effects: world_state![WorldKey::HasTarget => false],
            action_type: ActionType::Attack { target: Entity::PLACEHOLDER },
        },
        GoapAction {
            name: "move_to_target",
            cost: 3.0,
            preconditions: world_state![WorldKey::HasTarget => true],
            effects: world_state![WorldKey::AtTarget => true],
            action_type: ActionType::MoveTo { target: Vec2::ZERO },
        },
        GoapAction {
            name: "flank_target",
            cost: 3.0,
            preconditions: world_state![WorldKey::HasTarget => true, WorldKey::TargetVisible => true, WorldKey::FlankingPosition => false],
            effects: world_state![WorldKey::FlankingPosition => true, WorldKey::TacticalAdvantage => true, WorldKey::AtTarget => true],
            action_type: ActionType::FlankTarget { target_pos: Vec2::ZERO, flank_pos: Vec2::ZERO },
        },
        
        // Defensive
        GoapAction {
            name: "take_cover",
            cost: 2.0,
            preconditions: world_state![WorldKey::HasTarget => true, WorldKey::InCover => false, WorldKey::CoverAvailable => true],
            effects: world_state![WorldKey::InCover => true],
            action_type: ActionType::TakeCover,
        },
        GoapAction {
            name: "retreat",
            cost: 1.5,
            preconditions: world_state![WorldKey::IsInjured => true, WorldKey::Outnumbered => true, WorldKey::IsRetreating => false],
            effects: world_state![WorldKey::AtSafeDistance => true, WorldKey::IsRetreating => true, WorldKey::IsAlert => false],
            action_type: ActionType::Retreat { retreat_point: Vec2::ZERO },
        },
        
        // Support
        GoapAction {
            name: "call_for_help",
            cost: 1.5,
            preconditions: world_state![WorldKey::HasTarget => true, WorldKey::BackupCalled => false, WorldKey::NearbyAlliesAvailable => true],
            effects: world_state![WorldKey::BackupCalled => true],
            action_type: ActionType::CallForHelp,
        },
        GoapAction {
            name: "reload",
            cost: 2.0,
            preconditions: world_state![WorldKey::HasWeapon => true, WorldKey::WeaponLoaded => false],
            effects: world_state![WorldKey::WeaponLoaded => true],
            action_type: ActionType::Reload,
        },
        GoapAction {
            name: "tactical_reload",
            cost: 1.5,
            preconditions: world_state![WorldKey::HasWeapon => true, WorldKey::WeaponLoaded => true, WorldKey::HasTarget => false],
            effects: world_state![WorldKey::WeaponLoaded => true],
            action_type: ActionType::Reload,
        },
        
        // Advanced Tactics
        GoapAction {
            name: "use_medkit",
            cost: 2.5,
            preconditions: world_state![WorldKey::IsInjured => true, WorldKey::HasMedKit => true, WorldKey::InCover => true],
            effects: world_state![WorldKey::IsInjured => false, WorldKey::HasMedKit => false],
            action_type: ActionType::UseMedKit,
        },
        GoapAction {
            name: "throw_grenade",
            cost: 3.0,
            preconditions: world_state![WorldKey::HasGrenade => true, WorldKey::TargetGrouped => true, WorldKey::SafeThrowDistance => true],
            effects: world_state![WorldKey::HasGrenade => false, WorldKey::TargetGrouped => false],
            action_type: ActionType::ThrowGrenade { target_pos: Vec2::ZERO },
        },
        GoapAction {
            name: "activate_alarm",
            cost: 2.0,
            preconditions: world_state![WorldKey::HasTarget => true, WorldKey::NearAlarmPanel => true, WorldKey::FacilityAlert => false],
            effects: world_state![WorldKey::FacilityAlert => true, WorldKey::AllEnemiesAlerted => true, WorldKey::BackupCalled => true],
            action_type: ActionType::ActivateAlarm { panel_pos: Vec2::ZERO },
        },
        
        // Tactical Movement
        GoapAction {
            name: "find_better_cover",
            cost: 2.0,
            preconditions: world_state![WorldKey::InCover => true, WorldKey::UnderFire => true, WorldKey::BetterCoverAvailable => true],
            effects: world_state![WorldKey::InBetterCover => true, WorldKey::SafetyImproved => true, WorldKey::UnderFire => false],
            action_type: ActionType::FindBetterCover { new_cover_pos: Vec2::ZERO },
        },
        GoapAction {
            name: "suppressing_fire",
            cost: 1.5,
            preconditions: world_state![WorldKey::HasTarget => true, WorldKey::HasWeapon => true, WorldKey::AlliesAdvancing => true],
            effects: world_state![WorldKey::EnemySuppressed => true, WorldKey::AlliesAdvantage => true],
            action_type: ActionType::SuppressingFire { target_area: Vec2::ZERO },
        },
        GoapAction {
            name: "fighting_withdrawal",
            cost: 2.5,
            preconditions: world_state![WorldKey::Outnumbered => true, WorldKey::IsInjured => true, WorldKey::RetreatPathClear => true],
            effects: world_state![WorldKey::SafelyWithdrawing => true, WorldKey::TacticalRetreat => true, WorldKey::AtSafeDistance => true],
            action_type: ActionType::FightingWithdrawal { retreat_path: Vec2::ZERO },
        },
        
        // Weapon-Specific
        GoapAction {
            name: "pickup_better_weapon",
            cost: 1.0,
            preconditions: world_state![WorldKey::HasBetterWeapon => true, WorldKey::IsPanicked => false],
            effects: world_state![WorldKey::HasBetterWeapon => false],
            action_type: ActionType::MoveTo { target: Vec2::ZERO },
        },
        GoapAction {
            name: "panic_flee",
            cost: 0.5,
            preconditions: world_state![WorldKey::IsPanicked => true],
            effects: world_state![WorldKey::AtSafeDistance => true],
            action_type: ActionType::Retreat { retreat_point: Vec2::ZERO },
        },
        GoapAction {
            name: "maintain_weapon_range",
            cost: 1.5,
            preconditions: world_state![WorldKey::HasTarget => true, WorldKey::TooClose => true],
            effects: world_state![WorldKey::InWeaponRange => true, WorldKey::TooClose => false],
            action_type: ActionType::MaintainDistance,
        },
        GoapAction {
            name: "close_distance",
            cost: 2.0,
            preconditions: world_state![WorldKey::HasTarget => true, WorldKey::TooFar => true],
            effects: world_state![WorldKey::InWeaponRange => true, WorldKey::TooFar => false],
            action_type: ActionType::MoveTo { target: Vec2::ZERO },
        },
        GoapAction {
            name: "flamethrower_area_denial",
            cost: 2.0,
            preconditions: world_state![WorldKey::HasTarget => true, WorldKey::AgentsGroupedInRange => true, WorldKey::InWeaponRange => true],
            effects: world_state![WorldKey::ControllingArea => true, WorldKey::TacticalAdvantage => true],
            action_type: ActionType::Attack { target: Entity::PLACEHOLDER },
        },
        GoapAction {
            name: "minigun_suppression",
            cost: 1.5,
            preconditions: world_state![WorldKey::HasTarget => true, WorldKey::InWeaponRange => true, WorldKey::InCover => true],
            effects: world_state![WorldKey::SuppressingTarget => true, WorldKey::EnemySuppressed => true],
            action_type: ActionType::Attack { target: Entity::PLACEHOLDER },
        },
    ]
}

// === GOAL LIBRARY ===
fn create_goal_library() -> Vec<Goal> {
    vec![
        Goal {
            name: "panic_survival",
            priority: 15.0,
            desired_state: world_state![WorldKey::IsPanicked => false, WorldKey::AtSafeDistance => true],
        },
        Goal {
            name: "survival",
            priority: 12.0,
            desired_state: world_state![WorldKey::AtSafeDistance => true, WorldKey::IsInjured => false],
        },
        Goal {
            name: "eliminate_threat",
            priority: 10.0,
            desired_state: world_state![WorldKey::HasTarget => false],
        },
        Goal {
            name: "coordinate_defense",
            priority: 9.0,
            desired_state: world_state![WorldKey::FacilityAlert => true, WorldKey::AllEnemiesAlerted => true],
        },
        Goal {
            name: "tactical_advantage",
            priority: 8.0,
            desired_state: world_state![WorldKey::TacticalAdvantage => true, WorldKey::FlankingPosition => true],
        },
        Goal {
            name: "area_control",
            priority: 7.0,
            desired_state: world_state![WorldKey::ControllingArea => true, WorldKey::SuppressingTarget => true],
        },
        Goal {
            name: "thorough_search",
            priority: 6.0,
            desired_state: world_state![WorldKey::AreaSearched => true, WorldKey::HeardSound => false],
        },
        Goal {
            name: "investigate_disturbance",
            priority: 5.0,
            desired_state: world_state![WorldKey::HeardSound => false],
        },
        Goal {
            name: "weapon_upgrade",
            priority: 4.0,
            desired_state: world_state![WorldKey::HasBetterWeapon => false],
        },
        Goal {
            name: "patrol_area",
            priority: 1.0,
            desired_state: world_state![WorldKey::IsAlert => false],
        },
    ]
}

// === INTEGRATION COMPONENTS ===
#[derive(Component)]
pub struct CoverPoint {
    pub capacity: u8,
    pub current_users: u8,
    pub cover_direction: Vec2,
}

#[derive(Component)]
pub struct InCover {
    pub cover_entity: Entity,
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
            grenades: 0,
            tools: Vec::new(),
        }
    }
}

// === DEBUG CONFIGURATION ===
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
        Option<&WeaponState>
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
            weapon_state,
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

fn update_world_state_from_perception (
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
    weapon_state: Option<&WeaponState>,    
) {
    let enemy_pos = enemy_transform.translation.truncate();
    
    update_vision_direction(goap_agent, ai_state, patrol, vision, enemy_pos, agent_query);
    
    let (visible_agent, has_target) = update_basic_perception(
        enemy_transform, 
        vision, 
        agent_query, 
        ai_state
    );
    
    let tactical_state = assess_tactical_situation(
        enemy_pos,
        patrol,
        cover_query,
        all_enemy_query,
        current_enemy,
        health,
        agent_query,
        visible_agent
    );
    
    update_weapon_state(goap_agent, weapon_state);
    update_world_states(goap_agent, &tactical_state, has_target, visible_agent);
    update_ai_mode(goap_agent, ai_state, has_target, visible_agent);
}

fn update_vision_direction(
    goap_agent: &mut GoapAgent,
    ai_state: &AIState,
    patrol: &Patrol,
    vision: &mut Vision,
    enemy_pos: Vec2,
    agent_query: &Query<(Entity, &Transform), With<Agent>>,
) {
    match &ai_state.mode {
        AIMode::Patrol => {
            if let Some(target) = patrol.current_target() {
                let direction = target - enemy_pos;
                update_direction_if_valid(vision, direction);
            }
        },

        AIMode::Combat { target } => {
            if let Ok((_, target_transform)) = agent_query.get(*target) {
                let direction = target_transform.translation.truncate() - enemy_pos;
                update_direction_if_valid(vision, direction);
            }
        },

        AIMode::Investigate { location } => {
            let direction = *location - enemy_pos;
            update_direction_if_valid(vision, direction);
        },

        AIMode::Search { area } => {
            let direction = *area - enemy_pos;
            update_direction_if_valid(vision, direction);
        },

        AIMode::Panic => {
            apply_panic_state(goap_agent);
        },        
    }
}

// Helper function for panic state
fn apply_panic_state(goap_agent: &mut GoapAgent) {
    goap_agent.update_multiple([
        (WorldKey::IsAlert, true),
        (WorldKey::IsInvestigating, false),
        (WorldKey::IsPanicked, true),
        (WorldKey::IsRetreating, true),
    ]);
}

fn update_direction_if_valid(vision: &mut Vision, delta: Vec2) {
    let direction = delta.normalize_or_zero();
    if direction != Vec2::ZERO {
        vision.direction = direction;
    }
}

fn update_basic_perception(
    enemy_transform: &Transform,
    vision: &mut Vision,
    agent_query: &Query<(Entity, &Transform), With<Agent>>,
    ai_state: &mut AIState,
) -> (Option<Entity>, bool) {
    let visible_agent = check_line_of_sight_goap(enemy_transform, vision, agent_query);
    let has_target = visible_agent.is_some();
    
    if let Some(agent_entity) = visible_agent {
        if let Ok((_, agent_transform)) = agent_query.get(agent_entity) {
            ai_state.last_known_target = Some(agent_transform.translation.truncate());
        }
    }
    
    (visible_agent, has_target)
}

struct TacticalState {
    at_patrol_point: bool,
    cover_available: bool,
    nearby_allies: bool,
    is_injured: bool,
    outnumbered: bool,
    at_safe_distance: bool,
    target_grouped: bool,
    safe_throw_distance: bool,
    has_medkit: bool,
    has_grenade: bool,
    under_fire: bool,
    better_cover_available: bool,
    allies_advancing: bool,
    retreat_path_clear: bool,
}

fn assess_tactical_situation(
    enemy_pos: Vec2,
    patrol: &Patrol,
    cover_query: &Query<(Entity, &Transform, &CoverPoint), Without<Enemy>>,
    all_enemy_query: &Query<(Entity, &Transform), (With<Enemy>, Without<Dead>)>,
    current_enemy: Entity,
    health: &Health,
    agent_query: &Query<(Entity, &Transform), With<Agent>>,
    visible_agent: Option<Entity>,
) -> TacticalState {
    let at_patrol_point = is_at_patrol_point(enemy_pos, patrol);
    let cover_available = find_cover(enemy_pos, cover_query, None, false).is_some();
    let nearby_allies = has_nearby_allies(enemy_pos, all_enemy_query, current_enemy);
    let is_injured = health.0 < 50.0;
    
    let (agent_count, nearby_enemy_count) = count_entities_in_range(
        enemy_pos, 
        agent_query, 
        all_enemy_query, 
        current_enemy
    );
    
    let outnumbered = agent_count > nearby_enemy_count + 1;
    let at_safe_distance = !is_any_agent_in_range(enemy_pos, agent_query, 150.0);
    
    let (target_grouped, safe_throw_distance) = assess_group_targets(
        enemy_pos, 
        agent_query,
        300.0,
        80.0,
        (100.0, 250.0)
    );
    
    TacticalState {
        at_patrol_point,
        cover_available,
        nearby_allies,
        is_injured,
        outnumbered,
        at_safe_distance,
        target_grouped,
        safe_throw_distance,
        has_medkit: is_injured && rand::random::<f32>() < 0.3,
        has_grenade: target_grouped && rand::random::<f32>() < 0.2,
        under_fire: agent_count > 0 && is_any_agent_in_range(enemy_pos, agent_query, 120.0),
        better_cover_available: cover_available && cover_query.iter().count() > 1,
        allies_advancing: nearby_enemy_count > 0 && agent_count > 0,
        retreat_path_clear: check_retreat_path(enemy_pos, patrol, agent_query),
    }
}

fn is_at_patrol_point(enemy_pos: Vec2, patrol: &Patrol) -> bool {
    patrol.current_target()
        .map(|target| enemy_pos.distance(target) < 20.0)
        .unwrap_or(true)
}

fn has_nearby_allies(
    enemy_pos: Vec2,
    all_enemy_query: &Query<(Entity, &Transform), (With<Enemy>, Without<Dead>)>,
    current_enemy: Entity,
) -> bool {
    all_enemy_query.iter()
        .filter(|(entity, transform)| {
            *entity != current_enemy &&
            enemy_pos.distance(transform.translation.truncate()) <= 200.0
        })
        .count() > 0
}

fn count_entities_in_range(
    enemy_pos: Vec2,
    agent_query: &Query<(Entity, &Transform), With<Agent>>,
    all_enemy_query: &Query<(Entity, &Transform), (With<Enemy>, Without<Dead>)>,
    current_enemy: Entity,
) -> (usize, usize) {
    let agent_count = agent_query.iter()
        .filter(|(_, transform)| enemy_pos.distance(transform.translation.truncate()) <= 200.0)
        .count();
        
    let enemy_count = all_enemy_query.iter()
        .filter(|(entity, transform)| {
            *entity != current_enemy &&
            enemy_pos.distance(transform.translation.truncate()) <= 200.0
        })
        .count();
        
    (agent_count, enemy_count)
}

fn update_weapon_state(
    goap_agent: &mut GoapAgent,
    weapon_state: Option<&WeaponState>,
) {
    match weapon_state {
        Some(weapon) => {
            goap_agent.update_multiple([
                (WorldKey::WeaponLoaded, weapon.current_ammo > 0),
                (WorldKey::HasWeapon, true),
                // (WorldKey::NeedsReload, weapon.needs_reload()), // Uncomment if needed
            ]);
        }
        None => {
            goap_agent.update_multiple([
                (WorldKey::WeaponLoaded, true),
                (WorldKey::HasWeapon, true),
            ]);
        }
    }
}

fn update_world_states(
    goap_agent: &mut GoapAgent,
    tactical_state: &TacticalState,
    has_target: bool,
    visible_agent: Option<Entity>,
) {
    // Core perception states
    goap_agent.update_multiple([
        (WorldKey::TargetVisible, has_target),
        (WorldKey::HasTarget, has_target),
        (WorldKey::AtPatrolPoint, tactical_state.at_patrol_point),
        (WorldKey::CoverAvailable, tactical_state.cover_available),
        (WorldKey::NearbyAlliesAvailable, tactical_state.nearby_allies),
    ]);

    // Tactical assessment states
    goap_agent.update_multiple([
        (WorldKey::IsInjured, tactical_state.is_injured),
        (WorldKey::Outnumbered, tactical_state.outnumbered),
        (WorldKey::AtSafeDistance, tactical_state.at_safe_distance),
        (WorldKey::TargetGrouped, tactical_state.target_grouped),
        (WorldKey::SafeThrowDistance, tactical_state.safe_throw_distance),
        (WorldKey::UnderFire, tactical_state.under_fire),
        (WorldKey::BetterCoverAvailable, tactical_state.better_cover_available),
        (WorldKey::AlliesAdvancing, tactical_state.allies_advancing),
        (WorldKey::RetreatPathClear, tactical_state.retreat_path_clear),
    ]);

    // Inventory states
    goap_agent.update_multiple([
        (WorldKey::HasMedKit, tactical_state.has_medkit),
        (WorldKey::HasGrenade, tactical_state.has_grenade),
    ]);

    // Reset tactical positions if no target
    if !has_target {
        goap_agent.update_multiple([
            (WorldKey::FlankingPosition, false),
            (WorldKey::TacticalAdvantage, false),
        ]);
    }
}

fn update_ai_mode(
    goap_agent: &mut GoapAgent,
    ai_state: &mut AIState,
    has_target: bool,
    visible_agent: Option<Entity>,
) {
    match &ai_state.mode {
        AIMode::Patrol => {
            goap_agent.update_multiple([
                (WorldKey::IsAlert, false),
                (WorldKey::IsInvestigating, false),
                (WorldKey::IsRetreating, false),
            ]);

            if has_target {
                if let Some(target) = visible_agent {
                    ai_state.mode = AIMode::Combat { target };
                    goap_agent.update_world_state(WorldKey::IsAlert, true);
                    goap_agent.abort_plan();
                }
            }
        }
        AIMode::Combat { .. } => {
            goap_agent.update_multiple([
                (WorldKey::IsAlert, true),
                (WorldKey::IsInvestigating, false),
            ]);
        }
        AIMode::Investigate { .. } | AIMode::Search { .. } => {
            goap_agent.update_multiple([
                (WorldKey::IsAlert, true),
                (WorldKey::IsInvestigating, true),
            ]);
        }
        AIMode::Panic => {
            goap_agent.update_multiple([
                (WorldKey::IsAlert, true),
                (WorldKey::IsInvestigating, false),
                (WorldKey::IsPanicked, true),
                (WorldKey::IsRetreating, true),
            ]);
        }
    }
}

fn assess_group_targets(
    enemy_pos: Vec2,
    agent_query: &Query<(Entity, &Transform), With<Agent>>,
    detection_range: f32,
    group_proximity: f32,
    throw_range: (f32, f32),
) -> (bool, bool) {
    let agents_in_range: Vec<_> = agent_query.iter()
        .filter(|(_, transform)| {
            enemy_pos.distance(transform.translation.truncate()) <= detection_range
        })
        .collect();

    // Check if agents are grouped together
    let target_grouped = if agents_in_range.len() >= 2 {
        let positions: Vec<Vec2> = agents_in_range.iter()
            .map(|(_, t)| t.translation.truncate())
            .collect();
        
        positions.iter().any(|&pos1| {
            positions.iter()
                .filter(|&&pos2| pos1.distance(pos2) <= group_proximity)
                .count() >= 2
        })
    } else {
        false
    };

    // Check safe throw distance
    let safe_throw_distance = agents_in_range.first()
        .map(|(_, transform)| {
            let distance = enemy_pos.distance(transform.translation.truncate());
            distance >= throw_range.0 && distance <= throw_range.1
        })
        .unwrap_or(false);

    (target_grouped, safe_throw_distance)
}

fn is_any_agent_in_range(
    enemy_pos: Vec2,
    agent_query: &Query<(Entity, &Transform), With<Agent>>,
    range: f32,
) -> bool {
    agent_query.iter()
        .any(|(_, transform)| {
            enemy_pos.distance(transform.translation.truncate()) <= range
        })
}

fn check_retreat_path(
    enemy_pos: Vec2,
    patrol: &Patrol,
    agent_query: &Query<(Entity, &Transform), With<Agent>>,
) -> bool {
    patrol.current_target()
        .map(|patrol_point| {
            let to_patrol = (patrol_point - enemy_pos).normalize_or_zero();
            !agent_query.iter().any(|(_, agent_transform)| {
                let agent_pos = agent_transform.translation.truncate();
                let to_agent = (agent_pos - enemy_pos).normalize_or_zero();
                to_patrol.dot(to_agent) > 0.7 // Agent is in retreat direction
            })
        })
        .unwrap_or(true) // If no patrol point, assume path is clear
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
            if let Some((cover_entity, cover_pos)) = find_cover(enemy_transform.translation.truncate(), cover_query, None, false) {
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
                alert_level: 1,  // Add missing field
                source: AlertSource::Gunshot,  // Add missing field
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
                alert_level: 2,  // Add missing field  
                source: AlertSource::Alarm,  // Add missing field
                alert_type: AlertType::EnemySpotted, // Reuse existing type
            });
            
            info!("Enemy {} activated facility alarm!", enemy_entity.index());
        },
        
        // === NEW: TACTICAL MOVEMENT EXECUTION ===
        ActionType::FindBetterCover { new_cover_pos: _ } => {
            // Find the best available cover point (furthest from agents)
            if let Some((cover_entity, cover_pos)) = find_cover(
                enemy_transform.translation.truncate(), 
                cover_query, 
                Some(&agent_query),
                true
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

fn find_cover(
    enemy_pos: Vec2,
    cover_q: &Query<(Entity, &Transform, &CoverPoint), Without<Enemy>>,
    agent_q: Option<&Query<(Entity, &Transform), With<Agent>>>,
    use_score: bool,
) -> Option<(Entity, Vec2)> {
    let (mut best, mut val) = (None, if use_score { f32::MIN } else { f32::MAX });
    for (e, t, c) in cover_q.iter() {
        if c.current_users >= c.capacity { continue; }
        let p = t.translation.truncate();
        let d = enemy_pos.distance(p);
        if use_score {
            if d > 150.0 { continue; }
            let mut s = 100.0 - d;
            if let Some(aq) = agent_q {
                for (_, at) in aq.iter() { s += 0.5 * p.distance(at.translation.truncate()); }
            }
            if s > val { val = s; best = Some((e, p)); }
        } else if d < val { val = d; best = Some((e, p)); }
    }
    best
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
