// ui_iced/integration.rs - Bevy-Iced integration
use bevy::prelude::*;
use iced::{Application, Settings};
use std::sync::{Arc, Mutex};
use crate::ui_iced::{SubversiveUI, IcedUIBridge, UISharedState, sync_iced_ui};

// Plugin to integrate Iced with Bevy
pub struct IcedUIPlugin;

impl Plugin for IcedUIPlugin {
    fn build(&self, app: &mut App) {
        let shared_state = Arc::new(Mutex::new(UISharedState::default()));
        
        app.insert_resource(IcedUIBridge {
            state: shared_state.clone(),
        })
        .add_systems(Update, sync_iced_ui)
        .add_systems(Startup, setup_iced_ui.pipe(handle_iced_setup));
        
        // Launch Iced in separate thread
        let state_clone = shared_state.clone();
        std::thread::spawn(move || {
            let settings = Settings::with_flags(state_clone);
            SubversiveUI::run(settings).expect("Failed to run Iced UI");
        });
    }
}

fn setup_iced_ui() -> Arc<Mutex<UISharedState>> {
    Arc::new(Mutex::new(UISharedState::default()))
}

fn handle_iced_setup(_: In<Arc<Mutex<UISharedState>>>) {
    info!("Iced UI system initialized");
}