use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use leafwing_input_manager::prelude::*;  // Temporarily disabled

mod components;
mod systems;
mod states;
mod resources;
mod events;

use components::*;
use systems::*;
use states::*;
use resources::*;
use events::*;

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
        .init_state::<MissionState>()
        .init_resource::<GlobalGameData>()
        .init_resource::<MissionData>()
        .init_resource::<SelectionState>()
        .init_resource::<NeurovectorTargeting>()
        .init_resource::<InteractionState>()
        .init_resource::<InventoryState>()
        .init_resource::<CombatTargeting>()
        .add_event::<AgentActionEvent>()
        .add_event::<MissionEvent>()
        .add_event::<AlertEvent>()
        .add_event::<NeurovectorEvent>()
        .add_event::<InteractionEvent>()
        .add_event::<InteractionCompleteEvent>()
        .add_event::<DetectionEvent>()
        .add_event::<CombatEvent>()
        .add_event::<DeathEvent>()
        .add_systems(Startup, (
            setup_camera,
            setup_input,  // Temporarily disabled
            spawn_test_mission,
        ))
        .add_systems(Update, (
            // Core systems that always run
            handle_pause_input,
            camera_movement,
            selection_system,
            inventory_ui_render_system,
            equipment_notification_system,            
            // Mission-specific systems
            agent_movement_system,
            agent_action_system,
        ).run_if(in_state(GameState::Mission)))
        .add_systems(Update, (
            interaction_detection_system,
            interaction_system,
            interaction_progress_system,
            interaction_visual_system,
            inventory_management_system,
            inventory_ui_system,
            enemy_vision_visual_system,
            neurovector_system,
            neurovector_targeting_system,
            neurovector_cooldown_system,
            neurovector_visual_system,
            controlled_civilian_visual_system,
            mission_timer_system,
            visibility_system,
            alert_system,
        ).run_if(in_state(GameState::Mission)))
        .add_systems(Update, (
            // Paused state systems
            pause_ui_system,
            queued_orders_system,
        ).run_if(in_state(GameState::Mission).and_then(in_state(MissionState::Paused))))
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

fn setup_input(mut commands: Commands) {
    commands.spawn(InputManagerBundle::<PlayerAction> {
        input_map: InputMap::default()
            .insert(PlayerAction::Pause, KeyCode::Space)
            .insert(PlayerAction::Select, MouseButton::Left)
            .insert(PlayerAction::Move, MouseButton::Right)
            .build(),
        ..default()
    });
}