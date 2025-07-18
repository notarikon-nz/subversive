use bevy::prelude::*;
use crate::core::*;

pub fn timer_system(
    mut mission_data: ResMut<MissionData>,
    mut next_state: ResMut<NextState<GameState>>,
    mut post_mission: ResMut<PostMissionResults>,
    game_mode: Res<GameMode>,
    time: Res<Time>,
) {
    if game_mode.paused { return; }
    
    mission_data.timer += time.delta_seconds();
    
    // Check time limit
    if mission_data.timer >= mission_data.time_limit {
        *post_mission = PostMissionResults {
            success: false,
            time_taken: mission_data.timer,
            enemies_killed: mission_data.enemies_killed,
            terminals_accessed: mission_data.terminals_accessed,
            credits_earned: 0,
            alert_level: mission_data.alert_level,
        };
        
        next_state.set(GameState::PostMission);
        info!("Mission failed - time expired!");
    }
}

pub fn check_completion(
    mut next_state: ResMut<NextState<GameState>>,
    mission_data: Res<MissionData>,
    mut post_mission: ResMut<PostMissionResults>,
    agent_query: Query<&Inventory, With<Agent>>,
) {
    let all_objectives_complete = mission_data.objectives_completed >= mission_data.total_objectives;
    let agents_alive = agent_query.iter().count() > 0;
    
    if all_objectives_complete {
        let total_credits = agent_query.iter()
            .map(|inv| inv.currency)
            .sum::<u32>();
            
        *post_mission = PostMissionResults {
            success: true,
            time_taken: mission_data.timer,
            enemies_killed: mission_data.enemies_killed,
            terminals_accessed: mission_data.terminals_accessed,
            credits_earned: total_credits,
            alert_level: mission_data.alert_level,
        };
        
        next_state.set(GameState::PostMission);
        info!("Mission completed successfully!");
    } else if !agents_alive {
        *post_mission = PostMissionResults {
            success: false,
            time_taken: mission_data.timer,
            enemies_killed: mission_data.enemies_killed,
            terminals_accessed: mission_data.terminals_accessed,
            credits_earned: 0,
            alert_level: mission_data.alert_level,
        };
        
        next_state.set(GameState::PostMission);
        info!("Mission failed - all agents eliminated!");
    }
}

pub fn restart_system(
    mut commands: Commands,
    restart_check: Option<Res<ShouldRestart>>,
    entities: Query<Entity, (Without<Camera>, Without<Window>)>,
    mut mission_data: ResMut<MissionData>,
    mut game_mode: ResMut<GameMode>,
    mut selection: ResMut<SelectionState>,
    mut inventory_state: ResMut<InventoryState>,
    global_data: Res<GlobalData>,
) {
    if restart_check.is_some() {
        // Clear all entities except camera and window
        for entity in entities.iter() {
            commands.entity(entity).despawn_recursive();
        }
        
        // Reset all resources
        *mission_data = MissionData::default();
        *game_mode = GameMode::default();
        *selection = SelectionState::default();
        *inventory_state = InventoryState::default();
        
        // Remove restart flag
        commands.remove_resource::<ShouldRestart>();
        
        // Respawn mission with persistent agent data
        crate::spawn_agents(&mut commands, 3, &*global_data);
        crate::spawn_civilians(&mut commands, 5);
        crate::spawn_enemy(&mut commands);
        crate::spawn_terminals(&mut commands);
        
        info!("Mission restarted successfully!");
    }
}