// src/systems/ui/hub/global_map.rs - Simplified using UIBuilder
use bevy::prelude::*;
use crate::core::*;
use crate::systems::ui::builder::*;

#[derive(Component)]
pub struct InteractiveCity {
    pub city_id: String,
    pub accessible: bool,
}

#[derive(Component)]
pub struct MapContainer;

#[derive(Component)]
pub struct CityTooltip {
    pub city_id: String,
    pub hover_timer: f32,
}

#[derive(Clone, Resource, Default)]
pub struct GlobalMapState {
    pub selected_city: Option<String>,
    pub hovered_city: Option<String>,
    pub map_projection: Option<MapProjection>,
    pub city_positions: std::collections::HashMap<String, Vec2>,
    pub zoom: f32,
    pub pan_offset: Vec2,
    pub is_dragging: bool,
    pub last_mouse_pos: Option<Vec2>,
    pub hover_timer: f32,    
}

impl GlobalMapState {
    pub fn new() -> Self {
        Self {
            zoom: 1.0,
            pan_offset: Vec2::ZERO,
            hover_timer: 0.0,
            ..Default::default()
        }
    }
    // future implementation
    fn clamp_pan(&mut self, container_size: Vec2) {
        let max_offset = container_size * (self.zoom - 1.0) * 0.5;
        self.pan_offset.x = self.pan_offset.x.clamp(-max_offset.x, max_offset.x);
        self.pan_offset.y = self.pan_offset.y.clamp(-max_offset.y, max_offset.y);
    }
}

pub fn handle_input(
    input: &ButtonInput<KeyCode>,
    global_data: &mut GlobalData,
    hub_state: &mut super::HubState,
    cities_db: &CitiesDatabase,
    map_state: &mut GlobalMapState,
    windows: &Query<&Window>,
    cameras: &Query<(&Camera, &GlobalTransform)>,
    mouse: &ButtonInput<MouseButton>,
    city_query: &Query<(Entity, &Transform, &InteractiveCity)>,
) -> bool {
    let mut needs_rebuild = false;

    if mouse.just_pressed(MouseButton::Left) {
        if let Some(world_pos) = get_global_map_mouse_position(windows, cameras) {
            for (_, transform, interactive_city) in city_query.iter() {
                let city_pos = transform.translation.truncate();
                let distance = world_pos.distance(city_pos);
                
                if distance <= 12.0 && interactive_city.accessible {
                    map_state.selected_city = Some(interactive_city.city_id.clone());
                    global_data.cities_progress.current_city = interactive_city.city_id.clone();
                    needs_rebuild = true;
                    break;
                }
            }
        }
    }

    if let Some(world_pos) = get_global_map_mouse_position(windows, cameras) {
        let mut new_hovered = None;
        
        for (_, transform, interactive_city) in city_query.iter() {
            let city_pos = transform.translation.truncate();
            let distance = world_pos.distance(city_pos);
            
            if distance <= 15.0 {
                new_hovered = Some(interactive_city.city_id.clone());
                break;
            }
        }
        
        if map_state.hovered_city != new_hovered {
            map_state.hovered_city = new_hovered;
            needs_rebuild = true;
        }
    }

    // Wait day
    if input.just_pressed(KeyCode::KeyW) {
        global_data.current_day += 1;
        let current_day = global_data.current_day;
        
        for region in &mut global_data.regions {
            region.update_alert(current_day);
        }
        
        for (_, city_state) in global_data.cities_progress.city_states.iter_mut() {
            update_city_alert(city_state, current_day);
        }
        
        needs_rebuild = true;
    }

    // Launch mission
    if input.just_pressed(KeyCode::Enter) {
        if let Some(selected_city_id) = &map_state.selected_city {
            if cities_db.get_city(selected_city_id).is_some() {
                hub_state.active_tab = super::HubTab::Missions;
                needs_rebuild = true;
            }
        } else {
            hub_state.active_tab = super::HubTab::Missions;
            needs_rebuild = true;
        }
    }

    needs_rebuild
}

pub fn create_content(
    parent: &mut ChildSpawnerCommands, 
    global_data: &GlobalData, 
    hub_state: &super::HubState,
    cities_db: &CitiesDatabase,
    cities_progress: &CitiesProgress,
    map_state: &mut GlobalMapState,
) {
    parent.spawn(UIBuilder::content_area()).with_children(|content| {
        create_world_map_section(content, cities_db, &global_data, map_state);
        
        // Show tooltip if hovering and timer exceeded
        if let Some(hovered_city_id) = &map_state.hovered_city {
            if let Some(city) = cities_db.get_city(hovered_city_id) {
                create_city_tooltip(content, city, global_data);
            }
        }
        
        content.spawn(UIBuilder::nav_controls("Click Cities | W: Wait Day | ENTER: Launch Mission"));
    });
}

fn create_world_map_section(
    parent: &mut ChildSpawnerCommands,
    cities_db: &CitiesDatabase,
    global_data: &GlobalData,
    map_state: &mut GlobalMapState,
) {
    parent.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Px(550.0),
            border: UiRect::all(Val::Px(2.0)),
            padding: UiRect::all(Val::Px(0.0)),
            position_type: PositionType::Relative,
            ..default()
        },
        BorderColor(Color::srgb(0.3, 0.3, 0.3)),
        BackgroundColor(Color::srgb(0.05, 0.05, 0.1)),
    )).with_children(|map_container| {
        let map_width = 1200.0;
        let map_height = 550.0;
        map_state.map_projection = Some(MapProjection::new(map_width, map_height));
        
        let projection = map_state.map_projection.as_ref().unwrap();
        let accessible_cities = cities_db.get_accessible_cities(&global_data);
        let all_cities = cities_db.get_all_cities();
        
        for city in &all_cities {
            let pixel_pos = projection.lat_lon_to_pixel(&city.coordinates);
            map_state.city_positions.insert(city.id.clone(), pixel_pos);

            let is_accessible = accessible_cities.iter().any(|acc_city| acc_city.id == city.id);
            let is_selected = map_state.selected_city.as_ref() == Some(&city.id);
            let is_hovered = map_state.hovered_city.as_ref() == Some(&city.id);
            let city_state = global_data.cities_progress.get_city_state(&city.id);
            
            let city_color = if !is_accessible {
                Color::srgba(0.3, 0.3, 0.3, 0.4)
            } else if city_state.completed {
                Color::srgb(0.2, 0.8, 0.2)
            } else {
                match city_state.alert_level {
                    AlertLevel::Green => Color::srgb(0.2, 0.8, 0.2),
                    AlertLevel::Yellow => Color::srgb(0.8, 0.8, 0.2),
                    AlertLevel::Orange => Color::srgb(0.8, 0.5, 0.2),
                    AlertLevel::Red => Color::srgb(0.8, 0.2, 0.2),
                }
            };
            
            let circle_size = if is_selected { 20.0 } else if is_hovered { 16.0 } else { 16.0 };
            
            // City circle
            map_container.spawn((
                Node {
                    width: Val::Px(circle_size),
                    height: Val::Px(circle_size),
                    position_type: PositionType::Absolute,
                    left: Val::Px(pixel_pos.x - circle_size / 2.0),
                    top: Val::Px(pixel_pos.y - circle_size / 2.0),
                    border: UiRect::all(Val::Px(if is_selected { 1.0 } else { 1.0 })),
                    ..default()
                },
                BackgroundColor(city_color),
                BorderColor(if is_selected { 
                    Color::WHITE 
                } else if is_accessible {
                    Color::srgb(0.4, 0.4, 0.4)
                } else {
                    Color::srgba(0.2, 0.2, 0.2, 0.4)
                }),
                InteractiveCity {
                    city_id: city.id.clone(),
                    accessible: is_accessible,
                },
            ));
            
            // City label
            let text_color = if is_accessible { Color::WHITE } else { Color::srgba(0.5, 0.5, 0.5, 0.6) };
            
            map_container.spawn((
                UIBuilder::text(&city.name, 10.0, text_color),
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(pixel_pos.x - 40.0),
                    top: Val::Px(pixel_pos.y - 25.0),
                    width: Val::Px(80.0),
                    justify_content: JustifyContent::Center,
                    ..default()
                },
            ));
            
            // Corporation indicator
            if is_accessible {
                let corp_color = city.controlling_corp.color();
                map_container.spawn((
                    Node {
                        width: Val::Px(5.0),
                        height: Val::Px(5.0),
                        position_type: PositionType::Absolute,
                        left: Val::Px(pixel_pos.x + 5.0),
                        top: Val::Px(pixel_pos.y - 5.0),
                        ..default()
                    },
                    BackgroundColor(corp_color),
                ));
            }
        }
        
        // Draw connections between accessible cities

        // Map info overlay
        map_container.spawn((
            UIBuilder::text(&format!("Zoom: {:.1}x | Cities: {} total, {} accessible", 
                map_state.zoom, 
                cities_db.get_all_cities().len(), 
                cities_db.get_accessible_cities(&global_data).len()), 10.0, Color::srgb(0.8, 0.8, 0.2)),
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(5.0),
                left: Val::Px(5.0),
                ..default()
            },
        ));
    });
}

fn create_city_tooltip(
    parent: &mut ChildSpawnerCommands,
    city: &City,
    global_data: &GlobalData,
) {
    let city_state = global_data.cities_progress.get_city_state(&city.id);
    
    parent.spawn((
        Node {
            position_type: PositionType::Absolute,
            width: Val::Px(300.0),
            padding: UiRect::all(Val::Px(10.0)),
            left: Val::Px(50.0), // Fixed position for now - could be made dynamic
            top: Val::Px(50.0),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(5.0),
            border: UiRect::all(Val::Px(1.0)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.1, 0.1, 0.2, 0.9)),
        BorderColor(Color::srgb(0.3, 0.3, 0.3)),
        ZIndex(100),
        CityTooltip {
            city_id: city.id.clone(),
            hover_timer: 0.0,
        },
    )).with_children(|tooltip| {
        tooltip.spawn(UIBuilder::subtitle(&format!("{}, {}", city.name, city.country)));
        
        tooltip.spawn(UIBuilder::row(15.0)).with_children(|stats| {
            stats.spawn(UIBuilder::text(&format!("Pop: {}M", city.population), 12.0, Color::WHITE));
            stats.spawn(UIBuilder::text(&format!("Corruption: {}/10", city.corruption_level), 12.0, Color::srgb(0.8, 0.6, 0.2)));
            
            let alert_color = match city_state.alert_level {
                AlertLevel::Green => Color::srgb(0.2, 0.8, 0.2),
                AlertLevel::Yellow => Color::srgb(0.8, 0.8, 0.2),
                AlertLevel::Orange => Color::srgb(0.8, 0.5, 0.2),
                AlertLevel::Red => Color::srgb(0.8, 0.2, 0.2),
            };
            stats.spawn(UIBuilder::text(&format!("{:?}", city_state.alert_level), 12.0, alert_color));
        });
        
        tooltip.spawn(UIBuilder::text(&format!("{:?}", city.controlling_corp), 12.0, city.controlling_corp.color()));
        
        if city_state.completed {
            tooltip.spawn(UIBuilder::success_text(&format!("✓ COMPLETED ({} visits)", city_state.times_visited)));
        }
    });
}

fn get_global_map_mouse_position(
    windows: &Query<&Window>,
    cameras: &Query<(&Camera, &GlobalTransform)>,
) -> Option<Vec2> {
    let window = windows.single().ok()?;
    let (camera, camera_transform) = cameras.single().ok()?;
    
    if let Some(cursor_pos) = window.cursor_position() {
        // Convert to world coordinates and flip Y to match our coordinate system
        if let Ok(world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_pos) {
            Some(Vec2::new(world_pos.x, -(world_pos.y)))
        } else {
            None
        }
    } else {
        None
    }
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