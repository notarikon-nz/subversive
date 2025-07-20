use bevy::prelude::*;

#[derive(Resource, Default)]
pub struct EntityPool {
    inactive: Vec<Entity>,
}

#[derive(Component)]
pub struct Pooled;

#[derive(Component)]
pub struct Inactive;

impl EntityPool {
    pub fn get_or_spawn(&mut self, commands: &mut Commands) -> Entity {
        self.inactive.pop().unwrap_or_else(|| {
            commands.spawn((Pooled, Inactive)).id()
        })
    }
    
    pub fn return_entity(&mut self, entity: Entity, commands: &mut Commands) {
        commands.entity(entity)
            .insert(Inactive)
            .remove::<Visibility>()
            .remove::<Transform>();
        self.inactive.push(entity);
    }
}

pub fn cleanup_inactive_entities(
    mut pool: ResMut<EntityPool>,
    mut commands: Commands,
    query: Query<Entity, (With<Pooled>, With<Inactive>)>,
) {
    for entity in query.iter() {
        pool.inactive.retain(|&e| e != entity);
        
        // Safe cleanup with existence check
        if let Ok(mut entity_commands) = commands.get_entity(entity) {
            entity_commands.insert((
                Visibility::Hidden,
                Transform::from_translation(Vec3::new(10000.0, 10000.0, 0.0))
            ));
        }
    }
}