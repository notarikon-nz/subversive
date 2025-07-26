// src/systems/ui/builder.rs - Consolidated UI creation patterns
use bevy::prelude::*;

pub struct UIBuilder;

impl UIBuilder {
    // Text creation helpers
    pub fn text(text: &str, size: f32, color: Color) -> impl Bundle {
        (
            Text::new(text),
            TextFont { font_size: size, ..default() },
            TextColor(color),
        )
    }

    pub fn header(text: &str) -> impl Bundle {
        Self::text(text, 15.0, Color::WHITE)
    }

    pub fn title(text: &str) -> impl Bundle {
        Self::text(text, 14.0, Color::srgb(0.8, 0.8, 0.2))
    }

    pub fn subtitle(text: &str) -> impl Bundle {
        Self::text(text, 13.0, Color::WHITE)
    }

    pub fn body(text: &str) -> impl Bundle {
        Self::text(text, 12.0, Color::WHITE)
    }

    pub fn small(text: &str) -> impl Bundle {
        Self::text(text, 11.0, Color::srgb(0.7, 0.7, 0.7))
    }

    pub fn selected_text(text: &str, size: f32) -> impl Bundle {
        Self::text(text, size, Color::srgb(0.8, 0.8, 0.2))
    }

    pub fn error_text(text: &str) -> impl Bundle {
        Self::text(text, 12.0, Color::srgb(0.8, 0.2, 0.2))
    }

    pub fn success_text(text: &str) -> impl Bundle {
        Self::text(text, 12.0, Color::srgb(0.2, 0.8, 0.2))
    }

    // Node creation helpers
    pub fn column(gap: f32) -> Node {
        Node {
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(gap),
            ..default()
        }
    }

    pub fn row(gap: f32) -> Node {
        Node {
            flex_direction: FlexDirection::Row,
            column_gap: Val::Px(gap),
            ..default()
        }
    }

    pub fn panel(padding: f32) -> Node {
        Node {
            padding: UiRect::all(Val::Px(padding)),
            ..default()
        }
    }

    pub fn full_size() -> Node {
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            ..default()
        }
    }

    pub fn content_area() -> Node {
        Node {
            width: Val::Percent(100.0),
            flex_grow: 1.0,
            padding: UiRect::all(Val::Px(20.0)),
            height: Val::Px(510.0), // 550 - 2x 20px padding
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(15.0),
            ..default()
        }
    }

    // Background helpers
    pub fn bg_primary() -> BackgroundColor {
        BackgroundColor(Color::srgba(0.1, 0.1, 0.2, 0.3))
    }

    pub fn bg_secondary() -> BackgroundColor {
        BackgroundColor(Color::srgba(0.2, 0.2, 0.3, 0.3))
    }

    pub fn bg_selected() -> BackgroundColor {
        BackgroundColor(Color::srgba(0.2, 0.6, 0.2, 0.3))
    }

    pub fn bg_error() -> BackgroundColor {
        BackgroundColor(Color::srgba(0.6, 0.2, 0.2, 0.3))
    }

    // Combined helpers for common patterns
    pub fn section_panel() -> (Node, BackgroundColor) {
        (
            Node {
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(15.0)),
                row_gap: Val::Px(8.0),
                ..default()
            },
            Self::bg_primary().0.into()
        )
    }

    pub fn selection_item(is_selected: bool, prefix: &str, text: &str) -> String {
        format!("{}{}", if is_selected { "> " } else { "  " }, 
                format!("{}{}", prefix, text))
    }

    pub fn stat_display(label: &str, value: &str) -> String {
        format!("{}: {}", label, value)
    }

    pub fn credits_display(amount: u32) -> String {
        format!("Credits: {}", amount)
    }

    // Tab bar creation
    pub fn tab_button(title: &str, is_active: bool) -> (Node, BackgroundColor, impl Bundle) {
        let bg_color = if is_active { 
            Color::srgb(0.2, 0.6, 0.8) 
        } else { 
            Color::srgb(0.12, 0.12, 0.2) 
        };
        let text_color = if is_active { 
            Color::WHITE 
        } else { 
            Color::srgb(0.7, 0.7, 0.7) 
        };
        
        (
            Node {
                width: Val::Percent(18.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(bg_color),
            Self::text(title, 14.0, text_color)
        )
    }

    // Input helper
    pub fn nav_controls(controls: &str) -> impl Bundle {
        Self::text(controls, 12.0, Color::srgb(0.7, 0.7, 0.7))
    }
}

// Specialized builders for common UI patterns
pub struct ListBuilder {
    selected_index: usize,
}

impl ListBuilder {
    pub fn new(selected_index: usize) -> Self {
        Self { selected_index }
    }

    pub fn item<F>(&self, parent: &mut ChildSpawnerCommands, index: usize, text: &str, mut content_fn: F) 
    where F: FnMut(&mut ChildSpawnerCommands, bool)
    {
        let is_selected = index == self.selected_index;
        let (node, bg) = if is_selected {
            (UIBuilder::panel(8.0), UIBuilder::bg_selected())
        } else {
            (UIBuilder::panel(8.0), BackgroundColor(Color::NONE))
        };

        parent.spawn((node, bg))
            .with_children(|item_parent| {
                content_fn(item_parent, is_selected);
            });
    }
}

pub struct StatsBuilder<'a, 'b> {
    parent: &'a mut ChildSpawnerCommands<'b>,
}

impl<'a, 'b> StatsBuilder<'a, 'b> {
    pub fn new(parent: &'a mut ChildSpawnerCommands<'b>) -> Self {
        Self { parent }
    }

    pub fn stat(&mut self, label: &str, value: &str, color: Option<Color>) {
        let text_color = color.unwrap_or(Color::WHITE);
        self.parent.spawn(UIBuilder::text(&format!("{}: {}", label, value), 14.0, text_color));
    }

    pub fn credits(&mut self, amount: u32) {
        self.stat("Credits", &amount.to_string(), Some(Color::srgb(0.8, 0.8, 0.2)));
    }

    pub fn level(&mut self, level: u8, exp: u32, next_exp: u32) {
        self.stat("Level", &level.to_string(), None);
        self.stat("Experience", &format!("{}/{}", exp, next_exp), None);
    }
}