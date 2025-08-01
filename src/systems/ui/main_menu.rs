// src/systems/ui/main_menu.rs - Working version with proper error handling

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

pub fn setup_main_menu_egui(mut menu_state: ResMut<MainMenuState>) {


    menu_state.has_save = save_game_exists();
    menu_state.selected_index = 0;

    // Build options list
    menu_state.options.clear();
    if menu_state.has_save {
        menu_state.options.push((MenuOptionType::Continue, "Continue"));
    }
    menu_state.options.push((MenuOptionType::NewGame, "New Game"));
    menu_state.options.push((MenuOptionType::Settings, "Settings"));
    menu_state.options.push((MenuOptionType::Credits, "Credits"));
    menu_state.options.push((MenuOptionType::Quit, "Quit Game"));
}

pub fn main_menu_system_egui(
    mut contexts: EguiContexts,
    mut next_state: ResMut<NextState<GameState>>,
    mut menu_state: ResMut<MainMenuState>,
    mut app_exit: EventWriter<bevy::app::AppExit>,
    mut global_data: ResMut<GlobalData>,
    input: Res<ButtonInput<KeyCode>>,
    mut research_progress: ResMut<ResearchProgress>,
    mut territory_manager: ResMut<TerritoryManager>,
    mut progression_tracker: ResMut<ProgressionTracker>,
) {

    // Handle keyboard navigation - this works even without egui context
    if input.just_pressed(KeyCode::KeyW) || input.just_pressed(KeyCode::ArrowUp) {
        if menu_state.selected_index > 0 {
            menu_state.selected_index -= 1;
        } else {
            menu_state.selected_index = menu_state.options.len().saturating_sub(1);
        }
    }

    if input.just_pressed(KeyCode::KeyS) || input.just_pressed(KeyCode::ArrowDown) {
        menu_state.selected_index = (menu_state.selected_index + 1) % menu_state.options.len();
    }

    if input.just_pressed(KeyCode::Escape) {
        menu_state.selected_index = menu_state.options.len().saturating_sub(1);
    }

    if input.just_pressed(KeyCode::Space) || input.just_pressed(KeyCode::Enter) {
        if let Some((option_type, _)) = menu_state.options.get(menu_state.selected_index) {
            execute_menu_option(*option_type, &mut next_state, &mut app_exit, &mut global_data, &mut research_progress, &mut territory_manager, &mut progression_tracker);
        }
    }

    // Use the same pattern that works in your other screens
    if let Ok(ctx) = contexts.ctx_mut() {
        egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(egui::Color32::from_rgb(26, 26, 51)))
            .show(ctx, |ui| {
                ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                    let available_height = ui.available_height();
                    let menu_height = 400.0;
                    let top_padding = (available_height - menu_height) / 2.0;

                    ui.add_space(top_padding.max(20.0));

                    ui.label(egui::RichText::new("SUBVERSIVE")
                        .size(48.0)
                        .strong()
                        .color(egui::Color32::from_rgb(252, 255, 82)));

                    ui.add_space(50.0);

                    for (i, (option_type, text)) in menu_state.options.iter().enumerate() {
                        let is_selected = i == menu_state.selected_index;

                        let button_color = if is_selected {
                            egui::Color32::from_rgb(252, 255, 82)
                        } else {
                            egui::Color32::WHITE
                        };

                        let button = egui::Button::new(
                            egui::RichText::new(*text)
                                .size(24.0)
                                .color(button_color)
                        )
                        .fill(egui::Color32::TRANSPARENT)
                        .stroke(if is_selected {
                            egui::Stroke::new(2.0, button_color)
                        } else {
                            egui::Stroke::NONE
                        });

                        if ui.add_sized([200.0, 40.0], button).clicked() {
                            execute_menu_option(
                                *option_type,
                                &mut next_state,
                                &mut app_exit,
                                &mut global_data,
                                &mut research_progress,
                                &mut territory_manager,
                                &mut progression_tracker,
                            );
                        }

                        ui.add_space(10.0);
                    }

                    ui.add_space(20.0);
                    ui.add_space(50.0);
                    ui.label(egui::RichText::new("W/S or ↑/↓: Navigate | Enter/Space: Select | Esc: Quit")
                        .size(12.0)
                        .color(egui::Color32::from_rgb(128, 128, 128)));
                });
            });
    }
    // If context is not available, the keyboard controls above still work
}

fn execute_menu_option(
    option_type: MenuOptionType,
    next_state: &mut NextState<GameState>,
    app_exit: &mut EventWriter<bevy::app::AppExit>,
    global_data: &mut GlobalData,
    research_progress: &mut ResearchProgress,
    territory_manager: &mut TerritoryManager,
    progression_tracker: &mut ProgressionTracker,
) {
    match option_type {
        MenuOptionType::Continue => {
            if let Some((loaded_data, loaded_territory, loaded_progression)) = crate::systems::save::load_game() {
                *global_data = loaded_data;
                *territory_manager = loaded_territory;
                *progression_tracker = loaded_progression;
                next_state.set(GameState::GlobalMap);
            }
        },
        MenuOptionType::NewGame => {
            *global_data = GlobalData::default();
            *research_progress = ResearchProgress::default();
            *territory_manager = TerritoryManager::default();
            *progression_tracker = ProgressionTracker::default();
            crate::systems::save::save_game_complete(global_data, research_progress, territory_manager, progression_tracker);
            next_state.set(GameState::GlobalMap);
        },
        MenuOptionType::Settings => {
            next_state.set(GameState::Settings);
        },
        MenuOptionType::Credits => {
            next_state.set(GameState::Credits);
        },
        MenuOptionType::Quit => {
            app_exit.write(bevy::app::AppExit::Success);
        },
    }
}

pub fn cleanup_main_menu() {
    // Any cleanup if needed
}

// These can be empty since everything is handled in the main system
pub fn menu_input_system() {}
pub fn menu_mouse_system() {}
pub fn update_menu_visuals() {}