// src/main.rs - Fixed system tuple parentheses
use bevy::prelude::*;

// https://github.com/Noxime/steamworks-rs/tree/master

use bevy_rapier2d::prelude::*;
use bevy_light_2d::prelude::*;
use leafwing_input_manager::prelude::*;

use systems::ui::hub::{CyberneticsDatabase, HubStates, HubDatabases, HubProgress};
use systems::ui::hub::agents::AgentManagementState;
use systems::ui::{main_menu, settings, credits};
use systems::ui::{MainMenuState};


use systems::police::{load_police_config, PoliceResponse, PoliceEscalation};

mod core;
mod systems;

use core::*;
use core::factions;
use systems::*;
use pool::*;
use systems::scenes::*;
use systems::explosions::*;

fn main() {

    let (global_data, research_progress) = load_global_data_or_default();
    // systems::scenes::ensure_scenes_directory();
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
//        .add_plugins(Light2dPlugin) // bevy_light_2d
        .add_plugins(bevy_mod_imgui::ImguiPlugin::default())

        // .add_plugins(bevy::diagnostic::LogDiagnosticsPlugin::default())
        .add_plugins(bevy::diagnostic::FrameTimeDiagnosticsPlugin::default())
        // .add_plugins(bevy::diagnostic::EntityCountDiagnosticsPlugin::default())

        .register_type::<PlayerAction>()

        .init_state::<GameState>()

        .init_resource::<GameMode>()
        .init_resource::<FontsLoaded>()
        .init_resource::<SelectionState>()
        .init_resource::<MissionData>()
        .init_resource::<InventoryState>()
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
        
        .insert_resource(HubStates::default())
        .insert_resource(HubDatabases::default())
        .insert_resource(HubProgress::default())

        .init_resource::<SceneCache>()

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

        .add_systems(Startup, (
            fonts::load_fonts,
            setup_camera_and_input,
            audio::setup_audio,
            setup_attachments,
            apply_loaded_research_benefits,
            fonts::check_fonts_loaded,
            setup_urban_areas,
            setup_police_system,
            sprites::load_sprites,
        ))

        .add_systems(PostStartup, (
            preload_common_scenes,
        ))

        .add_systems(Update, (
            systems::input::handle_input,
            ui::screens::fps_system,
            pool::cleanup_inactive_entities,
            save::auto_save_system,
            save::save_input_system,
            audio::audio_system,
            scene_cache_debug_system,

        ))

        // MAIN MENU
        .add_systems(OnEnter(GameState::MainMenu), (
            main_menu::setup_main_menu,
        ))
        .add_systems(Update, (
            main_menu::menu_input_system,
            main_menu::menu_mouse_system,
            main_menu::update_menu_visuals,
        ).run_if(in_state(GameState::MainMenu)))
        .add_systems(OnExit(GameState::MainMenu), (
            main_menu::cleanup_main_menu,
        ))        

        // SETTINGS
        .add_systems(OnEnter(GameState::Settings), (
            settings::setup_settings,
        ))
        .add_systems(Update, (
            settings::settings_input,
        ).run_if(in_state(GameState::Settings)))
        .add_systems(OnExit(GameState::Settings), (
            settings::cleanup_settings,
        ))

        // CREDITS
        .add_systems(OnEnter(GameState::Credits), (
            credits::setup_credits,
        ))
        .add_systems(Update, (
            credits::credits_input,
        ).run_if(in_state(GameState::Credits)))
        .add_systems(OnExit(GameState::Credits), (
            credits::cleanup_credits,
        ))

        // UI HUB
        .add_systems(OnEnter(GameState::GlobalMap), (
            ui::cleanup_global_map_ui,
            ui::reset_hub_to_global_map,
        ))

        .add_systems(Update,(
            despawn::despawn_marked_entities,
        ).run_if(in_state(GameState::GlobalMap)))

        .add_systems(OnExit(GameState::GlobalMap), (
            mission::restart_system_optimized
        ))


        // MAIN GAME / MISSION

        .add_systems(OnEnter(GameState::Mission), (
            setup_mission_scene_optimized,
            (
                health_bars::spawn_health_bars,
                factions::setup_factions_system,
                factions::faction_color_system,
                // message_window::setup_message_window,
            ).after(setup_mission_scene_optimized),
        ))

        .add_systems(Update, (
            ui::hub::hub_system,
        ).run_if(in_state(GameState::GlobalMap)))


        // Core mission systems
        .add_systems(Update, (
            camera::movement,
            selection::system,
            movement::system,

            goap::goap_ai_system,

            ai::goap_sound_detection_system,
            ai::alert_system,
            ai::legacy_enemy_ai_system,
            ai::sound_detection_system,

            morale::morale_system,
            morale::civilian_morale_system,
            morale::flee_system,
        ).run_if(in_state(GameState::Mission)))

        
        // Combat and interaction systems
        .add_systems(Update, (            
            weapon_swap::weapon_drop_system,
            weapon_swap::weapon_pickup_system,
            weapon_swap::weapon_behavior_system,
            
            interaction::system,
            collision_feedback_system,

            combat::system,
            combat::death_system,
            damage_text_event_system,

            ui::world::system,
            ui::screens::inventory_system,
            ui::screens::pause_system,

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

        // agents visible and controllable

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

        // Environmental systems
        .add_systems(Update, (
            vehicles::vehicle_explosion_system,
            vehicles::explosion_damage_system,
            vehicles::vehicle_cover_system,
            vehicles::vehicle_spawn_system,

            day_night::day_night_system,
            day_night::lighting_system,
            day_night::time_ui_system,

            health_bars::update_health_bars,
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
            
        ).run_if(in_state(GameState::Mission)))

        .add_systems(Update, (
            explosions::explosion_damage_system,
            explosions::floating_text_system,
            explosions::handle_grenade_events,
            explosions::handle_vehicle_explosions,
            // NEW
            explosions::time_bomb_system,
            explosions::pending_explosion_system,
            explosions::status_effect_system,
            
            scanner::scanner_ui_system,
            scanner::scanner_cleanup_system,
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

        // POST MISSION
        .add_systems(OnEnter(GameState::PostMission), (
            ui::cleanup_mission_ui,
            health_bars::cleanup_dead_health_bars,
        ))

        .add_systems(Update, (
            mission::process_mission_results,  
            ui::screens::post_mission_ui_system,
            
        ).run_if(in_state(GameState::PostMission)))
        
        .run();
}

pub fn setup_mission_scene_optimized(
    mut commands: Commands,
    sprites: Res<GameSprites>,
    global_data: Res<GlobalData>,
    launch_data: Option<Res<MissionLaunchData>>,
    cities_db: Res<CitiesDatabase>,
    cities_progress: Res<CitiesProgress>,     // Resource Does Not Exist
    mut scene_cache: ResMut<SceneCache>,
    agents: Query<Entity, With<Agent>>,
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
    if let Some((mut loaded_global_data)) = crate::systems::save::load_game() {
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

// Add this system to your main.rs update systems for mission state
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

