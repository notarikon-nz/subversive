// src/systems/ui/post_mission.rs - egui version with correct API usage
use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
// PLACEHOLDER
use egui_plot::{Bar, BarChart, Line, PlotPoints}; /*Plot,*/ 
use crate::core::*;

#[derive(Resource, Default)]
pub struct PostMissionUIState {
    pub show_detailed_stats: bool,
    pub show_agent_performance: bool,
    pub animation_progress: f32,
}

pub fn post_mission_ui_system(
    mut contexts: EguiContexts,
    mut next_state: ResMut<NextState<GameState>>,
    mut processed: ResMut<PostMissionProcessed>,
    mut ui_state: ResMut<PostMissionUIState>,
    post_mission: Res<PostMissionResults>,
    global_data: Res<GlobalData>,
    input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
) {

    if !processed.0 {
        return;
    }

    // Handle input
    if input.just_pressed(KeyCode::KeyR) {
        processed.0 = false;
        next_state.set(GameState::GlobalMap);
        return;
    }
    
    if input.just_pressed(KeyCode::Escape) {
        std::process::exit(0);
    }

    // Update animation
    ui_state.animation_progress = (ui_state.animation_progress + time.delta_secs() * 2.0).min(1.0);

    // Full-screen overlay
    if let Ok(ctx) = contexts.ctx_mut() {
    egui::CentralPanel::default()
        .frame(egui::Frame::new().fill(egui::Color32::from_rgba_unmultiplied(0, 0, 0, 200)))
        .show(ctx, |ui| {
            
            // Main results window
            egui::Window::new("Mission Results")
                .collapsible(false)
                .resizable(true)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .default_width(600.0)
                .default_height(500.0)
                .show(ui.ctx(), |ui| {
                    
                    // Header with success/failure
                    ui.vertical_centered(|ui| {
                        let (title, title_color) = if post_mission.success {
                            ("🎯 MISSION SUCCESS", egui::Color32::GREEN)
                        } else {
                            ("💥 MISSION FAILED", egui::Color32::RED)
                        };
                        
                        ui.colored_label(title_color, egui::RichText::new(title).heading().strong());
                    });
                    
                    ui.separator();
                    
                    // Mission statistics with animated bars
                    ui.group(|ui| {
                        ui.heading("📊 MISSION STATISTICS");
                        
                        // Time and basic stats
                        ui.horizontal(|ui| {
                            ui.vertical(|ui| {
                                ui.label("Mission Duration:");
                                ui.colored_label(egui::Color32::YELLOW, format!("{:.1}s", post_mission.time_taken));
                            });
                            
                            ui.separator();
                            
                            ui.vertical(|ui| {
                                ui.label("Enemies Neutralized:");
                                ui.colored_label(egui::Color32::RED, format!("{}", post_mission.enemies_killed));
                            });
                            
                            ui.separator();
                            
                            ui.vertical(|ui| {
                                ui.label("Data Accessed:");
                                ui.colored_label(egui::Color32::BLUE, format!("{}", post_mission.terminals_accessed));
                            });
                        });
                        
                        ui.separator();
                        
                        // Performance chart
                        if ui.button("📈 Show Performance Chart").clicked() {
                            ui_state.show_detailed_stats = !ui_state.show_detailed_stats;
                        }
                        
                        if ui_state.show_detailed_stats {
                            ui.separator();
                            create_performance_chart(ui, &post_mission, ui_state.animation_progress);
                        }
                    });
                    
                    ui.separator();
                    
                    // Credits and rewards
                    ui.group(|ui| {
                        ui.heading("💰 REWARDS");
                        
                        if post_mission.success {
                            ui.horizontal(|ui| {
                                ui.label("Credits Earned:");
                                ui.colored_label(egui::Color32::YELLOW, format!("${}", post_mission.credits_earned));
                                
                                // Animated credit counter could go here
                                let animated_credits = (post_mission.credits_earned as f32 * ui_state.animation_progress) as u32;
                                if ui_state.animation_progress < 1.0 {
                                    ui.weak(format!("(+${})", animated_credits));
                                }
                            });
                            
                            // Bonus breakdown
                            ui.group(|ui| {
                                ui.label("Bonus Breakdown:");
                                ui.indent("bonus", |ui| {
                                    ui.label(format!("• Base Mission: ${}", post_mission.credits_earned / 2));
                                    ui.label(format!("• Speed Bonus: ${}", post_mission.credits_earned / 4));
                                    ui.label(format!("• Stealth Bonus: ${}", post_mission.credits_earned / 4));
                                });
                            });
                        } else {
                            ui.colored_label(egui::Color32::RED, "No credits earned - Mission failed");
                            ui.weak("Complete objectives to earn rewards");
                        }
                    });
                    
                    ui.separator();
                    
                    // Agent experience and progression
                    if post_mission.success {
                        ui.group(|ui| {
                            ui.heading("👥 AGENT PROGRESSION");
                            
                            let exp_gained = 10 + (post_mission.enemies_killed * 5);
                            ui.colored_label(egui::Color32::from_rgb(100, 200, 255), format!("Experience Gained: +{}", exp_gained));
                            
                            ui.separator();
                            
                            if ui.button("👁 Show Agent Details").clicked() {
                                ui_state.show_agent_performance = !ui_state.show_agent_performance;
                            }
                            
                            if ui_state.show_agent_performance {
                                ui.separator();
                                create_agent_progression_display(ui, &global_data, exp_gained);
                            }
                        });
                    } else {
                        ui.group(|ui| {
                            ui.heading("🏥 CASUALTY REPORT");
                            ui.colored_label(egui::Color32::RED, "Mission failure - checking agent status...");
                            
                            // Show which agents might be injured
                            for i in 0..3 {
                                if global_data.agent_recovery[i] > global_data.current_day {
                                    let days_left = global_data.agent_recovery[i] - global_data.current_day;
                                    ui.colored_label(
                                        egui::Color32::YELLOW, 
                                        format!("Agent {}: Recovering ({} days)", i + 1, days_left)
                                    );
                                } else {
                                    ui.colored_label(
                                        egui::Color32::GREEN, 
                                        format!("Agent {}: Uninjured", i + 1)
                                    );
                                }
                            }
                        });
                    }
                    
                    ui.separator();
                    
                    // Action buttons
                    ui.vertical_centered(|ui| {
                        ui.horizontal(|ui| {
                            if ui.button("🗺️ Return to Map (R)").clicked() || input.just_pressed(KeyCode::KeyR) {
                                processed.0 = false;
                                next_state.set(GameState::GlobalMap);
                            }
                            
                            if ui.button("❌ Quit Game (ESC)").clicked() {
                                std::process::exit(0);
                            }
                        });
                        
                        ui.separator();
                        ui.weak("R: Return to Map | ESC: Quit Game");
                    });
                });
        });
    }
}

fn create_performance_chart(ui: &mut egui::Ui, post_mission: &PostMissionResults, animation_progress: f32) {
    // PLACEHOLDER
    // Create bars with correct API

    let bars = vec![
        Bar::new(0.0, post_mission.enemies_killed as f64 * animation_progress as f64),
        Bar::new(1.0, post_mission.terminals_accessed as f64 * animation_progress as f64),
        Bar::new(2.0, (post_mission.credits_earned / 100) as f64 * animation_progress as f64),
        Bar::new(3.0, post_mission.time_taken as f64 * animation_progress as f64 / 10.0),
    ];
    
    // Create chart with correct constructor
    let chart = BarChart::new("Mission Performance", bars);

    /*
    Plot::new("mission_chart")
        .height(150.0)
        .show_axes([false, true])
        .show(ui, |plot_ui| {
            plot_ui.bar_chart(chart);
        });
    */

    // Add labels manually
    ui.horizontal(|ui| {
        ui.small("Enemies");
        ui.separator();
        ui.small("Terminals");
        ui.separator();
        ui.small("Credits/100");
        ui.separator();
        ui.small("Time/10s");
    });
}

fn create_agent_progression_display(ui: &mut egui::Ui, global_data: &GlobalData, exp_gained: u32) {
    for i in 0..3 {
        ui.horizontal(|ui| {
            ui.label(format!("Agent {}:", i + 1));
            
            let current_level = global_data.agent_levels[i];
            let current_exp = global_data.agent_experience[i];
            let next_level_exp = experience_for_level(current_level + 1);
            let new_exp = current_exp + exp_gained;
            
            // Level display
            ui.label(format!("Lv{}", current_level));
            
            // Experience bar
            let progress = current_exp as f32 / next_level_exp as f32;
            
            ui.add(egui::ProgressBar::new(progress.min(1.0))
                .text(format!("{}/{}", current_exp, next_level_exp)));
            
            // Show level up if applicable
            if new_exp >= next_level_exp {
                ui.colored_label(egui::Color32::YELLOW, "LEVEL UP!");
            } else {
                ui.colored_label(egui::Color32::GREEN, format!("+{} XP", exp_gained));
            }
        });
    }
    
    // Create line chart with correct API
    let points: PlotPoints = (0..3)
        .map(|i| {
            let current_exp = global_data.agent_experience[i] as f64;
            [i as f64, current_exp]
        })
        .collect();
    

    // Create line with correct constructor
    let line = Line::new("Agent Experience", points);
    
    /*
    Plot::new("agent_exp_chart")
        .height(100.0)
        .show_axes([true, true])
        .show(ui, |plot_ui| {
            plot_ui.line(line);
        });
    */
}