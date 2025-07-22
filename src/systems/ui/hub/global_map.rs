// src/systems/ui/hub/global_map.rs - Updated with interactive city map
use bevy::prelude::*;
use crate::core::*;
use super::HubTab;

// Components for interactive cities
#[derive(Component)]
pub struct InteractiveCity {
    pub city_id: String,
    pub accessible: bool,
}

#[derive(Component)]
pub struct CityTooltip;

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

    // Handle city navigation with arrow keys (fallback)
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

    // Handle mouse interaction with cities
    if mouse.just_pressed(MouseButton::Left) {
        if let Some(world_pos) = get_global_map_mouse_position(windows, cameras) {
            // Find clicked city
            for (entity, transform, interactive_city) in city_query.iter() {
                let city_pos = transform.translation.truncate();
                let distance = world_pos.distance(city_pos);
                
                if distance <= 15.0 && interactive_city.accessible {
                    // Select this city for mission
                    map_state.selected_city = Some(interactive_city.city_id.clone());
                    cities_progress.current_city = interactive_city.city_id.clone();
                    
                    // Update global data to match
                    if let Some(city) = cities_db.get_city(&interactive_city.city_id) {
                        info!("Selected city: {} for mission", city.name);
                    }
                    needs_rebuild = true;
                    break;
                }
            }
        }
    }

    // Handle mouse hover for tooltips
    if let Some(world_pos) = get_global_map_mouse_position(windows, cameras) {
        let mut new_hovered = None;
        
        for (entity, transform, interactive_city) in city_query.iter() {
            
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

    // Time advancement
    if input.just_pressed(KeyCode::KeyW) {
        global_data.current_day += 1;
        let current_day = global_data.current_day;
        
        // Update legacy regions for compatibility
        for region in &mut global_data.regions {
            region.update_alert(current_day);
        }
        
        // Update city alert levels
        for (city_id, city_state) in cities_progress.city_states.iter_mut() {
            update_city_alert(city_state, current_day);
        }
        
        needs_rebuild = true;
        info!("Waited 1 day. Current day: {}", current_day);
    }

    // Mission launch
    if input.just_pressed(KeyCode::Enter) {
        if let Some(selected_city_id) = &map_state.selected_city {
            if let Some(city) = cities_db.get_city(selected_city_id) {
                // Launch mission to selected city
                info!("Launching mission to {}", city.name);
                hub_state.active_tab = HubTab::Missions;
                needs_rebuild = true;
            }
        } else {
            // Fallback to legacy system
            hub_state.active_tab = HubTab::Missions;
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
    parent.spawn(Node {
        width: Val::Percent(100.0),
        flex_grow: 1.0,
        padding: UiRect::all(Val::Px(20.0)),
        flex_direction: FlexDirection::Column,
        row_gap: Val::Px(15.0),
        ..default()
    }).with_children(|content| {
        
        // World map section
        create_world_map_section(content, cities_db, cities_progress, map_state);

        // Agent status section
        // create_agent_status_section(content, global_data);
        
        // Selected city info
        create_selected_city_info(content, cities_db, cities_progress, map_state);
        
        // Controls help
        content.spawn((
            Text::new("Click on accessible cities to select | W: Wait Day | ENTER: Launch Mission"),
            TextFont { font_size: 14.0, ..default() },
            TextColor(Color::srgb(0.7, 0.7, 0.7)),
        ));
    });
}

fn create_agent_status_section(parent: &mut ChildSpawnerCommands, global_data: &GlobalData) {
    // parent.spawn((Text::new("AGENT STATUS:"),TextFont { font_size: 20.0, ..default() },TextColor(Color::WHITE),));
    
    for i in 0..3 {
        let level = global_data.agent_levels[i];
        let is_recovering = global_data.agent_recovery[i] > global_data.current_day;
        let recovery_days = if is_recovering { 
            global_data.agent_recovery[i] - global_data.current_day 
        } else { 0 };
        
        let color = if is_recovering { Color::srgb(0.5, 0.5, 0.5) } else { Color::srgb(0.2, 0.8, 0.2) };
        let status = if is_recovering {
            format!("Agent {}: Level {} - RECOVERING ({} days)", i + 1, level, recovery_days)
        } else {
            format!("Agent {}: Level {} - READY", i + 1, level)
        };
        
        parent.spawn((
            Text::new(status),
            TextFont { font_size: 8.0, ..default() }, // WAS 16.0
            TextColor(color),
        ));
    }
}

fn create_world_map_section(
    parent: &mut ChildSpawnerCommands,
    cities_db: &CitiesDatabase,
    cities_progress: &CitiesProgress,
    map_state: &mut GlobalMapState,
) {
    // Create map container
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
        // Initialize map projection
        let map_width = 1200.0; // get screen width = padding
        let map_height = 500.0;
        map_state.map_projection = Some(MapProjection::new(map_width, map_height));
        
        let projection = map_state.map_projection.as_ref().unwrap();
        let accessible_cities = cities_db.get_accessible_cities(cities_progress);
        let all_cities = cities_db.get_all_cities(); // Get all cities for rendering
        
        // Draw ALL cities (both accessible and inaccessible)
        for city in &all_cities {
            
            let pixel_pos = projection.lat_lon_to_pixel(&city.coordinates);
            let city_state = cities_progress.get_city_state(&city.id);
            
            // Store the actual rendered position
            map_state.city_positions.insert(city.id.clone(), pixel_pos);

            let is_accessible = accessible_cities.iter().any(|acc_city| acc_city.id == city.id);
            let is_selected = map_state.selected_city.as_ref() == Some(&city.id);
            let is_hovered = map_state.hovered_city.as_ref() == Some(&city.id);
            let is_completed = city_state.completed;
            
            // City circle color based on status
            let city_color = if !is_accessible {
                Color::srgba(0.3, 0.3, 0.3, 0.4) // Dark grey for inaccessible
            } else if is_completed {
                Color::srgb(0.2, 0.8, 0.2) // Green for completed
            } else {
                match city_state.alert_level {
                    AlertLevel::Green => Color::srgb(0.2, 0.8, 0.2),
                    AlertLevel::Yellow => Color::srgb(0.8, 0.8, 0.2),
                    AlertLevel::Orange => Color::srgb(0.8, 0.5, 0.2),
                    AlertLevel::Red => Color::srgb(0.8, 0.2, 0.2),
                }
            };
            
            let circle_size = if is_selected {
                20.0
            } else if is_hovered {
                16.0
            } else {
                12.0
            };
            
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
                    Color::srgba(0.2, 0.2, 0.2, 0.4) // Darker border for inaccessible
                }),
                InteractiveCity {
                    city_id: city.id.clone(),
                    accessible: is_accessible,
                },
            ));
            
            // City name label (dimmer for inaccessible cities)
            let text_color = if is_accessible {
                Color::WHITE
            } else {
                Color::srgba(0.5, 0.5, 0.5, 0.6) // Semi-transparent for inaccessible
            };
            
            map_container.spawn((
                Text::new(&city.name),
                TextFont { font_size: 10.0, ..default() },
                TextColor(text_color),
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(pixel_pos.x - 40.0),
                    top: Val::Px(pixel_pos.y - 25.0),
                    width: Val::Px(80.0),
                    justify_content: JustifyContent::Center,
                    ..default()
                },
            ));
            
            // Corporation indicator (only for accessible cities)
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
        
        // Draw connections between accessible cities only
        for city in &accessible_cities {
            let city_pos = projection.lat_lon_to_pixel(&city.coordinates);
            
            for connection_id in &city.connections {
                if let Some(connected_city) = cities_db.get_city(connection_id) {
                    if accessible_cities.iter().any(|c| c.id == *connection_id) {
                        let connected_pos = projection.lat_lon_to_pixel(&connected_city.coordinates);
                        
                        // Draw connection line (simplified as a colored rectangle)
                        let line_length = city_pos.distance(connected_pos);
                        let angle = (connected_pos.y - city_pos.y).atan2(connected_pos.x - city_pos.x);
                        
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
        
        // Debug info overlay
        map_container.spawn((
            Text::new(format!("Cities: {} total, {} accessible", all_cities.len(), accessible_cities.len())),
            TextFont { font_size: 10.0, ..default() },
            TextColor(Color::srgb(0.8, 0.8, 0.2)),
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
            
            parent.spawn((
                Node {
                    flex_direction: FlexDirection::Column,
                    padding: UiRect::all(Val::Px(15.0)),
                    row_gap: Val::Px(8.0),
                    margin: UiRect::top(Val::Px(20.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.1, 0.1, 0.2, 0.5)),
            )).with_children(|city_info| {
                // City header
                city_info.spawn((
                    Text::new(format!("{}, {}", city.name, city.country)),
                    TextFont { font_size: 20.0, ..default() },
                    TextColor(Color::WHITE),
                ));
                
                // City stats
                city_info.spawn(Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(30.0),
                    ..default()
                }).with_children(|stats| {
                    stats.spawn((
                        Text::new(format!("Population: {}M", city.population)),
                        TextFont { font_size: 14.0, ..default() },
                        TextColor(Color::WHITE),
                    ));
                    
                    stats.spawn((
                        Text::new(format!("Corruption: {}/10", city.corruption_level)),
                        TextFont { font_size: 14.0, ..default() },
                        TextColor(Color::srgb(0.8, 0.6, 0.2)),
                    ));
                    
                    stats.spawn((
                        Text::new(format!("Alert: {:?}", city_state.alert_level)),
                        TextFont { font_size: 14.0, ..default() },
                        TextColor(match city_state.alert_level {
                            AlertLevel::Green => Color::srgb(0.2, 0.8, 0.2),
                            AlertLevel::Yellow => Color::srgb(0.8, 0.8, 0.2),
                            AlertLevel::Orange => Color::srgb(0.8, 0.5, 0.2),
                            AlertLevel::Red => Color::srgb(0.8, 0.2, 0.2),
                        }),
                    ));
                });
                
                // Corporation control
                city_info.spawn((
                    Text::new(format!("Controlled by: {:?}", city.controlling_corp)),
                    TextFont { font_size: 14.0, ..default() },
                    TextColor(city.controlling_corp.color()),
                ));
                
                // City traits
                if !city.traits.is_empty() {
                    let traits_text = city.traits.iter()
                        .map(|trait_item| format!("{:?}", trait_item))
                        .collect::<Vec<_>>()
                        .join(", ");
                    
                    city_info.spawn((
                        Text::new(format!("Traits: {}", traits_text)),
                        TextFont { font_size: 12.0, ..default() },
                        TextColor(Color::srgb(0.8, 0.8, 0.8)),
                    ));
                }
                
                // Mission status
                if city_state.completed {
                    city_info.spawn((
                        Text::new(format!("✓ COMPLETED (Visited {} times)", city_state.times_visited)),
                        TextFont { font_size: 14.0, ..default() },
                        TextColor(Color::srgb(0.2, 0.8, 0.2)),
                    ));
                } else {
                    city_info.spawn((
                        Text::new("Available for missions"),
                        TextFont { font_size: 14.0, ..default() },
                        TextColor(Color::srgb(0.8, 0.8, 0.2)),
                    ));
                }
            });
        }
    } else {
        parent.spawn((
            Text::new("\nClick on a city to view details and select for missions"),
            TextFont { font_size: 16.0, ..default() },
            TextColor(Color::srgb(0.6, 0.6, 0.6)),
        ));
    }
}

fn update_city_alert(city_state: &mut CityState, current_day: u32) {
    // Simple alert decay system
    if current_day > city_state.last_mission_day + 7 {
        city_state.alert_level = match city_state.alert_level {
            AlertLevel::Red => AlertLevel::Orange,
            AlertLevel::Orange => AlertLevel::Yellow,
            AlertLevel::Yellow => AlertLevel::Green,
            AlertLevel::Green => AlertLevel::Green,
        };
    }
}
