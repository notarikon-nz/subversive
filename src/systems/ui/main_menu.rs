// src/systems/ui/main_menu.rs - Optimized with better code reuse

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use crate::core::*;
use crate::systems::save::save_game_exists;

#[derive(PartialEq, Clone, Copy)]
pub enum MenuOptionType {
    Continue,
    NewGame,
    Settings,
    Credits,
    Quit,
}

#[derive(Resource)]
pub struct MainMenuState {
    pub selected_index: usize,
    pub has_save: bool,
    pub options: Vec<(MenuOptionType, &'static str)>,
}

impl Default for MainMenuState {
    fn default() -> Self {
        Self {
            selected_index: 0,
            has_save: false,
            options: Vec::new(),
        }
    }
}

use crate::systems::input::{MenuInput};

pub fn setup_main_menu_egui(mut menu_state: ResMut<MainMenuState>) {
    menu_state.has_save = save_game_exists();
    menu_state.selected_index = 0;
    menu_state.options.clear();

    if menu_state.has_save {
        menu_state.options.push((MenuOptionType::Continue, "Continue"));
    }
    menu_state.options.extend([
        (MenuOptionType::NewGame, "New Game"),
        (MenuOptionType::Settings, "Settings"),
        (MenuOptionType::Credits, "Credits"),
        (MenuOptionType::Quit, "Quit Game"),
    ]);
}

pub fn main_menu_system_egui(
    mut contexts: EguiContexts,
    mut next_state: ResMut<NextState<GameState>>,
    mut menu_state: ResMut<MainMenuState>,
    mut app_exit: EventWriter<bevy::app::AppExit>,
    mut global_data: ResMut<GlobalData>,
    keyboard: Res<ButtonInput<KeyCode>>,
    gamepads: Query<&Gamepad>,
    mut research_progress: ResMut<ResearchProgress>,
    mut territory_manager: ResMut<TerritoryManager>,
    mut progression_tracker: ResMut<CampaignProgressionTracker>,
) {
    let option_count = menu_state.options.len();

    let input = MenuInput::new(&keyboard, &gamepads);
    // Handle navigation
    if input.up {
        menu_state.selected_index = menu_state.selected_index.checked_sub(1).unwrap_or(option_count - 1);
    } else if input.down {
        menu_state.selected_index = (menu_state.selected_index + 1) % option_count;
    } else if input.back {
        menu_state.selected_index = option_count - 1;
    } else if input.select {
        if let Some(&(option_type, _)) = menu_state.options.get(menu_state.selected_index) {
            execute_menu_option(option_type, &mut next_state, &mut app_exit, &mut global_data, &mut research_progress, &mut territory_manager, &mut progression_tracker);
        }
    }

    // Render UI
    if let Ok(ctx) = contexts.ctx_mut() {
        egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(egui::Color32::from_rgb(26, 26, 51)))
            .show(ctx, |ui| {
                ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                    let top_padding = ((ui.available_height() - 400.0) / 2.0).max(20.0);
                    ui.add_space(top_padding);

                    ui.label(egui::RichText::new("SUBVERSIVE").size(48.0).strong()
                        .color(egui::Color32::from_rgb(252, 255, 82)));
                    ui.add_space(50.0);

                    for (i, &(option_type, text)) in menu_state.options.iter().enumerate() {
                        let selected = i == menu_state.selected_index;
                        let color = if selected { egui::Color32::from_rgb(252, 255, 82) } else { egui::Color32::WHITE };

                        let button = egui::Button::new(egui::RichText::new(text).size(24.0).color(color))
                            .fill(egui::Color32::TRANSPARENT)
                            .stroke(if selected { egui::Stroke::new(2.0, color) } else { egui::Stroke::NONE });

                        if ui.add_sized([200.0, 40.0], button).clicked() {
                            execute_menu_option(option_type, &mut next_state, &mut app_exit, &mut global_data, &mut research_progress, &mut territory_manager, &mut progression_tracker);
                        }
                        ui.add_space(10.0);
                    }

                    ui.add_space(70.0);
                    ui.label(egui::RichText::new("W/S/D-Pad: Navigate | Enter/A: Select | Esc/B: Quit")
                        .size(12.0).color(egui::Color32::from_rgb(128, 128, 128)));
                });
            });
    }
}


fn execute_menu_option(
    option_type: MenuOptionType,
    next_state: &mut NextState<GameState>,
    app_exit: &mut EventWriter<bevy::app::AppExit>,
    global_data: &mut GlobalData,
    research_progress: &mut ResearchProgress,
    territory_manager: &mut TerritoryManager,
    progression_tracker: &mut CampaignProgressionTracker,
) {
    use MenuOptionType::*;

    match option_type {
        Continue => {
            if let Some((data, territory, progression)) = crate::systems::save::load_game() {
                *global_data = data;
                *territory_manager = territory;
                *progression_tracker = progression;
                next_state.set(GameState::GlobalMap);
            }
        },
        NewGame => {
            *global_data = GlobalData::default();
            *research_progress = ResearchProgress::default();
            *territory_manager = TerritoryManager::default();
            *progression_tracker = CampaignProgressionTracker::default();
            crate::systems::save::save_game_complete(global_data, research_progress, territory_manager, progression_tracker);
            next_state.set(GameState::GlobalMap);
        },
        Settings => next_state.set(GameState::Settings),
        Credits => next_state.set(GameState::Credits),
        Quit => {
            app_exit.write(bevy::app::AppExit::Success);
        },
    }
}
