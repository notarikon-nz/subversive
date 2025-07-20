use bevy::prelude::*;
use leafwing_input_manager::prelude::*;
use crate::core::*;

pub fn system(
    mut commands: Commands,
    input: Query<&ActionState<PlayerAction>>,
    mut action_events: EventReader<ActionEvent>,
    mut audio_events: EventWriter<AudioEvent>,
    mut neurovector_query: Query<(&Transform, &mut NeurovectorCapability), With<Agent>>,
    mut target_query: Query<(Entity, &Transform, &mut Sprite), (With<NeurovectorTarget>, Without<NeurovectorControlled>)>,
    mut controlled_query: Query<(Entity, &Transform, &mut Sprite), With<NeurovectorControlled>>,
    game_mode: Res<GameMode>,
    windows: Query<&Window>,
    cameras: Query<(&Camera, &GlobalTransform)>,
    time: Res<Time>,
) {
    if game_mode.paused { return; }

    // Update cooldowns
    for (_, mut neurovector) in neurovector_query.iter_mut() {
        if neurovector.current_cooldown > 0.0 {
            neurovector.current_cooldown -= time.delta_secs();
        }
    }

    // Handle targeting mode
    if let Some(TargetingMode::Neurovector { agent }) = &game_mode.targeting {
        let Ok(action_state) = input.get_single() else { return; };
        
        if action_state.just_pressed(&PlayerAction::Select) {
            if let Some(target) = find_neurovector_target(*agent, &neurovector_query, &target_query, &windows, &cameras) {
                // Directly execute the neurovector control instead of sending an event
                execute_neurovector_control(&mut commands, *agent, target, &mut neurovector_query, &mut audio_events);
            }
        }
    }

    // Process neurovector actions from events
    for event in action_events.read() {
        if let Action::NeurovectorControl { target } = event.action {
            execute_neurovector_control(&mut commands, event.entity, target, &mut neurovector_query, &mut audio_events);
        }
    }

    // Update visual feedback
    update_controlled_visuals(&mut controlled_query);
}

fn find_neurovector_target(
    agent: Entity,
    neurovector_query: &Query<(&Transform, &mut NeurovectorCapability), With<Agent>>,
    target_query: &Query<(Entity, &Transform, &mut Sprite), (With<NeurovectorTarget>, Without<NeurovectorControlled>)>,
    windows: &Query<&Window>,
    cameras: &Query<(&Camera, &GlobalTransform)>,
) -> Option<Entity> {
    let (agent_transform, neurovector) = neurovector_query.get(agent).ok()?;
    let mouse_pos = get_world_mouse_position(windows, cameras)?;
    
    if neurovector.current_cooldown > 0.0 {
        return None;
    }

    let mut closest_target = None;
    let mut closest_distance = f32::INFINITY;

    for (entity, transform, _) in target_query.iter() {
        let target_pos = transform.translation.truncate();
        let agent_distance = agent_transform.translation.truncate().distance(target_pos);
        let mouse_distance = mouse_pos.distance(target_pos);

        if agent_distance <= neurovector.range && mouse_distance < 25.0 && mouse_distance < closest_distance {
            closest_distance = mouse_distance;
            closest_target = Some(entity);
        }
    }

    closest_target
}

fn execute_neurovector_control(
    commands: &mut Commands,
    agent: Entity,
    target: Entity,
    neurovector_query: &mut Query<(&Transform, &mut NeurovectorCapability), With<Agent>>,
    audio_events: &mut EventWriter<AudioEvent>, 
) {
    let Ok((_, mut neurovector)) = neurovector_query.get_mut(agent) else { return; };

    if neurovector.controlled.len() < neurovector.max_targets as usize {
        commands.entity(target).insert(NeurovectorControlled { controller: agent });
        neurovector.controlled.push(target);
        neurovector.current_cooldown = neurovector.cooldown;

        // Play neurovector sound
        audio_events.write(AudioEvent {
            sound: AudioType::Neurovector,
            volume: 0.5,
        });

        info!("Neurovector control successful");
    }
}

fn update_controlled_visuals(controlled_query: &mut Query<(Entity, &Transform, &mut Sprite), With<NeurovectorControlled>>) {
    for (_, _, mut sprite) in controlled_query.iter_mut() {
        sprite.color = Color::srgb(0.8, 0.3, 0.8); // Purple when controlled
    }
}