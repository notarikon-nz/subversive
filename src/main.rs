// src/main.rs - Fixed system tuple parentheses
use bevy::prelude::*;

// https://github.com/Noxime/steamworks-rs/tree/master

use bevy_rapier2d::prelude::*;
use bevy_light_2d::prelude::*;
use leafwing_input_manager::prelude::*;
use systems::ui::hub::{AgentManagementState, CyberneticsDatabase};
use testing_spawn::*;

mod core;
mod systems;

use core::*;
use core::factions;
use systems::*;
use pool::*;
use systems::scenes::*;
use systems::police_escalation::*;
use systems::explosions::*;


// temp
use std::path::Path;
//

fn main() {

    // temp
    let cities_db = CitiesDatabase::load();
    cities_db.save();
    // 

    let (global_data, research_progress) = load_global_data_or_default();
    systems::scenes::ensure_scenes_directory();
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
        .add_plugins(Light2dPlugin) // bevy_light_2d
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

        .insert_resource(GameConfig::load())
        .insert_resource(global_data)
        .insert_resource(research_progress)
        .insert_resource(ResearchDatabase::load())
        .insert_resource(CyberneticsDatabase::load())
        .insert_resource(TraitsDatabase::load())
        .insert_resource(AttachmentDatabase::load())
        .insert_resource(LoreDatabase::load())

        .insert_resource(ImguiState {
            demo_window_open: true,
        })

        .init_resource::<UIState>()
        .init_resource::<PostMissionProcessed>()
        .init_resource::<EntityPool>()
        .init_resource::<SelectionDrag>()
        .init_resource::<GoapConfig>()
        .init_resource::<HubState>()
        .init_resource::<UnlockedAttachments>()
        .init_resource::<ManufactureState>()
        .init_resource::<PoliceResponse>()
        .init_resource::<FormationState>()
        .init_resource::<CivilianSpawner>()
        .init_resource::<PowerGrid>()
        .insert_resource(CyberneticsDatabase::load())
        .insert_resource(AgentManagementState::default())
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

        .add_systems(Startup, (
            fonts::load_fonts,
            setup_camera_and_input,
            audio::setup_audio,
            sprites::load_sprites,
            setup_attachments,
            apply_loaded_research_benefits,
            fonts::check_fonts_loaded,
            setup_urban_areas,
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

        .add_systems(OnEnter(GameState::PostMission), (
            ui::cleanup_mission_ui,
            health_bars::cleanup_health_bars_system,
        ))

        .add_systems(OnEnter(GameState::GlobalMap), (
            ui::cleanup_global_map_ui,
            ui::reset_hub_to_global_map,
        ))

        .add_systems(OnEnter(GameState::Mission), (
            ui::cleanup_global_map_ui,
            setup_mission_scene_optimized,
            health_bars::spawn_health_bar_system,
            factions::setup_factions_system,
            factions::faction_color_system,
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
            combat::system,
            combat::death_system,
            goap::goap_config_system,
            goap::goap_debug_system,
            ui::world::system,
            ui::screens::inventory_system,
            ui::screens::pause_system,
        ).run_if(in_state(GameState::Mission)))

        // Mission management systems
        .add_systems(Update, (            
            mission::timer_system,
            mission::check_completion,
            mission::restart_system_optimized,
            cover::cover_management_system,
            cover::cover_exit_system,
            quicksave::quicksave_system,
            reload::reload_system,
            panic_spread::panic_spread_system,
            panic_spread::panic_morale_reduction_system,
            police::police_tracking_system,
            police::police_spawn_system,
        ).run_if(in_state(GameState::Mission)))

        // Area control and formations
        .add_systems(Update, (            
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
            health_bars::update_health_bars_system,
        ).run_if(in_state(GameState::Mission)))

        // Debug systems
        .add_systems(Update, (
            testing_spawn::goap_debug_display_system,
            testing_spawn::debug_selection_visual_system,
            testing_spawn::simple_visual_debug_system,
            testing_spawn::patrol_debug_system,
            testing_spawn::faction_visualization_system,
        ).run_if(in_state(GameState::Mission)))

        // Urban simulation
        .add_systems(Update, (
            urban_simulation::urban_civilian_spawn_system,
            urban_simulation::crowd_dynamics_system,
            urban_simulation::daily_routine_system,
            urban_simulation::urban_cleanup_system,
            urban_simulation::urban_debug_system,
            civilian_spawn::civilian_wander_system,
            civilian_spawn::civilian_cleanup_system,
        ).run_if(in_state(GameState::Mission)))

        // Police escalation
        .add_systems(Update, (
            police_escalation::police_incident_tracking_system,
            police_escalation::police_spawn_system,
            police_escalation::police_cleanup_system,
            police_escalation::police_deescalation_system,
            police_escalation::police_debug_system,
            explosions::explosion_damage_system,
            explosions::floating_text_system,
            explosions::handle_grenade_events,
            explosions::handle_vehicle_explosions,            
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
            npc_barks::chat_bubble_system,
            npc_barks::bark_cooldown_system,
            hacking_feedback::hack_progress_visualization,
            hacking_feedback::hack_status_indicator_system,
            hacking_feedback::device_visual_feedback_system,
            hacking_feedback::hack_interruption_system,
            hacking_feedback::hack_notification_system,            
        ).run_if(in_state(GameState::Mission)))

        // Post mission
        .add_systems(Update, (
            mission::process_mission_results,  
            ui::screens::post_mission_ui_system,
        ).run_if(in_state(GameState::PostMission)))
        
        .run();
}

pub fn dummy_fn() {

}

pub fn setup_mission_scene_optimized(
    mut commands: Commands,
    sprites: Res<GameSprites>,
    global_data: Res<GlobalData>,
    cities_db: Res<CitiesDatabase>,           // NEW
    cities_progress: Res<CitiesProgress>,     // NEW
    mut scene_cache: ResMut<SceneCache>,
    agents: Query<Entity, With<Agent>>,
) {
    // Clean up existing agents
    for entity in agents.iter() {
        if agents.get(entity).is_ok() {
            commands.entity(entity).despawn();
        }
    }

    // Get selected city context
    let selected_city = if !cities_progress.current_city.is_empty() {
        cities_db.get_city(&cities_progress.current_city)
    } else {
        None
    };

    let scene_name = if let Some(city) = selected_city {
        // Map cities to appropriate scenes
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
            spawn_from_scene_with_city(&mut commands, &scene, &*global_data, &sprites, selected_city);
            info!("Loaded scene: {} for city: {}", scene_name, 
                  selected_city.map_or("None", |c| &c.name));
        },
        None => {
            error!("Failed to load scene: {}. Creating fallback.", scene_name);
            spawn_fallback_mission(&mut commands, &*global_data, &sprites);
        }
    }
}

fn setup_camera_and_input(mut commands: Commands) {
    commands.spawn(Camera2d);
    
    let input_map = InputMap::default()
        .with(PlayerAction::Pause, KeyCode::Space)
        .with(PlayerAction::Select, MouseButton::Left)
        .with(PlayerAction::Move, MouseButton::Right)
        .with(PlayerAction::Neurovector, KeyCode::KeyN)
        .with(PlayerAction::Combat, KeyCode::KeyF)
        .with(PlayerAction::Interact, KeyCode::KeyE)
        .with(PlayerAction::Inventory, KeyCode::KeyI)
        .with(PlayerAction::Reload, KeyCode::KeyR);
    
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
    if let Some(loaded_data) = crate::systems::save::load_game() {
        let research_progress = loaded_data.research_progress.clone();
        (loaded_data, research_progress)
    } else {
        (GlobalData::default(), ResearchProgress::default())
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
