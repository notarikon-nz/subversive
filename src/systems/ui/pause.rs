// src/systems/ui/pause.rs - egui version
use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use crate::core::*;
use crate::systems::input::{MenuInput};

// Simple pause system using egui modal dialog
pub fn pause_system(
    mut contexts: EguiContexts,
    mut next_state: ResMut<NextState<GameState>>,
    mut post_mission: ResMut<PostMissionResults>,
    game_mode: Res<GameMode>,
    keyboard: Res<ButtonInput<KeyCode>>,
    gamepads: Query<&Gamepad>, 
    mission_data: Res<MissionData>,
) {
    if !game_mode.paused {
        return;
    }

    let input = MenuInput::new(&keyboard, &gamepads);
    
    // Create modal window
    if let Ok(ctx) = contexts.ctx_mut() {
    egui::Window::new("Game Paused")
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            ui.set_min_width(300.0);
            
            // Title
            ui.vertical_centered(|ui| {
                ui.heading("⏸ MISSION PAUSED");
                ui.separator();
            });
            
            // Mission info
            ui.group(|ui| {
                ui.label("MISSION STATUS:");
                ui.horizontal(|ui| {
                    ui.label("Time:");
                    ui.colored_label(egui::Color32::YELLOW, format!("{:.1}s", mission_data.timer));
                });
                ui.horizontal(|ui| {
                    ui.label("Enemies:");
                    ui.colored_label(egui::Color32::RED, format!("{}", mission_data.enemies_killed));
                });
                ui.horizontal(|ui| {
                    ui.label("Alert:");
                    let alert_color = match mission_data.alert_level {
                        AlertLevel::Green => egui::Color32::GREEN,
                        AlertLevel::Yellow => egui::Color32::YELLOW,
                        AlertLevel::Orange => egui::Color32::from_rgb(255, 165, 0),
                        AlertLevel::Red => egui::Color32::RED,
                    };
                    ui.colored_label(alert_color, format!("{:?}", mission_data.alert_level));
                });
            });
            
            ui.separator();
            
            // Action buttons
            ui.vertical_centered(|ui| {
                if ui.button("📋 Resume Mission (SPACE)").clicked() || input.select {
                    // Resume handled by existing game_mode.paused logic
                }
                
                ui.separator();
                
                if ui.button("⚠️ Abort Mission (Q)").clicked() || input.option {
                    // Set mission as failed/aborted
                    *post_mission = PostMissionResults {
                        success: false,
                        time_taken: mission_data.timer,
                        enemies_killed: mission_data.enemies_killed,
                        terminals_accessed: mission_data.terminals_accessed,
                        credits_earned: 0, // No credits for abort
                        alert_level: mission_data.alert_level,
                    };
                    
                    // Go to post-mission
                    next_state.set(GameState::PostMission);
                }
                
                ui.separator();
                
                // Settings button (future feature)
                if ui.button("⚙️ Settings").clicked() {
                    // TODO: Open in-mission settings
                }
            });
            
            ui.separator();
            
            // Controls help
            ui.vertical_centered(|ui| {
                ui.weak("CONTROLS:");
                ui.weak("SPACE - Resume Mission");
                ui.weak("Q - Abort Mission");
                ui.weak("ESC - Quick Settings");
            });
            
            // Warning about abort
            ui.separator();
            ui.vertical_centered(|ui| {
                ui.colored_label(
                    egui::Color32::from_rgb(200, 100, 100), 
                    "⚠️ Aborting will count as mission failure"
                );
                ui.colored_label(
                    egui::Color32::from_rgb(200, 100, 100), 
                    "⚠️ Agents may be injured during extraction"
                );
            });
        });
    }
}