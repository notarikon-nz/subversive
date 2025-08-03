// src/systems/mission.rs - Updated for Bevy 0.16
use bevy::prelude::*;
use crate::core::*;
use crate::core::research::{calculate_research_xp_bonus, calculate_research_credit_bonus};

pub fn timer_system(
    mut mission_data: ResMut<MissionData>,
    mut next_state: ResMut<NextState<GameState>>,
    mut post_mission: ResMut<PostMissionResults>,
    game_mode: Res<GameMode>,
    time: Res<Time>,
) {
    if game_mode.paused { return; }

    mission_data.timer += time.delta_secs();

    if mission_data.timer >= mission_data.time_limit {
        *post_mission = PostMissionResults {
            success: false,
            time_taken: mission_data.timer,
            enemies_killed: mission_data.enemies_killed,
            terminals_accessed: mission_data.terminals_accessed,
            credits_earned: 0,
            alert_level: mission_data.alert_level,
        };
        info!("Time Limit Exceeded - Mission Failed");
        next_state.set(GameState::PostMission);
    }
}

pub fn check_completion(
    mut next_state: ResMut<NextState<GameState>>,
    mission_data: Res<MissionData>,
    mut post_mission: ResMut<PostMissionResults>,
    agent_query: Query<&Inventory, With<Agent>>,
) {
    if agent_query.is_empty() && mission_data.timer < 1.0 {
        return; // Skip check if agents haven't spawned yet
    }

    let objectives_complete = mission_data.objectives_completed >= mission_data.total_objectives;
    let agents_alive = !agent_query.is_empty();

    if objectives_complete {
        let credits_earned = agent_query.iter().map(|inv| inv.currency).sum();
        *post_mission = PostMissionResults {
            success: true,
            time_taken: mission_data.timer,
            enemies_killed: mission_data.enemies_killed,
            terminals_accessed: mission_data.terminals_accessed,
            credits_earned,
            alert_level: mission_data.alert_level,
        };
        info!("Objectives Completed - Mission Success");
        next_state.set(GameState::PostMission);
    } else if !agents_alive {
        info!("Agents Deceased - Mission Failed");
        *post_mission = PostMissionResults::default();
        next_state.set(GameState::PostMission);
    }
}

pub fn restart_system_optimized(
    mut commands: Commands,
    restart_check: Option<Res<ShouldRestart>>,
    ui_entities: Query<Entity, Or<(With<Node>, With<Text>)>>,
    sprites: Option<Res<GameSprites>>,
    scene_cache: ResMut<SceneCache>,
    mut mission_data: ResMut<MissionData>,
    mut game_mode: ResMut<GameMode>,
    mut selection: ResMut<SelectionState>,
    mut inventory_state: ResMut<InventoryState>,
    global_data: Res<GlobalData>,
) {
    // Early return if sprites not loaded yet
    let Some(sprites) = sprites else {
        warn!("Sprites not loaded yet, skipping mission restart");
        return;
    };

    if restart_check.is_none() { return; }

    let mut despawn_count = 0;

    // Despawn UI entities
    for entity in ui_entities.iter() {
        if let Ok(entity_commands) = commands.get_entity(entity) {
            commands.entity(entity).insert(MarkedForDespawn);
            despawn_count += 1;
        }
    }

    info!("Restarting mission - despawned {} UI entities", despawn_count);

    // Reset game state (same as before)
    *mission_data = MissionData::default();
    *game_mode = GameMode::default();
    *selection = SelectionState::default();
    *inventory_state = InventoryState::default();

    commands.remove_resource::<ShouldRestart>();
}

pub fn process_mission_results(
    mut global_data: ResMut<GlobalData>,
    mut processed: ResMut<PostMissionProcessed>,
    post_mission: Res<PostMissionResults>,
    agent_query: Query<&Agent>,
    cities_db: Res<CitiesDatabase>,
    launch_data: Option<Res<MissionLaunchData>>,

    // ADD THESE NEW PARAMETERS:
    mut territory_manager: ResMut<TerritoryManager>,
    mut progression_tracker: ResMut<CampaignProgressionTracker>,
    campaign_db: Option<Res<NeoSingaporeCampaignDatabase>>, // Optional for now
) {
    if processed.0 { return; }

    info!("process_mission_results");

    let region_idx = global_data.selected_region;
    global_data.current_day += 1;
    let current_day = global_data.current_day;

    if post_mission.success {

        info!("mission successful");

        global_data.credits += post_mission.credits_earned;

        let exp_gained = 10 + (post_mission.enemies_killed * 5);
        let recovery_days = if post_mission.time_taken > 240.0 { 2 } else { 1 };

        for (i, _) in agent_query.iter().enumerate().take(3) {
            global_data.agent_experience[i] += exp_gained;
            global_data.agent_recovery[i] = current_day + recovery_days;

            let required_exp = experience_for_level(global_data.agent_levels[i] + 1);
            if global_data.agent_experience[i] >= required_exp && global_data.agent_levels[i] < 10 {
                global_data.agent_levels[i] += 1;
            }
        }

        // UNLOCK CONNECTED CITIES
        if let Some(launch_data) = launch_data.as_ref() {
            let newly_unlocked = cities_db.unlock_connected_cities(&launch_data.city_id, &mut global_data.cities_progress);

            if !newly_unlocked.is_empty() {
                info!("Mission success in {} unlocked {} new cities: {:?}",
                      launch_data.city_id, newly_unlocked.len(), newly_unlocked);
            }

            // Mark the completed city
            global_data.cities_progress.complete_city(&launch_data.city_id, current_day);
            

            // 0.2.17 - Establish territory control
            if !territory_manager.is_liberated(&launch_data.city_id) {

                if let Some(district) = campaign_db
                    .as_ref()
                    .and_then(|db| db.get_district(&launch_data.city_id)) 
                {
                    territory_manager.establish_control(
                        launch_data.city_id.clone(), 
                        district, 
                        current_day
                    );
                }
                // territory_manager.establish_control(launch_data.city_id.clone(), current_day);

                info!("Established control over {}", launch_data.city_id);

                // Check if this completes a campaign chapter (if campaign DB available)
                if let Some(campaign_db) = campaign_db.as_ref() {
                    if let Some(chapter) = campaign_db.campaign.acts.iter()
                        .flat_map(|act| &act.operations)
                        .find(|c| c.district_id == launch_data.city_id) {
                            // opreation id & district id
                        progression_tracker.complete_operation(chapter.id, chapter.district_id.clone());
                    }
                }
            }
        }

        if post_mission.enemies_killed > 0 || post_mission.time_taken >= 180.0 {
            global_data.regions[region_idx].raise_alert(current_day);
        }
    } else {
        global_data.regions[region_idx].raise_alert(current_day);
        global_data.regions[region_idx].raise_alert(current_day);
        for i in 0..3 {
            global_data.agent_recovery[i] = current_day + 3;
        }
    }

    for region in &mut global_data.regions {
        region.update_alert(current_day);
    }


    
    processed.0 = true;
}

#[derive(Component)]
pub struct PostMissionUI;
