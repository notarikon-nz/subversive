use bevy::prelude::*;
use bevy::audio::{Volume};
use crate::core::*;

#[derive(Resource)]
pub struct GameAudio {
    pub gunshot: Handle<AudioSource>,
    pub terminal_access: Handle<AudioSource>,
    pub footstep: Handle<AudioSource>,
    pub alert: Handle<AudioSource>,
    pub neurovector: Handle<AudioSource>,
}

#[derive(Event)]
pub struct AudioEvent {
    pub sound: AudioType,
    pub volume: f32,
}

#[derive(Clone)]
pub enum AudioType {
    Gunshot,
    TerminalAccess,
    Footstep,
    Alert,
    Neurovector,
}

impl Default for AudioEvent {
    fn default() -> Self {
        Self {
            sound: AudioType::Gunshot,
            volume: 0.5,
        }
    }
}

pub fn setup_audio(mut commands: Commands, asset_server: Res<AssetServer>) {
    let audio = GameAudio {
        gunshot: asset_server.load("audio/gunshot.ogg"),
        terminal_access: asset_server.load("audio/terminal.ogg"),
        footstep: asset_server.load("audio/footstep.ogg"),
        alert: asset_server.load("audio/alert.ogg"),
        neurovector: asset_server.load("audio/neurovector.ogg"),
    };
    
    commands.insert_resource(audio);
}

pub fn audio_system(
    mut commands: Commands,
    mut audio_events: EventReader<AudioEvent>,
    audio: Res<GameAudio>,
) {
    for event in audio_events.read() {
        let source = match event.sound {
            AudioType::Gunshot => &audio.gunshot,
            AudioType::TerminalAccess => &audio.terminal_access,
            AudioType::Footstep => &audio.footstep,
            AudioType::Alert => &audio.alert,
            AudioType::Neurovector => &audio.neurovector,
        };
        
        commands.spawn(AudioBundle {
            source: source.clone(),
            settings: PlaybackSettings::DESPAWN.with_volume(bevy::audio::Volume::new(event.volume)),
        });
    }
}

// Helper function for easy audio triggering
pub fn play_sound(audio_events: &mut EventWriter<AudioEvent>, sound: AudioType, volume: f32) {
    audio_events.send(AudioEvent { sound, volume });
}