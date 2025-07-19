use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use leafwing_input_manager::prelude::*;

mod core;
mod systems;

use core::*;
use systems::*;
use systems::ai::*;
use pool::*;

fn load_global_data_or_default() -> GlobalData {
    if let Some(loaded_data) = crate::systems::save::load_game() {
        info!("Save file loaded successfully! Day {}, Credits: {}", 
              loaded_data.current_day, loaded_data.credits);
        loaded_data
    } else {
        info!("No save file found, starting new game");
        GlobalData::default()
    }
}

fn main() {
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
        .insert_resource(load_global_data_or_default())
        .init_resource::<UIState>()
        .init_resource::<PostMissionProcessed>()
        .init_resource::<EntityPool>()
        .init_resource::<SelectionDrag>()
        .init_resource::<GoapConfig>()
        .init_resource::<HubState>()
        .init_resource::<UnlockedAttachments>()
        .init_resource::<ManufactureState>()  // Add manufacture state        
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