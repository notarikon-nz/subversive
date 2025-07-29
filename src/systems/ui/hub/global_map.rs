// src/systems/ui/hub/global_map.rs - egui version
use bevy::prelude::*;
use bevy_egui::egui;
use crate::core::*;

// Keep the InteractiveCity component for compatibility
#[derive(Component)]
pub struct InteractiveCity {
    pub city_id: String,
    pub accessible: bool,
}

#[derive(Default)]
pub struct GlobalMapState {
    pub selected_city: Option<String>,
    pub hovered_city: Option<String>,
    pub map_projection: Option<MapProjection>,
    pub city_positions: std::collections::HashMap<String, Vec2>,
}


pub fn show_global_map(
    ui: &mut egui::Ui,
    global_data: &mut GlobalData,
    cities_db: &CitiesDatabase,
    input: &ButtonInput<KeyCode>,
    windows: &Query<&Window>,
    cameras: &Query<(&Camera, &GlobalTransform)>,
    mouse: &ButtonInput<MouseButton>,
    city_query: &Query<(Entity, &Transform, &InteractiveCity)>,
) {
    // Create a local state for the map
    let mut map_state = GlobalMapState::default();
    map_state.selected_city = Some(global_data.cities_progress.current_city.clone());

    // Do we really need this?
    ui.heading("GLOBAL OPERATIONS MAP");
    
    ui.separator();
    
    // Add the visual map here
    draw_visual_map(ui, cities_db, global_data, &mut map_state);
    
    ui.separator();

    // Wait day button - check keyboard availability
    let wait_clicked = ui.button("⏰ Wait Day (W)").clicked();
    let wait_key = input.just_pressed(KeyCode::KeyW);

    // Wait day button
    if wait_clicked || wait_key {
        global_data.current_day += 1;
        let current_day = global_data.current_day;
        
        // Update region alerts
        for region in &mut global_data.regions {
            region.update_alert(current_day);
        }
        
        // Update city alerts
        for (_, city_state) in global_data.cities_progress.city_states.iter_mut() {
            update_city_alert(city_state, current_day);
        }
    }
    
    ui.separator();
    
    // Current city selection
    ui.horizontal(|ui| {
        ui.label("Selected City:");
        if global_data.cities_progress.current_city.is_empty() {
            ui.colored_label(egui::Color32::GRAY, "None selected");
        } else {
            if let Some(city) = cities_db.get_city(&global_data.cities_progress.current_city) {
                ui.colored_label(egui::Color32::YELLOW, &city.name);
                ui.separator();
                ui.label(format!("Pop: {}M", city.population));
                ui.separator();
                ui.label(format!("Corruption: {}/10", city.corruption_level));
            }
        }
    });
    
    ui.separator();
    
    ui.collapsing("City List", |ui| {
        egui::ScrollArea::vertical()
            .max_height(200.0)
            .show(ui, |ui| {
                show_city_list(ui, global_data, cities_db, &mut map_state);
            });
    });

}

fn draw_visual_map(
    ui: &mut egui::Ui,
    cities_db: &CitiesDatabase,
    global_data: &mut GlobalData,
    map_state: &mut GlobalMapState,
) {

    // Allocate space for the map
    let available_size = ui.available_size();
    let map_size = egui::Vec2::new(
        available_size.x.min(1200.0).max(600.0), 
        400.0  // Fixed height for the map
    );
    
    let (response, painter) = ui.allocate_painter(map_size, egui::Sense::click());
    let map_rect = response.rect;
    
    // Dark blue background
    painter.rect_filled(
        map_rect,
        0.0,
        egui::Color32::from_rgb(13, 13, 26),
    );

    // Initialize map projection
    if map_state.map_projection.is_none() {
        map_state.map_projection = Some(MapProjection::new(map_rect.width(), map_rect.height()));
    }
    
    let projection = map_state.map_projection.as_ref().unwrap();
    let accessible_cities = cities_db.get_accessible_cities(&global_data);
    let all_cities = cities_db.get_all_cities();
    
    let mut clicked_city = None;

    // Draw all cities
    for city in &all_cities {
        let pixel_pos = projection.lat_lon_to_pixel(&city.coordinates);
        map_state.city_positions.insert(city.id.clone(), pixel_pos);
        
        let is_accessible = accessible_cities.iter().any(|acc_city| acc_city.id == city.id);
        let is_selected = map_state.selected_city.as_ref() == Some(&city.id);
        let is_hovered = map_state.hovered_city.as_ref() == Some(&city.id);
        let city_state = global_data.cities_progress.get_city_state(&city.id);
        
        // City color based on state
        let city_color = if !is_accessible {
            egui::Color32::from_rgba_unmultiplied(77, 77, 77, 102)
        } else if city_state.completed {
            egui::Color32::from_rgb(51, 204, 51)
        } else {
            match city_state.alert_level {
                AlertLevel::Green => egui::Color32::from_rgb(51, 204, 51),
                AlertLevel::Yellow => egui::Color32::from_rgb(204, 204, 51),
                AlertLevel::Orange => egui::Color32::from_rgb(204, 128, 51),
                AlertLevel::Red => egui::Color32::from_rgb(204, 51, 51),
                _ => egui::Color32::WHITE,
            }
        };
        
        let circle_radius = if is_selected { 8.0 } else if is_hovered { 6.0 } else { 6.0 };
        
        // Convert to screen coordinates
        let city_screen_pos = egui::Pos2::new(
            map_rect.left() + pixel_pos.x,
            map_rect.top() + pixel_pos.y
        );
        
        // Draw city circle
        painter.circle_filled(
            city_screen_pos,
            circle_radius,
            city_color,
        );
        
        // Draw border
        let border_color = if is_selected {
            egui::Color32::WHITE
        } else if is_accessible {
            egui::Color32::from_rgb(102, 102, 102)
        } else {
            egui::Color32::from_rgba_unmultiplied(51, 51, 51, 102)
        };
        
        painter.circle_stroke(
            city_screen_pos,
            circle_radius,
            egui::Stroke::new(1.0, border_color),
        );
        
        // Corporation indicator
        if is_accessible {
            let corp_color = match city.controlling_corp {
                Corporation::Omnicorp => egui::Color32::from_rgb(0, 100, 200),
                Corporation::Syndicate => egui::Color32::from_rgb(200, 0, 100),
                Corporation::Helix => egui::Color32::from_rgb(100, 200, 0),
                Corporation::Independent => egui::Color32::GRAY,
                _ => egui::Color32::WHITE,
            };
            
            painter.rect_filled(
                egui::Rect::from_center_size(
                    egui::Pos2::new(city_screen_pos.x + 5.0, city_screen_pos.y - 5.0),
                    egui::Vec2::splat(5.0)
                ),
                0.0,
                corp_color,
            );
        }
        
        // City label
        let text_color = if is_accessible {
            egui::Color32::WHITE
        } else {
            egui::Color32::from_rgba_unmultiplied(128, 128, 128, 153)
        };
        
        painter.text(
            egui::Pos2::new(city_screen_pos.x, city_screen_pos.y - 20.0),
            egui::Align2::CENTER_CENTER,
            &city.name,
            egui::FontId::proportional(10.0),
            text_color,
        );
        
        // Check for mouse interaction
        // City position is stored, so only need to and bottom offsets
        if let Some(hover_pos) = response.hover_pos() {
            let distance = ((hover_pos.x - city_screen_pos.x).powi(2) + 
                           (hover_pos.y - city_screen_pos.y).powi(2)).sqrt();
            
            if distance <= 15.0 {
                map_state.hovered_city = Some(city.id.clone());
                
                if response.clicked() && is_accessible {
                    clicked_city = Some(city.id.clone());
                }
            }
        }
    }
    
    // Update selected city after the loop
    if let Some(city_id) = clicked_city {
        map_state.selected_city = Some(city_id.clone());
        global_data.cities_progress.current_city = city_id;
    }
    
    // Info overlay
    painter.text(
        egui::Pos2::new(map_rect.left() + 5.0, map_rect.top() + 5.0),
        egui::Align2::LEFT_TOP,
        format!(
            "Cities: {} total, {} accessible",
            all_cities.len(),
            accessible_cities.len()
        ),
        egui::FontId::proportional(10.0),
        egui::Color32::from_rgb(204, 204, 51),
    );
    
    // Show tooltip for hovered city
    if let Some(hovered_city_id) = &map_state.hovered_city {
        if let Some(city) = cities_db.get_city(hovered_city_id) {
            response.on_hover_ui(|ui| {
                show_city_tooltip_content(ui, city, global_data);
            });
        }
    }
}

fn show_city_tooltip_content(
    ui: &mut egui::Ui,
    city: &City,
    global_data: &GlobalData,
) {
        let city_state = global_data.cities_progress.get_city_state(&city.id);
        
        ui.strong(format!("{}, {}", city.name, city.country));
        
        ui.horizontal(|ui| {
            ui.label(format!("Pop: {}M", city.population));
            ui.separator();
            ui.label(format!("Corruption: {}/10", city.corruption_level));
            ui.separator();
            
            let alert_color = match city_state.alert_level {
                AlertLevel::Green => egui::Color32::from_rgb(51, 204, 51),
                AlertLevel::Yellow => egui::Color32::from_rgb(204, 204, 51),
                AlertLevel::Orange => egui::Color32::from_rgb(204, 128, 51),
                AlertLevel::Red => egui::Color32::from_rgb(204, 51, 51),
                _ => egui::Color32::WHITE,
            };
            
            ui.colored_label(alert_color, format!("{:?}", city_state.alert_level));
        });
        
        let corp_color = match city.controlling_corp {
            Corporation::Omnicorp => egui::Color32::from_rgb(0, 100, 200),
            Corporation::Syndicate => egui::Color32::from_rgb(200, 0, 100),
            Corporation::Helix => egui::Color32::from_rgb(100, 200, 0),
            Corporation::Independent => egui::Color32::GRAY,
            _ => egui::Color32::WHITE,
        };
        ui.colored_label(corp_color, format!("{:?}", city.controlling_corp));
        
        if city_state.completed {
            ui.colored_label(egui::Color32::GREEN, format!("✓ COMPLETED ({} visits)", city_state.times_visited));
        }
}

fn show_city_list(
    ui: &mut egui::Ui,
    global_data: &mut GlobalData,
    cities_db: &CitiesDatabase,
    map_state: &mut GlobalMapState,
) {
    let accessible_cities = cities_db.get_accessible_cities(global_data);
    let all_cities = cities_db.get_all_cities();
    
    ui.group(|ui| {
        ui.label(format!("ACCESSIBLE CITIES ({}/{})", accessible_cities.len(), all_cities.len()));
        
        for city in &accessible_cities {
            let city_state = global_data.cities_progress.get_city_state(&city.id);
            let is_selected = map_state.selected_city.as_ref() == Some(&city.id);
            
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    if ui.selectable_label(is_selected, &city.name).clicked() {
                        map_state.selected_city = Some(city.id.clone());
                        global_data.cities_progress.current_city = city.id.clone();
                    }
                    
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let alert_color = match city_state.alert_level {
                            AlertLevel::Green => egui::Color32::GREEN,
                            AlertLevel::Yellow => egui::Color32::YELLOW,
                            AlertLevel::Orange => egui::Color32::from_rgb(255, 165, 0),
                            AlertLevel::Red => egui::Color32::RED,
                            _ => egui::Color32::WHITE,
                        };
                        ui.colored_label(alert_color, format!("{:?}", city_state.alert_level));
                    });
                });
                
                if city_state.completed {
                    ui.colored_label(egui::Color32::GREEN, format!("✓ COMPLETED ({} visits)", city_state.times_visited));
                }
            });
        }
    });
}

fn update_city_alert(city_state: &mut CityState, current_day: u32) {
    if current_day > city_state.last_mission_day + 7 {
        city_state.alert_level = match city_state.alert_level {
            AlertLevel::Red => AlertLevel::Orange,
            AlertLevel::Orange => AlertLevel::Yellow,
            AlertLevel::Yellow => AlertLevel::Green,
            AlertLevel::Green => AlertLevel::Green,
        };
    }
}

