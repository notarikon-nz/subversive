// src/data/goap_goals.rs
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