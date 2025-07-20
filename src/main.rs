// src/main.rs - Clean version with core systems only
use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use leafwing_input_manager::prelude::*;

mod core;
mod systems;

use core::*;
use systems::*;
use systems::ai::*;
use pool::*;

fn main() {
    let (global_data, research_progress) = load_global_data_or_default();
    systems::scenes::ensure_scenes_directory();

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
        .register_type::<PlayerAction>()

        .init_state::<GameState>()

        .init_resource::<GameMode>()
        .init_resource::<SelectionState>()
        .init_resource::<MissionData>()
        .init_resource::<InventoryState>()
        .init_resource::<PostMissionResults>()
        .init_resource::<MissionState>()

        .insert_resource(global_data)
        .insert_resource(research_progress)
        .insert_resource(ResearchDatabase::load())
        .init_resource::<UIState>()
        .init_resource::<PostMissionProcessed>()
        .init_resource::<EntityPool>()
        .init_resource::<SelectionDrag>()
        .init_resource::<GoapConfig>()
        .init_resource::<HubState>()
        .init_resource::<UnlockedAttachments>()
        .init_resource::<ManufactureState>()

        .add_event::<ActionEvent>()
        .add_event::<CombatEvent>()
        .add_event::<AudioEvent>()
        .add_event::<AlertEvent>()

        .add_systems(Startup, (
            setup_camera_and_input,
            audio::setup_audio,
            sprites::load_sprites,
            setup_attachments,
            apply_loaded_research_benefits,
        ))

        .add_systems(Update, (
            input::handle_input,
            ui::screens::fps_system,
            pool::cleanup_inactive_entities,
            save::auto_save_system,
            save::save_input_system,
            audio::audio_system,
        ))

        .add_systems(OnEnter(GameState::PostMission), (
            ui::cleanup_mission_ui,
        ))

        .add_systems(OnEnter(GameState::GlobalMap), (
            ui::cleanup_global_map_ui,
            ui::reset_hub_to_global_map,
        ))
        .add_systems(Update, (
            ui::hub_system,
        ).run_if(in_state(GameState::GlobalMap)))

        .add_systems(OnEnter(GameState::Mission), (
            ui::cleanup_global_map_ui,
            setup_mission_scene,
        ))

        .add_systems(Update, (
            // Core gameplay systems
            camera::movement,
            selection::system,
            movement::system,                    // FIXED: Core movement system
            
            // AI systems
            goap::goap_ai_system,
            goap::goap_patrol_advancement_system, // ADDED: Patrol advancement
            ai::goap_sound_detection_system,
            ai::alert_system,
            ai::legacy_enemy_ai_system,
            ai::sound_detection_system,
            
            // Interaction systems
            neurovector::system,
            interaction::system,
            combat::system,
            combat::death_system,
            goap::goap_config_system,
            goap::goap_debug_system,
            
        ).run_if(in_state(GameState::Mission)))
        
        .add_systems(Update, (            
            ui::world::system,
            ui::screens::inventory_system,
            ui::screens::pause_system,
            mission::timer_system,
            mission::check_completion,
            mission::restart_system,
            cover::cover_management_system,
            cover::cover_exit_system,
            quicksave::quicksave_system,
            reload::reload_system,

        ).run_if(in_state(GameState::Mission)))

        .add_systems(Update, (
            mission::process_mission_results,  
            ui::screens::post_mission_ui_system,
        ).run_if(in_state(GameState::PostMission)))
        
        .run();
}

fn setup_mission_scene(
    mut commands: Commands,
    sprites: Res<GameSprites>,
    global_data: Res<GlobalData>,
    agents: Query<Entity, With<Agent>>,
) {
    // Clean up any existing agents first
    for entity in agents.iter() {
        commands.entity(entity).despawn();
    }
    
    let scene_name = match global_data.selected_region {
        0 => "mission1",
        1 => "mission2", 
        2 => "mission3",
        _ => "mission1",
    };
    
    let scene = crate::systems::scenes::load_scene(scene_name);
    crate::systems::scenes::spawn_from_scene(&mut commands, &scene, &*global_data, &sprites);
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