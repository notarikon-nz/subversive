// src/core/goap_config.rs - Optional debugging and configuration
use bevy::prelude::*;
use crate::core::goap::*;

#[derive(Resource)]
pub struct GoapConfig {
    pub debug_enabled: bool,
    pub planning_interval: f32,
    pub max_plan_depth: usize,
    pub action_costs: ActionCosts,
    pub goal_priorities: GoalPriorities,
}

#[derive(Clone)]
pub struct ActionCosts {
    pub patrol: f32,
    pub investigate: f32,
    pub attack: f32,
    pub move_to_target: f32,
    pub reload: f32,
    pub call_for_help: f32,
}

#[derive(Clone)]
pub struct GoalPriorities {
    pub eliminate_threat: f32,
    pub investigate_disturbance: f32,
    pub patrol_area: f32,
}

impl Default for GoapConfig {
    fn default() -> Self {
        Self {
            debug_enabled: false,
            planning_interval: 2.0,
            max_plan_depth: 10,
            action_costs: ActionCosts {
                patrol: 1.0,
                investigate: 2.0,
                attack: 1.0,
                move_to_target: 3.0,
                reload: 2.0,
                call_for_help: 1.5,
            },
            goal_priorities: GoalPriorities {
                eliminate_threat: 10.0,
                investigate_disturbance: 5.0,
                patrol_area: 1.0,
            },
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
        
        // Draw current goal
        if let Some(goal) = &goap_agent.current_goal {
            gizmos.circle_2d(pos + Vec2::new(0.0, 40.0), 8.0, Color::srgb(0.8, 0.8, 0.2));
            
            // You could add text rendering here if needed
            // For now, we'll just use colored circles to indicate different goals
            let goal_color = match goal.name {
                "eliminate_threat" => Color::srgb(1.0, 0.2, 0.2),
                "investigate_disturbance" => Color::srgb(1.0, 0.8, 0.2),
                "patrol_area" => Color::srgb(0.2, 1.0, 0.2),
                _ => Color::WHITE,
            };
            
            gizmos.circle_2d(pos + Vec2::new(0.0, 35.0), 4.0, goal_color);
        }
        
        // Draw plan length indicator
        let plan_length = goap_agent.current_plan.len() as f32;
        if plan_length > 0.0 {
            gizmos.line_2d(
                pos + Vec2::new(-10.0, -30.0),
                pos + Vec2::new(-10.0 + (plan_length * 4.0), -30.0),
                Color::srgb(0.3, 0.8, 0.8),
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
    }
    
    // Adjust planning interval with +/-
    if keyboard.pressed(KeyCode::Equal) {
        config.planning_interval += 0.1;
        info!("GOAP Planning Interval: {:.1}s", config.planning_interval);
    }
    
    if keyboard.pressed(KeyCode::Minus) && config.planning_interval > 0.1 {
        config.planning_interval -= 0.1;
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
        // Update action costs
        for action in &mut goap_agent.available_actions {
            action.cost = match action.name {
                "patrol" => config.action_costs.patrol,
                "investigate" => config.action_costs.investigate,
                "attack" => config.action_costs.attack,
                "move_to_target" => config.action_costs.move_to_target,
                "reload" => config.action_costs.reload,
                "call_for_help" => config.action_costs.call_for_help,
                _ => action.cost,
            };
        }
        
        // Update goal priorities
        for goal in &mut goap_agent.goals {
            goal.priority = match goal.name {
                "eliminate_threat" => config.goal_priorities.eliminate_threat,
                "investigate_disturbance" => config.goal_priorities.investigate_disturbance,
                "patrol_area" => config.goal_priorities.patrol_area,
                _ => goal.priority,
            };
        }
        
        // Force replanning with new costs/priorities
        goap_agent.abort_plan();
    }
}