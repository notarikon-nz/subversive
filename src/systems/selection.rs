use bevy::prelude::*;
use leafwing_input_manager::prelude::*;
use crate::core::*;

pub fn system(
    mut commands: Commands,
    input: Query<&ActionState<PlayerAction>>,
    mut selection: ResMut<SelectionState>,
    selectable_query: Query<(Entity, &Selectable, &Transform), With<Agent>>,
    selected_query: Query<Entity, With<Selected>>,
    windows: Query<&Window>,
    cameras: Query<(&Camera, &GlobalTransform)>,
) {
    let Ok(action_state) = input.get_single() else { return; };

    if action_state.just_pressed(&PlayerAction::Select) {
        let Some(world_pos) = get_world_mouse_position(&windows, &cameras) else { return; };

        // Clear previous selection
        for entity in selected_query.iter() {
            commands.entity(entity).remove::<Selected>();
        }
        selection.selected.clear();

        // Find closest selectable agent
        let mut closest = None;
        let mut closest_distance = f32::INFINITY;

        for (entity, selectable, transform) in selectable_query.iter() {
            let distance = world_pos.distance(transform.translation.truncate());
            if distance < selectable.radius && distance < closest_distance {
                closest_distance = distance;
                closest = Some(entity);
            }
        }

        // Select the closest entity
        if let Some(entity) = closest {
            commands.entity(entity).insert(Selected);
            selection.selected.push(entity);
            info!("Agent selected");
        }
    }
}