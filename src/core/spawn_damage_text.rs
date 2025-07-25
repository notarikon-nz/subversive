use bevy::prelude::*;
use crate::core::events::DamageTextEvent;
use crate::systems::explosions::CombatTextSettings;

pub fn damage_text_event_system(
    mut damage_text_events: EventReader<DamageTextEvent>,
    mut commands: Commands,
    combat_text_settings: Res<CombatTextSettings>,
) {
    for event in damage_text_events.read() {
        spawn_damage_text(&mut commands, event.position, event.damage, &combat_text_settings);
    }
}

pub fn spawn_damage_text(
    commands: &mut Commands,
    position: Vec2,
    damage: f32,
    settings: &CombatTextSettings,
) {
    let damage_text = format!("{:.0}", damage);
    let text_color = if damage >= 50.0 {
        Color::srgb(1.0, 0.2, 0.2)
    } else if damage >= 25.0 {
        Color::srgb(1.0, 0.8, 0.2)
    } else {
        Color::srgb(1.0, 1.0, 0.2)
    };
    
    commands.spawn((
        Text2d::new(damage_text),
        TextFont {
            font_size: settings.font_size,
            ..default()
        },
        TextColor(text_color),
        Transform::from_translation((position + Vec2::new(0.0, 30.0)).extend(100.0)),
        crate::systems::explosions::FloatingText {
            lifetime: 1.0,
            velocity: Vec2::new(
                (rand::random::<f32>() - 0.5) * 20.0,
                50.0,
            ),
        },
    ));
}
