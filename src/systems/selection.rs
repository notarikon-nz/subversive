use bevy::prelude::*;
use crate::core::*;

// AGENT SELECTION VIA 1,2,3
#[derive(Component)]
pub struct AgentIndex(pub usize);

pub fn system(
    mut commands: Commands,
    mut selection: ResMut<SelectionState>,
    mut drag_state: ResMut<SelectionDrag>,
    selectable_query: Query<(Entity, &Selectable, &Transform), With<Agent>>,
    selected_query: Query<Entity, With<Selected>>,
    mouse: Res<ButtonInput<MouseButton>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    windows: Query<&Window>,
    cameras: Query<(&Camera, &GlobalTransform)>,
    selection_box_query: Query<Entity, (With<SelectionBox>, Without<MarkedForDespawn>)>,
) {
    let Some(world_pos) = get_world_mouse_position(&windows, &cameras) else { 
        return; 
    };
    
    if drag_state.dragging {
        
        if mouse.just_released(MouseButton::Left) {
            complete_drag_selection(&mut commands, &mut selection, &drag_state, &selectable_query, false, &selected_query);
            
            drag_state.dragging = false;
            
            // Safe cleanup selection box
            for entity in selection_box_query.iter() {
                commands.entity(entity).insert(MarkedForDespawn);
            }
        } else {
            drag_state.current_pos = world_pos;
            update_selection_box(&mut commands, &drag_state, &selection_box_query);
        }
    }
    
    // Rest of selection logic...
    if mouse.just_pressed(MouseButton::Left) {
        let clicked_agent = find_agent_at_position(world_pos, &selectable_query);
        
        if let Some(agent) = clicked_agent {
            clear_selection(&mut commands, &mut selection, &selected_query);
            add_to_selection(&mut commands, &mut selection, agent);
        } else {
            drag_state.dragging = true;
            drag_state.start_pos = world_pos;
            drag_state.current_pos = world_pos;
            clear_selection(&mut commands, &mut selection, &selected_query);
        }
    }
}

fn toggle_agent_selection(
    commands: &mut Commands,
    selection: &mut SelectionState,
    world_pos: Vec2,
    selectable_query: &Query<(Entity, &Selectable, &Transform), With<Agent>>,
) {
    if let Some(entity) = find_agent_at_position(world_pos, selectable_query) {
        if selection.selected.contains(&entity) {
            // Remove from selection
            commands.entity(entity).remove::<Selected>();
            selection.selected.retain(|&e| e != entity);
        } else {
            add_to_selection(commands, selection, entity);
        }
    }
}

fn find_agent_at_position(
    world_pos: Vec2,
    selectable_query: &Query<(Entity, &Selectable, &Transform), With<Agent>>,
) -> Option<Entity> {
    let mut closest = None;
    let mut closest_distance = f32::INFINITY;

    for (entity, selectable, transform) in selectable_query.iter() {
        let distance = world_pos.distance(transform.translation.truncate());
        if distance < selectable.radius && distance < closest_distance {
            closest_distance = distance;
            closest = Some(entity);
        }
    }

    closest
}

fn clear_selection(
    commands: &mut Commands,
    selection: &mut SelectionState,
    selected_query: &Query<Entity, With<Selected>>,
) {
    for entity in selected_query.iter() {
        commands.entity(entity).remove::<Selected>();
    }
    selection.selected.clear();
}

fn add_to_selection(
    commands: &mut Commands,
    selection: &mut SelectionState,
    entity: Entity,
) {
    commands.entity(entity).insert(Selected);
    if !selection.selected.contains(&entity) {
        selection.selected.push(entity);
    }
}


fn update_selection_box(
    commands: &mut Commands,
    drag_state: &SelectionDrag,
    selection_box_query: &Query<Entity, (With<SelectionBox>, Without<MarkedForDespawn>)>,
) {
    // Safe cleanup existing selection box
    for entity in selection_box_query.iter() {
        commands.entity(entity).insert(MarkedForDespawn);
    }
    
    // Create new selection box
    let min_x = drag_state.start_pos.x.min(drag_state.current_pos.x);
    let max_x = drag_state.start_pos.x.max(drag_state.current_pos.x);
    let min_y = drag_state.start_pos.y.min(drag_state.current_pos.y);
    let max_y = drag_state.start_pos.y.max(drag_state.current_pos.y);
    
    let width = max_x - min_x;
    let height = max_y - min_y;
    let center = Vec2::new(min_x + width / 2.0, min_y + height / 2.0);
    
    if width > 5.0 || height > 5.0 {
        commands.spawn((
            Sprite {
                color: Color::srgba(0.2, 0.8, 0.2, 0.3),
                custom_size: Some(Vec2::new(width, height)),
                ..default()
            },
            Transform::from_translation(center.extend(10.0)),
            SelectionBox {
                start: drag_state.start_pos,
                end: drag_state.current_pos,
            },
        ));
    }
}

fn complete_drag_selection(
    commands: &mut Commands,
    selection: &mut SelectionState,
    drag_state: &SelectionDrag,
    selectable_query: &Query<(Entity, &Selectable, &Transform), With<Agent>>,
    shift_held: bool,
    selected_query: &Query<Entity, With<Selected>>,
) {
    let min_x = drag_state.start_pos.x.min(drag_state.current_pos.x);
    let max_x = drag_state.start_pos.x.max(drag_state.current_pos.x);
    let min_y = drag_state.start_pos.y.min(drag_state.current_pos.y);
    let max_y = drag_state.start_pos.y.max(drag_state.current_pos.y);
    
    // Only proceed if the selection box is significant
    let width = max_x - min_x;
    let height = max_y - min_y;
    
    if width > 5.0 || height > 5.0 {
        // Clear existing selection if not holding shift
        if !shift_held {
            clear_selection(commands, selection, selected_query);
        }
        
        // Select all agents within the box
        for (entity, _, transform) in selectable_query.iter() {
            let pos = transform.translation.truncate();
            
            if pos.x >= min_x && pos.x <= max_x && pos.y >= min_y && pos.y <= max_y {
                if shift_held && selection.selected.contains(&entity) {
                    // Shift+drag on already selected agent = deselect
                    commands.entity(entity).remove::<Selected>();
                    selection.selected.retain(|&e| e != entity);
                } else {
                    add_to_selection(commands, selection, entity);
                }
            }
        }
    }
}

// Modified selection system to handle agent-specific selection
pub fn handle_agent_selection_events(
    mut action_events: EventReader<ActionEvent>,
    mut selection: ResMut<SelectionState>,
    agents: Query<(Entity, &AgentIndex, &Transform), With<Agent>>,
    mut camera_query: Query<&mut Transform, (With<Camera>, Without<Agent>)>,
) {
    for event in action_events.read() {
        match event.action {
            Action::SelectAgent(idx) => {
                // Find agent with matching index
                for (entity, agent_idx, _) in agents.iter() {
                    if agent_idx.0 == idx {
                        selection.selected.clear();
                        selection.selected.push(entity);
                        break;
                    }
                }
            }
            Action::CenterCameraOnAgent(idx) => {
                // Find agent and center camera without changing selection
                for (entity, agent_idx, agent_transform) in agents.iter() {
                    if agent_idx.0 == idx {
                        if let Ok(mut camera_transform) = camera_query.single_mut() {
                            camera_transform.translation.x = agent_transform.translation.x;
                            camera_transform.translation.y = agent_transform.translation.y;
                        }
                        break;
                    }
                }
            }
            _ => {}
        }
    }
}