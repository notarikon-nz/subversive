// src/systems/interactive_decals_demo.rs - Demo system for testing interactive decals
use bevy::prelude::*;
use leafwing_input_manager::prelude::*;
use crate::core::*;
use crate::systems::*;
use crate::systems::explosions::*;

// === DEMO KEYBINDINGS ===

#[derive(Actionlike, PartialEq, Eq, Hash, Clone, Copy, Debug, Reflect)]
pub enum DecalDemoAction {
    SpawnOilSpill,
    SpawnGasSpill,
    SpawnElectricPuddle,
    SpawnFuelBarrel,
    CreateExplosion,
    IgniteAll,
    ClearDecals,
}

// === DEMO SYSTEM ===

pub fn interactive_decals_demo_system(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    input: Query<&ActionState<DecalDemoAction>>,
    cameras: Query<(&Camera, &GlobalTransform)>,
    windows: Query<&Window>,
    decal_query: Query<Entity, With<InteractiveDecal>>,
    explodable_query: Query<Entity, With<Explodable>>,
    fire_query: Query<Entity, With<OnFire>>,
) {
    let Ok(action_state) = input.single() else { return; };
    
    // Get mouse world position
    let Some(mouse_pos) = get_world_mouse_position(&windows, &cameras) else { return; };
    
    // Handle demo actions
    if action_state.just_pressed(&DecalDemoAction::SpawnOilSpill) {
        spawn_oil_spill(&mut commands, mouse_pos, 50.0);
    }
    
    if action_state.just_pressed(&DecalDemoAction::SpawnGasSpill) {
        spawn_gasoline_spill(&mut commands, mouse_pos, 45.0);
    }
    
    if action_state.just_pressed(&DecalDemoAction::SpawnElectricPuddle) {
        spawn_electric_puddle(&mut commands, mouse_pos, 40.0);
    }
    
    if action_state.just_pressed(&DecalDemoAction::SpawnFuelBarrel) {
        spawn_explodable(&mut commands, mouse_pos, ExplodableType::FuelBarrel);
    }
    
    if keyboard.just_pressed(KeyCode::Digit0) {
        spawn_explosion(
            &mut commands,
            mouse_pos,
            60.0,
            80.0,
            ExplosionType::Grenade,
        );
    }
    
    if keyboard.just_pressed(KeyCode::KeyF) {
        // Ignite all flammable decals for testing
        for entity in decal_query.iter() {
            commands.entity(entity).insert(OnFire {
                intensity: 1.0,
                spread_timer: 1.0,
                burn_timer: 30.0,
            });
        }
        info!("Ignited all flammable decals!");
    }
    
    if action_state.just_pressed(&DecalDemoAction::ClearDecals) {
        // Clean up all decals and explodables
        for entity in decal_query.iter() {
            commands.entity(entity).insert(MarkedForDespawn);
        }
        for entity in explodable_query.iter() {
            commands.entity(entity).insert(MarkedForDespawn);
        }
        for entity in fire_query.iter() {
            commands.entity(entity).insert(MarkedForDespawn);
        }
    }
}

// === DEMO SETUP ===

pub fn setup_interactive_decals_demo(mut commands: Commands) {
    // Create demo input map
    let input_map = InputMap::default()
        .with(DecalDemoAction::SpawnOilSpill, KeyCode::Digit1)
        .with(DecalDemoAction::SpawnGasSpill, KeyCode::Digit2)
        .with(DecalDemoAction::SpawnElectricPuddle, KeyCode::Digit3)
        .with(DecalDemoAction::SpawnFuelBarrel, KeyCode::Digit4)
        .with(DecalDemoAction::CreateExplosion, KeyCode::Digit5)
        .with(DecalDemoAction::IgniteAll, KeyCode::KeyF)
        .with(DecalDemoAction::ClearDecals, KeyCode::KeyC);
    
    commands.spawn((
        input_map,
        ActionState::<DecalDemoAction>::default(),
    ));
}

// === PRE-BUILT TEST SCENARIOS ===

/// Creates a pre-built test scenario for chain reactions
pub fn setup_chain_reaction_test_scenario(mut commands: Commands) {
    
    // Gas station scenario
    let center = Vec2::new(200.0, 100.0);
    
    // Central gasoline spill
    spawn_gasoline_spill(&mut commands, center, 60.0);
    
    // Surrounding fuel barrels
    for i in 0..6 {
        let angle = (i as f32 / 6.0) * std::f32::consts::TAU;
        let pos = center + Vec2::new(angle.cos(), angle.sin()) * 100.0;
        spawn_explodable(&mut commands, pos, ExplodableType::FuelBarrel);
    }
    
    // Oil spills connecting some barrels
    for i in 0..3 {
        let angle = (i as f32 / 3.0) * std::f32::consts::TAU;
        let start = center + Vec2::new(angle.cos(), angle.sin()) * 80.0;
        let end = center + Vec2::new((angle + 1.0).cos(), (angle + 1.0).sin()) * 80.0;
        create_gasoline_trail(&mut commands, start, end, 25.0);
    }
    
    // Some cars nearby with oil leaks
    for i in 0..4 {
        let pos = center + Vec2::new(
            (i as f32 * 80.0) - 120.0,
            200.0,
        );
        spawn_oil_spill(&mut commands, pos, 35.0);
    }
    
}

/// Creates an industrial zone with mixed hazards
pub fn setup_industrial_zone_scenario(mut commands: Commands) {
    
    let base = Vec2::new(-200.0, -100.0);
    
    // Large oil spill from "ruptured pipeline"
    spawn_oil_spill(&mut commands, base, 80.0);
    
    // Electrical hazards from "damaged transformer"
    spawn_electric_puddle(&mut commands, base + Vec2::new(100.0, 0.0), 60.0);
    spawn_electric_puddle(&mut commands, base + Vec2::new(120.0, 30.0), 40.0);
    
    // Gas canisters scattered around
    for i in 0..8 {
        let pos = base + Vec2::new(
            (rand::random::<f32>() - 0.5) * 200.0,
            (rand::random::<f32>() - 0.5) * 150.0,
        );
        spawn_explodable(&mut commands, pos, ExplodableType::GasCanister);
    }
    
    // Power cells near electrical area
    for i in 0..3 {
        let pos = base + Vec2::new(100.0 + i as f32 * 30.0, 60.0);
        spawn_explodable(&mut commands, pos, ExplodableType::PowerCell);
    }
    
}

// === PERFORMANCE TESTING ===

/// Stress test system that creates many decals
pub fn create_stress_test_scenario(mut commands: Commands) {
    
    let grid_size = 20;
    let spacing = 30.0;
    let start = Vec2::new(-300.0, -300.0);
    
    for x in 0..grid_size {
        for y in 0..grid_size {
            let pos = start + Vec2::new(x as f32 * spacing, y as f32 * spacing);
            
            match (x) % 4 {
                0 => { spawn_oil_spill(&mut commands, pos, 35.0); },
                1 => { spawn_gasoline_spill(&mut commands, pos, 30.0); },
                2 => { spawn_electric_puddle(&mut commands, pos, 30.0); },
                3 => { spawn_explodable(&mut commands, pos, ExplodableType::GasCanister); },
                _ => {}
            }
        }
    }
    
}
