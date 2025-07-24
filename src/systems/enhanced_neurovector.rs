// src/systems/enhanced_neurovector.rs
use bevy::prelude::*;
use leafwing_input_manager::prelude::*;
use crate::core::*;

#[derive(Component)]
pub struct ControlledCivilian {
    pub controller: Entity,
    pub combat_capable: bool,
    pub follow_distance: f32,
}

impl Default for ControlledCivilian {
    fn default() -> Self {
        Self {
            controller: Entity::PLACEHOLDER,
            combat_capable: true,
            follow_distance: 50.0,
        }
    }
}

pub fn enhanced_neurovector_system(
    mut commands: Commands,
    input: Query<&ActionState<PlayerAction>>,
    mut action_events: EventReader<ActionEvent>,
    mut audio_events: EventWriter<AudioEvent>,
    mut neurovector_query: Query<(&Transform, &mut NeurovectorCapability), With<Agent>>,
    target_query: Query<(Entity, &Transform, &mut Sprite), (With<NeurovectorTarget>, Without<ControlledCivilian>)>,
    game_mode: Res<GameMode>,
    windows: Query<&Window>,
    cameras: Query<(&Camera, &GlobalTransform)>,
    time: Res<Time>,
) {
    for (_, mut neurovector) in neurovector_query.iter_mut() {
        if neurovector.current_cooldown > 0.0 {
            neurovector.current_cooldown -= time.delta_secs();
        }
    }

    if let Some(TargetingMode::Neurovector { agent }) = &game_mode.targeting {
        let Ok(action_state) = input.single() else { return; };
        
        if action_state.just_pressed(&PlayerAction::Move) {
            if let Some(targets) = find_neurovector_targets(*agent, &neurovector_query, &target_query, &windows, &cameras) {
                execute_mass_neurovector_control(&mut commands, *agent, targets, &mut neurovector_query, &mut audio_events);
            }
        }
    }

    for event in action_events.read() {
        if let Action::NeurovectorControl { target } = event.action {
            execute_single_neurovector_control(&mut commands, event.entity, target, &mut neurovector_query, &mut audio_events);
        }
    }
}

fn find_neurovector_targets(
    agent: Entity,
    neurovector_query: &Query<(&Transform, &mut NeurovectorCapability), With<Agent>>,
    target_query: &Query<(Entity, &Transform, &mut Sprite), (With<NeurovectorTarget>, Without<ControlledCivilian>)>,
    windows: &Query<&Window>,
    cameras: &Query<(&Camera, &GlobalTransform)>,
) -> Option<Vec<Entity>> {
    let (agent_transform, neurovector) = neurovector_query.get(agent).ok()?;
    let mouse_pos = get_world_mouse_position(windows, cameras)?;
    
    if neurovector.current_cooldown > 0.0 { return None; }

    let agent_pos = agent_transform.translation.truncate();
    let mut targets = Vec::new();

    for (entity, transform, _) in target_query.iter() {
        let target_pos = transform.translation.truncate();
        let agent_distance = agent_pos.distance(target_pos);
        let mouse_distance = mouse_pos.distance(target_pos);

        if agent_distance <= neurovector.range && mouse_distance < 80.0 {
            targets.push(entity);
            if targets.len() >= neurovector.max_targets as usize {
                break;
            }
        }
    }

    if targets.is_empty() { None } else { Some(targets) }
}

fn execute_mass_neurovector_control(
    commands: &mut Commands,
    agent: Entity,
    targets: Vec<Entity>,
    neurovector_query: &mut Query<(&Transform, &mut NeurovectorCapability), With<Agent>>,
    audio_events: &mut EventWriter<AudioEvent>,
) {
    let Ok((_, mut neurovector)) = neurovector_query.get_mut(agent) else { return; };

    for target in targets {
        commands.entity(target).insert(ControlledCivilian {
            controller: agent,
            combat_capable: true,
            follow_distance: 50.0,
        });
        neurovector.controlled.push(target);
    }

    neurovector.current_cooldown = neurovector.cooldown * 0.5;

    audio_events.write(AudioEvent {
        sound: AudioType::Neurovector,
        volume: 0.8,
    });
}

fn execute_single_neurovector_control(
    commands: &mut Commands,
    agent: Entity,
    target: Entity,
    neurovector_query: &mut Query<(&Transform, &mut NeurovectorCapability), With<Agent>>,
    audio_events: &mut EventWriter<AudioEvent>,
) {
    let Ok((_, mut neurovector)) = neurovector_query.get_mut(agent) else { return; };

    if neurovector.controlled.len() < neurovector.max_targets as usize {
        commands.entity(target).insert(ControlledCivilian {
            controller: agent,
            combat_capable: true,
            follow_distance: 50.0,
        });
        neurovector.controlled.push(target);
        neurovector.current_cooldown = neurovector.cooldown;

        audio_events.write(AudioEvent {
            sound: AudioType::Neurovector,
            volume: 0.5,
        });
    }
}

pub fn controlled_civilian_behavior_system(
    mut controlled_query: Query<(Entity, &Transform, &ControlledCivilian, &mut Sprite), With<Civilian>>,
    agent_query: Query<&Transform, (With<Agent>, Without<Civilian>)>,
    enemy_query: Query<(Entity, &Transform), (With<Enemy>, Without<Dead>)>,
    mut action_events: EventWriter<ActionEvent>,
    mut commands: Commands,
) {
    for (civilian_entity, civilian_transform, controlled, mut sprite) in controlled_query.iter_mut() {
        sprite.color = Color::srgb(0.8, 0.3, 0.8);
        
        if let Ok(controller_transform) = agent_query.get(controlled.controller) {
            let civilian_pos = civilian_transform.translation.truncate();
            let controller_pos = controller_transform.translation.truncate();
            let distance_to_controller = civilian_pos.distance(controller_pos);

            if controlled.combat_capable {
                if let Some(enemy_entity) = find_nearest_enemy(&enemy_query, civilian_pos, 100.0) {
                    action_events.write(ActionEvent {
                        entity: civilian_entity,
                        action: Action::Attack(enemy_entity),
                    });
                    
                    commands.entity(civilian_entity).insert(WeaponState::new_from_type(&WeaponType::Pistol));
                    continue;
                }
            }

            if distance_to_controller > controlled.follow_distance {
                let follow_pos = controller_pos + (civilian_pos - controller_pos).normalize_or_zero() * controlled.follow_distance;
                action_events.write(ActionEvent {
                    entity: civilian_entity,
                    action: Action::MoveTo(follow_pos),
                });
            }
        }
    }
}

fn find_nearest_enemy(
    enemy_query: &Query<(Entity, &Transform), (With<Enemy>, Without<Dead>)>,
    position: Vec2,
    max_range: f32,
) -> Option<Entity> {
    enemy_query.iter()
        .filter(|(_, transform)| position.distance(transform.translation.truncate()) <= max_range)
        .min_by(|(_, a), (_, b)| {
            let dist_a = position.distance(a.translation.truncate());
            let dist_b = position.distance(b.translation.truncate());
            dist_a.partial_cmp(&dist_b).unwrap_or(std::cmp::Ordering::Equal)
        })
        .map(|(entity, _)| entity)
}

pub fn controlled_civilian_visual_system(
    mut gizmos: Gizmos,
    controlled_query: Query<&Transform, With<ControlledCivilian>>,
    agent_query: Query<(&Transform, &NeurovectorCapability), With<Agent>>,
) {
    for (agent_transform, neurovector) in agent_query.iter() {
        for &controlled_entity in &neurovector.controlled {
            if let Ok(controlled_transform) = controlled_query.get(controlled_entity) {
                gizmos.line_2d(
                    agent_transform.translation.truncate(),
                    controlled_transform.translation.truncate(),
                    Color::srgb(0.8, 0.3, 0.8),
                );
                
                gizmos.circle_2d(
                    controlled_transform.translation.truncate(),
                    25.0,
                    Color::srgba(0.8, 0.3, 0.8, 0.4),
                );
            }
        }
    }
}