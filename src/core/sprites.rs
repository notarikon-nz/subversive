use bevy::prelude::*;

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
    info!("Loading sprites...");
    
    let sprites = GameSprites {
        agent: asset_server.load("sprites/agent.png"),
        civilian: asset_server.load("sprites/civilian.png"),
        enemy: asset_server.load("sprites/enemy.png"),
        terminal_objective: asset_server.load("sprites/terminal_red.png"),
        terminal_equipment: asset_server.load("sprites/terminal_blue.png"),
        terminal_intel: asset_server.load("sprites/terminal_green.png"),
    };
    
    commands.insert_resource(sprites);
    info!("Sprites resource created!");
}

// FIXED: Much more visible fallback sprites with borders
pub fn create_agent_sprite(sprites: &GameSprites) -> (Sprite, Transform) {
    (
        Sprite {
            image: sprites.agent.clone(),
            custom_size: Some(Vec2::new(24.0, 24.0)),
            color: Color::srgb(0.1, 1.0, 0.1), // BRIGHT GREEN - very visible
            ..default()
        },
        Transform::default(),
    )
}

pub fn create_civilian_sprite(sprites: &GameSprites) -> (Sprite, Transform) {
    (
        Sprite {
            image: sprites.civilian.clone(),
            custom_size: Some(Vec2::new(18.0, 18.0)),
            color: Color::srgb(1.0, 1.0, 0.1), // BRIGHT YELLOW - very visible
            ..default()
        },
        Transform::default(),
    )
}

pub fn create_enemy_sprite(sprites: &GameSprites) -> (Sprite, Transform) {
    (
        Sprite {
            image: sprites.enemy.clone(),
            custom_size: Some(Vec2::new(22.0, 22.0)),
            color: Color::srgb(1.0, 0.1, 0.1), // BRIGHT RED - very visible
            ..default()
        },
        Transform::default(),
    )
}

pub fn create_terminal_sprite(sprites: &GameSprites, terminal_type: &crate::core::TerminalType) -> (Sprite, Transform) {
    let (texture, fallback_color) = match terminal_type {
        crate::core::TerminalType::Objective => (&sprites.terminal_objective, Color::srgb(1.0, 0.1, 0.1)),
        crate::core::TerminalType::Equipment => (&sprites.terminal_equipment, Color::srgb(0.1, 0.5, 1.0)),
        crate::core::TerminalType::Intel => (&sprites.terminal_intel, Color::srgb(0.1, 1.0, 0.3)),
    };
    
    (
        Sprite {
            image: texture.clone(),
            custom_size: Some(Vec2::new(20.0, 20.0)),
            color: fallback_color,
            ..default()
        },
        Transform::default(),
    )
}

// NEW: Create a simple colored rectangle sprite as a guaranteed fallback
pub fn create_colored_rectangle(size: Vec2, color: Color) -> Sprite {
    Sprite {
        color,
        custom_size: Some(size),
        ..default()
    }
}
