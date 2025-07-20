// src/main.rs - Updated for Bevy 0.16 + latest dependencies
use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use leafwing_input_manager::prelude::*;

mod core;
mod systems;

use core::*;
use systems::*;
use systems::ai::*;
use pool::*;

// Resource to track if initial scene has been spawned
#[derive(Resource, Default)]
pub struct InitialSceneSpawned(pub bool);

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

        .init_resource::<ResearchProgress>()        
        .init_resource::<UIState>()
        .init_resource::<PostMissionProcessed>()
        .init_resource::<EntityPool>()
        .init_resource::<SelectionDrag>()
        .init_resource::<GoapConfig>()
        .init_resource::<HubState>()
        .init_resource::<UnlockedAttachments>()
        .init_resource::<ManufactureState>()
        .init_resource::<InitialSceneSpawned>() // NEW

        .add_event::<ActionEvent>()
        .add_event::<CombatEvent>()
        .add_event::<AudioEvent>()
        .add_event::<AlertEvent>()

       .add_systems(Startup, (
            setup_camera_and_input,
            audio::setup_audio,
            sprites::load_sprites,
            // sprites::spawn_initial_scene, // UGH
            setup_attachments,
            apply_loaded_research_benefits,
        ))

        .add_systems(Update, (
            spawn_scene_simple,
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
        .add_systems(Update, (
            ui::hub_system,
        ).run_if(in_state(GameState::GlobalMap)))


        .add_systems(OnEnter(GameState::Mission), (
            ui::cleanup_global_map_ui,
            reset_initial_scene_flag,   // Reset the flag when entering mission
        ))

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
            debug_entity_counts,
        ).run_if(in_state(GameState::Mission)))

        .add_systems(Update, (
            mission::process_mission_results,  
            ui::screens::post_mission_ui_system,
        ).run_if(in_state(GameState::PostMission)))
        
        .run();
}

fn spawn_scene_simple(
    mut commands: Commands,
    sprites: Res<GameSprites>,
    global_data: Res<GlobalData>,
    mut spawned: ResMut<InitialSceneSpawned>,
    agents: Query<Entity, With<Agent>>,
) {
    // Don't spawn if already spawned or agents exist
    if spawned.0 || !agents.is_empty() {
        return;
    }
    
    info!("Spawning initial scene...");
    
    let scene_name = match global_data.selected_region {
        0 => "mission1",
        1 => "mission2", 
        2 => "mission3",
        _ => "mission1",
    };
    
    let scene = crate::systems::scenes::load_scene(scene_name);
    crate::systems::scenes::spawn_from_scene(&mut commands, &scene, &*global_data, &sprites);
    
    spawned.0 = true;
    info!("Initial scene spawned!");
}
fn reset_initial_scene_flag(
    mut spawned: ResMut<InitialSceneSpawned>,
) {
    spawned.0 = false;
    info!("Reset initial scene flag for new mission");
}

// FIXED: Input setup for leafwing-input-manager 0.17.1
fn setup_camera_and_input(mut commands: Commands) {
    commands.spawn(Camera2d);
    
    // Updated: Use InputMap directly instead of deprecated InputManagerBundle
    let input_map = InputMap::default()
        .with(PlayerAction::Pause, KeyCode::Space)
        .with(PlayerAction::Select, MouseButton::Left)
        .with(PlayerAction::Move, MouseButton::Right)
        .with(PlayerAction::Neurovector, KeyCode::KeyN)
        .with(PlayerAction::Combat, KeyCode::KeyF)
        .with(PlayerAction::Interact, KeyCode::KeyE)
        .with(PlayerAction::Inventory, KeyCode::KeyI);
    
    commands.spawn((
        input_map,
        ActionState::<PlayerAction>::default(),
    ));
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
    apply_research_unlocks(
        &global_data.research_progress,
        &research_db,
        &mut unlocked_attachments,
    );
    
    info!("Applied research benefits for {} completed projects", 
          global_data.research_progress.completed.len());
}

fn debug_entity_counts(
    agents: Query<Entity, With<Agent>>,
    enemies: Query<Entity, With<Enemy>>,
    civilians: Query<Entity, With<Civilian>>,
    terminals: Query<Entity, With<Terminal>>,
    game_state: Res<State<GameState>>,
) {
    if *game_state.get() == GameState::Mission {
        static mut LAST_COUNT_TIME: f32 = 0.0;
        static mut FRAME_COUNT: u32 = 0;
        
        unsafe {
            FRAME_COUNT += 1;
            if FRAME_COUNT % 60 == 0 { // Every second
                let agent_count = agents.iter().count();
                let enemy_count = enemies.iter().count();
                let civilian_count = civilians.iter().count();
                let terminal_count = terminals.iter().count();
                
                info!("Entity counts - Agents: {}, Enemies: {}, Civilians: {}, Terminals: {}", 
                      agent_count, enemy_count, civilian_count, terminal_count);
            }
        }
    }
}