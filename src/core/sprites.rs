use bevy::prelude::*;
use crate::core::*;
use crate::systems::*;

#[derive(Resource)]
pub struct GameSprites {
    pub agent: Handle<Image>,
    pub civilian: Handle<Image>,
    pub enemy: Handle<Image>,
    pub terminal_objective: Handle<Image>,
    pub terminal_equipment: Handle<Image>,
    pub terminal_intel: Handle<Image>,
}

pub fn load_sprites(mut commands: Commands, asset_server: Res<AssetServer>) {
    let sprites = GameSprites {
        agent: asset_server.load("sprites/agent.png"),
        civilian: asset_server.load("sprites/civilian.png"),
        enemy: asset_server.load("sprites/enemy.png"),
        terminal_objective: asset_server.load("sprites/terminal_red.png"),
        terminal_equipment: asset_server.load("sprites/terminal_blue.png"),
        terminal_intel: asset_server.load("sprites/terminal_green.png"),
    };
    
    commands.insert_resource(sprites);
}

pub fn spawn_initial_scene(
    mut commands: Commands,
    sprites: Res<GameSprites>,
    global_data: Res<GlobalData>,
) {
    let scene = crate::systems::scenes::load_scene("mission1");
    crate::systems::scenes::spawn_from_scene(&mut commands, &scene, &global_data, &sprites);
}

// Update spawn functions to use sprites instead of colored rectangles
pub fn create_agent_sprite(sprites: &GameSprites) -> SpriteBundle {
    SpriteBundle {
        texture: sprites.agent.clone(),
        sprite: Sprite {
            custom_size: Some(Vec2::new(24.0, 24.0)),
            color: Color::srgb(0.2, 0.8, 0.2), // Fallback color
            ..default()
        },
        ..default()
    }
}

pub fn create_civilian_sprite(sprites: &GameSprites) -> SpriteBundle {
    SpriteBundle {
        texture: sprites.civilian.clone(),
        sprite: Sprite {
            custom_size: Some(Vec2::new(18.0, 18.0)),
            color: Color::srgb(0.8, 0.8, 0.2), // Fallback color
            ..default()
        },
        ..default()
    }
}

pub fn create_enemy_sprite(sprites: &GameSprites) -> SpriteBundle {
    SpriteBundle {
        texture: sprites.enemy.clone(),
        sprite: Sprite {
            custom_size: Some(Vec2::new(22.0, 22.0)),
            color: Color::srgb(0.8, 0.2, 0.2), // Fallback color
            ..default()
        },
        ..default()
    }
}

pub fn create_terminal_sprite(sprites: &GameSprites, terminal_type: &crate::core::TerminalType) -> SpriteBundle {
    let (texture, fallback_color) = match terminal_type {
        crate::core::TerminalType::Objective => (&sprites.terminal_objective, Color::srgb(0.9, 0.2, 0.2)),
        crate::core::TerminalType::Equipment => (&sprites.terminal_equipment, Color::srgb(0.2, 0.5, 0.9)),
        crate::core::TerminalType::Intel => (&sprites.terminal_intel, Color::srgb(0.2, 0.8, 0.3)),
    };
    
    SpriteBundle {
        texture: texture.clone(),
        sprite: Sprite {
            custom_size: Some(Vec2::new(20.0, 20.0)),
            color: fallback_color, // This will show if texture fails to load
            ..default()
        },
        ..default()
    }
}

