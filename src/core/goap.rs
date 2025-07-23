// src/core/goap.rs
use bevy::prelude::*;
use std::collections::{HashMap, VecDeque};
use crate::systems::ai::AIMode;
use crate::core::factions::Faction;

macro_rules! world_state {
    ( $( $key:expr => $value:expr ),* $(,)? ) => {{
        let mut map = HashMap::new();
        $( map.insert($key, $value); )*
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
    Patrol, MoveTo { target: Vec2 }, Attack { target: Entity },
    Investigate { location: Vec2 }, Search { area: Vec2 }, Reload,
    CallForHelp, TakeCover, FlankTarget { target_pos: Vec2, flank_pos: Vec2 },
    SearchArea { center: Vec2, radius: f32 }, Retreat { retreat_point: Vec2 },
    UseMedKit, ThrowGrenade { target_pos: Vec2 }, ActivateAlarm { panel_pos: Vec2 },
    FindBetterCover { new_cover_pos: Vec2 }, SuppressingFire { target_area: Vec2 },
    FightingWithdrawal { retreat_path: Vec2 }, MaintainDistance,
}

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
            available_actions: create_action_library(),
            goals: create_goal_library(),
            planning_cooldown: 0.0,
        };
        agent.setup_initial_world_state();
        agent
    }
}

impl GoapAgent {
    fn setup_initial_world_state(&mut self) {
        self.world_state = world_state![
            WorldKey::HasWeapon => true, WorldKey::WeaponLoaded => true, WorldKey::IsAlert => false,
            WorldKey::HasTarget => false, WorldKey::TargetVisible => false, WorldKey::HeardSound => false,
            WorldKey::AtPatrolPoint => true, WorldKey::AtLastKnownPosition => false, WorldKey::AtTarget => false,
            WorldKey::IsInvestigating => false, WorldKey::InCover => false, WorldKey::CoverAvailable => false,
            WorldKey::UnderFire => false, WorldKey::BackupCalled => false, WorldKey::NearbyAlliesAvailable => false,
            WorldKey::FlankingPosition => false, WorldKey::TacticalAdvantage => false, WorldKey::AreaSearched => false,
            WorldKey::IsRetreating => false, WorldKey::AtSafeDistance => false, WorldKey::Outnumbered => false,
            WorldKey::IsInjured => false, WorldKey::HasMedKit => false, WorldKey::HasGrenade => false,
            WorldKey::TargetGrouped => false, WorldKey::SafeThrowDistance => false, WorldKey::NearAlarmPanel => false,
            WorldKey::FacilityAlert => false, WorldKey::AllEnemiesAlerted => false, WorldKey::BetterCoverAvailable => false,
            WorldKey::InBetterCover => false, WorldKey::SafetyImproved => false, WorldKey::AlliesAdvancing => false,
            WorldKey::EnemySuppressed => false, WorldKey::AlliesAdvantage => false, WorldKey::RetreatPathClear => false,
            WorldKey::SafelyWithdrawing => false, WorldKey::TacticalRetreat => false, WorldKey::IsPanicked => false,
            WorldKey::HasBetterWeapon => false, WorldKey::InWeaponRange => false, WorldKey::TooClose => false,
            WorldKey::TooFar => false, WorldKey::ControllingArea => false, WorldKey::SuppressingTarget => false,
            WorldKey::AgentsGroupedInRange => false,
        ];
    }
    
    pub fn update_world_state(&mut self, key: WorldKey, value: bool) {
        self.world_state.insert(key, value);
    }

    pub fn update_multiple(&mut self, updates: impl IntoIterator<Item = (WorldKey, bool)>) {
        for (key, value) in updates {
            self.update_world_state(key, value);
        }
    }    

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
        
        for _ in 0..10 {
            if remaining_goals.is_empty() { break; }
            
            if let Some(action) = self.find_satisfying_action(&remaining_goals) {
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
    
    fn find_satisfying_action(&self, goals: &WorldState) -> Option<GoapAction> {
        self.available_actions.iter()
            .find(|action| action.effects.iter().any(|(key, &value)| goals.get(key) == Some(&value)))
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

// === EXTERNAL DATA ===
fn create_action_library() -> Vec<GoapAction> {
    include!("../data/goap_actions.rs")
}

fn create_goal_library() -> Vec<Goal> {
    include!("../data/goap_goals.rs")
}

// === INTEGRATION COMPONENTS ===
#[derive(Component)]
pub struct CoverPoint {
    pub capacity: u8,
    pub current_users: u8,
    pub cover_direction: Vec2,
}

#[derive(Component)]
pub struct InCover { pub cover_entity: Entity }

#[derive(Component)]
pub struct AlarmPanel { pub activated: bool, pub range: f32 }

#[derive(Component, Default)]
pub struct Equipment { pub medkits: u8, pub grenades: u8, pub tools: Vec<String> }

// === MAIN AI SYSTEM ===
use crate::core::*;
use crate::systems::ai::AIState;

pub fn goap_ai_system(
    mut commands: Commands,
    mut enemy_query: Query<(Entity, &Transform, &mut AIState, &mut GoapAgent, &mut Vision,
        &Patrol, &Health, &Faction, Option<&WeaponState>), (With<Enemy>, Without<Dead>)>,
    agent_query: Query<(Entity, &Transform), With<Agent>>,
    all_enemy_query: Query<(Entity, &Transform, &Faction), (With<Enemy>, Without<Dead>)>,
    cover_query: Query<(Entity, &Transform, &CoverPoint), Without<Enemy>>,
    mut action_events: EventWriter<ActionEvent>,
    mut audio_events: EventWriter<AudioEvent>,
    mut alert_events: EventWriter<AlertEvent>,
    time: Res<Time>,
    game_mode: Res<GameMode>,
) {
    if game_mode.paused { return; }

    for (enemy_entity, enemy_transform, mut ai_state, mut goap_agent, mut vision, patrol, health, faction, weapon_state) in enemy_query.iter_mut() {
        goap_agent.planning_cooldown -= time.delta_secs();
        
        update_world_state_from_perception(&mut goap_agent, enemy_transform, &mut vision, faction,
            enemy_entity, &agent_query, &all_enemy_query, &mut ai_state, patrol, &cover_query, health, weapon_state);
        
        let should_replan = goap_agent.current_plan.is_empty() || goap_agent.planning_cooldown <= 0.0 ||
                          plan_invalidated(&goap_agent, &ai_state, health);
        
        if should_replan {
            goap_agent.plan();
            goap_agent.planning_cooldown = match (health.0 < 30.0, *goap_agent.world_state.get(&WorldKey::HasTarget).unwrap_or(&false)) {
                (true, _) => 0.3,
                (_, true) => 0.5,
                _ => 2.0,
            };
        }
        
        if let Some(action) = goap_agent.get_next_action() {
            execute_goap_action(&action, enemy_entity, enemy_transform, &mut ai_state,
                &mut action_events, &mut audio_events, &mut alert_events, patrol,
                &agent_query, &all_enemy_query, &vision, &cover_query, &mut commands);
        }
    }
}

fn update_world_state_from_perception(
    goap_agent: &mut GoapAgent, enemy_transform: &Transform, vision: &mut Vision, faction: &Faction,
    current_entity: Entity, agent_query: &Query<(Entity, &Transform), With<Agent>>,
    enemy_query: &Query<(Entity, &Transform, &Faction), (With<Enemy>, Without<Dead>)>,
    ai_state: &mut AIState, patrol: &Patrol, cover_query: &Query<(Entity, &Transform, &CoverPoint), Without<Enemy>>,
    health: &Health, weapon_state: Option<&WeaponState>,    
) {
    let enemy_pos = enemy_transform.translation.truncate();
    
    update_vision_direction(goap_agent, ai_state, patrol, vision, enemy_pos, current_entity, agent_query, enemy_query);
    
    let visible_hostile = check_line_of_sight_goap(enemy_transform, vision, faction, current_entity, agent_query, enemy_query);
    let has_target = visible_hostile.is_some();
    
    if let Some(target_entity) = visible_hostile {
        if let Some(pos) = get_entity_position(target_entity, agent_query, enemy_query) {
            ai_state.last_known_target = Some(pos);
        }
    }
    
    let enemy_positions: Vec<(Entity, Vec2)> = enemy_query.iter()
        .map(|(entity, transform, _)| (entity, transform.translation.truncate()))
        .collect();
    
    let tactical_state = assess_tactical_situation(enemy_pos, patrol, cover_query, &enemy_positions,
        current_entity, health, agent_query, visible_hostile);
    
    update_weapon_state(goap_agent, weapon_state);
    update_world_states(goap_agent, &tactical_state, has_target, visible_hostile);
    update_ai_mode(goap_agent, ai_state, has_target, visible_hostile);
}

struct TacticalState {
    at_patrol_point: bool, cover_available: bool, nearby_allies: bool, is_injured: bool,
    outnumbered: bool, at_safe_distance: bool, target_grouped: bool, safe_throw_distance: bool,
    has_medkit: bool, has_grenade: bool, under_fire: bool, better_cover_available: bool,
    allies_advancing: bool, retreat_path_clear: bool,
}

fn assess_tactical_situation(enemy_pos: Vec2, patrol: &Patrol, cover_query: &Query<(Entity, &Transform, &CoverPoint), Without<Enemy>>,
    enemy_positions: &[(Entity, Vec2)], current_enemy: Entity, health: &Health,
    agent_query: &Query<(Entity, &Transform), With<Agent>>, visible_hostile: Option<Entity>) -> TacticalState {
    
    let at_patrol_point = patrol.current_target().map(|t| enemy_pos.distance(t) < 20.0).unwrap_or(true);
    let cover_available = find_cover(enemy_pos, cover_query, None, false).is_some();
    let nearby_allies = enemy_positions.iter().filter(|(e, p)| *e != current_enemy && enemy_pos.distance(*p) <= 200.0).count() > 0;
    let is_injured = health.0 < 50.0;
    
    let (agent_count, enemy_count) = count_entities_in_range(enemy_pos, agent_query, enemy_positions, current_enemy);
    let outnumbered = agent_count > enemy_count + 1;
    let at_safe_distance = !agent_query.iter().any(|(_, t)| enemy_pos.distance(t.translation.truncate()) <= 150.0);
    
    let (target_grouped, safe_throw_distance) = assess_group_targets(enemy_pos, agent_query, 300.0, 80.0, (100.0, 250.0));
    
    TacticalState {
        at_patrol_point, cover_available, nearby_allies, is_injured, outnumbered, at_safe_distance,
        target_grouped, safe_throw_distance,
        has_medkit: is_injured && rand::random::<f32>() < 0.3,
        has_grenade: target_grouped && rand::random::<f32>() < 0.2,
        under_fire: agent_count > 0 && agent_query.iter().any(|(_, t)| enemy_pos.distance(t.translation.truncate()) <= 120.0),
        better_cover_available: cover_available && cover_query.iter().count() > 1,
        allies_advancing: enemy_count > 0 && agent_count > 0,
        retreat_path_clear: check_retreat_path(enemy_pos, patrol, agent_query),
    }
}

fn count_entities_in_range(enemy_pos: Vec2, agent_query: &Query<(Entity, &Transform), With<Agent>>,
    enemy_positions: &[(Entity, Vec2)], current_enemy: Entity) -> (usize, usize) {
    let agent_count = agent_query.iter().filter(|(_, t)| enemy_pos.distance(t.translation.truncate()) <= 200.0).count();
    let enemy_count = enemy_positions.iter().filter(|(e, p)| *e != current_enemy && enemy_pos.distance(*p) <= 200.0).count();
    (agent_count, enemy_count)
}

fn update_vision_direction(goap_agent: &mut GoapAgent, ai_state: &AIState, patrol: &Patrol, vision: &mut Vision,
    enemy_pos: Vec2, current_entity: Entity, agent_query: &Query<(Entity, &Transform), With<Agent>>,
    enemy_query: &Query<(Entity, &Transform, &Faction), (With<Enemy>, Without<Dead>)>) {
    
    let direction = match &ai_state.mode {
        AIMode::Patrol => patrol.current_target().map(|t| t - enemy_pos),
        AIMode::Combat { target } => get_entity_position(*target, agent_query, enemy_query).map(|p| p - enemy_pos),
        AIMode::Investigate { location } | AIMode::Search { area: location } => Some(*location - enemy_pos),
        AIMode::Panic => { goap_agent.update_multiple([(WorldKey::IsPanicked, true), (WorldKey::IsRetreating, true)]); None },
    };
    
    if let Some(dir) = direction {
        let normalized = dir.normalize_or_zero();
        if normalized != Vec2::ZERO { vision.direction = normalized; }
    }
}

fn update_weapon_state(goap_agent: &mut GoapAgent, weapon_state: Option<&WeaponState>) {
    let (loaded, has_weapon) = weapon_state.map_or((true, true), |w| (w.current_ammo > 0, true));
    goap_agent.update_multiple([(WorldKey::WeaponLoaded, loaded), (WorldKey::HasWeapon, has_weapon)]);
}

fn update_world_states(goap_agent: &mut GoapAgent, tactical_state: &TacticalState, has_target: bool, visible_agent: Option<Entity>) {
    goap_agent.update_multiple([
        (WorldKey::TargetVisible, has_target), (WorldKey::HasTarget, has_target),
        (WorldKey::AtPatrolPoint, tactical_state.at_patrol_point), (WorldKey::CoverAvailable, tactical_state.cover_available),
        (WorldKey::NearbyAlliesAvailable, tactical_state.nearby_allies), (WorldKey::IsInjured, tactical_state.is_injured),
        (WorldKey::Outnumbered, tactical_state.outnumbered), (WorldKey::AtSafeDistance, tactical_state.at_safe_distance),
        (WorldKey::TargetGrouped, tactical_state.target_grouped), (WorldKey::SafeThrowDistance, tactical_state.safe_throw_distance),
        (WorldKey::UnderFire, tactical_state.under_fire), (WorldKey::BetterCoverAvailable, tactical_state.better_cover_available),
        (WorldKey::AlliesAdvancing, tactical_state.allies_advancing), (WorldKey::RetreatPathClear, tactical_state.retreat_path_clear),
        (WorldKey::HasMedKit, tactical_state.has_medkit), (WorldKey::HasGrenade, tactical_state.has_grenade),
    ]);

    if !has_target {
        goap_agent.update_multiple([(WorldKey::FlankingPosition, false), (WorldKey::TacticalAdvantage, false)]);
    }
}

fn update_ai_mode(goap_agent: &mut GoapAgent, ai_state: &mut AIState, has_target: bool, visible_agent: Option<Entity>) {
    match &ai_state.mode {
        AIMode::Patrol => {
            goap_agent.update_multiple([(WorldKey::IsAlert, false), (WorldKey::IsInvestigating, false), (WorldKey::IsRetreating, false)]);
            if has_target {
                if let Some(target) = visible_agent {
                    ai_state.mode = AIMode::Combat { target };
                    goap_agent.update_world_state(WorldKey::IsAlert, true);
                    goap_agent.abort_plan();
                }
            }
        }
        AIMode::Combat { .. } => goap_agent.update_multiple([(WorldKey::IsAlert, true), (WorldKey::IsInvestigating, false)]),
        AIMode::Investigate { .. } | AIMode::Search { .. } => goap_agent.update_multiple([(WorldKey::IsAlert, true), (WorldKey::IsInvestigating, true)]),
        AIMode::Panic => goap_agent.update_multiple([(WorldKey::IsAlert, true), (WorldKey::IsPanicked, true), (WorldKey::IsRetreating, true)]),
    }
}

fn assess_group_targets(enemy_pos: Vec2, agent_query: &Query<(Entity, &Transform), With<Agent>>,
    detection_range: f32, group_proximity: f32, throw_range: (f32, f32)) -> (bool, bool) {
    
    let agents_in_range: Vec<_> = agent_query.iter()
        .filter(|(_, t)| enemy_pos.distance(t.translation.truncate()) <= detection_range).collect();

    let target_grouped = if agents_in_range.len() >= 2 {
        let positions: Vec<Vec2> = agents_in_range.iter().map(|(_, t)| t.translation.truncate()).collect();
        positions.iter().any(|&pos1| positions.iter().filter(|&&pos2| pos1.distance(pos2) <= group_proximity).count() >= 2)
    } else { false };

    let safe_throw_distance = agents_in_range.first().map(|(_, t)| {
        let distance = enemy_pos.distance(t.translation.truncate());
        distance >= throw_range.0 && distance <= throw_range.1
    }).unwrap_or(false);

    (target_grouped, safe_throw_distance)
}

fn check_retreat_path(enemy_pos: Vec2, patrol: &Patrol, agent_query: &Query<(Entity, &Transform), With<Agent>>) -> bool {
    patrol.current_target().map(|patrol_point| {
        let to_patrol = (patrol_point - enemy_pos).normalize_or_zero();
        !agent_query.iter().any(|(_, agent_transform)| {
            let agent_pos = agent_transform.translation.truncate();
            let to_agent = (agent_pos - enemy_pos).normalize_or_zero();
            to_patrol.dot(to_agent) > 0.7
        })
    }).unwrap_or(true)
}

fn plan_invalidated(goap_agent: &GoapAgent, ai_state: &AIState, health: &Health) -> bool {
    let critically_injured = health.0 < 30.0;
    let planning_survival = goap_agent.current_goal.as_ref().map(|g| g.name.contains("survival")).unwrap_or(false);
    
    if critically_injured && !planning_survival { return true; }
    
    let outnumbered = *goap_agent.world_state.get(&WorldKey::Outnumbered).unwrap_or(&false);
    let has_tactical_goal = goap_agent.current_goal.as_ref()
        .map(|g| ["tactical_advantage", "survival", "panic_survival"].contains(&g.name)).unwrap_or(false);
    
    if outnumbered && !has_tactical_goal { return true; }
    
    let is_panicked = *goap_agent.world_state.get(&WorldKey::IsPanicked).unwrap_or(&false);
    if is_panicked && !planning_survival { return true; }
    
    match &ai_state.mode {
        AIMode::Combat { .. } => !goap_agent.world_state.get(&WorldKey::HasTarget).unwrap_or(&false),
        AIMode::Panic => true,
        _ => false,
    }
}

fn check_line_of_sight_goap(enemy_transform: &Transform, vision: &Vision, faction: &Faction, current_entity: Entity,
    agent_query: &Query<(Entity, &Transform), With<Agent>>, enemy_query: &Query<(Entity, &Transform, &Faction), (With<Enemy>, Without<Dead>)>) -> Option<Entity> {
    
    let enemy_pos = enemy_transform.translation.truncate();
    
    for (agent_entity, agent_transform) in agent_query.iter() {
        if in_vision_cone(enemy_pos, agent_transform.translation.truncate(), vision) {
            return Some(agent_entity);
        }
    }
    
    for (other_entity, other_transform, other_faction) in enemy_query.iter() {
        if other_entity != current_entity && faction.is_hostile_to(other_faction) {
            if in_vision_cone(enemy_pos, other_transform.translation.truncate(), vision) {
                return Some(other_entity);
            }
        }
    }
    
    None
}

fn in_vision_cone(observer_pos: Vec2, target_pos: Vec2, vision: &Vision) -> bool {
    let to_target = target_pos - observer_pos;
    let distance = to_target.length();
    
    if distance <= vision.range && distance > 1.0 {
        let target_direction = to_target.normalize();
        let dot_product = vision.direction.dot(target_direction);
        let angle_cos = (vision.angle / 2.0).cos();
        dot_product >= angle_cos
    } else {
        false
    }
}

fn get_entity_position(entity: Entity, agent_query: &Query<(Entity, &Transform), With<Agent>>,
    enemy_query: &Query<(Entity, &Transform, &Faction), (With<Enemy>, Without<Dead>)>) -> Option<Vec2> {
    
    if let Ok((_, transform)) = agent_query.get(entity) {
        Some(transform.translation.truncate())
    } else if let Ok((_, transform, _)) = enemy_query.get(entity) {
        Some(transform.translation.truncate())
    } else {
        None
    }
}

fn execute_goap_action(action: &GoapAction, enemy_entity: Entity, enemy_transform: &Transform, ai_state: &mut AIState,
    action_events: &mut EventWriter<ActionEvent>, audio_events: &mut EventWriter<AudioEvent>, alert_events: &mut EventWriter<AlertEvent>,
    patrol: &Patrol, agent_query: &Query<(Entity, &Transform), With<Agent>>, all_enemy_query: &Query<(Entity, &Transform, &Faction), (With<Enemy>, Without<Dead>)>,
    vision: &Vision, cover_query: &Query<(Entity, &Transform, &CoverPoint), Without<Enemy>>, commands: &mut Commands) {
    
    match &action.action_type {
        ActionType::Patrol => {
            if let Some(target) = patrol.current_target() {
                ai_state.mode = AIMode::Patrol;
                action_events.write(ActionEvent { entity: enemy_entity, action: Action::MoveTo(target) });
            }
        },
        ActionType::MoveTo { target } => {
            action_events.write(ActionEvent { entity: enemy_entity, action: Action::MoveTo(*target) });
        },
        ActionType::Attack { .. } => {
            if let Some(target_entity) = find_any_hostile_target(enemy_transform, vision, agent_query, all_enemy_query) {
                if let Some(pos) = get_entity_position(target_entity, agent_query, all_enemy_query) {
                    let distance = enemy_transform.translation.truncate().distance(pos);
                    ai_state.mode = AIMode::Combat { target: target_entity };
                    
                    if distance <= 150.0 {
                        action_events.write(ActionEvent { entity: enemy_entity, action: Action::Attack(target_entity) });
                    } else {
                        action_events.write(ActionEvent { entity: enemy_entity, action: Action::MoveTo(pos) });
                    }
                }
            } else if let Some(last_pos) = ai_state.last_known_target {
                action_events.write(ActionEvent { entity: enemy_entity, action: Action::MoveTo(last_pos) });
                ai_state.mode = AIMode::Investigate { location: last_pos };
            }
        },
        ActionType::FlankTarget { .. } => {
            if let Some(agent_entity) = find_closest_agent(enemy_transform, agent_query) {
                if let Ok((_, agent_transform)) = agent_query.get(agent_entity) {
                    let agent_pos = agent_transform.translation.truncate();
                    let enemy_pos = enemy_transform.translation.truncate();
                    let to_agent = (agent_pos - enemy_pos).normalize_or_zero();
                    let flank_offset = Vec2::new(-to_agent.y, to_agent.x) * 80.0;
                    let flank_position = agent_pos + flank_offset;
                    
                    action_events.write(ActionEvent { entity: enemy_entity, action: Action::MoveTo(flank_position) });
                    ai_state.mode = AIMode::Combat { target: agent_entity };
                }
            }
        },
        ActionType::Investigate { .. } => {
            let investigation_target = ai_state.last_known_target.unwrap_or(Vec2::ZERO);
            ai_state.mode = AIMode::Investigate { location: investigation_target };
            action_events.write(ActionEvent { entity: enemy_entity, action: Action::MoveTo(investigation_target) });
        },
        ActionType::SearchArea { center, radius } => {
            let search_center = ai_state.last_known_target.unwrap_or(*center);
            let enemy_pos = enemy_transform.translation.truncate();
            let angle = (enemy_pos.x + enemy_pos.y) * 0.1;
            let search_offset = Vec2::new(angle.cos(), angle.sin()) * radius;
            let search_point = search_center + search_offset;
            
            action_events.write(ActionEvent { entity: enemy_entity, action: Action::MoveTo(search_point) });
            ai_state.mode = AIMode::Search { area: search_center };
        },
        ActionType::Search { area } => {
            ai_state.mode = AIMode::Search { area: *area };
        },
        ActionType::TakeCover => {
            if let Some((cover_entity, cover_pos)) = find_cover(enemy_transform.translation.truncate(), cover_query, None, false) {
                action_events.write(ActionEvent { entity: enemy_entity, action: Action::MoveTo(cover_pos) });
                commands.entity(enemy_entity).insert(InCover { cover_entity });
            }
        },
        ActionType::Retreat { .. } => {
            let enemy_pos = enemy_transform.translation.truncate();
            let retreat_direction = if let Some(agent_entity) = find_closest_agent(enemy_transform, agent_query) {
                if let Ok((_, agent_transform)) = agent_query.get(agent_entity) {
                    -(agent_transform.translation.truncate() - enemy_pos).normalize_or_zero()
                } else { Vec2::new(1.0, 0.0) }
            } else {
                patrol.current_target().map(|p| (p - enemy_pos).normalize_or_zero()).unwrap_or(Vec2::new(1.0, 0.0))
            };
            
            let retreat_point = enemy_pos + retreat_direction * 120.0;
            action_events.write(ActionEvent { entity: enemy_entity, action: Action::MoveTo(retreat_point) });
            ai_state.mode = AIMode::Patrol;
        },
        ActionType::CallForHelp => {
            audio_events.write(AudioEvent { sound: AudioType::Alert, volume: 1.0 });
            alert_events.write(AlertEvent {
                alerter: enemy_entity, position: enemy_transform.translation.truncate(),
                alert_level: 1, source: AlertSource::Gunshot, alert_type: AlertType::CallForHelp,
            });
        },
        ActionType::Reload => {
            action_events.write(ActionEvent { entity: enemy_entity, action: Action::Reload });
        },
        ActionType::UseMedKit => {
            audio_events.write(AudioEvent { sound: AudioType::Alert, volume: 0.3 });
        },
        ActionType::ThrowGrenade { .. } => {
            let throw_target = if let Some(agent_entity) = find_closest_agent(enemy_transform, agent_query) {
                if let Ok((_, agent_transform)) = agent_query.get(agent_entity) {
                    agent_transform.translation.truncate() + Vec2::new(20.0, 0.0)
                } else { enemy_transform.translation.truncate() + Vec2::new(50.0, 0.0) }
            } else { enemy_transform.translation.truncate() + Vec2::new(50.0, 0.0) };
            
            audio_events.write(AudioEvent { sound: AudioType::Alert, volume: 1.0 });
        },
        ActionType::ActivateAlarm { .. } => {
            audio_events.write(AudioEvent { sound: AudioType::Alert, volume: 1.0 });
            alert_events.write(AlertEvent {
                alerter: enemy_entity, position: enemy_transform.translation.truncate(),
                alert_level: 2, source: AlertSource::Alarm, alert_type: AlertType::EnemySpotted,
            });
        },
        ActionType::FindBetterCover { .. } => {
            if let Some((cover_entity, cover_pos)) = find_cover(enemy_transform.translation.truncate(), cover_query, Some(&agent_query), true) {
                action_events.write(ActionEvent { entity: enemy_entity, action: Action::MoveTo(cover_pos) });
                commands.entity(enemy_entity).insert(InCover { cover_entity });
            }
        },
        ActionType::SuppressingFire { .. } => {
            if let Some(agent_entity) = find_closest_agent(enemy_transform, agent_query) {
                action_events.write(ActionEvent { entity: enemy_entity, action: Action::Attack(agent_entity) });
                audio_events.write(AudioEvent { sound: AudioType::Gunshot, volume: 0.8 });
            }
        },
        ActionType::MaintainDistance => {
            if let Some(agent_entity) = find_closest_agent(enemy_transform, agent_query) {
                if let Ok((_, agent_transform)) = agent_query.get(agent_entity) {
                    let agent_pos = agent_transform.translation.truncate();
                    let enemy_pos = enemy_transform.translation.truncate();
                    let away_direction = (enemy_pos - agent_pos).normalize_or_zero();
                    let retreat_pos = enemy_pos + away_direction * 80.0;
                    
                    action_events.write(ActionEvent { entity: enemy_entity, action: Action::MoveTo(retreat_pos) });
                }
            }
        },
        ActionType::FightingWithdrawal { .. } => {
            let enemy_pos = enemy_transform.translation.truncate();
            let retreat_target = if let Some(patrol_point) = patrol.current_target() {
                let to_patrol = (patrol_point - enemy_pos).normalize_or_zero();
                enemy_pos + to_patrol * 100.0
            } else {
                if let Some(agent_entity) = find_closest_agent(enemy_transform, agent_query) {
                    if let Ok((_, agent_transform)) = agent_query.get(agent_entity) {
                        let away_from_agent = (enemy_pos - agent_transform.translation.truncate()).normalize_or_zero();
                        enemy_pos + away_from_agent * 100.0
                    } else { enemy_pos + Vec2::new(100.0, 0.0) }
                } else { enemy_pos + Vec2::new(100.0, 0.0) }
            };
            
            action_events.write(ActionEvent { entity: enemy_entity, action: Action::MoveTo(retreat_target) });
            
            if let Some(agent_entity) = find_closest_agent(enemy_transform, agent_query) {
                action_events.write(ActionEvent { entity: enemy_entity, action: Action::Attack(agent_entity) });
            }
            
            ai_state.mode = AIMode::Patrol;
        },
    }
}

fn find_cover(enemy_pos: Vec2, cover_q: &Query<(Entity, &Transform, &CoverPoint), Without<Enemy>>,
    agent_q: Option<&Query<(Entity, &Transform), With<Agent>>>, use_score: bool) -> Option<(Entity, Vec2)> {
    
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

fn find_closest_agent(enemy_transform: &Transform, agent_query: &Query<(Entity, &Transform), With<Agent>>) -> Option<Entity> {
    let enemy_pos = enemy_transform.translation.truncate();
    agent_query.iter()
        .min_by(|(_, a_transform), (_, b_transform)| {
            let a_distance = enemy_pos.distance(a_transform.translation.truncate());
            let b_distance = enemy_pos.distance(b_transform.translation.truncate());
            a_distance.partial_cmp(&b_distance).unwrap_or(std::cmp::Ordering::Equal)
        })
        .map(|(entity, _)| entity)
}

fn find_any_hostile_target(enemy_transform: &Transform, vision: &Vision, agent_query: &Query<(Entity, &Transform), With<Agent>>,
    all_enemy_query: &Query<(Entity, &Transform, &Faction), (With<Enemy>, Without<Dead>)>) -> Option<Entity> {
    find_closest_agent(enemy_transform, agent_query)
}