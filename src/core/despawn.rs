use bevy::prelude::*;
use crate::core::components::*;

pub fn despawn_marked_entities(
    mut commands: Commands,
    query: Query<Entity, With<MarkedForDespawn>>,
) {
    for entity in query.iter() {
        commands.entity(entity).despawn(); // ← Now safe
    }
}