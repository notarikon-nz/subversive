// src/systems/ui/layout/constants.rs - UI Constants and Helper Functions
use bevy::prelude::*;

// === LAYOUT CONSTANTS ===
pub const PADDING_SMALL: Val = Val::Px(10.0);
pub const PADDING_MEDIUM: Val = Val::Px(20.0);
pub const PADDING_LARGE: Val = Val::Px(30.0);

pub const SPACING_TINY: Val = Val::Px(5.0);
pub const SPACING_SMALL: Val = Val::Px(10.0);
pub const SPACING_MEDIUM: Val = Val::Px(20.0);
pub const SPACING_LARGE: Val = Val::Px(50.0);

pub const BUTTON_HEIGHT_SMALL: Val = Val::Px(30.0);
pub const BUTTON_HEIGHT_MEDIUM: Val = Val::Px(50.0);
pub const BUTTON_HEIGHT_LARGE: Val = Val::Px(70.0);

pub const PANEL_WIDTH_SMALL: Val = Val::Px(300.0);
pub const PANEL_WIDTH_MEDIUM: Val = Val::Px(500.0);
pub const PANEL_WIDTH_LARGE: Val = Val::Px(800.0);

// === COLOR CONSTANTS ===
pub const COLOR_CYBERPUNK_YELLOW: Color = Color::srgb(0.99, 1.0, 0.32);
pub const COLOR_CYBERPUNK_CYAN: Color = Color::srgb(0.0, 1.0, 1.0);
pub const COLOR_CYBERPUNK_MAGENTA: Color = Color::srgb(1.0, 0.0, 0.59);
pub const COLOR_DARK_BACKGROUND: Color = Color::srgba(0.1, 0.1, 0.2, 0.9);
pub const COLOR_PANEL_BACKGROUND: Color = Color::srgba(0.15, 0.15, 0.3, 0.8);
pub const COLOR_BUTTON_BACKGROUND: Color = Color::srgba(0.2, 0.2, 0.4, 0.8);
pub const COLOR_BUTTON_HOVER: Color = Color::srgba(0.3, 0.3, 0.5, 0.8);

// === FONT CONSTANTS ===
pub const DEFAULT_FONT_PATH: &str = "fonts/orbitron.ttf";
pub const DEFAULT_FONT_SIZE: f32 = 16.0;
pub const TITLE_FONT_SIZE: f32 = 48.0;
pub const SUBTITLE_FONT_SIZE: f32 = 24.0;
pub const HEADER_FONT_SIZE: f32 = 18.0;
pub const BODY_FONT_SIZE: f32 = 14.0;

// === BORDER CONSTANTS ===
pub const BORDER_RADIUS_SMALL: Val = Val::Px(4.0);
pub const BORDER_RADIUS_MEDIUM: Val = Val::Px(8.0);
pub const BORDER_RADIUS_LARGE: Val = Val::Px(16.0);

// === HELPER FUNCTIONS ===

/// Create a standard text bundle with cyberpunk styling
pub fn create_text(text: &str, font: Handle<Font>, size: f32, color: Color) -> impl Bundle {
    (
        Text::new(text),
        TextFont { font, font_size: size, ..default() },
        TextColor(color),
    )
}

/// Create title text with standard cyberpunk yellow styling
pub fn create_title_text(text: &str, font: Handle<Font>) -> impl Bundle {
    create_text(text, font, TITLE_FONT_SIZE, COLOR_CYBERPUNK_YELLOW)
}

/// Create subtitle text with cyan styling
pub fn create_subtitle_text(text: &str, font: Handle<Font>) -> impl Bundle {
    create_text(text, font, SUBTITLE_FONT_SIZE, COLOR_CYBERPUNK_CYAN)
}

/// Create header text with magenta styling
pub fn create_header_text(text: &str, font: Handle<Font>) -> impl Bundle {
    create_text(text, font, HEADER_FONT_SIZE, COLOR_CYBERPUNK_MAGENTA)
}

/// Create body text with white styling
pub fn create_body_text(text: &str, font: Handle<Font>) -> impl Bundle {
    create_text(text, font, BODY_FONT_SIZE, Color::WHITE)
}

/// Create a standard button bundle
pub fn create_button(width: Val, height: Val) -> impl Bundle {
    (
        Button,
        Node {
            width,
            height,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        BackgroundColor(COLOR_BUTTON_BACKGROUND),
        BorderRadius::all(BORDER_RADIUS_SMALL),
    )
}

/// Create a container node with flex column layout
pub fn create_column_container() -> impl Bundle {
    Node {
        flex_direction: FlexDirection::Column,
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
    }
}

/// Create a container node with flex row layout
pub fn create_row_container() -> impl Bundle {
    Node {
        flex_direction: FlexDirection::Row,
        justify_content: JustifyContent::SpaceBetween,
        align_items: AlignItems::Center,
        ..default()
    }
}

/// Create a full-screen container
pub fn create_fullscreen_container() -> impl Bundle {
    (
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            flex_direction: FlexDirection::Column,
            ..default()
        },
        BackgroundColor(COLOR_DARK_BACKGROUND),
    )
}

/// Create a standard panel
pub fn create_panel(width: Val, padding: Val) -> impl Bundle {
    (
        Node {
            width,
            padding: UiRect::all(padding),
            margin: UiRect::all(PADDING_MEDIUM),
            flex_direction: FlexDirection::Column,
            row_gap: SPACING_SMALL,
            ..default()
        },
        BackgroundColor(COLOR_PANEL_BACKGROUND),
        BorderRadius::all(BORDER_RADIUS_MEDIUM),
    )
}

/// Create spacing element
pub fn create_spacer(size: Val) -> impl Bundle {
    Node {
        width: size,
        height: size,
        ..default()
    }
}

/// Standard button interaction colors
pub fn get_button_colors(interaction: Interaction) -> Color {
    match interaction {
        Interaction::Pressed => COLOR_BUTTON_HOVER,
        Interaction::Hovered => COLOR_BUTTON_HOVER,
        Interaction::None => COLOR_BUTTON_BACKGROUND,
    }
}

/// CSS-like margin shorthand parser
pub fn parse_margin(value: &str) -> UiRect {
    parse_ui_rect_values(value)
}

/// CSS-like padding shorthand parser
pub fn parse_padding(value: &str) -> UiRect {
    parse_ui_rect_values(value)
}

fn parse_ui_rect_values(value: &str) -> UiRect {
    let parts: Vec<&str> = value.split_whitespace().collect();
    match parts.len() {
        1 => {
            let val = parse_length_value(parts[0]);
            UiRect::all(val)
        },
        2 => {
            let vertical = parse_length_value(parts[0]);
            let horizontal = parse_length_value(parts[1]);
            UiRect::axes(horizontal, vertical)
        },
        4 => UiRect {
            top: parse_length_value(parts[0]),
            right: parse_length_value(parts[1]),
            bottom: parse_length_value(parts[2]),
            left: parse_length_value(parts[3]),
        },
        _ => UiRect::all(Val::Px(0.0)),
    }
}

fn parse_length_value(value: &str) -> Val {
    match value {
        "auto" => Val::Auto,
        value if value.ends_with('%') => {
            let num = value.trim_end_matches('%').parse().unwrap_or(0.0);
            Val::Percent(num)
        },
        value if value.ends_with("px") => {
            let num = value.trim_end_matches("px").parse().unwrap_or(0.0);
            Val::Px(num)
        },
        value => {
            // Try to parse as number, assume px
            value.parse::<f32>().map(Val::Px).unwrap_or(Val::Auto)
        }
    }
}