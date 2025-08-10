// src/systems/ui/layout/mod.rs - Main UI Layout System
use bevy::prelude::*;
use bevy::ecs::world::EntityWorldMut;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub mod constants;
pub use constants::*;

/// Main UI layout definition loaded from external files
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UILayout {
    pub name: String,
    pub root: UIElement,
    #[serde(default)]
    pub variables: HashMap<String, String>,
    #[serde(default)]
    pub actions: HashMap<String, ActionBinding>,
}

/// Individual UI element with all properties
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UIElement {
    #[serde(rename = "type")]
    pub element_type: ElementType,
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default)]
    pub class: Option<String>,
    #[serde(default)]
    pub text: Option<String>,
    #[serde(default)]
    pub style: ElementStyle,
    #[serde(default)]
    pub children: Vec<UIElement>,
    #[serde(default)]
    pub action: Option<String>,
    #[serde(default)]
    pub component: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ElementType {
    Container,
    Text,
    Button,
    Panel,
    Spacer,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ElementStyle {
    #[serde(default)]
    pub width: Option<String>,
    #[serde(default)]
    pub height: Option<String>,
    #[serde(default)]
    pub padding: Option<String>,
    #[serde(default)]
    pub margin: Option<String>,
    #[serde(default)]
    pub background_color: Option<String>,
    #[serde(default)]
    pub text_color: Option<String>,
    #[serde(default)]
    pub font_size: Option<f32>,
    #[serde(default)]
    pub justify_content: Option<String>,
    #[serde(default)]
    pub align_items: Option<String>,
    #[serde(default)]
    pub flex_direction: Option<String>,
    #[serde(default)]
    pub border_radius: Option<String>,
    #[serde(default)]
    pub gap: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionBinding {
    pub component: String,
    #[serde(default)]
    pub parameters: HashMap<String, String>,
}

/// Resource to track loaded layouts
#[derive(Resource, Default)]
pub struct UILayoutCache {
    layouts: HashMap<String, UILayout>,
}

impl UILayoutCache {
    pub fn load_layout(&mut self, name: &str, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        let layout: UILayout = serde_json::from_str(&content)?;
        self.layouts.insert(name.to_string(), layout);
        Ok(())
    }

    pub fn get_layout(&self, name: &str) -> Option<&UILayout> {
        self.layouts.get(name)
    }

    pub fn preload_layouts(&mut self) {
        let layouts_to_load = [
            ("main_menu", "ui_layouts/main_menu.json"),
            ("settings", "ui_layouts/settings.json"),
            ("credits", "ui_layouts/credits.json"),
            ("hub", "ui_layouts/hub.json"),
        ];

        for (name, path) in layouts_to_load {
            if let Err(e) = self.load_layout(name, path) {
                warn!("Failed to load UI layout {}: {}", name, e);
            } else {
                info!("Loaded UI layout: {}", name);
            }
        }
    }
}

/// Main function to spawn UI from layout
pub fn spawn_ui_from_layout(
    commands: &mut Commands,
    asset_server: &AssetServer,
    layout_cache: &UILayoutCache,
    layout_name: &str,
    variables: Option<&HashMap<String, String>>,
) -> Option<Entity> {
    let layout = layout_cache.get_layout(layout_name)?;
    
    // Merge provided variables with layout variables
    let mut resolved_vars = layout.variables.clone();
    if let Some(vars) = variables {
        resolved_vars.extend(vars.clone());
    }

    Some(spawn_element(commands, asset_server, &layout.root, &resolved_vars, &layout.actions))
}

/// Recursive function to spawn individual elements
fn spawn_element(
    commands: &mut Commands,
    asset_server: &AssetServer,
    element: &UIElement,
    variables: &HashMap<String, String>,
    actions: &HashMap<String, ActionBinding>,
) -> Entity {
    let mut entity_commands = commands.spawn(create_base_bundle(element, asset_server, variables));
    apply_element_components(&mut entity_commands, element, asset_server, variables, actions);
    let entity = entity_commands.id();

    // Spawn children - inline to avoid type issues
    if !element.children.is_empty() {
        commands.entity(entity).with_children(|parent| {
            for child in &element.children {
                let mut child_commands = parent.spawn(create_base_bundle(child, asset_server, variables));
                apply_element_components(&mut child_commands, child, asset_server, variables, actions);
                let child_entity = child_commands.id();
                
                // Handle grandchildren
                if !child.children.is_empty() {
                    parent.commands().entity(child_entity).with_children(|grandparent| {
                        for grandchild in &child.children {
                            let mut grandchild_commands = grandparent.spawn(create_base_bundle(grandchild, asset_server, variables));
                            apply_element_components(&mut grandchild_commands, grandchild, asset_server, variables, actions);
                        }
                    });
                }
            }
        });
    }

    entity
}

/// Extract common component application logic
fn apply_element_components(
    entity_commands: &mut EntityCommands,
    element: &UIElement,
    asset_server: &AssetServer,
    variables: &HashMap<String, String>,
    actions: &HashMap<String, ActionBinding>,
) {
    // Add type-specific components
    match element.element_type {
        ElementType::Button => {
            entity_commands.insert(Button);
            if let Some(action_name) = &element.action {
                if let Some(action) = actions.get(action_name) {
                    add_action_component(entity_commands, action);
                }
            }
        },
        ElementType::Text => {
            if let Some(text) = &element.text {
                let resolved_text = resolve_variables(text, variables);
                entity_commands.insert(create_text_bundle(&resolved_text, &element.style, asset_server));
            }
        },
        _ => {} // Container, Panel, Spacer use base bundle only
    }

    // Add custom component if specified
    if let Some(component) = &element.component {
        add_custom_component(entity_commands, component);
    }
}

/// Create base bundle with Node and styling
fn create_base_bundle(
    element: &UIElement,
    asset_server: &AssetServer,
    variables: &HashMap<String, String>,
) -> impl Bundle {
    let style = &element.style;
    let margin_str = resolve_variables(&style.margin.as_deref().unwrap_or("0"), variables);
    info!("Parsing margin: {} for element", margin_str);
    
    (
        Node {
            width: parse_val(&resolve_variables(&style.width.as_deref().unwrap_or("auto"), variables)),
            height: parse_val(&resolve_variables(&style.height.as_deref().unwrap_or("auto"), variables)),
            padding: parse_ui_rect(&resolve_variables(&style.padding.as_deref().unwrap_or("0"), variables)),
            margin: parse_ui_rect(&resolve_variables(&style.margin.as_deref().unwrap_or("0"), variables)),
            justify_content: parse_justify_content(&style.justify_content.as_deref().unwrap_or("start")),
            align_items: parse_align_items(&style.align_items.as_deref().unwrap_or("stretch")),
            flex_direction: parse_flex_direction(&style.flex_direction.as_deref().unwrap_or("column")),
            row_gap: parse_val(&resolve_variables(&style.gap.as_deref().unwrap_or("0"), variables)),
            column_gap: parse_val(&resolve_variables(&style.gap.as_deref().unwrap_or("0"), variables)),
            ..default()
        },
        BackgroundColor(parse_color(&style.background_color.as_deref().unwrap_or("transparent"))),
        BorderRadius::all(parse_val(&resolve_variables(&style.border_radius.as_deref().unwrap_or("0"), variables))),
    )
}

fn create_text_bundle(text: &str, style: &ElementStyle, asset_server: &AssetServer) -> impl Bundle {
    (
        Text::new(text),
        TextFont {
            font: asset_server.load(DEFAULT_FONT_PATH),
            font_size: style.font_size.unwrap_or(DEFAULT_FONT_SIZE),
            ..default()
        },
        TextColor(parse_color(&style.text_color.as_deref().unwrap_or("#FFFFFF"))),
    )
}

// Helper functions for parsing style values
fn parse_val(value: &str) -> Val {
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
        _ => Val::Auto,
    }
}

fn parse_ui_rect(value: &str) -> UiRect {
    let parts: Vec<&str> = value.split_whitespace().collect();
    match parts.len() {
        1 => UiRect::all(parse_val(parts[0])),
        2 => UiRect::axes(parse_val(parts[0]), parse_val(parts[1])),
        4 => UiRect {
            top: parse_val(parts[0]),
            right: parse_val(parts[1]),
            bottom: parse_val(parts[2]),
            left: parse_val(parts[3]),
        },
        _ => UiRect::all(Val::Px(0.0)),
    }
}

fn parse_color(value: &str) -> Color {
    match value {
        "transparent" => Color::NONE,
        "white" => Color::WHITE,
        "black" => Color::BLACK,
        "red" => Color::srgb(1.0, 0.0, 0.0),
        "green" => Color::srgb(0.0, 1.0, 0.0),
        "blue" => Color::srgb(0.0, 0.0, 1.0),
        "yellow" => Color::srgb(0.99, 1.0, 0.32), // Cyberpunk yellow
        "cyan" => Color::srgb(0.0, 1.0, 1.0),
        "magenta" => Color::srgb(1.0, 0.0, 0.59),
        hex if hex.starts_with('#') && hex.len() == 7 => {
            let r = u8::from_str_radix(&hex[1..3], 16).unwrap_or(255) as f32 / 255.0;
            let g = u8::from_str_radix(&hex[3..5], 16).unwrap_or(255) as f32 / 255.0;
            let b = u8::from_str_radix(&hex[5..7], 16).unwrap_or(255) as f32 / 255.0;
            Color::srgb(r, g, b)
        },
        rgba if rgba.starts_with("rgba(") && rgba.ends_with(')') => {
            let inner = &rgba[5..rgba.len()-1];
            let parts: Vec<f32> = inner.split(',').map(|s| s.trim().parse().unwrap_or(0.0)).collect();
            if parts.len() >= 4 {
                Color::srgba(parts[0]/255.0, parts[1]/255.0, parts[2]/255.0, parts[3])
            } else {
                Color::WHITE
            }
        },
        _ => Color::WHITE,
    }
}

fn parse_justify_content(value: &str) -> JustifyContent {
    match value {
        "start" => JustifyContent::Start,
        "end" => JustifyContent::End,
        "center" => JustifyContent::Center,
        "space-between" => JustifyContent::SpaceBetween,
        "space-around" => JustifyContent::SpaceAround,
        "space-evenly" => JustifyContent::SpaceEvenly,
        _ => JustifyContent::Start,
    }
}

fn parse_align_items(value: &str) -> AlignItems {
    match value {
        "start" => AlignItems::Start,
        "end" => AlignItems::End,
        "center" => AlignItems::Center,
        "stretch" => AlignItems::Stretch,
        _ => AlignItems::Stretch,
    }
}

fn parse_flex_direction(value: &str) -> FlexDirection {
    match value {
        "row" => FlexDirection::Row,
        "column" => FlexDirection::Column,
        "row-reverse" => FlexDirection::RowReverse,
        "column-reverse" => FlexDirection::ColumnReverse,
        _ => FlexDirection::Column,
    }
}

fn resolve_variables(text: &str, variables: &HashMap<String, String>) -> String {
    let mut result = text.to_string();
    for (key, value) in variables {
        result = result.replace(&format!("{{{}}}", key), value);
    }
    result
}

fn add_action_component(entity_commands: &mut EntityCommands, action: &ActionBinding) {
    match action.component.as_str() {
        "BackButton" => { entity_commands.insert(crate::systems::ui::settings::BackButton); },
        "VolumeControl" => { entity_commands.insert(crate::systems::ui::settings::SettingsControl::Volume); },
        "QualityControl" => { entity_commands.insert(crate::systems::ui::settings::SettingsControl::Quality); },
        _ => warn!("Unknown action component: {}", action.component),
    }
}

fn add_custom_component(entity_commands: &mut EntityCommands, component: &str) {
    match component {
        "SettingsUI" => { entity_commands.insert(crate::systems::ui::settings::SettingsUI); },
        "CreditsUI" => { entity_commands.insert(crate::systems::ui::credits::CreditsUI); },
        "VolumeBar" => { entity_commands.insert(crate::systems::ui::settings::VolumeBar); },
        _ => warn!("Unknown custom component: {}", component),
    }
}

// Example usage function for integration
pub fn setup_ui_layout_system(mut commands: Commands) {
    let mut cache = UILayoutCache::default();
    cache.preload_layouts();
    commands.insert_resource(cache);
}

pub fn spawn_settings_ui_from_layout(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    layout_cache: Res<UILayoutCache>,
) {
    let mut variables = HashMap::new();
    variables.insert("screen_title".to_string(), "SETTINGS".to_string());
    variables.insert("back_button_text".to_string(), "Back to Menu (ESC)".to_string());
    
    if let Some(_entity) = spawn_ui_from_layout(&mut commands, &asset_server, &layout_cache, "settings", Some(&variables)) {
        info!("Settings UI spawned from layout");
    } else {
        error!("Failed to spawn settings UI from layout");
    }
}