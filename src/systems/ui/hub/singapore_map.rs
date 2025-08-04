// src/systems/ui/hub/singapore_map.rs - Vector-based Singapore Map
use bevy::prelude::*;
use bevy_egui::egui;
use crate::core::*;
use std::collections::HashMap;

// === SINGAPORE MAP STRUCTURES ===

#[derive(Debug, Default, Clone, Resource)]
pub struct SingaporeVectorMap {
    pub districts: HashMap<String, DistrictGeometry>,
    pub mrt_system: MRTNetwork,
    pub landmarks: Vec<MapLandmark>,
    pub coastline: Vec<Vec2>,
    pub viewport: MapViewport,
    pub bounds: MapBounds,
}

#[derive(Debug, Clone)]
pub struct DistrictGeometry {
    pub id: String,
    pub boundary: Vec<Vec2>,           // District polygon in Singapore coordinates
    pub center: Vec2,                  // Center point for labels/icons
    pub area: f32,                     // Area for scaling
    pub control_color: egui::Color32,  // Dynamic color based on control
    pub border_style: BorderStyle,     // Visual style
}

#[derive(Debug, Default, Clone)]
pub struct MRTNetwork {
    pub lines: HashMap<String, MRTLine>,
    pub stations: HashMap<String, MRTStation>,
}

#[derive(Debug, Clone)]
pub struct MRTLine {
    pub name: String,
    pub color: egui::Color32,
    pub path: Vec<Vec2>,              // Line path coordinates
    pub stations: Vec<String>,        // Station IDs along this line
}

#[derive(Debug, Clone)]
pub struct MRTStation {
    pub id: String,
    pub name: String,
    pub position: Vec2,
    pub lines: Vec<String>,           // Which MRT lines serve this station
    pub interchange: bool,            // Major interchange station
}

#[derive(Debug, Clone)]
pub struct MapLandmark {
    pub name: String,
    pub position: Vec2,
    pub landmark_type: LandmarkType,
    pub importance: u8,               // 1-5, affects visibility at zoom levels
}

#[derive(Debug, Clone)]
pub enum LandmarkType {
    Airport,
    Port,
    Government,
    Corporate,
    University,
    Hospital,
    Shopping,
    Tourist,
}

#[derive(Debug, Clone)]
pub enum BorderStyle {
    Controlled,      // Solid line, liberation colors
    Contested,       // Dashed line, warning colors
    Corporate,       // Thick line, corporate colors
    Neutral,         // Thin line, gray
}

#[derive(Debug, Clone)]
pub struct MapViewport {
    pub center: Vec2,
    pub zoom: f32,
    pub rotation: f32,
}

#[derive(Debug, Default, Clone)]
pub struct MapBounds {
    pub min: Vec2,
    pub max: Vec2,
}

// === SINGAPORE MAP STATE ===
#[derive(Default, Resource)]
pub struct SingaporeMapState {
    pub selected_district: Option<String>,
    pub hovered_district: Option<String>,
    pub viewport: MapViewport,
    pub show_mrt: bool,
    pub show_landmarks: bool,
    pub show_surveillance: bool,
    pub show_population: bool,
}

impl Default for MapViewport {
    fn default() -> Self {
        // Center on Singapore (approximate)
        Self {
            center: Vec2::new(103.8198, 1.3521), // Singapore coordinates
            zoom: 1.0,
            rotation: 0.0,
        }
    }
}

// === MAP IMPLEMENTATION ===
impl SingaporeVectorMap {
    pub fn new() -> Self {
        Self {
            districts: create_singapore_districts(),
            mrt_system: create_mrt_network(),
            landmarks: create_landmarks(),
            coastline: create_coastline(),
            viewport: MapViewport::default(),
            bounds: MapBounds {
                min: Vec2::new(103.6, 1.2),    // Southwest Singapore
                max: Vec2::new(104.0, 1.5),    // Northeast Singapore
            },
        }
    }

    pub fn update_from_game_state(&mut self, territory_manager: &TerritoryManager, campaign_db: &NeoSingaporeCampaignDatabase) {

        for (district_id, geometry) in self.districts.iter_mut() {

            if let Some(control) = territory_manager.get_district(district_id) {

                geometry.control_color = match control.control_level {
                    ControlLevel::Corporate => egui::Color32::from_rgb(120, 120, 120),
                    ControlLevel::Contested => egui::Color32::from_rgb(255, 165, 0), // Orange
                    ControlLevel::Liberated => egui::Color32::from_rgb(100, 200, 100), // Light green
                    ControlLevel::Secured => egui::Color32::from_rgb(50, 150, 50), // Green
                    ControlLevel::Autonomous => egui::Color32::from_rgb(0, 255, 100), // Bright green
                };
                
                geometry.border_style = match control.control_level {
                    ControlLevel::Corporate => BorderStyle::Corporate,
                    ControlLevel::Contested => BorderStyle::Contested,
                    _ => BorderStyle::Controlled,
                };
            
            } else if let Some(district_data) = campaign_db.get_district(district_id) {
                // Uncontrolled district - show corporate control

                geometry.control_color = match district_data.controlling_corp {
                    Corporation::Nexus => egui::Color32::from_rgb(0, 100, 200),     // Blue
                    Corporation::Omnicorp => egui::Color32::from_rgb(200, 100, 0),  // Orange
                    Corporation::Helix => egui::Color32::from_rgb(100, 0, 200),     // Purple
                    Corporation::Aegis => egui::Color32::from_rgb(200, 0, 0),       // Red
                    Corporation::Syndicate => egui::Color32::from_rgb(150, 150, 150), // Gray
                    _ => egui::Color32::GRAY,
                };

                geometry.border_style = BorderStyle::Corporate;

            }
        }
    }

    fn get_control_color(&self, control: &DistrictControl) -> egui::Color32 {
        match control.control_level {
            ControlLevel::Corporate => egui::Color32::from_rgb(120, 120, 120),
            ControlLevel::Contested => egui::Color32::from_rgb(255, 165, 0), // Orange
            ControlLevel::Liberated => egui::Color32::from_rgb(100, 200, 100), // Light green
            ControlLevel::Secured => egui::Color32::from_rgb(50, 150, 50), // Green
            ControlLevel::Autonomous => egui::Color32::from_rgb(0, 255, 100), // Bright green
        }
    }

    fn get_corporate_color(&self, corp: &Corporation) -> egui::Color32 {
        match corp {
            Corporation::Nexus => egui::Color32::from_rgb(0, 100, 200),     // Blue
            Corporation::Omnicorp => egui::Color32::from_rgb(200, 100, 0),  // Orange
            Corporation::Helix => egui::Color32::from_rgb(100, 0, 200),     // Purple
            Corporation::Aegis => egui::Color32::from_rgb(200, 0, 0),       // Red
            Corporation::Syndicate => egui::Color32::from_rgb(150, 150, 150), // Gray
            _ => egui::Color32::GRAY,
        }
    }

    fn get_border_style(&self, control: &DistrictControl) -> BorderStyle {
        match control.control_level {
            ControlLevel::Corporate => BorderStyle::Corporate,
            ControlLevel::Contested => BorderStyle::Contested,
            _ => BorderStyle::Controlled,
        }
    }

    pub fn world_to_screen(&self, world_pos: Vec2, screen_rect: egui::Rect) -> egui::Pos2 {
        // Convert Singapore lat/lon to screen coordinates
        let normalized = Vec2::new(
            (world_pos.x - self.bounds.min.x) / (self.bounds.max.x - self.bounds.min.x),
            1.0 - (world_pos.y - self.bounds.min.y) / (self.bounds.max.y - self.bounds.min.y), // Flip Y
        );

        let screen_pos = Vec2::new(
            screen_rect.left() + normalized.x * screen_rect.width(),
            screen_rect.top() + normalized.y * screen_rect.height(),
        );

        egui::Pos2::new(screen_pos.x, screen_pos.y)
    }

    pub fn screen_to_world(&self, screen_pos: egui::Pos2, screen_rect: egui::Rect) -> Vec2 {
        let normalized = Vec2::new(
            (screen_pos.x - screen_rect.left()) / screen_rect.width(),
            (screen_pos.y - screen_rect.top()) / screen_rect.height(),
        );

        Vec2::new(
            self.bounds.min.x + normalized.x * (self.bounds.max.x - self.bounds.min.x),
            self.bounds.max.y - normalized.y * (self.bounds.max.y - self.bounds.min.y), // Flip Y back
        )
    }
}

// === MAIN MAP UI FUNCTION ===
pub fn show_singapore_map (
    ui: &mut egui::Ui,
    global_data: &mut GlobalData,
    territory_manager: &TerritoryManager,
    campaign_db: &NeoSingaporeCampaignDatabase,
    map_state: &mut SingaporeMapState,
    singapore_map: &mut SingaporeVectorMap,
    input: &ButtonInput<KeyCode>,
) {
    // Update map with current game state
    singapore_map.update_from_game_state(territory_manager, campaign_db);

    ui.heading("NEO-SINGAPORE OPERATIONS");
    ui.separator();

    // Map controls
    ui.horizontal(|ui| {
        ui.checkbox(&mut map_state.show_mrt, "MRT Lines");
        ui.checkbox(&mut map_state.show_landmarks, "Landmarks");
        ui.checkbox(&mut map_state.show_surveillance, "Surveillance");
        ui.checkbox(&mut map_state.show_population, "Population");
    });

    ui.separator();

    // Main map area
    let available_size = ui.available_size();
    let map_size = egui::Vec2::new(
        available_size.x.min(1000.0).max(600.0),
        available_size.y.min(600.0).max(400.0),
    );

    let (response, painter) = ui.allocate_painter(map_size, egui::Sense::click_and_drag());
    let map_rect = response.rect;

    // Dark background
    painter.rect_filled(map_rect, 0.0, egui::Color32::from_rgb(8, 12, 20));

    // Draw map layers
    draw_coastline(&painter, singapore_map, map_rect);
    draw_districts(&painter, singapore_map, map_rect, map_state, territory_manager);
    
    if map_state.show_mrt {
        draw_mrt_system(&painter, singapore_map, map_rect);
    }
    
    if map_state.show_landmarks {
        draw_landmarks(&painter, singapore_map, map_rect);
    }

    // Handle interactions
    handle_map_interactions(&response, map_state, singapore_map, map_rect, global_data);

    // Info panel
    draw_map_info_panel(ui, map_state, territory_manager, campaign_db);

    // Wait day button - check keyboard availability
    let wait_clicked = ui.button("⏰ Wait Day (W)").clicked();
    let wait_key = input.just_pressed(KeyCode::KeyW);

    // Wait day button
    if wait_clicked || wait_key {
        global_data.current_day += 1;
        let current_day = global_data.current_day;
    }

}

// === DRAWING FUNCTIONS ===
fn draw_coastline(painter: &egui::Painter, map: &SingaporeVectorMap, screen_rect: egui::Rect) {
    if map.coastline.len() < 2 { return; }

    let screen_points: Vec<egui::Pos2> = map.coastline.iter()
        .map(|&world_pos| map.world_to_screen(world_pos, screen_rect))
        .collect();

    painter.add(egui::Shape::line(
        screen_points,
        egui::Stroke::new(2.0, egui::Color32::from_rgb(70, 130, 180)),
    ));
}

fn draw_districts(
    painter: &egui::Painter,
    map: &SingaporeVectorMap,
    screen_rect: egui::Rect,
    map_state: &SingaporeMapState,
    territory_manager: &TerritoryManager,
) {
    for (district_id, geometry) in &map.districts {
        // Convert boundary to screen coordinates
        let screen_points: Vec<egui::Pos2> = geometry.boundary.iter()
            .map(|&world_pos| map.world_to_screen(world_pos, screen_rect))
            .collect();

        if screen_points.len() < 3 { continue; }

        // Fill district
        let fill_color = if map_state.selected_district.as_ref() == Some(district_id) {
            brighten_color(geometry.control_color, 1.3)
        } else if map_state.hovered_district.as_ref() == Some(district_id) {
            brighten_color(geometry.control_color, 1.1)
        } else {
            geometry.control_color
        };

        painter.add(egui::Shape::convex_polygon(
            screen_points.clone(),
            fill_color,
            egui::Stroke::NONE,
        ));

        // District border
        let border_stroke = match geometry.border_style {
            BorderStyle::Controlled => egui::Stroke::new(2.0, egui::Color32::WHITE),
            BorderStyle::Contested => egui::Stroke::new(2.0, egui::Color32::YELLOW),
            BorderStyle::Corporate => egui::Stroke::new(1.5, egui::Color32::GRAY),
            BorderStyle::Neutral => egui::Stroke::new(1.0, egui::Color32::DARK_GRAY),
        };

        painter.add(egui::Shape::closed_line(screen_points, border_stroke));

        // District label
        let center_screen = map.world_to_screen(geometry.center, screen_rect);
        let label_text = if let Some(district_data) = territory_manager.get_district(district_id) {
            format!("{}\n{:.0}%", district_id.replace("_", " "), district_data.population_support * 100.0)
        } else {
            district_id.replace("_", " ")
        };

        painter.text(
            center_screen,
            egui::Align2::CENTER_CENTER,
            label_text,
            egui::FontId::proportional(9.0),
            egui::Color32::WHITE,
        );
    }
}

fn draw_mrt_system(painter: &egui::Painter, map: &SingaporeVectorMap, screen_rect: egui::Rect) {
    // Draw MRT lines
    for line in map.mrt_system.lines.values() {
        let screen_points: Vec<egui::Pos2> = line.path.iter()
            .map(|&world_pos| map.world_to_screen(world_pos, screen_rect))
            .collect();

        if screen_points.len() >= 2 {
            painter.add(egui::Shape::line(
                screen_points,
                egui::Stroke::new(3.0, line.color),
            ));
        }
    }

    // Draw MRT stations
    for station in map.mrt_system.stations.values() {
        let screen_pos = map.world_to_screen(station.position, screen_rect);
        let radius = if station.interchange { 4.0 } else { 3.0 };

        painter.circle_filled(screen_pos, radius, egui::Color32::WHITE);
        painter.circle_stroke(screen_pos, radius, egui::Stroke::new(1.0, egui::Color32::BLACK));
    }
}

fn draw_landmarks(painter: &egui::Painter, map: &SingaporeVectorMap, screen_rect: egui::Rect) {
    for landmark in &map.landmarks {
        let screen_pos = map.world_to_screen(landmark.position, screen_rect);
        
        let (icon, color) = match landmark.landmark_type {
            LandmarkType::Airport => ("✈", egui::Color32::LIGHT_BLUE),
            LandmarkType::Port => ("⚓", egui::Color32::LIGHT_GRAY),
            LandmarkType::Government => ("🏛", egui::Color32::YELLOW),
            LandmarkType::Corporate => ("🏢", egui::Color32::RED),
            LandmarkType::University => ("🎓", egui::Color32::GREEN),
            LandmarkType::Hospital => ("🏥", egui::Color32::from_rgb(255, 100, 100)),
            LandmarkType::Shopping => ("🛍", egui::Color32::LIGHT_GREEN),
            LandmarkType::Tourist => ("📸", egui::Color32::LIGHT_YELLOW),
        };

        painter.text(
            screen_pos,
            egui::Align2::CENTER_CENTER,
            icon,
            egui::FontId::proportional(12.0),
            color,
        );
    }
}

fn handle_map_interactions(
    response: &egui::Response,
    map_state: &mut SingaporeMapState,
    singapore_map: &SingaporeVectorMap,
    screen_rect: egui::Rect,
    global_data: &mut GlobalData,
) {
    // Handle hover and click detection
    if let Some(hover_pos) = response.hover_pos() {
        let world_pos = singapore_map.screen_to_world(hover_pos, screen_rect);
        
        // Find hovered district
        map_state.hovered_district = None;
        for (district_id, geometry) in &singapore_map.districts {
            if point_in_polygon(world_pos, &geometry.boundary) {
                map_state.hovered_district = Some(district_id.clone());
                break;
            }
        }
    }

    // Handle clicks
    if response.clicked() {
        if let Some(district_id) = &map_state.hovered_district {
            map_state.selected_district = Some(district_id.clone());
            // Update global data to reflect selection
            // global_data.selected_district = district_id.clone();
        }
    }
}

fn draw_map_info_panel(
    ui: &mut egui::Ui,
    map_state: &SingaporeMapState,
    territory_manager: &TerritoryManager,
    campaign_db: &NeoSingaporeCampaignDatabase,
) {
    ui.separator();
    
    ui.horizontal(|ui| {
        ui.label("Selected:");
        if let Some(district_id) = &map_state.selected_district {
            if let Some(district_data) = campaign_db.get_district(district_id) {
                ui.strong(&district_data.name);
                ui.separator();
                
                if let Some(control) = territory_manager.get_district(district_id) {
                    ui.colored_label(
                        egui::Color32::GREEN,
                        format!("Liberation: {:.0}%", control.population_support * 100.0)
                    );
                    ui.separator();
                    ui.label(format!("Status: {:?}", control.liberation_status));
                } else {
                    ui.colored_label(
                        egui::Color32::RED,
                        format!("Corporate: {:?}", district_data.controlling_corp)
                    );
                }
            }
        } else {
            ui.colored_label(egui::Color32::GRAY, "None");
        }
    });
}

// === UTILITY FUNCTIONS ===
fn brighten_color(color: egui::Color32, factor: f32) -> egui::Color32 {
    let [r, g, b, a] = color.to_array();
    egui::Color32::from_rgba_unmultiplied(
        (r as f32 * factor).min(255.0) as u8,
        (g as f32 * factor).min(255.0) as u8,
        (b as f32 * factor).min(255.0) as u8,
        a,
    )
}

fn point_in_polygon(point: Vec2, polygon: &[Vec2]) -> bool {
    let mut inside = false;
    let mut j = polygon.len() - 1;

    for i in 0..polygon.len() {
        let pi = polygon[i];
        let pj = polygon[j];

        if ((pi.y > point.y) != (pj.y > point.y)) &&
           (point.x < (pj.x - pi.x) * (point.y - pi.y) / (pj.y - pi.y) + pi.x) {
            inside = !inside;
        }
        j = i;
    }

    inside
}

// === DATA CREATION FUNCTIONS ===
fn create_singapore_districts() -> HashMap<String, DistrictGeometry> {
    // This would be populated with actual Singapore district boundaries
    // For now, create simplified rectangular districts
    let mut districts = HashMap::new();
    
    // Marina Bay (central, small)
    districts.insert("marina_bay".to_string(), DistrictGeometry {
        id: "marina_bay".to_string(),
        boundary: vec![
            Vec2::new(103.85, 1.28),
            Vec2::new(103.86, 1.28),
            Vec2::new(103.86, 1.29),
            Vec2::new(103.85, 1.29),
        ],
        center: Vec2::new(103.855, 1.285),
        area: 1.0,
        control_color: egui::Color32::GRAY,
        border_style: BorderStyle::Corporate,
    });

    // Add more districts...
    // TODO: Replace with actual GIS data conversion
    
    districts
}

fn create_mrt_network() -> MRTNetwork {
    let mut lines = HashMap::new();
    let mut stations = HashMap::new();

    // Example: North-South Line (Red)
    lines.insert("NSL".to_string(), MRTLine {
        name: "North-South Line".to_string(),
        color: egui::Color32::RED,
        path: vec![
            Vec2::new(103.83, 1.45), // Jurong East
            Vec2::new(103.85, 1.40), // City Hall
            Vec2::new(103.85, 1.35), // Raffles Place
            Vec2::new(103.86, 1.30), // Marina Bay
        ],
        stations: vec!["JE".to_string(), "CH".to_string(), "RP".to_string(), "MB".to_string()],
    });

    // Add stations
    stations.insert("MB".to_string(), MRTStation {
        id: "MB".to_string(),
        name: "Marina Bay".to_string(),
        position: Vec2::new(103.86, 1.30),
        lines: vec!["NSL".to_string()],
        interchange: false,
    });

    MRTNetwork { lines, stations }
}

fn create_landmarks() -> Vec<MapLandmark> {
    vec![
        MapLandmark {
            name: "Changi Airport".to_string(),
            position: Vec2::new(103.99, 1.36),
            landmark_type: LandmarkType::Airport,
            importance: 5,
        },
        MapLandmark {
            name: "Marina Bay Sands".to_string(),
            position: Vec2::new(103.86, 1.28),
            landmark_type: LandmarkType::Corporate,
            importance: 4,
        },
        // Add more landmarks...
    ]
}

fn create_coastline() -> Vec<Vec2> {
    // Simplified Singapore coastline
    vec![
        Vec2::new(103.6, 1.2),
        Vec2::new(103.7, 1.15),
        Vec2::new(103.85, 1.15),
        Vec2::new(104.0, 1.2),
        Vec2::new(104.0, 1.45),
        Vec2::new(103.9, 1.5),
        Vec2::new(103.7, 1.5),
        Vec2::new(103.6, 1.4),
        Vec2::new(103.6, 1.2), // Close the polygon
    ]
}