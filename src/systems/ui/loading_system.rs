use bevy::prelude::*;
use crate::core::*;

pub fn loading_system(
    mut next_state: ResMut<NextState<GameState>>,
    mut frame_count: Local<u32>,
) {
    *frame_count += 1;
    if *frame_count > 10 {  // Wait 10 frames
        next_state.set(GameState::MainMenu);
    }
}