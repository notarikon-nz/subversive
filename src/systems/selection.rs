use bevy::prelude::*;
use leafwing_input_manager::prelude::*;
use crate::core::*;
use crate::systems::*;

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
    selection_box_query: Query<Entity, With<SelectionBox>>,
) {
    let Some(world_pos) = get_world_mouse_position(&windows, &cameras) else { return; };
    let shift_held = keyboard.pressed(KeyCode::ShiftLeft) || keyboard.pressed(KeyCode::ShiftRight);

    // Handle mouse press
    if mouse.just_pressed(MouseButton::Left) {
        if shift_held {
            // Shift+click: toggle individual agent
            toggle_agent_selection(&mut commands, &mut selection, world_pos, &selectable_query);
        } else {
            // Check if clicking on an agent for single selection
            let clicked_agent = find_agent_at_position(world_pos, &selectable_query);
            
            if clicked_agent.is_some() {
                // Single select the clicked agent
                clear_selection(&mut commands, &mut selection, &selected_query);
                if let Some(entity) = clicked_agent {
                    add_to_selection(&mut commands, &mut selection, entity);
                }
            } else {
                // Start drag selection
                drag_state.dragging = true;
                drag_state.start_pos = world_pos;
                drag_state.current_pos = world_pos;
                
                // Clear current selection if not holding shift
                clear_selection(&mut commands, &mut selection, &selected_query);
            }
        }
    }

    // Handle mouse drag
    if drag_state.dragging {
        drag_state.current_pos = world_pos;
        
        // Update selection box visual
        update_selection_box(&mut commands, &drag_state, &selection_box_query);
        
        // Handle mouse release
        if mouse.just_released(MouseButton::Left) {
            // Complete drag selection
            complete_drag_selection(&mut commands, &mut selection, &drag_state, &selectable_query, shift_held, &selected_query);
            
            // Clean up drag state and selection box
            drag_state.dragging = false;
            for entity in selection_box_query.iter() {
                commands.entity(entity).despawn();
            }
        }
    }
    
    // Also clean up selection box if drag state gets reset somehow
    if !drag_state.dragging && !selection_box_query.is_empty() {
        for entity in selection_box_query.iter() {
            commands.entity(entity).despawn();
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
            // Add to selection
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
    selection_box_query: &Query<Entity, With<SelectionBox>>,
) {
    // Remove existing selection box
    for entity in selection_box_query.iter() {
        commands.entity(entity).despawn();
    }
    
    // Create new selection box
    let min_x = drag_state.start_pos.x.min(drag_state.current_pos.x);
    let max_x = drag_state.start_pos.x.max(drag_state.current_pos.x);
    let min_y = drag_state.start_pos.y.min(drag_state.current_pos.y);
    let max_y = drag_state.start_pos.y.max(drag_state.current_pos.y);
    
    let width = max_x - min_x;
    let height = max_y - min_y;
    let center = Vec2::new(min_x + width / 2.0, min_y + height / 2.0);
    
    if width > 5.0 || height > 5.0 { // Only show box if drag is significant
        commands.spawn((
            Sprite {
                color: Color::srgba(0.2, 0.8, 0.2, 0.3),
                custom_size: Some(Vec2::new(width, height)),
                ..default()
            },
            Transform::from_translation(center.extend(10.0)), // High Z for visibility
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
                    // Add to selection
                    add_to_selection(commands, selection, entity);
                }
            }
        }
        
        info!("Selected {} agents", selection.selected.len());
    }
}