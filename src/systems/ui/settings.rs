// src/systems/ui/settings.rs - egui version
use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use crate::core::*;
use crate::systems::input::{MenuInput};

pub fn settings_system_egui(
    mut contexts: EguiContexts,
    mut next_state: ResMut<NextState<GameState>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    gamepads: Query<&Gamepad>,
) {

    let input = MenuInput::new(&keyboard, &gamepads);
    // Handle navigation
    if input.up {

    } else if input.down {

    } else if input.back || keyboard.just_pressed(KeyCode::Escape) {
        next_state.set(GameState::MainMenu);
    } else if input.select {

    }

    if let Ok(ctx) = contexts.ctx_mut() {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(100.0);
                
                ui.label(egui::RichText::new("SETTINGS")
                    .size(32.0)
                    .strong()
                    .color(egui::Color32::from_rgb(252, 255, 82)));
                
                ui.add_space(50.0);
                
                ui.group(|ui| {
                    ui.set_min_width(400.0);
                    ui.vertical_centered(|ui| {
                        ui.label("Settings coming soon...");
                        ui.add_space(20.0);
                        
                        // Placeholder settings
                        ui.horizontal(|ui| {
                            ui.label("Master Volume:");
                            ui.add(egui::Slider::new(&mut 50, 0..=100));
                        });
                        
                        ui.horizontal(|ui| {
                            ui.label("Graphics Quality:");
                            egui::ComboBox::from_label("")
                                .selected_text("High")
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(&mut (), (), "Low");
                                    ui.selectable_value(&mut (), (), "Medium");
                                    ui.selectable_value(&mut (), (), "High");
                                });
                        });
                    });
                });
                
                ui.add_space(50.0);
                
                if ui.button(egui::RichText::new("Back to Menu (ESC)")
                    .size(16.0)
                    .color(egui::Color32::WHITE))
                    .clicked() {
                    next_state.set(GameState::MainMenu);
                }
            });
        });
    }
}