// src/systems/ui/hub/global_map.rs - Simplified using UIBuilder
use bevy::prelude::*;
use crate::core::*;
use crate::systems::ui::builder::*;

#[derive(Component)]
pub struct InteractiveCity {
    pub city_id: String,
    pub accessible: bool,
}

#[derive(Clone, Resource, Default)]
pub struct GlobalMapState {
    pub selected_city: Option<String>,
    pub hovered_city: Option<String>,
    pub map_projection: Option<MapProjection>,
    pub city_positions: std::collections::HashMap<String, Vec2>,
}

pub fn handle_input(
    input: &ButtonInput<KeyCode>,
    global_data: &mut GlobalData,
    hub_state: &mut super::HubState,
    cities_db: &CitiesDatabase,
    cities_progress: &mut CitiesProgress,
    map_state: &mut GlobalMapState,
    windows: &Query<&Window>,
    cameras: &Query<(&Camera, &GlobalTransform)>,
    mouse: &ButtonInput<MouseButton>,
    city_query: &Query<(Entity, &Transform, &InteractiveCity)>,
) -> bool {
    let mut needs_rebuild = false;

    if input.just_pressed(KeyCode::ArrowUp) && hub_state.selected_region > 0 {
        hub_state.selected_region -= 1;
        global_data.selected_region = hub_state.selected_region;
        needs_rebuild = true;
    }
    
    if input.just_pressed(KeyCode::ArrowDown) && hub_state.selected_region < global_data.regions.len() - 1 {
        hub_state.selected_region += 1;
        global_data.selected_region = hub_state.selected_region;
        needs_rebuild = true;
    }

    if mouse.just_pressed(MouseButton::Left) {
        if let Some(world_pos) = get_global_map_mouse_position(windows, cameras) {
            for (_, transform, interactive_city) in city_query.iter() {
                let city_pos = transform.translation.truncate();
                let distance = world_pos.distance(city_pos);
                
                if distance <= 15.0 && interactive_city.accessible {
                    map_state.selected_city = Some(interactive_city.city_id.clone());
                    cities_progress.current_city = interactive_city.city_id.clone();
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

    if input.just_pressed(KeyCode::KeyW) {
        global_data.current_day += 1;
        let current_day = global_data.current_day;
        
        for region in &mut global_data.regions {
            region.update_alert(current_day);
        }
        
        for (_, city_state) in cities_progress.city_states.iter_mut() {
            update_city_alert(city_state, current_day);
        }
        
        needs_rebuild = true;
    }

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
        create_world_map_section(content, cities_db, cities_progress, map_state);
        create_selected_city_info(content, cities_db, cities_progress, map_state);
        content.spawn(UIBuilder::nav_controls("Click Cities | W: Wait Day | ENTER: Launch Mission"));
    });
}

fn create_world_map_section(
    parent: &mut ChildSpawnerCommands,
    cities_db: &CitiesDatabase,
    cities_progress: &CitiesProgress,
    map_state: &mut GlobalMapState,
) {
    parent.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Px(400.0),
            border: UiRect::all(Val::Px(2.0)),
            padding: UiRect::all(Val::Px(10.0)),
            position_type: PositionType::Relative,
            ..default()
        },
        BorderColor(Color::srgb(0.3, 0.3, 0.3)),
        BackgroundColor(Color::srgb(0.05, 0.05, 0.1)),
    )).with_children(|map_container| {
        let map_width = 1200.0;
        let map_height = 500.0;
        map_state.map_projection = Some(MapProjection::new(map_width, map_height));
        
        let projection = map_state.map_projection.as_ref().unwrap();
        let accessible_cities = cities_db.get_accessible_cities(cities_progress);
        let all_cities = cities_db.get_all_cities();
        
        for city in &all_cities {
            let pixel_pos = projection.lat_lon_to_pixel(&city.coordinates);
            map_state.city_positions.insert(city.id.clone(), pixel_pos);

            let is_accessible = accessible_cities.iter().any(|acc_city| acc_city.id == city.id);
            let is_selected = map_state.selected_city.as_ref() == Some(&city.id);
            let is_hovered = map_state.hovered_city.as_ref() == Some(&city.id);
            let city_state = cities_progress.get_city_state(&city.id);
            
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
            
            let circle_size = if is_selected { 20.0 } else if is_hovered { 16.0 } else { 12.0 };
            
            // City circle
            map_container.spawn((
                Node {
                    width: Val::Px(circle_size),
                    height: Val::Px(circle_size),
                    position_type: PositionType::Absolute,
                    left: Val::Px(pixel_pos.x - circle_size / 2.0),
                    top: Val::Px(pixel_pos.y - circle_size / 2.0),
                    border: UiRect::all(Val::Px(if is_selected { 3.0 } else { 1.0 })),
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
                        width: Val::Px(6.0),
                        height: Val::Px(6.0),
                        position_type: PositionType::Absolute,
                        left: Val::Px(pixel_pos.x + 8.0),
                        top: Val::Px(pixel_pos.y - 8.0),
                        ..default()
                    },
                    BackgroundColor(corp_color),
                ));
            }
        }
        
        // Draw connections between accessible cities
        for city in &accessible_cities {
            let city_pos = projection.lat_lon_to_pixel(&city.coordinates);
            
            for connection_id in &city.connections {
                if let Some(connected_city) = cities_db.get_city(connection_id) {
                    if accessible_cities.iter().any(|c| c.id == *connection_id) {
                        let connected_pos = projection.lat_lon_to_pixel(&connected_city.coordinates);
                        let line_length = city_pos.distance(connected_pos);
                        
                        map_container.spawn((
                            Node {
                                width: Val::Px(line_length),
                                height: Val::Px(1.0),
                                position_type: PositionType::Absolute,
                                left: Val::Px(city_pos.x),
                                top: Val::Px(city_pos.y),
                                ..default()
                            },
                            BackgroundColor(Color::srgba(0.4, 0.4, 0.4, 0.3)),
                        ));
                    }
                }
            }
        }
        
        map_container.spawn((
            UIBuilder::text(&format!("Cities: {} total, {} accessible", all_cities.len(), accessible_cities.len()), 10.0, Color::srgb(0.8, 0.8, 0.2)),
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(5.0),
                left: Val::Px(5.0),
                ..default()
            },
        ));
    });
}

fn create_selected_city_info(
    parent: &mut ChildSpawnerCommands,
    cities_db: &CitiesDatabase,
    cities_progress: &CitiesProgress,
    map_state: &GlobalMapState,
) {
    if let Some(selected_city_id) = &map_state.selected_city {
        if let Some(city) = cities_db.get_city(selected_city_id) {
            let city_state = cities_progress.get_city_state(selected_city_id);
            
            let (panel_node, _) = UIBuilder::section_panel();
            parent.spawn((panel_node, BackgroundColor(Color::srgba(0.1, 0.1, 0.2, 0.5)))).with_children(|city_info| {
                city_info.spawn(UIBuilder::subtitle(&format!("{}, {}", city.name, city.country)));
                
                city_info.spawn(UIBuilder::row(30.0)).with_children(|stats| {
                    let mut stats_builder = StatsBuilder::new(stats);
                    stats_builder.stat("Population", &format!("{}M", city.population), None);
                    stats_builder.stat("Corruption", &format!("{}/10", city.corruption_level), Some(Color::srgb(0.8, 0.6, 0.2)));
                    
                    let alert_color = match city_state.alert_level {
                        AlertLevel::Green => Color::srgb(0.2, 0.8, 0.2),
                        AlertLevel::Yellow => Color::srgb(0.8, 0.8, 0.2),
                        AlertLevel::Orange => Color::srgb(0.8, 0.5, 0.2),
                        AlertLevel::Red => Color::srgb(0.8, 0.2, 0.2),
                    };
                    stats_builder.stat("Alert", &format!("{:?}", city_state.alert_level), Some(alert_color));
                });
                
                city_info.spawn(UIBuilder::text(&format!("Controlled by: {:?}", city.controlling_corp), 14.0, city.controlling_corp.color()));
                
                if !city.traits.is_empty() {
                    let traits_text = city.traits.iter()
                        .map(|trait_item| format!("{:?}", trait_item))
                        .collect::<Vec<_>>()
                        .join(", ");
                    
                    city_info.spawn(UIBuilder::text(&format!("Traits: {}", traits_text), 12.0, Color::srgb(0.8, 0.8, 0.8)));
                }
                
                if city_state.completed {
                    city_info.spawn(UIBuilder::success_text(&format!("✓ COMPLETED (Visited {} times)", city_state.times_visited)));
                } else {
                    city_info.spawn(UIBuilder::text("Available for missions", 14.0, Color::srgb(0.8, 0.8, 0.2)));
                }
            });
        }
    } else {
        parent.spawn(UIBuilder::text("Click on a city to view details and select for missions", 16.0, Color::srgb(0.6, 0.6, 0.6)));
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

