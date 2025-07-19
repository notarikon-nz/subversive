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
        .add_plugins(RapierDebugRenderPlugin::default())
        .add_plugins(InputManagerPlugin::<PlayerAction>::default())
        .init_state::<GameState>()
        .init_resource::<GameMode>()
        .init_resource::<SelectionState>()
        .init_resource::<MissionData>()
        .init_resource::<InventoryState>()
        .init_resource::<PostMissionResults>()
        .insert_resource(global_data)                    // Insert loaded global data
        .insert_resource(research_progress)              // Insert loaded research progress
        .insert_resource(ResearchDatabase::load())
        .init_resource::<ResearchProgress>()        
        .init_resource::<UIState>()
        .init_resource::<PostMissionProcessed>()
        .init_resource::<EntityPool>()
        .init_resource::<SelectionDrag>()
        .init_resource::<GoapConfig>()
        .init_resource::<HubState>()
        .init_resource::<UnlockedAttachments>()
        .init_resource::<ManufactureState>()
        .init_resource::<ResearchProgress>()
        .add_event::<ActionEvent>()
        .add_event::<CombatEvent>()
        .add_event::<AudioEvent>()
        .add_event::<AlertEvent>()
        .add_systems(Startup, (
            setup_camera_and_input,
            setup_physics, 
            audio::setup_audio,
            sprites::load_sprites,
            setup_attachments,
            apply_loaded_research_benefits,
        ))
        .add_systems(Update, (
            sprites::spawn_initial_scene.run_if(resource_exists::<GameSprites>).run_if(run_once()),
            input::handle_input,
            ui::screens::fps_system,
            pool::cleanup_inactive_entities,
            save::auto_save_system,
            save::save_input_system,
            audio::audio_system,
            goap::goap_config_system,
            debug_research_system,
        ))
        .add_systems(OnEnter(GameState::PostMission), (
            ui::cleanup_mission_ui,
        ))
        .add_systems(OnEnter(GameState::GlobalMap), (
            ui::cleanup_global_map_ui,
            ui::reset_hub_to_global_map,
        ))
        .add_systems(OnEnter(GameState::Mission), (
            ui::cleanup_global_map_ui,
        ))        
        .add_systems(Update, (
            ui::hub_system,
        ).run_if(in_state(GameState::GlobalMap)))

        .add_systems(Update, (
            camera::movement,
            selection::system,
            movement::system,
            goap::goap_ai_system,
            ai::goap_sound_detection_system,
            ai::alert_system,
            ai::legacy_enemy_ai_system,
            ai::sound_detection_system,
            neurovector::system,
            interaction::system,
            combat::system,
            combat::death_system,
        ).run_if(in_state(GameState::Mission)))
        .add_systems(Update, (            
            ui::world::system,
            ui::screens::inventory_system,
            ui::screens::pause_system,
            mission::timer_system,
            mission::check_completion,
            mission::restart_system,
            goap::goap_debug_system,     
            goap::apply_goap_config_system, 
            cover::cover_management_system,
            cover::cover_exit_system,
            quicksave::quicksave_system,
        ).run_if(in_state(GameState::Mission)))

        .add_systems(Update, (
            mission::process_mission_results,  
            ui::screens::post_mission_ui_system,
        ).run_if(in_state(GameState::PostMission)))
        .run();
}

fn setup_physics(mut rapier_config: ResMut<RapierConfiguration>) {
    rapier_config.gravity = Vec2::ZERO;
}

// Split setup into two parts:
fn setup_camera_and_input(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
    
    commands.spawn(InputManagerBundle::<PlayerAction> {
        input_map: InputMap::default()
            .insert(PlayerAction::Pause, KeyCode::Space)
            .insert(PlayerAction::Select, MouseButton::Left)
            .insert(PlayerAction::Move, MouseButton::Right)
            .insert(PlayerAction::Neurovector, KeyCode::KeyN)
            .insert(PlayerAction::Combat, KeyCode::KeyF)
            .insert(PlayerAction::Interact, KeyCode::KeyE)
            .insert(PlayerAction::Inventory, KeyCode::KeyI)
            .build(),
        ..default()
    });
}

fn setup_attachments(mut commands: Commands) {
    let attachment_db = AttachmentDatabase::load();
    
    // Start with basic attachments unlocked
    let mut unlocked = UnlockedAttachments::default();
    unlocked.attachments.insert("red_dot".to_string());
    unlocked.attachments.insert("iron_sights".to_string());
    unlocked.attachments.insert("tactical_grip".to_string());
    
    let attachment_count = attachment_db.attachments.len();
    commands.insert_resource(attachment_db);
    commands.insert_resource(unlocked);
    
    info!("Attachment system initialized with {} attachments", attachment_count);
}


fn load_global_data_or_default() -> (GlobalData, ResearchProgress) {
    if let Some(loaded_data) = crate::systems::save::load_game() {
        let research_progress = loaded_data.research_progress.clone();
        info!("Save file loaded successfully! Day {}, Credits: {}, Research: {} projects", 
              loaded_data.current_day, 
              loaded_data.credits,
              research_progress.completed.len());
        (loaded_data, research_progress)
    } else {
        info!("No save file found, starting new game");
        (GlobalData::default(), ResearchProgress::default())
    }
}

fn apply_loaded_research_benefits(
    global_data: Res<GlobalData>,
    research_db: Res<ResearchDatabase>,
    mut unlocked_attachments: ResMut<UnlockedAttachments>,
) {
    // Apply all research benefits from loaded save data
    apply_research_unlocks(
        &global_data.research_progress,
        &research_db,
        &mut unlocked_attachments,
        // Note: We can't mutate global_data here since it's Res, not ResMut
        // But weapon/tool unlocks don't need to modify GlobalData
    );
    
    info!("Applied research benefits for {} completed projects", 
          global_data.research_progress.completed.len());
}

// TEMPORARY to debug research state
pub fn debug_research_system(
    input: Res<ButtonInput<KeyCode>>,
    global_data: Res<GlobalData>,
    research_db: Res<ResearchDatabase>,
) {
    if input.just_pressed(KeyCode::F9) {
        info!("=== RESEARCH DEBUG ===");
        info!("Completed projects: {:?}", global_data.research_progress.completed);
        info!("Credits invested: {}", global_data.research_progress.credits_invested);
        
        let available = research_db.get_available_projects(&global_data.research_progress);
        info!("Available projects: {}", available.len());
        
        let completed = research_db.get_completed_projects(&global_data.research_progress);
        info!("Completed projects: {}", completed.len());
        for project in completed {
            info!("  - {}", project.name);
        }
    }
}