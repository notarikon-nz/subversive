// src/systems/ui/hub/mod.rs - New egui-based hub system
use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use crate::core::*;
use serde::{Deserialize, Serialize};

pub mod agents;
pub mod research;
pub mod manufacture;
pub mod missions;
pub mod global_map;

// Much simpler state management - no complex rebuilding needed
#[derive(Resource, Default)]
pub struct HubState {
    pub active_tab: HubTab,
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub enum HubTab {
    #[default] 
    GlobalMap, 
    Research, 
    Agents, 
    Manufacture, 
    Missions,
}

// All the databases in one place for easy access
#[derive(Resource)]
pub struct HubDatabases {
    pub research_db: ResearchDatabase,
    pub cybernetics_db: CyberneticsDatabase,
    pub attachment_db: AttachmentDatabase,
    pub cities_db: CitiesDatabase,
}

impl Default for HubDatabases {
    fn default() -> Self {
        Self {
            research_db: ResearchDatabase::load(),
            cybernetics_db: CyberneticsDatabase::load(),
            attachment_db: AttachmentDatabase::load(),
            cities_db: CitiesDatabase::load(),
        }
    }
}

#[derive(Debug, Default, Clone, Resource, Serialize, Deserialize)]
pub struct CyberneticsDatabase {
    pub cybernetics: Vec<CyberneticUpgrade>,
}

impl CyberneticsDatabase {
    pub fn load() -> Self {
        std::fs::read_to_string("data/cybernetics.json")
            .ok()
            .and_then(|content| serde_json::from_str(&content).ok())
            .unwrap_or_else(|| Self { cybernetics: Vec::new() })
    }
}

// Main hub system - much simpler than before
pub fn hub_system(
    mut contexts: EguiContexts,
    mut commands: Commands,
    mut next_state: ResMut<NextState<GameState>>,
    mut global_data: ResMut<GlobalData>,
    mut hub_state: ResMut<HubState>,
    hub_databases: Res<HubDatabases>,
    mut research_progress: ResMut<ResearchProgress>,
    mut unlocked_attachments: ResMut<UnlockedAttachments>,
    mut agent_query: Query<&mut Inventory, With<Agent>>,
    input: Res<ButtonInput<KeyCode>>,
    windows: Query<&Window>,
    cameras: Query<(&Camera, &GlobalTransform)>,
    mouse: Res<ButtonInput<MouseButton>>,
    city_query: Query<(Entity, &Transform, &global_map::InteractiveCity)>,
) {
    // Handle tab switching with Q/E
    if input.just_pressed(KeyCode::KeyQ) {
        hub_state.active_tab = match hub_state.active_tab {
            HubTab::GlobalMap => HubTab::Missions,
            HubTab::Research => HubTab::GlobalMap,
            HubTab::Agents => HubTab::Research,
            HubTab::Manufacture => HubTab::Agents,
            HubTab::Missions => HubTab::Manufacture,
        };
    }
    
    if input.just_pressed(KeyCode::KeyE) {
        hub_state.active_tab = match hub_state.active_tab {
            HubTab::GlobalMap => HubTab::Research,
            HubTab::Research => HubTab::Agents,
            HubTab::Agents => HubTab::Manufacture,
            HubTab::Manufacture => HubTab::Missions,
            HubTab::Missions => HubTab::GlobalMap,
        };
    }

    // Global exit handler
    if input.just_pressed(KeyCode::Escape) {
        std::process::exit(0);
    }

    if let Ok(ctx) = contexts.ctx_mut() {
    
        // Apply cyberpunk theme
        setup_cyberpunk_theme(ctx);
        
        // Top bar with navigation and info
        egui::TopBottomPanel::top("top_bar")
            .exact_height(50.0)
            .show(ctx, |ui| {
            ui.horizontal_centered(|ui| {
                // Left side - progress info
                let accessible_cities = hub_databases.cities_db.get_accessible_cities(&global_data).len();
                let total_cities = hub_databases.cities_db.get_all_cities().len();
                ui.colored_label(egui::Color32::YELLOW, format!("Cities: {}/{}", accessible_cities, total_cities));
                
                ui.separator();
                
                // Center section - 60% width
                ui.allocate_ui_with_layout(
                    egui::vec2(ui.available_width() * 0.75, ui.available_height()), // 0.75 because 20% is already used
                    egui::Layout::top_down(egui::Align::Center),
                    |ui| {
                        ui.horizontal_centered(|ui| {
                            ui.label("Q");
                            for tab in [HubTab::GlobalMap, HubTab::Research, HubTab::Agents, HubTab::Manufacture, HubTab::Missions] {
                                let is_active = hub_state.active_tab == tab;
                                let text = match tab {
                                    HubTab::GlobalMap => "Map",
                                    HubTab::Research => "Research", 
                                    HubTab::Agents => "Agents",
                                    HubTab::Manufacture => "Gear",
                                    HubTab::Missions => "Mission",
                                };
                                
                                if ui.selectable_label(is_active, text).clicked() {
                                    hub_state.active_tab = tab;
                                }
                            }
                            ui.label("E");
                        });
                    }
                );
                
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.colored_label(egui::Color32::YELLOW, format!("${}", global_data.credits));
                    ui.label(format!("Day {}", global_data.current_day));
                });
            });
        });
        
        // Bottom bar with controls
        egui::TopBottomPanel::bottom("bottom_bar")
            .exact_height(30.0)
            .show(ctx, |ui| {
            ui.horizontal_centered(|ui| {
                let controls = match hub_state.active_tab {
                    HubTab::GlobalMap => "Click Cities | W: Wait Day | ENTER: Mission | Q/E: Tabs",
                    HubTab::Research => "↑↓: Navigate | ENTER: Purchase | Q/E: Tabs",
                    HubTab::Agents => "←→: Agent | 1-3: View | ↑↓: Navigate | ENTER: Install | Q/E: Tabs",
                    HubTab::Manufacture => "1-3: Agent | ↑↓: Slots | ←→: Attachments | ENTER: Modify | Q/E: Tabs",
                    HubTab::Missions => "ENTER: Launch | Q/E: Tabs | ESC: Quit",
                };
                ui.weak(controls);
            });
        });
        

        
        // Main content area
        egui::CentralPanel::default().show(ctx, |ui| {

            match hub_state.active_tab {
                HubTab::GlobalMap => global_map::show_global_map(
                    ui, 
                    &mut global_data, 
                    &hub_databases.cities_db,
                    &input,
                    &windows,
                    &cameras,
                    &mouse,
                    &city_query,
                ),
                HubTab::Research => research::show_research(
                    ui,
                    &mut global_data,
                    &mut research_progress,
                    &hub_databases.research_db,
                    &mut unlocked_attachments,
                    &input,
                ),
                HubTab::Agents => agents::show_agents(
                    ui,
                    &mut global_data,
                    &hub_databases.cybernetics_db,
                    &input,
                ),
                HubTab::Manufacture => manufacture::show_manufacture(
                    ui,
                    &mut global_data,
                    &hub_databases.attachment_db,
                    &unlocked_attachments,
                    &mut agent_query,
                    &input,
                ),
                HubTab::Missions => missions::show_missions(
                    ui,
                    &global_data,
                    &hub_databases.cities_db,
                    &mut commands,
                    &mut next_state,
                    &input,
                ),
            }
        });
    }
}

pub fn setup_cyberpunk_theme(ctx: &egui::Context) {
    let mut style = (*ctx.style()).clone();
    
    // Dark cyberpunk colors
    style.visuals.window_fill = egui::Color32::from_rgba_premultiplied(10, 10, 20, 240);
    style.visuals.panel_fill = egui::Color32::from_rgba_premultiplied(15, 15, 25, 200);
    style.visuals.faint_bg_color = egui::Color32::from_rgba_premultiplied(20, 20, 40, 100);
    
    // Text colors - these are methods, not fields
    style.visuals.override_text_color = Some(egui::Color32::WHITE);
    
    // Selection colors
    style.visuals.selection.bg_fill = egui::Color32::from_rgb(50, 150, 50);
    style.visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(30, 30, 50);
    style.visuals.widgets.active.bg_fill = egui::Color32::from_rgb(50, 50, 80);
    
    // Borders
    style.visuals.widgets.noninteractive.bg_stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(60, 60, 80));
    
    // Spacing - use proper margin constructor
    style.spacing.item_spacing = egui::vec2(8.0, 6.0);
    style.spacing.window_margin = egui::Margin::symmetric(12, 12);
    
    ctx.set_style(style);
}

// Reset function for entering hub
pub fn reset_hub_to_global_map(mut hub_state: ResMut<HubState>) {
    hub_state.active_tab = HubTab::GlobalMap;
}