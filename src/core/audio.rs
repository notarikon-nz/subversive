use bevy::prelude::*;
use bevy::audio::PlaybackSettings;  // Remove Volume import

#[derive(Resource)]
pub struct GameAudio {
    pub gunshot: Handle<AudioSource>,
    pub terminal_access: Handle<AudioSource>,
    pub footstep: Handle<AudioSource>,
    pub alert: Handle<AudioSource>,
    pub neurovector: Handle<AudioSource>,
    pub reload: Handle<AudioSource>,
    pub reload_complete: Handle<AudioSource>,
    //
    pub cursor_target: Handle<AudioSource>,
    pub cursor_interact: Handle<AudioSource>,
    pub cursor_hack: Handle<AudioSource>,

    // 0.2.10
    pub gate_open: Handle<AudioSource>,
    pub gate_close: Handle<AudioSource>,
    pub door_open: Handle<AudioSource>,
    pub door_close: Handle<AudioSource>,
    pub access_granted: Handle<AudioSource>,
    pub access_denied: Handle<AudioSource>,
    pub card_swipe: Handle<AudioSource>,    
    pub money_dispense: Handle<AudioSource>,    
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
    Reload,
    ReloadComplete,    
    Explosion,
    CursorTarget,
    CursorInteract,
    CursorHack,
    // 0.2.10
    GateOpen,
    GateClose,
    DoorOpen,
    DoorClose,
    AccessGranted,
    AccessDenied,
    CardSwipe,    
    MoneyDispense,
    // 0.2.16
    FootstepSnow,
    FootstepWet,
    LightBuzz,     // For flickering lights
    PowerDown,     // When lights go out
    PowerUp,       // When lights come back 
    GlassBreak,      // Street light destruction
    ElectricalBuzz,  // Flickering lights

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
        reload: asset_server.load("audio/reload.ogg"),
        reload_complete: asset_server.load("audio/reload_complete.ogg"),
        // 
        cursor_target: asset_server.load("audio/cursor_target.ogg"),
        cursor_interact: asset_server.load("audio/cursor_interact.ogg"),
        cursor_hack: asset_server.load("audio/cursor_hack.ogg"),
        // 0.2.10
        gate_open: asset_server.load("audio/gate_open.ogg"),
        gate_close: asset_server.load("audio/gate_close.ogg"),
        door_open: asset_server.load("audio/door_open.ogg"),
        door_close: asset_server.load("audio/door_close.ogg"),
        access_granted: asset_server.load("audio/access_granted.ogg"),
        access_denied: asset_server.load("audio/access_denied.ogg"),
        card_swipe: asset_server.load("audio/card_swipe.ogg"),
        money_dispense: asset_server.load("audio/money_dispense.ogg"),
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
            AudioType::Reload => &audio.reload,
            AudioType::ReloadComplete => &audio.reload_complete,
            AudioType::MoneyDispense => &audio.money_dispense,
            _ => &audio.alert, // PLACEHOLDER
        };
        
        // FIXED: Use default settings for now, volume control can be added later
        commands.spawn((
            AudioPlayer::new(source.clone()),
            PlaybackSettings::DESPAWN,  // Use default volume for now
        ));
    }
}

// Helper function for easy audio triggering
pub fn play_sound(audio_events: &mut EventWriter<AudioEvent>, sound: AudioType, volume: f32) {
    audio_events.write(AudioEvent { sound, volume });
}