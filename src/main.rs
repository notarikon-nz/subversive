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
        .add_systems(Startup, setup_physics)
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
        .add_systems(Startup, (setup, setup_physics))
        .add_systems(Update, (
            input::handle_input,
            ui::fps_system,
            pool::cleanup_inactive_entities,
            save::auto_save_system,
            save::save_input_system,            
        ))
        .add_systems(Update, (
            ui::global_map_system,
        ).run_if(in_state(GameState::GlobalMap)))
        .add_systems(Update, (
            camera::movement,
            selection::system,
            movement::system,
            neurovector::system,
            interaction::system,
            combat::system,
            combat::death_system,
            ui::system,
            ui::inventory_system,
            ui::pause_system,
            mission::timer_system,
            mission::check_completion,
            mission::restart_system,
        ).run_if(in_state(GameState::Mission)))
        .add_systems(Update, (
            mission::post_mission_system, 
        ).run_if(in_state(GameState::PostMission)))
        .run();
}

fn setup_physics(mut rapier_config: ResMut<RapierConfiguration>) {
    rapier_config.gravity = Vec2::ZERO;
}

fn setup(mut commands: Commands) {
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
    systems::scenes::spawn_from_scene(&mut commands, &scene, &global_data);
}

/*
fn setup(mut commands: Commands) {
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

    // Spawn test scenario
    let global_data = load_game().unwrap_or_default();
    commands.insert_resource(global_data.clone());

    spawn_agents(&mut commands, 3, &global_data);
    spawn_civilians(&mut commands, 5);
    spawn_enemy(&mut commands, &global_data);
    spawn_terminals(&mut commands);

}


fn spawn_agents(commands: &mut Commands, count: usize, global_data: &GlobalData) {
    for i in 0..count {
        commands.spawn((
            SpriteBundle {
                sprite: Sprite {
                    color: Color::srgb(0.2, 0.8, 0.2),
                    custom_size: Some(Vec2::new(20.0, 20.0)),
                    ..default()
                },
                transform: Transform::from_translation(Vec3::new(
                    -200.0 + i as f32 * 50.0,
                    0.0,
                    1.0,
                )),
                ..default()
            },
            Agent {experience:0, level:1 },
            Health(100.0),
            MovementSpeed(150.0),
            Controllable,
            Selectable { radius: 15.0 },
            Vision::new(150.0, 60.0),
            NeurovectorCapability::default(),
            Inventory::default(),
            RigidBody::Dynamic,
            Collider::ball(10.0),
            Velocity::default(),
            Damping { linear_damping: 10.0, angular_damping: 10.0 },
        ));
    }
}

fn spawn_civilians(commands: &mut Commands, count: usize) {
    for i in 0..count {
        commands.spawn((
            SpriteBundle {
                sprite: Sprite {
                    color: Color::srgb(0.8, 0.8, 0.2),
                    custom_size: Some(Vec2::new(15.0, 15.0)),
                    ..default()
                },
                transform: Transform::from_translation(Vec3::new(
                    100.0 + i as f32 * 60.0,
                    100.0 + (i as f32 * 20.0).sin() * 50.0,
                    1.0,
                )),
                ..default()
            },
            Civilian,
            Health(50.0),
            MovementSpeed(100.0),
            Controllable,
            NeurovectorTarget,
            RigidBody::Dynamic,
            Collider::ball(7.5),
            Velocity::default(),
            Damping { linear_damping: 10.0, angular_damping: 10.0 },
        ));
    }
}

fn spawn_enemy(commands: &mut Commands, global_data: &GlobalData) {
    let region = &global_data.regions[global_data.selected_region];
    let difficulty = region.mission_difficulty_modifier();
    
    let enemy_health = 100.0 * difficulty;
    let enemy_speed = 120.0 * difficulty;
    let vision_range = 120.0 * difficulty;
    
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: alert_color(region.alert_level),
                custom_size: Some(Vec2::new(18.0, 18.0)),
                ..default()
            },
            transform: Transform::from_translation(Vec3::new(200.0, -100.0, 1.0)),
            ..default()
        },
        Enemy,
        Health(enemy_health),
        MovementSpeed(enemy_speed),
        Vision::new(vision_range, 45.0),
        Patrol::new(vec![
            Vec2::new(200.0, -100.0),
            Vec2::new(300.0, -100.0),
            Vec2::new(300.0, 50.0),
            Vec2::new(200.0, 50.0),
        ]),
        RigidBody::Dynamic,
        Collider::ball(9.0),
        Velocity::default(),
        Damping { linear_damping: 10.0, angular_damping: 10.0 },
    ));
}

fn alert_color(alert_level: AlertLevel) -> Color {
    match alert_level {
        AlertLevel::Green => Color::srgb(0.8, 0.2, 0.2),      // Normal red
        AlertLevel::Yellow => Color::srgb(0.8, 0.5, 0.2),     // Orange-red
        AlertLevel::Orange => Color::srgb(0.8, 0.8, 0.2),     // Yellow-red  
        AlertLevel::Red => Color::srgb(1.0, 0.2, 0.2),        // Bright red
    }
}
*/

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