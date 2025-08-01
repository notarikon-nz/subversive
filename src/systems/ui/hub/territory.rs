// src/systems/ui/hub/territory.rs - Territory Control UI
use bevy::prelude::*;
use bevy_egui::egui;
use crate::core::*;
use crate::core::game_state::{GlobalData};
use crate::systems::ui::hub::{HubState, HubTab};

pub fn show_territory_control(
    ui: &mut egui::Ui,
    territory_manager: &mut TerritoryManager,
    progression_tracker: &ProgressionTracker,
    cities_db: &CitiesDatabase,
    campaign_db: &ExtendedCampaignDatabase,
    global_data: &GlobalData,
) {
    ui.heading("TERRITORY CONTROL");
    ui.separator();

    // Overview section
    ui.horizontal(|ui| {
        ui.colored_label(egui::Color32::YELLOW, format!("Controlled Cities: {}", territory_manager.territory_count));
        ui.separator();
        ui.colored_label(egui::Color32::GREEN, format!("Daily Income: {} credits",
            if territory_manager.territory_count > 0 {
                territory_manager.total_territory_income / territory_manager.territory_count as u32
            } else { 0 }));
        ui.separator();
        ui.colored_label(egui::Color32::BLUE, format!("Campaign: {}/{}",
            progression_tracker.campaign_progress.current_chapter,
            progression_tracker.campaign_progress.total_chapters));
    });

    ui.separator();

    // Controlled territories list
    if !territory_manager.controlled_cities.is_empty() {
        ui.collapsing("Controlled Territories", |ui| {
            egui::ScrollArea::vertical().max_height(300.0).show(ui, |ui| {
                // Collect tax rate changes to apply after iteration
                let mut tax_changes = Vec::new();

                for (city_id, territory) in territory_manager.controlled_cities.iter() {
                    if let Some(city) = cities_db.get_city(city_id) {
                        let tax_change = show_territory_details(ui, city, territory);
                        if let Some(new_rate) = tax_change {
                            tax_changes.push((city_id.clone(), new_rate));
                        }
                        ui.separator();
                    }
                }

                // Apply tax changes after iteration is complete
                for (city_id, new_rate) in tax_changes {
                    territory_manager.set_tax_rate(&city_id, new_rate);
                }
            });
        });
    } else {
        ui.colored_label(egui::Color32::GRAY, "No territories under control");
        ui.weak("Complete missions successfully to establish control over cities.");
    }

    ui.separator();

    // Campaign progress
    show_campaign_progress(ui, progression_tracker, campaign_db, global_data);

    ui.separator();

    // Win conditions
    show_win_conditions(ui, &progression_tracker.win_conditions, territory_manager);
}

fn show_territory_details(
    ui: &mut egui::Ui,
    city: &City,
    territory: &TerritoryControl,
) -> Option<f32> {
    let mut tax_change = None;

    ui.group(|ui| {
        // City header
        ui.horizontal(|ui| {
            ui.strong(&city.name);

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let control_color = match territory.control_level {
                    ControlLevel::None => egui::Color32::GRAY,
                    ControlLevel::Contested => egui::Color32::from_rgb(255, 165, 0),
                    ControlLevel::Established => egui::Color32::YELLOW,
                    ControlLevel::Dominant => egui::Color32::GREEN,
                };
                ui.colored_label(control_color, format!("{:?}", territory.control_level));
            });
        });

        // Control strength bar
        ui.horizontal(|ui| {
            ui.label("Control:");
            let control_rect = ui.allocate_space(egui::vec2(100.0, 10.0)).1;
            ui.painter().rect_filled(control_rect, 0.0, egui::Color32::DARK_GRAY);

            let fill_width = control_rect.width() * territory.control_strength;
            let fill_rect = egui::Rect::from_min_size(
                control_rect.min,
                egui::vec2(fill_width, control_rect.height())
            );
            ui.painter().rect_filled(fill_rect, 0.0, egui::Color32::GREEN);

            ui.label(format!("{:.0}%", territory.control_strength * 100.0));
        });

        // Resistance level
        if territory.resistance_level > 0.1 {
            ui.horizontal(|ui| {
                ui.label("Resistance:");
                let resistance_color = if territory.resistance_level > 0.7 {
                    egui::Color32::RED
                } else if territory.resistance_level > 0.4 {
                    egui::Color32::from_rgb(255, 165, 0)
                } else {
                    egui::Color32::YELLOW
                };
                ui.colored_label(resistance_color, format!("{:.0}%", territory.resistance_level * 100.0));
            });
        }

        // Tax rate slider
        ui.horizontal(|ui| {
            ui.label("Tax Rate:");
            let max_rate = territory.control_level.max_tax_rate();
            let mut tax_rate = territory.tax_rate;

            if ui.add(egui::Slider::new(&mut tax_rate, 0.0..=max_rate)
                .step_by(0.01)
                .suffix("%")
                .custom_formatter(|n, _| format!("{:.1}", n * 100.0))
                .custom_parser(|s| s.parse::<f64>().map(|n| n / 100.0).ok()))
                .changed()
            {
                tax_change = Some(tax_rate);
            }
        });

        // Income information
        ui.horizontal(|ui| {
            let daily_income = (city.population as f32 * 1000.0 * territory.tax_rate * territory.control_strength) as u32;
            ui.colored_label(egui::Color32::GREEN, format!("Daily Income: {} credits", daily_income));
            ui.separator();
            ui.label(format!("Total Collected: {}", territory.total_tax_collected));
            ui.separator();
            ui.label(format!("Days Controlled: {}", territory.days_controlled));
        });

        // Warning for high resistance
        if territory.resistance_level > 0.7 {
            ui.colored_label(egui::Color32::RED, "⚠ High resistance - reduce tax rate to maintain control");
        }

        if territory.control_strength < 0.2 {
            ui.colored_label(egui::Color32::RED, "⚠ Control weakening - territory at risk");
        }
    });

    tax_change
}

fn show_campaign_progress(
    ui: &mut egui::Ui,
    progression_tracker: &ProgressionTracker,
    campaign_db: &ExtendedCampaignDatabase,
    global_data: &GlobalData,
) {
    ui.collapsing("Campaign Progress", |ui| {
        let progress = &progression_tracker.campaign_progress;

        // Progress bar
        ui.horizontal(|ui| {
            ui.label("Campaign:");
            let progress_rect = ui.allocate_space(egui::vec2(200.0, 12.0)).1;
            ui.painter().rect_filled(progress_rect, 0.0, egui::Color32::DARK_GRAY);

            let fill_width = progress_rect.width() * (progress.current_chapter as f32 / progress.total_chapters as f32);
            let fill_rect = egui::Rect::from_min_size(
                progress_rect.min,
                egui::vec2(fill_width, progress_rect.height())
            );
            ui.painter().rect_filled(fill_rect, 0.0, egui::Color32::BLUE);

            ui.label(format!("{}/{}", progress.current_chapter, progress.total_chapters));
        });

        // Current act info
        if let Some(current_act) = campaign_db.get_current_act(progress.current_chapter) {
            ui.group(|ui| {
                ui.colored_label(egui::Color32::BLUE, format!("Act {}: {}",
                    current_act.id, current_act.title));
                ui.weak(&current_act.description);

                // Show current chapter within act
                if let Some(current_chapter) = current_act.chapters.iter()
                    .find(|c| c.id == progress.current_chapter) {
                    ui.separator();
                    ui.colored_label(egui::Color32::YELLOW, format!("Chapter {}: {}",
                        current_chapter.id, current_chapter.title));
                    ui.weak(&current_chapter.story_beat);
                }
            });
        }

        // Next chapters preview
        ui.collapsing("Upcoming Chapters", |ui| {
            let completed_cities: std::collections::HashSet<String> =
                global_data.cities_progress.city_states.iter()
                    .filter(|(_, state)| state.completed)
                    .map(|(id, _)| id.clone())
                    .collect();

            // Show next few chapters from current act
            if let Some(current_act) = campaign_db.get_current_act(progress.current_chapter) {
                for chapter in current_act.chapters.iter()
                    .filter(|c| c.id > progress.current_chapter)
                    .take(3) {

                    let available = chapter.prerequisites.iter()
                        .all(|prereq| completed_cities.contains(prereq));
                    let color = if available { egui::Color32::WHITE } else { egui::Color32::GRAY };

                    ui.horizontal(|ui| {
                        ui.colored_label(color, format!("Ch.{}: {}", chapter.id, chapter.title));
                        if !available {
                            ui.colored_label(egui::Color32::RED, "[LOCKED]");
                        }
                    });
                }
            }
        });
    });
}

fn show_win_conditions(
    ui: &mut egui::Ui,
    win_conditions: &WinConditions,
    territory_manager: &TerritoryManager,
) {
    ui.collapsing("Victory Conditions", |ui| {
        // Cities controlled
        let cities_color = if territory_manager.territory_count >= win_conditions.min_cities_controlled {
            egui::Color32::GREEN
        } else {
            egui::Color32::RED
        };
        ui.horizontal(|ui| {
            ui.colored_label(cities_color, format!("Cities Controlled: {}/{}",
                territory_manager.territory_count, win_conditions.min_cities_controlled));
            if territory_manager.territory_count >= win_conditions.min_cities_controlled {
                ui.colored_label(egui::Color32::GREEN, "✓");
            }
        });

        // Daily income
        let daily_income = if territory_manager.territory_count > 0 {
            territory_manager.total_territory_income / territory_manager.territory_count as u32
        } else {
            0
        };
        let income_color = if daily_income >= win_conditions.min_daily_income {
            egui::Color32::GREEN
        } else {
            egui::Color32::RED
        };
        ui.horizontal(|ui| {
            ui.colored_label(income_color, format!("Daily Income: {}/{}",
                daily_income, win_conditions.min_daily_income));
            if daily_income >= win_conditions.min_daily_income {
                ui.colored_label(egui::Color32::GREEN, "✓");
            }
        });

        // Campaign completion
        if win_conditions.campaign_complete {
            ui.colored_label(egui::Color32::GREEN, "🏆 CAMPAIGN COMPLETE - VICTORY ACHIEVED!");
        } else {
            ui.colored_label(egui::Color32::YELLOW, "Campaign in progress...");
        }
    });
}

// Helper function to integrate with existing hub system
pub fn integrate_territory_tab(
    ui: &mut egui::Ui,
    hub_state: &mut HubState,
    territory_manager: &mut TerritoryManager,
    progression_tracker: &ProgressionTracker,
    cities_db: &CitiesDatabase,
    campaign_db: &ExtendedCampaignDatabase,
    global_data: &GlobalData,
) {
    if ui.selectable_label(matches!(hub_state.active_tab, HubTab::Territory), "Territory").clicked() {
        hub_state.active_tab = HubTab::Territory;
    }

    if matches!(hub_state.active_tab, HubTab::Territory) {
        show_territory_control(ui, territory_manager, progression_tracker, cities_db, campaign_db, global_data);
    }
}
