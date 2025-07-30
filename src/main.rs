// src/main.rs - Fixed system tuple parentheses
use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPlugin};
use bevy_rapier2d::prelude::*;
use bevy::time::common_conditions::on_timer;
use leafwing_input_manager::prelude::*;
use std::sync::Arc; // fonts

use systems::interactive_decals::*;
use systems::explosion_decal_integration::*;            
use systems::police::{load_police_config, PoliceResponse, PoliceEscalation};

mod core;
mod systems;

use core::*;
use core::factions;
use systems::*;
use pool::*;
use systems::scenes::*;
use systems::explosions::*;
use systems::projectiles::*;
use systems::ui::{loading_system};

// USER INTERFACE
use systems::ui::hub::{CyberneticsDatabase, HubState, HubDatabases};
use systems::ui::hub::agents::AgentManagementState;
use systems::ui::{main_menu, settings, credits};
use systems::ui::{MainMenuState};
use systems::ui::screens::InventoryUIState;
use systems::ui::post_mission::{PostMissionUIState};

fn main() {

    let (global_data, research_progress) = load_global_data_or_default();
    ensure_data_directories();

    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Subversive".to_string(),
                resolution: (1280.0, 720.0).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0))
        .add_plugins(InputManagerPlugin::<PlayerAction>::default())
        .add_plugins(bevy::diagnostic::FrameTimeDiagnosticsPlugin::default())
        .add_plugins(EguiPlugin::default()) // 0.2.5.4

        .register_type::<PlayerAction>()
        .register_type::<DecalDemoAction>()

        .init_state::<GameState>()

        .init_resource::<GameMode>()
        .init_resource::<FontsLoaded>()
        .init_resource::<SelectionState>()
        .init_resource::<MissionData>()
        .init_resource::<InventoryState>()
        .init_resource::<InventoryUIState>()
        .init_resource::<PostMissionResults>()
        .init_resource::<MissionState>()
        .init_resource::<DayNightCycle>()
        .init_resource::<PoliceEscalation>()
        .init_resource::<CombatTextSettings>()
        .init_resource::<AgentManagementState>()
        .init_resource::<CitiesDatabase>()
        .init_resource::<CitiesProgress>()
        .init_resource::<MessageLog>()
        .init_resource::<ScannerState>()
        .init_resource::<MainMenuState>()
        .init_resource::<ProjectilePool>()
        .init_resource::<ContinuousAttackState>()
        .init_resource::<DecalSettings>()
        .init_resource::<InteractiveDecalSettings>()
        .init_resource::<PathfindingGrid>() // 0.2.5.3

        .init_resource::<PostMissionUIState>() // 0.2.5.4
        
        .init_resource::<StartupFrameCount>()


        .insert_resource(GameConfig::load())
        .insert_resource(global_data)
        .insert_resource(research_progress)

        .insert_resource(ResearchDatabase::load())
        .insert_resource(CyberneticsDatabase::load())
        .insert_resource(TraitsDatabase::load())
        .insert_resource(AttachmentDatabase::load())
        .insert_resource(LoreDatabase::load())
        .insert_resource(CitiesDatabase::load())
        .insert_resource(WeaponDatabase::load())
        .insert_resource(CyberneticsDatabase::load())

        .init_resource::<UIState>()
        .init_resource::<PostMissionProcessed>()
        .init_resource::<EntityPool>()
        .init_resource::<SelectionDrag>()
        .init_resource::<HubState>()
        .init_resource::<UnlockedAttachments>()
        .init_resource::<ManufactureState>()
        .init_resource::<PoliceResponse>()
        .init_resource::<FormationState>()
        .init_resource::<CivilianSpawner>()
        .init_resource::<PowerGrid>()
        .insert_resource(AgentManagementState::default())

        // 0.2.9
        .init_resource::<TrafficSystem>()
        .init_resource::<RoadGrid>()

        // 0.2.10
        .init_resource::<BankingNetwork>()

        .insert_resource(HubState::default())
        .insert_resource(HubDatabases::default())
        // .insert_resource(HubProgress::default())
        
        .init_resource::<SceneCache>()
        .init_resource::<MinimapSettings>()

        .add_event::<ActionEvent>()
        .add_event::<CombatEvent>()
        .add_event::<AudioEvent>()
        .add_event::<AlertEvent>()
        .add_event::<GrenadeEvent>()
        .add_event::<BarkEvent>()
        .add_event::<LoreAccessEvent>()
        .add_event::<HackAttemptEvent>() 
        .add_event::<HackCompletedEvent>()
        .add_event::<PowerGridEvent>()
        .add_event::<DamageTextEvent>()

        // 0.2.10
        .add_event::<AccessEvent>()
        .add_event::<GateStateChange>()
        .add_event::<DoorStateChange>()        

        // 0.2.12
        .add_event::<ScientistRecruitmentEvent>()
        .add_event::<ResearchCompletedEvent>()
        .add_event::<ResearchSabotageEvent>()        

        .add_systems(Startup, (
            fonts::load_fonts,
            load_egui_fonts,
            setup_camera_and_input,
            audio::setup_audio,
            setup_attachments,
            apply_loaded_research_benefits,
            fonts::check_fonts_loaded,
            setup_urban_areas,
            setup_police_system,
            sprites::load_sprites,
            pathfinding::setup_pathfinding_grid, // 0.2.5.3

            setup_cyberpunk2077_theme, // 0.2.5.4

            setup_traffic_system, // 0.2.9
            setup_banking_network, // 0.2.10
        ))

        .add_systems(Startup, (
            // Cursor and interaction systems
            cursor::load_cursor_sprites,
            interaction_prompts::load_interaction_sprites,
            cursor::hide_system_cursor,
            // Initialize settings resources
            setup_cursor_settings,
            setup_prompt_settings,
        ))        

        .add_systems(PostStartup, (
            preload_common_scenes,
            main_menu::setup_main_menu_egui,
        ))

        .add_systems(Update, loading_system::loading_system.run_if(in_state(GameState::Loading)))

        .add_systems(Update, (
            ui::fps_system,

            pool::cleanup_inactive_entities,
            save::auto_save_system,
            save::save_input_system,
            audio::audio_system,
            scene_cache_debug_system,

        ))

        .add_systems(Update, (
            cursor_enhancements::cursor_detection_system,
            cursor_enhancements::cursor_sprite_system,
            cursor_enhancements::cursor_audio_system,

            cursor_enhancements::weapon_specific_cursor_system,
            cursor_enhancements::range_indicator_system,

            advanced_prompts::advanced_prompt_system,
            advanced_prompts::distance_fade_system,
        ).chain())    // Leave .chain() to ensure proper execution order

        .add_systems(Update, (
            // Cleanup systems - run less frequently for performance
            interaction_prompts::cleanup_orphaned_prompts,
        ).run_if(on_timer(std::time::Duration::from_secs_f32(1.0)))) // Only run every second

        // MAIN MENU
        .add_systems(Update, (
            main_menu::main_menu_system_egui,
        ).run_if(in_state(GameState::MainMenu)))

        // SETTINGS
        .add_systems(Update, (
            settings::settings_system_egui,
        ).run_if(in_state(GameState::Settings)))

        // CREDITS
        .add_systems(Update, (
            credits::credits_system_egui,
        ).run_if(in_state(GameState::Credits)))

        // UI HUB
        .add_systems(OnEnter(GameState::GlobalMap), (
            ui::cleanup_global_map_ui,
            ui::reset_hub_to_global_map,
        ))

        .add_systems(Update,(
            despawn::despawn_marked_entities,
        ).run_if(in_state(GameState::GlobalMap)))

        .add_systems(OnExit(GameState::GlobalMap), 
            mission::restart_system_optimized
        )


        // MAIN GAME / MISSION

        .add_systems(OnEnter(GameState::Mission), (
            setup_mission_scene_optimized,
            (
                health_bars::spawn_agent_status_bars,
                health_bars::spawn_enemy_health_bars,
                factions::setup_factions_system,
                factions::faction_color_system,
                // message_window::setup_message_window,
                setup_interactive_decals_demo,
                setup_minimap,
            ).after(setup_mission_scene_optimized),
        ))

        // 0.2.12
        .add_systems(Update, (
            // Research progression (daily)
            research_progress_system,
            scientist_loyalty_system,
            
            // Scientist interactions
            scientist_interaction_system,
            scientist_productivity_system,
            
            // Research facilities and espionage
            research_facility_interaction_system,
            research_facility_security_system,
            research_sabotage_system,
            
            // Spawning systems (mission state)
        ).run_if(in_state(GameState::GlobalMap)))

        .add_systems(Update, (
            sync_egui_mouse_input,
            ui::hub::hub_input_system,
            ui::hub::hub_ui_system,
            ui::hub::hub_interaction_system,
        ).chain().run_if(in_state(GameState::GlobalMap)))

        // Core mission systems
        .add_systems(Update, (
            camera::movement,
            selection::system,
            systems::input::handle_input,

            goap::goap_ai_system,

            ai::goap_sound_detection_system,
            ai::alert_system,
            ai::legacy_enemy_ai_system,
            ai::sound_detection_system,

            morale::morale_system,
            morale::civilian_morale_system,
            morale::flee_system,
        ).run_if(in_state(GameState::Mission)))

        // MOVEMENT SYSTEMS 0.2.5.3
        // Replaces original movement::system
        .add_systems(Update, (
            // movement::system,
            pathfinding::update_pathfinding_grid,
            pathfinding::add_pathfinding_to_agents,
            pathfinding::pathfinding_movement_system, 
        ).run_if(in_state(GameState::Mission)))        
        
        // Combat and interaction systems
        .add_systems(Update, (            
            weapon_swap::weapon_drop_system,
            weapon_swap::weapon_pickup_system,
            weapon_swap::weapon_behavior_system,
            
            interaction::system,
            collision_feedback_system,

            combat::process_attack_events,
            combat::enemy_combat_system,

            combat::system,
            
            death::death_system,
            death::explodable_death_system,
            combat::auto_reload_system,
            combat::cleanup_miss_targets,

            damage_text_event_system,

            ui::world::system,
            ui::screens::inventory_system,
            ui::pause_system,

        ).run_if(in_state(GameState::Mission)))

        .add_systems(Update, (
            // Death and decal systems
            decals::decal_fade_system,
            decals::decal_cleanup_system,
            death::corpse_cleanup_system,
            
            // Add decals for projectile impacts
            // projectile_impact_decals,
            enhanced_projectile_impact_decals,
            explosion_scorch_decals,

            // Minimap systems
            minimap::update_minimap_system,
            minimap::apply_minimap_research_benefits,
            minimap::minimap_toggle_system,            
        ).run_if(in_state(GameState::Mission)))

        .add_systems(Update, (

            projectiles::unified_projectile_system,
            projectiles::impact_effect_system,            
                        
        ).run_if(in_state(GameState::Mission)))

        // Mission management systems
        .add_systems(Update, (            

            cover::cover_management_system,
            cover::cover_exit_system,

            quicksave::quicksave_system,

            reload::reload_system,

            panic_spread::panic_spread_system,
            panic_spread::panic_morale_reduction_system,


        ).run_if(in_state(GameState::Mission)))

        // Area control and formations
        .add_systems(Update, (
            
            // CRASH RISK
            area_control::weapon_area_control_system,
            
            area_control::area_effect_system,
            area_control::suppression_movement_system,
            
            formations::formation_input_system,
            formations::formation_movement_system,
            formations::formation_visual_system,

            enhanced_neurovector::enhanced_neurovector_system,
            enhanced_neurovector::controlled_civilian_behavior_system,
            enhanced_neurovector::controlled_civilian_visual_system,
        ).run_if(in_state(GameState::Mission)))

        // 0.2.9
        .add_systems(Update, (
            // Traffic core systems
            traffic::traffic_spawn_system,
            traffic::traffic_movement_system,
            traffic::traffic_visual_effects_system,
            traffic::traffic_collision_system,
            traffic::traffic_cleanup_system,
            
            // Civilian Handling
            traffic_upgrades::civilian_traffic_interaction_system,
            traffic_upgrades::traffic_light_vehicle_system,

            // Emergency and military systems
            emergency_response_system,
            military_convoy_system,
            
        ).run_if(in_state(GameState::Mission)))

        // 0.2.10
        .add_systems(Update, (
            // Access control systems
            access_control::motion_sensor_system,
            access_control::access_control_system,
            access_control::access_control_prompts,
            access_control::gate_door_visual_system,
            access_control::gate_door_audio_system,

            hacking_financial::atm_hacking_system,
            hacking_financial::billboard_influence_system,
            hacking_financial::terminal_account_data_system,
            hacking_financial::financial_interaction_prompts,
                        
        ).run_if(in_state(GameState::Mission)))

        // Environmental systems
        .add_systems(Update, (
            vehicles::vehicle_explosion_system,
            //vehicles::explosion_damage_system,
            vehicles::vehicle_cover_system,
            vehicles::vehicle_spawn_system,

            day_night::day_night_system,
            day_night::lighting_system,
            day_night::time_ui_system,

            health_bars::update_agent_status_bars,
            health_bars::update_enemy_health_bars,
        ).run_if(in_state(GameState::Mission)))

        // Urban simulation
        .add_systems(Update, (
            urban_simulation::urban_civilian_spawn_system,
            urban_simulation::crowd_dynamics_system,
            urban_simulation::daily_routine_system,
            urban_simulation::urban_cleanup_system,
            urban_simulation::urban_debug_system,

            // civilian_spawn::civilian_wander_system,

            message_window::update_message_window,
            message_window::message_scroll_system,
            civilian_spawn::civilian_cleanup_system,

        ).run_if(in_state(GameState::Mission)))
        
        // Police escalation
        .add_systems(Update, (
            // CORE
            police::police_tracking_system,
            police::police_spawn_system,
            // ESCALATION
            police::police_incident_tracking_system,
            police::police_spawn_system,
            police::police_cleanup_system,
            police::police_deescalation_system,
            
            weapons::enemy_weapon_update_system,
        ).run_if(in_state(GameState::Mission)))

        .add_systems(Update, (
            // explosions::explosion_damage_system,
            enhanced_explosion_damage_system,
            explosions::time_bomb_system,
            explosions::pending_explosion_system,
            explosions::status_effect_system,
            explosions::floating_text_system,
            // explosions::handle_grenade_events,
            enhanced_handle_grenade_events,
            // explosions::handle_vehicle_explosions,
            enhanced_handle_vehicle_explosions,

            scanner::scanner_ui_system,
            scanner::scanner_cleanup_system,
        ).run_if(in_state(GameState::Mission)))

        .add_systems(Update, (
            // Movement and interaction systems
            interactive_decal_movement_system,
            electrical_hazard_system,
            stuck_entities_system,
            
            // Fire systems
            fire_ignition_system,
            fire_burn_system,
            
            // 0.2.12
            research_gameplay::spawn_scientists_in_mission,
        ).run_if(in_state(GameState::Mission)))        

        // Hacking and infrastructure
        .add_systems(Update, (
            lore::lore_interaction_system,
            lore::lore_notification_system,
            hacking_feedback::enhanced_hacking_system,
            hack_recovery_system,
            power_grid_system,
            power_grid_management_system,
            street_light_system,
            traffic_light_system,
            security_camera_system,
            automated_turret_system,
            security_door_system,
            power_grid_debug_system,
        ).run_if(in_state(GameState::Mission)))

        // NPC communication and feedback
        .add_systems(Update, (
            npc_barks::goap_bark_system,
            npc_barks::combat_bark_system,
            npc_barks::bark_handler_system,
            npc_barks::update_bubble_system,

            hacking_feedback::hack_progress_visualization,
            hacking_feedback::hack_status_indicator_system,
            hacking_feedback::device_visual_feedback_system,
            hacking_feedback::hack_interruption_system,
            hacking_feedback::hack_notification_system,
            
            mission::timer_system,
            mission::check_completion,
            
            // ALWAYS LAST
            despawn::despawn_marked_entities,
        ).run_if(in_state(GameState::Mission)))

        // TESTING & DEBUG
        .add_systems(Update, (
            debug_pathfinding_grid,
            interactive_decals_demo_system,
            research_debug_system,
        ).run_if(in_state(GameState::Mission)))

        .add_systems(OnExit(GameState::Mission), (
            minimap::cleanup_minimap_ui,
            cursor_memory_cleanup,
        ))

        // POST MISSION
        .add_systems(OnEnter(GameState::PostMission), (
            ui::cleanup_mission_ui,
        ))

        .add_systems(Update, (
            mission::process_mission_results,  
            ui::post_mission_ui_system,
            
        ).run_if(in_state(GameState::PostMission)))
        
        .run();
}

// === TESTING SCENARIOS ===

/*
Great chain reaction scenarios to test:

1. **The Gas Station**: 
   - Place fuel barrels near gasoline spills
   - Shoot or explode near the spills
   - Watch fire spread and trigger barrel explosions

2. **The Parking Lot**:
   - Multiple cars with oil spills
   - One explosion should create fire that spreads between vehicles
   - Chain reaction of vehicle explosions

3. **The Industrial Zone**:
   - Mix of oil spills, gas spills, and electrical puddles
   - Electric puddles damage entities walking through
   - Fire spreads between flammable areas

4. **The Convoy Ambush**:
   - Fuel truck creates massive gasoline spill when destroyed
   - Fire spreads to nearby vehicles and explodables
   - Creates area denial as players/enemies avoid burning zones
*/


pub fn setup_mission_scene_optimized(
    mut commands: Commands,
    sprites: Res<GameSprites>,
    global_data: Res<GlobalData>,
    launch_data: Option<Res<MissionLaunchData>>,
    cities_db: Res<CitiesDatabase>,
    cities_progress: Res<CitiesProgress>,     // Resource Does Not Exist
    mut scene_cache: ResMut<SceneCache>,
    agents: Query<Entity, With<Agent>>,
    mut power_grid: ResMut<PowerGrid>,
) {

    info!("setup_mission_scene_optimized");
    // Clean up existing agents
    // Doesn't cause the warnings
    // Removing leaves us with six agents?!

    for entity in agents.iter() {
        if agents.get(entity).is_ok() {
            commands.entity(entity).insert(MarkedForDespawn); 
        }
    }

    let selected_city = if let Some(launch_data) = launch_data {
        info!("Looking for city: '{}'", launch_data.city_id); // Fix this
        let city = cities_db.get_city(&launch_data.city_id);
        if city.is_none() {
            info!("City '{}' not found in database!", launch_data.city_id); // Fix this too
            info!("Available cities: {:?}", cities_db.get_all_cities().iter().map(|c| &c.id).take(5).collect::<Vec<_>>());
        }
        city
    } else {
        info!("Launch data not available");
        None
    };

    let scene_name = if let Some(city) = selected_city {
        match city.traits.first() {
            Some(CityTrait::FinancialHub) => "mission_corporate",
            Some(CityTrait::DrugCartels) => "mission_syndicate", 
            Some(CityTrait::Underground) => "mission_underground",
            _ => "mission1", // fallback
        }
    } else {
        // Legacy fallback
        match global_data.selected_region {
            0 => "mission1",
            1 => "mission2", 
            2 => "mission3",
            _ => "mission1",
        }
    };
    
    match load_scene_cached(&mut scene_cache, scene_name) {
        Some(scene) => {
            spawn_from_scene(&mut commands, &scene, &*global_data, &sprites);
            info!("Loaded scene: {} for city: {}", scene_name, 
                  selected_city.map_or("None", |c| &c.name));
        },
        None => {
            error!("Failed to load scene: {}. Creating fallback.", scene_name);
        }
    }

    spawn_oil_spill(&mut commands, Vec2::new(100.0, 100.0), 50.0);
    spawn_gasoline_spill(&mut commands, Vec2::new(200.0, 100.0), 40.0);
    spawn_explodable(&mut commands, Vec2::new(250.0, 100.0), ExplodableType::FuelBarrel);

       // NEW: Add hackable test objects
    spawn_hackable_test_objects(&mut commands, &sprites, &mut power_grid);

    setup_chain_reaction_test_scenario(commands);
}

fn setup_camera_and_input(mut commands: Commands) {
    // FIXED: Proper 2D camera setup for Bevy 0.16.1
    commands.spawn((
        Camera2d,
        Camera {
            clear_color: ClearColorConfig::Custom(Color::srgb(0.1, 0.1, 0.2)), // Dark blue background
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, 1000.0), // Ensure camera is above sprites
    ));
    
    let input_map = InputMap::default()
        .with(PlayerAction::Pause, KeyCode::Space)
        .with(PlayerAction::Select, MouseButton::Left)
        .with(PlayerAction::Move, MouseButton::Right)
        .with(PlayerAction::Attack, MouseButton::Right)
        .with(PlayerAction::Neurovector, KeyCode::KeyN)
        .with(PlayerAction::Combat, KeyCode::KeyF)
        .with(PlayerAction::Interact, KeyCode::KeyE)
        .with(PlayerAction::Inventory, KeyCode::KeyI)
        .with(PlayerAction::Reload, KeyCode::KeyR)
        .with(PlayerAction::SetTimeBomb, KeyCode::KeyT);
    
    commands.spawn((
        input_map,
        ActionState::<PlayerAction>::default(),
    ));

}

fn setup_attachments(mut commands: Commands) {
    let attachment_db = AttachmentDatabase::load();
    let mut unlocked = UnlockedAttachments::default();
    unlocked.attachments.insert("red_dot".to_string());
    unlocked.attachments.insert("iron_sights".to_string());
    unlocked.attachments.insert("tactical_grip".to_string());
    
    commands.insert_resource(attachment_db);
    commands.insert_resource(unlocked);
}

fn load_global_data_or_default() -> (GlobalData, ResearchProgress) {
    if let Some(mut loaded_global_data) = crate::systems::save::load_game() {
        // Merge the loaded cities progress into global data
        loaded_global_data.cities_progress = loaded_global_data.cities_progress;
        let research_progress = loaded_global_data.research_progress.clone();
        (loaded_global_data, research_progress)
    } else {
        let global_data = GlobalData::default(); // This now includes cities_progress
        let research_progress = global_data.research_progress.clone();
        (global_data, research_progress)
    }
}

fn apply_loaded_research_benefits(
    global_data: Res<GlobalData>,
    research_db: Res<ResearchDatabase>,
    mut unlocked_attachments: ResMut<UnlockedAttachments>,
) {
    apply_research_unlocks(
        &global_data.research_progress,
        &research_db,
        &mut unlocked_attachments,
    );
}



fn ensure_data_directories() {
    let directories = [
        "data/config",
        "data/attachments", 
        "scenes"
    ];
    
    for dir in directories {
        if std::fs::create_dir_all(dir).is_err() {
            error!("Failed to create directory: {}", dir);
        }
    }
    
    // Check for required files
    let required_files = [
        "data/config/game.json",
        "data/research.json",
        "data/cybernetics.json",
        "data/traits.json",
        "data/attachments/tier1.json",
    ];
    
    for file in required_files {
        if !std::path::Path::new(file).exists() {
            warn!("Missing data file: {} - game may not function properly", file);
        }
    }
}

pub fn preload_common_scenes(mut scene_cache: ResMut<SceneCache>) {
    let common_scenes = ["mission1", "mission2", "mission3"];
    scene_cache.preload_scenes(&common_scenes);
    info!("Preloaded {} scenes at startup", common_scenes.len());
}

fn setup_urban_areas(mut commands: Commands) {
    commands.insert_resource(UrbanAreas::default());
}

pub fn collision_feedback_system(
    mut collision_events: EventReader<CollisionEvent>,
    mut units: Query<&mut Velocity, Or<(With<Agent>, With<Civilian>, With<Enemy>)>>,
) {
    for collision_event in collision_events.read() {
        if let CollisionEvent::Started(e1, e2, _) = collision_event {
            // Check if both entities are units before proceeding
            if units.get(*e1).is_ok() && units.get(*e2).is_ok() {
                let separation_force = Vec2::new(fastrand::f32() - 0.5, fastrand::f32() - 0.5) * 50.0;
                
                // Apply opposite forces to separate the entities
                if let Ok(mut vel1) = units.get_mut(*e1) {
                    vel1.linvel += separation_force;
                }
                if let Ok(mut vel2) = units.get_mut(*e2) {
                    vel2.linvel -= separation_force; // Opposite direction
                }
            }
        }
    }
}

fn setup_police_system(mut commands: Commands) {
    // Load configuration from file
    let config = load_police_config();
    
    // Insert as resources
    commands.insert_resource(config);
    commands.insert_resource(PoliceResponse::default());
    commands.insert_resource(PoliceEscalation::default());
}

fn setup_egui_theme(mut contexts: EguiContexts) {
    if let Ok(ctx) = contexts.ctx_mut() {
        ui::setup_cyberpunk_theme(ctx);
    }
}


fn setup_cyberpunk2077_theme(mut contexts: EguiContexts) {
    if let Ok(ctx) = contexts.ctx_mut() {
        let mut style = (*ctx.style()).clone();
        
        // Cyberpunk 2077 color palette
        let cp_yellow = egui::Color32::from_rgb(252, 255, 82);      // Signature yellow
        let cp_cyan = egui::Color32::from_rgb(0, 255, 255);        // Bright cyan
        let cp_magenta = egui::Color32::from_rgb(255, 0, 150);     // Hot pink/magenta
        let cp_dark_bg = egui::Color32::from_rgb(8, 8, 12);        // Very dark background
        let cp_panel_bg = egui::Color32::from_rgb(16, 18, 24);     // Dark panel
        let cp_accent_bg = egui::Color32::from_rgb(24, 28, 35);    // Slightly lighter
        let cp_border = egui::Color32::from_rgb(252, 255, 82);     // Yellow borders
        
        // Background colors - very dark with blue tint
        style.visuals.window_fill = cp_dark_bg;
        style.visuals.panel_fill = cp_panel_bg;
        style.visuals.faint_bg_color = cp_accent_bg;
        style.visuals.extreme_bg_color = cp_dark_bg;
        
        // Text colors - bright yellow as primary
        style.visuals.override_text_color = Some(cp_yellow);
        
        // Widget colors
        style.visuals.widgets.noninteractive.bg_fill = cp_panel_bg;
        style.visuals.widgets.noninteractive.bg_stroke = egui::Stroke::new(1.0, cp_border);
        
        style.visuals.widgets.inactive.bg_fill = cp_accent_bg;
        style.visuals.widgets.inactive.bg_stroke = egui::Stroke::new(1.0, cp_border);
        
        style.visuals.widgets.hovered.bg_fill = cp_magenta.gamma_multiply(0.3);
        style.visuals.widgets.hovered.bg_stroke = egui::Stroke::new(2.0, cp_magenta);
        
        style.visuals.widgets.active.bg_fill = cp_cyan.gamma_multiply(0.3);
        style.visuals.widgets.active.bg_stroke = egui::Stroke::new(2.0, cp_cyan);
        
        // Selection colors - bright cyan
        style.visuals.selection.bg_fill = cp_cyan.gamma_multiply(0.4);
        style.visuals.selection.stroke = egui::Stroke::new(2.0, cp_cyan);
        
        // Button colors
        style.visuals.widgets.open.bg_fill = cp_yellow.gamma_multiply(0.2);
        style.visuals.widgets.open.bg_stroke = egui::Stroke::new(2.0, cp_yellow);
        
        // Hyperlink colors
        style.visuals.hyperlink_color = cp_cyan;
        
        // Window styling - sharp corners, prominent borders
        // style.visuals.window_rounding = egui::Rounding::ZERO;
        style.visuals.window_stroke = egui::Stroke::new(2.0, cp_border);
        style.visuals.window_shadow = egui::epaint::Shadow::NONE;
        
        // Panel styling
        style.visuals.panel_fill = cp_panel_bg;
        
        // Spacing - tighter, more compact
        style.spacing.item_spacing = egui::vec2(6.0, 4.0);
        style.spacing.window_margin = egui::Margin::symmetric(8, 8);
        style.spacing.button_padding = egui::vec2(8.0, 4.0);
        style.spacing.menu_margin = egui::Margin::symmetric(4, 4);
        
        // Scrollbar colors
        style.visuals.widgets.inactive.bg_fill = cp_accent_bg;
        
        ctx.set_style(style);
    }
}



fn sync_egui_mouse_input(
    contexts: EguiContexts,
    windows: Query<&Window>,
    mouse: Res<ButtonInput<MouseButton>>,
) {

    // let mut ctx = contexts.ctx_mut();
    let ctx = egui::Context::default();
    
    // Get window and cursor position
    if let Ok(window) = windows.single() {

        if let Some(cursor_pos) = window.cursor_position() {
            // Convert Bevy's Y-down to egui's Y-up coordinate system
            
            let egui_pos = egui::pos2(
                cursor_pos.x,
                window.height() - cursor_pos.y
            );
            
            // Create raw input for egui
            let mut raw_input = ctx.input_mut(|i| i.clone());
            
            // Update pointer position
            raw_input.events.push(egui::Event::PointerMoved(egui_pos));
            
            // Handle mouse button
            if mouse.just_pressed(MouseButton::Left) {
                raw_input.events.push(egui::Event::PointerButton {
                    pos: egui_pos,
                    button: egui::PointerButton::Primary,
                    pressed: true,
                    modifiers: Default::default(),
                });
            }
            
            if mouse.just_released(MouseButton::Left) {
                raw_input.events.push(egui::Event::PointerButton {
                    pos: egui_pos,
                    button: egui::PointerButton::Primary,
                    pressed: false,
                    modifiers: Default::default(),
                });
            }
            
            // Send the input to egui
            ctx.input_mut(|i| *i = raw_input);
        }
    }
}


pub fn load_egui_fonts(mut contexts: EguiContexts, mut has_run: Local<bool>) {

    if *has_run {
        return;
    }    

    info!("Loading egui fonts...");

    if let Ok(ctx) = contexts.ctx_mut() {

        let mut fonts = egui::FontDefinitions::default();

        fonts.font_data.insert(
            "orbitron".to_owned(),
            Arc::new(egui::FontData::from_static(include_bytes!("../assets/fonts/orbitron.ttf"))),
        );

        // Set it as the highest-priority font 
        for family in [&egui::FontFamily::Proportional] {
            fonts
                .families
                .get_mut(family)
                .unwrap()
                .insert(0, "orbitron".to_owned());
        }

        ctx.set_fonts(fonts);

        *has_run = true; // Prevent reapplying every frame

        info!("Fonts Loaded Successfully!");
    }
}


// === CURSOR FUNCTIONS ===
fn setup_cursor_settings(mut commands: Commands) {
    commands.insert_resource(CursorSettings {
        show_range_indicator: true,
        cursor_scale: 1.0,
        animation_speed: 2.0,
        sound_enabled: true,
    });
}

fn setup_prompt_settings(mut commands: Commands) {
    commands.insert_resource(PromptSettings {
        max_distance: 100.0,
        fade_distance: 80.0,
        animation_enabled: true,
        show_tooltips: true,
        stack_prompts: true,
    });
}

pub fn cursor_memory_cleanup(
    mut commands: Commands,
    cursor_query: Query<Entity, (With<CursorEntity>, Without<MarkedForDespawn>)>,
    prompt_query: Query<Entity, (With<InteractionPrompt>, Without<CursorEntity>, Without<MarkedForDespawn>)>,
    game_state: Res<State<GameState>>,
) {
    // Clean up cursor/prompt entities when leaving mission
    if game_state.is_changed() && !matches!(*game_state.get(), GameState::Mission) {
        // Remove all cursor entities
        for entity in cursor_query.iter() {
            commands.entity(entity).insert(MarkedForDespawn);
        }
        
        // Remove all prompt entities
        for entity in prompt_query.iter() {
            commands.entity(entity).insert(MarkedForDespawn);
        }
    }
}


fn setup_banking_network(mut commands: Commands) {
    let banking_network = BankingNetwork {
        banks: vec![
            Bank {
                id: "MegaBank".to_string(),
                name: "MegaBank Corporation".to_string(),
                total_funds: 1000000,
                security_level: 4,
            },
            Bank {
                id: "CyberCredit".to_string(),
                name: "CyberCredit Union".to_string(),
                total_funds: 500000,
                security_level: 3,
            },
            Bank {
                id: "DataVault".to_string(),
                name: "DataVault Financial".to_string(),
                total_funds: 2000000,
                security_level: 5,
            },
        ],
        stolen_accounts: Vec::new(),
    };
    
    commands.insert_resource(banking_network);
}

// NEW: Function to spawn hackable test objects
fn spawn_hackable_test_objects(
    commands: &mut Commands,
    sprites: &GameSprites,
    power_grid: &mut ResMut<PowerGrid>,
) {
    // Security Camera - Easy hack
    let camera_entity = commands.spawn((
        Sprite {
            color: Color::srgb(0.3, 0.3, 0.3),
            custom_size: Some(Vec2::new(16.0, 12.0)),
            ..default()
        },
        Transform::from_translation(Vec3::new(-50.0, 150.0, 1.0)),
        GlobalTransform::default(),
        Visibility::default(),
        ViewVisibility::default(),
        bevy_rapier2d::prelude::RigidBody::Fixed,
        bevy_rapier2d::prelude::Collider::cuboid(8.0, 6.0),
    )).id();
    
    make_hackable(commands, camera_entity, DeviceType::Camera);
    
    // ATM - Medium security
    let atm_entity = commands.spawn((
        Sprite {
            color: Color::srgb(0.2, 0.4, 0.6),
            custom_size: Some(Vec2::new(20.0, 30.0)),
            ..default()
        },
        Transform::from_translation(Vec3::new(150.0, -100.0, 1.0)),
        GlobalTransform::default(),
        Visibility::default(),
        ViewVisibility::default(),
        bevy_rapier2d::prelude::RigidBody::Fixed,
        bevy_rapier2d::prelude::Collider::cuboid(10.0, 15.0),
    )).id();
    
    make_hackable(commands, atm_entity, DeviceType::ATM);
    
    // Turret - High security
    let turret_entity = commands.spawn((
        Sprite {
            color: Color::srgb(0.6, 0.2, 0.2),
            custom_size: Some(Vec2::new(24.0, 24.0)),
            ..default()
        },
        Transform::from_translation(Vec3::new(0.0, 200.0, 1.0)),
        GlobalTransform::default(),
        Visibility::default(),
        ViewVisibility::default(),
        bevy_rapier2d::prelude::RigidBody::Fixed,
        bevy_rapier2d::prelude::Collider::ball(12.0),
    )).id();
    
    setup_hackable_turret(commands, turret_entity);
    
    // Power Station - Networked device
    let power_station_entity = commands.spawn((
        Sprite {
            color: Color::srgb(0.8, 0.8, 0.2),
            custom_size: Some(Vec2::new(40.0, 40.0)),
            ..default()
        },
        Transform::from_translation(Vec3::new(-200.0, -50.0, 1.0)),
        GlobalTransform::default(),
        Visibility::default(),
        ViewVisibility::default(),
        bevy_rapier2d::prelude::RigidBody::Fixed,
        bevy_rapier2d::prelude::Collider::cuboid(20.0, 20.0),
    )).id();
    
    make_hackable_networked(
        commands, 
        power_station_entity, 
        DeviceType::PowerStation, 
        "test_grid".to_string(),
        power_grid
    );
    
    // Connected street lights (affected by power station)
    for i in 0..3 {
        let light_entity = commands.spawn((
            Sprite {
                color: Color::srgb(0.9, 0.9, 0.7),
                custom_size: Some(Vec2::new(8.0, 24.0)),
                ..default()
            },
            Transform::from_translation(Vec3::new(-150.0 + i as f32 * 40.0, -20.0, 1.0)),
            GlobalTransform::default(),
            Visibility::default(),
            ViewVisibility::default(),
            bevy_rapier2d::prelude::RigidBody::Fixed,
            bevy_rapier2d::prelude::Collider::cuboid(4.0, 12.0),
        )).id();
        
        make_hackable_networked(
            commands, 
            light_entity, 
            DeviceType::StreetLight, 
            "test_grid".to_string(),
            power_grid
        );
    }
    
    // Simple door - Quick hack
    let door_entity = commands.spawn((
        Sprite {
            color: Color::srgb(0.4, 0.2, 0.1),
            custom_size: Some(Vec2::new(8.0, 32.0)),
            ..default()
        },
        Transform::from_translation(Vec3::new(300.0, 0.0, 1.0)),
        GlobalTransform::default(),
        Visibility::default(),
        ViewVisibility::default(),
        bevy_rapier2d::prelude::RigidBody::Fixed,
        bevy_rapier2d::prelude::Collider::cuboid(4.0, 16.0),
    )).id();
    
    setup_hackable_door(commands, door_entity);
    
    info!("Spawned hackable test objects: camera, ATM, turret, power station, 3 street lights, door");
}


pub fn research_debug_system(
    input: Res<ButtonInput<KeyCode>>,
    mut research_progress: ResMut<ResearchProgress>,
    mut global_data: ResMut<GlobalData>,
    scientist_query: Query<(Entity, &Scientist)>,
) {

    
    if input.just_pressed(KeyCode::F9) {
        // Collect the project IDs first (this ends the immutable borrow)
        let project_ids: Vec<String> = research_progress.active_queue
            .iter()
            .map(|active| active.project_id.clone())
            .collect();
        
        // Now we can mutably borrow research_progress
        for project_id in project_ids {
            research_progress.completed.insert(project_id);
        }
        research_progress.active_queue.clear();
        info!("DEBUG: Completed all active research");
    }
    
    if input.just_pressed(KeyCode::F10) {
        // Debug: Add test scientist
        info!("DEBUG: Scientist debug info:");
        for (entity, scientist) in scientist_query.iter() {
            info!("  {} - {:?} - Recruited: {} - Productivity: {:.2}", 
                  scientist.name, scientist.specialization, scientist.is_recruited, scientist.productivity_bonus);
        }
    }
    
    if input.just_pressed(KeyCode::F11) {
        // Debug: Add 10000 credits
        global_data.credits += 10000;
        info!("DEBUG: Added $10,000 credits");
    }
}