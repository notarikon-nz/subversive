// src/systems/ui/credits.rs - egui version
use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use crate::core::*;
use crate::systems::input::{MenuInput};

pub fn credits_system_egui(
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
                
                ui.label(egui::RichText::new("CREDITS")
                    .size(32.0)
                    .strong()
                    .color(egui::Color32::from_rgb(252, 255, 82)));
                
                ui.add_space(50.0);
                
                ui.group(|ui| {
                    ui.set_min_width(500.0);
                    ui.vertical_centered(|ui| {
                        ui.label(egui::RichText::new("SUBVERSIVE")
                            .size(24.0)
                            .strong()
                            .color(egui::Color32::from_rgb(0, 255, 255)));
                        
                        ui.add_space(20.0);
                        
                        ui.label("Copyright (c) 2005 Matt Orsborn. All rights reserved.");
                        ui.add_space(10.0);
                        ui.label("Developed with Bevy Engine, Rapier2D and eGUI");
                        
                        ui.add_space(30.0);
                        
                        ui.label(egui::RichText::new("Special Thanks:")
                            .strong()
                            .color(egui::Color32::from_rgb(255, 0, 150)));
                        
                        ui.add_space(10.0);
                        ui.label("• Bevy Community");
                        ui.label("• egui Contributors");
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