// src/core/fonts.rs - Custom font system for Bevy 0.16.1
use bevy::prelude::*;
use crate::core::*;


#[derive(Resource)]
pub struct GameFonts {
    pub main_font: Handle<Font>,
    pub ui_font: Handle<Font>,
    pub monospace_font: Handle<Font>,
}

pub fn load_fonts(mut commands: Commands, asset_server: Res<AssetServer>) {
    info!("Loading custom fonts...");
    
    let fonts = GameFonts {
        // Load your custom fonts from the assets/fonts/ directory
        main_font: asset_server.load("fonts/orbitron.ttf"),
        ui_font: asset_server.load("fonts/cousine.ttf"), 
        monospace_font: asset_server.load("fonts/courier_prime.ttf"),
    };
    
    commands.insert_resource(fonts);
    info!("Custom fonts loaded!");
}

// Helper function to create text with custom font
pub fn create_text_with_font(
    text: &str,
    font: Handle<Font>,
    font_size: f32,
    color: Color,
) -> (Text, TextFont, TextColor) {
    (
        Text::new(text),
        TextFont { 
            font,
            font_size,
            ..default()
        },
        TextColor(color),
    )
}

// Convenient text creation functions
pub fn create_title_text(text: &str, fonts: &GameFonts) -> (Text, TextFont, TextColor) {
    create_text_with_font(text, fonts.main_font.clone(), 32.0, Color::WHITE)
}

pub fn create_ui_text(text: &str, fonts: &GameFonts) -> (Text, TextFont, TextColor) {
    create_text_with_font(text, fonts.ui_font.clone(), 16.0, Color::WHITE)
}

pub fn create_monospace_text(text: &str, fonts: &GameFonts) -> (Text, TextFont, TextColor) {
    create_text_with_font(text, fonts.monospace_font.clone(), 14.0, Color::srgb(0.8, 0.8, 0.8))
}

// Update existing UI systems to use custom fonts
pub fn update_hub_header_with_font(
    parent: &mut ChildSpawnerCommands, 
    global_data: &GlobalData,
    fonts: &GameFonts,
) {
    parent.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Px(80.0),
            flex_shrink: 0.0,
            justify_content: JustifyContent::SpaceBetween,
            align_items: AlignItems::Center,
            padding: UiRect::all(Val::Px(20.0)),
            ..default()
        },
        BackgroundColor(Color::srgb(0.15, 0.15, 0.25)),
    )).with_children(|header| {
        // Use custom font for title
        let (text, font, color) = create_title_text("SUBVERSIVE", fonts);
        header.spawn((text, font, color));
        
        header.spawn(Node {
            flex_direction: FlexDirection::Row,
            column_gap: Val::Px(30.0),
            ..default()
        }).with_children(|info| {
            let (day_text, day_font, day_color) = create_ui_text(
                &format!("Day {}", global_data.current_day), 
                fonts
            );
            info.spawn((day_text, day_font, day_color));
            
            let (credits_text, credits_font, credits_color) = create_text_with_font(
                &format!("Credits: {}", global_data.credits),
                fonts.ui_font.clone(),
                18.0,
                Color::srgb(0.8, 0.8, 0.2)
            );
            info.spawn((credits_text, credits_font, credits_color));
        });
    });
}

// Example of updating existing UI with fonts
impl GameFonts {
    pub fn spawn_text(&self, parent: &mut ChildSpawnerCommands, text: &str, style: FontStyle) {
        let (text_component, font_component, color_component) = match style {
            FontStyle::Title => create_title_text(text, self),
            FontStyle::UI => create_ui_text(text, self),
            FontStyle::Monospace => create_monospace_text(text, self),
            FontStyle::Custom { font_size, color } => create_text_with_font(
                text, 
                self.main_font.clone(), 
                font_size, 
                color
            ),
        };
        
        parent.spawn((text_component, font_component, color_component));
    }
}

#[derive(Clone)]
pub enum FontStyle {
    Title,
    UI,
    Monospace,
    Custom { font_size: f32, color: Color },
}

// Font loading state to ensure fonts are loaded before UI
#[derive(Resource, Default)]
pub struct FontsLoaded(pub bool);

pub fn check_fonts_loaded(
    fonts: Option<Res<GameFonts>>,
    mut fonts_loaded: ResMut<FontsLoaded>,
) {
    if fonts.is_some() && !fonts_loaded.0 {
        fonts_loaded.0 = true;
        info!("Fonts are ready for use!");
    }
}