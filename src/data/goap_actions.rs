// src/data/goap_actions.rs
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