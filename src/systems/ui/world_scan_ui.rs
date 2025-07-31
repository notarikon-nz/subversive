// src/systems/ui/world_scan_ui.rs - Complete UI integration for World Scan System
use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use crate::core::*;
use crate::systems::world_scan::*;

#[derive(Resource, Default)]
pub struct WorldScanUIState {
    pub show_scanner_window: bool,
    pub show_intel_window: bool,
    pub show_network_window: bool,
    pub selected_entity: Option<Entity>,
    pub intel_filter: IntelFilter,
    pub auto_scan: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum IntelFilter {
    All,
    Threats,
    Financial,
    Infrastructure,
    Personnel,
}

impl Default for IntelFilter {
    fn default() -> Self { Self::All }
}

pub fn world_scan_ui_system(
    mut contexts: EguiContexts,
    mut ui_state: ResMut<WorldScanUIState>,
    scan_state: Res<WorldScanState>,
    selection: Res<SelectionState>,
    scanner_query: Query<(Entity, &WorldScanner), With<Agent>>,
    transform_query: Query<&Transform>,
    game_mode: Res<GameMode>,
) {
    if game_mode.paused { return; }
    
    let Ok(ctx) = contexts.ctx_mut() else { return; };

    // Find active scanner
    let active_scanner = selection.selected.iter()
        .filter_map(|&entity| scanner_query.get(entity).ok())
        .next();

    // Main scanner control window
    if ui_state.show_scanner_window || active_scanner.is_some() {
        draw_scanner_control_window(ctx, &mut ui_state, &scan_state, active_scanner);
    }

    // Intel database window
    if ui_state.show_intel_window {
        draw_intel_database_window(ctx, &mut ui_state, &scan_state, &transform_query);
    }

    // Network topology window
    if ui_state.show_network_window {
        draw_network_topology_window(ctx, &mut ui_state, &scan_state);
    }

    // Scan results overlay (temporary popup)
    if let Some(last_scan) = scan_state.scan_history.last() {
        if !last_scan.intel_gathered.is_empty() {
            draw_scan_results_popup(ctx, last_scan);
        }
    }
}

fn draw_scanner_control_window(
    ctx: &egui::Context,
    ui_state: &mut WorldScanUIState,
    scan_state: &WorldScanState,
    active_scanner: Option<(Entity, &WorldScanner)>,
) {
    egui::Window::new("🛰 World Scanner")
        .default_pos(egui::pos2(10.0, 10.0))
        .default_size(egui::vec2(280.0, 200.0))
        .resizable(false)
        .show(ctx, |ui| {
            if let Some((entity, scanner)) = active_scanner {
                // Energy display
                ui.horizontal(|ui| {
                    ui.label("Energy:");
                    let energy_ratio = scanner.energy / scanner.max_energy;
                    let color = if energy_ratio > 0.6 {
                        egui::Color32::from_rgb(0, 255, 100)
                    } else if energy_ratio > 0.3 {
                        egui::Color32::from_rgb(255, 255, 0)
                    } else {
                        egui::Color32::from_rgb(255, 100, 0)
                    };
                    
                    let progress = egui::ProgressBar::new(energy_ratio)
                        .fill(color)
                        .text(format!("{:.0}/{:.0}", scanner.energy, scanner.max_energy));
                    ui.add(progress);
                });

                ui.separator();

                // Scan mode selection
                ui.horizontal(|ui| {
                    ui.label("Mode:");
                    ui.selectable_value(&mut ui_state.auto_scan, false, "Manual");
                    ui.selectable_value(&mut ui_state.auto_scan, true, "Auto");
                });

                // Current scan mode display
                ui.horizontal(|ui| {
                    ui.label("Target:");
                    let mode_text = match scan_state.scan_mode {
                        ScanMode::Infrastructure => "🏭 Infrastructure",
                        ScanMode::Security => "🛡️ Security",
                        ScanMode::Financial => "💰 Financial",
                        ScanMode::Personnel => "👥 Personnel",
                        ScanMode::All => "🌐 All Systems",
                    };
                    ui.colored_label(egui::Color32::from_rgb(100, 200, 255), mode_text);
                });

                ui.separator();

                // Scan controls
                ui.horizontal(|ui| {
                    let can_scan = scanner.energy >= scanner.scan_cost;
                    
                    if ui.add_enabled(can_scan, egui::Button::new("🔍 SCAN"))
                        .on_hover_text("Perform world scan [Enter]")
                        .clicked() 
                    {
                        // Trigger scan event - would be handled by input system
                    }

                    if ui.button("📊 Intel")
                        .on_hover_text("Open intelligence database")
                        .clicked() 
                    {
                        ui_state.show_intel_window = !ui_state.show_intel_window;
                    }

                    if ui.button("🔗 Network")
                        .on_hover_text("Show network topology")
                        .clicked() 
                    {
                        ui_state.show_network_window = !ui_state.show_network_window;
                    }
                });

                // Scanner stats
                ui.separator();
                ui.small(format!("Range: {:.0}m", scanner.range));
                ui.small(format!("Level: {}", scanner.scan_level));
                ui.small(format!("Cost: {:.0} energy/scan", scanner.scan_cost));

                // Quick stats
                ui.separator();
                ui.horizontal(|ui| {
                    ui.small("Discovered:");
                    ui.colored_label(
                        egui::Color32::from_rgb(100, 255, 100),
                        format!("{}", scan_state.discovered_entities.len())
                    );
                    
                    ui.small("Networks:");
                    ui.colored_label(
                        egui::Color32::from_rgb(255, 200, 100),
                        format!("{}", scan_state.network_topology.len())
                    );
                });

            } else {
                ui.vertical_centered(|ui| {
                    ui.colored_label(egui::Color32::GRAY, "No scanner equipped");
                    ui.small("Select an agent with scanning capability");
                });
            }

            // Controls help
            ui.separator();
            ui.small("Controls:");
            ui.small("Tab: Change mode | Enter: Scan");
            ui.small("O: Toggle overlays | Q: Basic scanner");
        });
}

fn draw_intel_database_window(
    ctx: &egui::Context,
    ui_state: &mut WorldScanUIState,
    scan_state: &WorldScanState,
    transform_query: &Query<&Transform>,
) {
    egui::Window::new("📋 Intelligence Database")
        .default_pos(egui::pos2(300.0, 10.0))
        .default_size(egui::vec2(400.0, 500.0))
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Filter:");
                ui.selectable_value(&mut ui_state.intel_filter, IntelFilter::All, "All");
                ui.selectable_value(&mut ui_state.intel_filter, IntelFilter::Threats, "Threats");
                ui.selectable_value(&mut ui_state.intel_filter, IntelFilter::Financial, "Financial");
                ui.selectable_value(&mut ui_state.intel_filter, IntelFilter::Infrastructure, "Infrastructure");
                ui.selectable_value(&mut ui_state.intel_filter, IntelFilter::Personnel, "Personnel");
            });

            ui.separator();

            egui::ScrollArea::vertical()
                .max_height(400.0)
                .show(ui, |ui| {
                    for (entity, scannable) in &scan_state.discovered_entities {
                        if should_show_entity(scannable, ui_state.intel_filter) {
                            draw_entity_intel_card(ui, *entity, scannable, transform_query, ui_state);
                        }
                    }
                });
        });
}

fn draw_network_topology_window(
    ctx: &egui::Context,
    ui_state: &mut WorldScanUIState,
    scan_state: &WorldScanState,
) {
    egui::Window::new("🔗 Network Topology")
        .default_pos(egui::pos2(710.0, 10.0))
        .default_size(egui::vec2(300.0, 400.0))
        .show(ctx, |ui| {
            if scan_state.network_topology.is_empty() {
                ui.vertical_centered(|ui| {
                    ui.colored_label(egui::Color32::GRAY, "No networks discovered");
                    ui.small("Scan infrastructure to map networks");
                });
                return;
            }

            egui::ScrollArea::vertical()
                .max_height(350.0)
                .show(ui, |ui| {
                    for (network_id, entities) in &scan_state.network_topology {
                        ui.group(|ui| {
                            ui.horizontal(|ui| {
                                let network_color = get_network_color(network_id);
                                let color_rect = egui::Rect::from_min_size(
                                    ui.cursor().min,
                                    egui::vec2(12.0, 12.0)
                                );
                                ui.painter().rect_filled(color_rect, 2.0, network_color);
                                ui.allocate_space(egui::vec2(16.0, 12.0));
                                
                                ui.strong(network_id);
                                ui.colored_label(
                                    egui::Color32::GRAY,
                                    format!("({} nodes)", entities.len())
                                );
                            });

                            ui.indent("network_nodes", |ui| {
                                for (i, entity) in entities.iter().enumerate() {
                                    if i >= 5 { // Limit display
                                        ui.small(format!("... and {} more", entities.len() - 5));
                                        break;
                                    }
                                    
                                    if let Some(scannable) = scan_state.discovered_entities.get(entity) {
                                        let type_icon = get_entity_type_icon(&scannable.entity_type);
                                        let threat_color = get_threat_color(scannable.threat_level);
                                        
                                        ui.horizontal(|ui| {
                                            ui.colored_label(threat_color, type_icon);
                                            ui.small(get_entity_type_name(&scannable.entity_type));
                                        });
                                    }
                                }
                            });
                        });
                    }
                });
        });
}

fn draw_scan_results_popup(ctx: &egui::Context, scan_record: &ScanRecord) {
    if scan_record.intel_gathered.is_empty() { return; }

    egui::Window::new("📡 Scan Results")
        .anchor(egui::Align2::CENTER_TOP, egui::vec2(0.0, 100.0))
        .collapsible(false)
        .resizable(false)
        .auto_sized()
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.colored_label(egui::Color32::from_rgb(100, 255, 100), "✓");
                ui.strong(format!("Scan Complete - {} entities found", scan_record.entities_found));
            });

            ui.separator();

            for intel in &scan_record.intel_gathered {
                ui.horizontal(|ui| {
                    ui.colored_label(egui::Color32::from_rgb(200, 200, 255), "•");
                    ui.small(intel);
                });
            }
        });
}

fn draw_entity_intel_card(
    ui: &mut egui::Ui,
    entity: Entity,
    scannable: &ScannableEntity,
    transform_query: &Query<&Transform>,
    ui_state: &mut WorldScanUIState,
) {
    ui.group(|ui| {
        ui.horizontal(|ui| {
            // Entity type icon and name
            let type_icon = get_entity_type_icon(&scannable.entity_type);
            let threat_color = get_threat_color(scannable.threat_level);
            
            ui.colored_label(threat_color, type_icon);
            ui.strong(get_entity_type_name(&scannable.entity_type));
            
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.small_button("📍").on_hover_text("Focus camera").clicked() {
                    ui_state.selected_entity = Some(entity);
                }
                
                // Threat level indicator
                let threat_text = format!("{:?}", scannable.threat_level);
                ui.colored_label(threat_color, threat_text);
            });
        });

        // Entity-specific details
        match &scannable.entity_type {
            ScannableType::PowerStation { capacity, network_id } => {
                ui.small(format!("Capacity: {} MW", capacity));
                ui.small(format!("Network: {}", network_id));
            }
            ScannableType::ATM { bank_id, funds } => {
                ui.small(format!("Bank: {}", bank_id));
                ui.colored_label(egui::Color32::from_rgb(100, 255, 100), format!("${}", funds));
            }
            ScannableType::Enemy { faction, weapon, patrol_route } => {
                ui.small(format!("Faction: {}", faction));
                ui.small(format!("Weapon: {}", weapon));
                if let Some(route) = patrol_route {
                    ui.small(format!("Patrol: {} waypoints", route.len()));
                }
            }
            ScannableType::SecurityCamera { fov, detection_range } => {
                ui.small(format!("FOV: {:.0}°", fov.to_degrees()));
                ui.small(format!("Range: {:.0}m", detection_range));
            }
            ScannableType::AutomatedTurret { damage, range } => {
                ui.colored_label(egui::Color32::from_rgb(255, 100, 100), format!("Damage: {:.0}", damage));
                ui.small(format!("Range: {:.0}m", range));
            }
            _ => {}
        }

        // Intel value
        match &scannable.intel_value {
            IntelValue::Financial(value) => {
                ui.colored_label(egui::Color32::from_rgb(100, 255, 100), format!("Value: ${}", value));
            }
            IntelValue::Tactical(info) => {
                ui.colored_label(egui::Color32::from_rgb(255, 200, 100), info);
            }
            IntelValue::Research(project) => {
                ui.colored_label(egui::Color32::from_rgb(100, 200, 255), format!("Research: {}", project));
            }
            IntelValue::Network(networks) => {
                ui.small(format!("Networks: {}", networks.join(", ")));
            }
            IntelValue::None => {}
        }
    });
}

// === HELPER FUNCTIONS ===
fn should_show_entity(scannable: &ScannableEntity, filter: IntelFilter) -> bool {
    match filter {
        IntelFilter::All => true,
        IntelFilter::Threats => !matches!(scannable.threat_level, ThreatLevel::None),
        IntelFilter::Financial => matches!(scannable.entity_type, 
            ScannableType::ATM { .. } | ScannableType::Terminal { .. }),
        IntelFilter::Infrastructure => matches!(scannable.entity_type,
            ScannableType::PowerStation { .. } | ScannableType::SecurityCamera { .. } | 
            ScannableType::AutomatedTurret { .. } | ScannableType::AccessPoint { .. }),
        IntelFilter::Personnel => matches!(scannable.entity_type,
            ScannableType::Enemy { .. } | ScannableType::Civilian { .. } | 
            ScannableType::Scientist { .. }),
    }
}

fn get_entity_type_icon(entity_type: &ScannableType) -> &'static str {
    match entity_type {
        ScannableType::PowerStation { .. } => "⚡",
        ScannableType::SecurityCamera { .. } => "📹",
        ScannableType::AutomatedTurret { .. } => "🎯",
        ScannableType::AccessPoint { .. } => "🚪",
        ScannableType::ATM { .. } => "💳",
        ScannableType::Terminal { .. } => "💻",
        ScannableType::Enemy { .. } => "👤",
        ScannableType::Civilian { .. } => "👥",
        ScannableType::Scientist { .. } => "🧑‍🔬",
        ScannableType::Vehicle { .. } => "🚗",
        ScannableType::Loot { .. } => "📦",
        ScannableType::ResearchData { .. } => "📊",
    }
}

fn get_entity_type_name(entity_type: &ScannableType) -> String {
    match entity_type {
        ScannableType::PowerStation { .. } => "Power Station".to_string(),
        ScannableType::SecurityCamera { .. } => "Security Camera".to_string(),
        ScannableType::AutomatedTurret { .. } => "Automated Turret".to_string(),
        ScannableType::AccessPoint { .. } => "Access Point".to_string(),
        ScannableType::ATM { .. } => "ATM".to_string(),
        ScannableType::Terminal { .. } => "Terminal".to_string(),
        ScannableType::Enemy { .. } => "Hostile".to_string(),
        ScannableType::Civilian { .. } => "Civilian".to_string(),
        ScannableType::Scientist { .. } => "Scientist".to_string(),
        ScannableType::Vehicle { vehicle_type, .. } => format!("Vehicle ({})", vehicle_type),
        ScannableType::Loot { item_type, .. } => format!("Loot ({})", item_type),
        ScannableType::ResearchData { category, .. } => format!("Research ({})", category),
    }
}

fn get_threat_color(threat_level: ThreatLevel) -> egui::Color32 {
    match threat_level {
        ThreatLevel::None => egui::Color32::from_rgb(100, 255, 100),
        ThreatLevel::Low => egui::Color32::from_rgb(200, 255, 100),
        ThreatLevel::Medium => egui::Color32::from_rgb(255, 200, 100),
        ThreatLevel::High => egui::Color32::from_rgb(255, 150, 100),
        ThreatLevel::Critical => egui::Color32::from_rgb(255, 100, 100),
    }
}

fn get_network_color(network_id: &str) -> egui::Color32 {
    match network_id {
        id if id.contains("power") => egui::Color32::from_rgb(255, 255, 100),
        id if id.contains("security") => egui::Color32::from_rgb(255, 100, 100),
        id if id.contains("financial") => egui::Color32::from_rgb(100, 255, 100),
        _ => egui::Color32::from_rgb(150, 150, 150),
    }
}

// === INTEGRATION HELPER ===
pub fn setup_world_scan_ui_system(app: &mut App) {
    app.init_resource::<WorldScanUIState>()
       .add_systems(Update, world_scan_ui_system.run_if(in_state(GameState::Mission)));
}