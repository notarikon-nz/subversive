use bevy::prelude::*;
use crate::core::*;

pub fn reload_system(
    mut agent_query: Query<(&mut WeaponState, &Inventory), With<Agent>>,
    mut enemy_query: Query<&mut WeaponState, (With<Enemy>, Without<Agent>)>,
    mut action_events: EventReader<ActionEvent>,
    mut audio_events: EventWriter<AudioEvent>,
    time: Res<Time>,
    game_mode: Res<GameMode>,
) {
    if game_mode.paused { return; }
    
    // Update reload timers for all entities
    for (mut weapon_state, inventory) in agent_query.iter_mut() {
        update_reload_timer(&mut weapon_state, &time, &mut audio_events);
        
        // Apply attachment modifiers if weapon config changed
        if let Some(weapon_config) = &inventory.equipped_weapon {
            weapon_state.apply_attachment_modifiers(weapon_config);
        }
    }
    
    for mut weapon_state in enemy_query.iter_mut() {
        update_reload_timer(&mut weapon_state, &time, &mut audio_events);
    }
    
    // Process reload action events
    for event in action_events.read() {
        if let Action::Reload = event.action {
            if let Ok((mut weapon_state, _)) = agent_query.get_mut(event.entity) {
                if !weapon_state.is_reloading && weapon_state.current_ammo < weapon_state.max_ammo {
                    weapon_state.start_reload();
                    
                    // play_sound
                    audio_events.write(AudioEvent {
                        sound: AudioType::Reload,
                        volume: 0.4,
                    });
                }
            } else if let Ok(mut weapon_state) = enemy_query.get_mut(event.entity) {
                if !weapon_state.is_reloading && weapon_state.current_ammo < weapon_state.max_ammo {
                    weapon_state.start_reload();
                }
            }
        }
    }
}

fn update_reload_timer(
    weapon_state: &mut WeaponState,
    time: &Time,
    audio_events: &mut EventWriter<AudioEvent>,
) {
    if weapon_state.is_reloading {
        weapon_state.reload_timer -= time.delta_secs();
        
        if weapon_state.reload_timer <= 0.0 {
            weapon_state.complete_reload();
            
            // Play reload complete sound
            audio_events.write(AudioEvent {
                sound: AudioType::ReloadComplete,
                volume: 0.3,
            });
        }
    }
}