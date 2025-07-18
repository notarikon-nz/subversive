use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use leafwing_input_manager::prelude::*;

mod core;
mod systems;

use core::*;
use systems::*;
use pool::*;

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
        .init_resource::<GlobalData>()
        .init_resource::<UIState>()
        .init_resource::<PostMissionProcessed>()
        .init_resource::<EntityPool>()
        .add_event::<ActionEvent>()
        .add_event::<CombatEvent>()
        .add_event::<AudioEvent>()
        .add_systems(Startup, (
            setup_camera_and_input,
            setup_physics, 
            audio::setup_audio,
            sprites::load_sprites))
        .add_systems(Update, (
            sprites::spawn_initial_scene.run_if(resource_exists::<GameSprites>).run_if(run_once()),
            input::handle_input,
            ui::fps_system,
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
        ))
        .add_systems(OnEnter(GameState::Mission), (
            ui::cleanup_global_map_ui,  // Clean up when starting mission too
        ))        
        .add_systems(Update, (
            ui::global_map_system,
        ).run_if(in_state(GameState::GlobalMap)))

        .add_systems(Update, (
            camera::movement,
            selection::system,
            movement::system,
            ai::enemy_ai_system,        
            ai::sound_detection_system, 
            neurovector::system,
            interaction::system,
            combat::system,
            combat::death_system,
            ui::system,              // gizmos/world-space UI
            ui::inventory_system,    
            ui::pause_system,        // bevy_ui
            ui::fps_system,          // simple toggle system
            mission::timer_system,
            mission::check_completion,
            mission::restart_system,
        ).run_if(in_state(GameState::Mission)))

        .add_systems(Update, (
            mission::process_mission_results,  // Separated from UI
            ui::post_mission_ui_system,        // New clean UI system
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

/*
fn setup(mut commands: Commands, sprites: Res<GameSprites>) {
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

    // Load scene instead of hardcoded spawning
    let global_data = GlobalData::default();
    let scene = systems::scenes::load_scene("mission1");
    systems::scenes::spawn_from_scene(&mut commands, &scene, &global_data, &sprites);
}
*/


/*
fn spawn_terminals(commands: &mut Commands) {
    let terminals = [
        (Vec3::new(320.0, -50.0, 1.0), Color::srgb(0.9, 0.2, 0.2), TerminalType::Objective),
        (Vec3::new(150.0, -80.0, 1.0), Color::srgb(0.2, 0.5, 0.9), TerminalType::Equipment),
        (Vec3::new(50.0, 120.0, 1.0), Color::srgb(0.2, 0.8, 0.3), TerminalType::Intel),
    ];

    for (pos, color, terminal_type) in terminals {
        commands.spawn((
            SpriteBundle {
                sprite: Sprite {
                    color,
                    custom_size: Some(Vec2::new(20.0, 20.0)),
                    ..default()
                },
                transform: Transform::from_translation(pos),
                ..default()
            },
            Terminal { terminal_type, range: 30.0, accessed: false },
            Selectable { radius: 15.0 },
        ));
    }
}
    */