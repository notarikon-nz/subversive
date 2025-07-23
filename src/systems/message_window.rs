// Add this to your systems module (e.g., systems/message_window.rs)
use bevy::prelude::*;
use std::collections::VecDeque;

const MAX_MESSAGES: usize = 100;
const VISIBLE_LINES: usize = 10;
const FONT_SIZE: f32 = 8.0;
const LINE_HEIGHT: f32 = 10.0;
const WINDOW_PADDING: f32 = 4.0;

#[derive(Resource)]
pub struct MessageLog {
    messages: VecDeque<String>,
    scroll_offset: usize,
}

impl Default for MessageLog {
    fn default() -> Self {
        Self {
            messages: VecDeque::with_capacity(MAX_MESSAGES),
            scroll_offset: 0,
        }
    }
}

impl MessageLog {
    pub fn add(&mut self, message: String) {
        self.messages.push_back(message);
        if self.messages.len() > MAX_MESSAGES {
            self.messages.pop_front();
        }
        self.scroll_offset = 0; // Reset to bottom on new message
    }
}

#[derive(Component)]
pub struct MessageWindow;

#[derive(Component)]
pub struct MessageText;

pub fn setup_message_window(mut commands: Commands) {
    // Create container
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            right: Val::Px(8.0),
            bottom: Val::Px(8.0),
            width: Val::Percent(30.0),
            height: Val::Px(VISIBLE_LINES as f32 * LINE_HEIGHT + WINDOW_PADDING * 2.0),
            padding: UiRect::all(Val::Px(WINDOW_PADDING)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.5)),
        MessageWindow,
    ))
    .with_children(|parent| {
        parent.spawn((
            Text::default(),
            TextFont {
                font_size: FONT_SIZE,
                ..default()
            },
            TextColor(Color::srgb(0.8, 0.8, 0.8)),
            MessageText,
        ));
    });
}

pub fn update_message_window(
    log: Res<MessageLog>,
    mut text_query: Query<&mut Text, With<MessageText>>,
) {
    if !log.is_changed() {
        return;
    }
    
    let Ok(mut text) = text_query.single_mut() else { return };
    
    let start = log.messages.len().saturating_sub(VISIBLE_LINES + log.scroll_offset);
    let end = log.messages.len().saturating_sub(log.scroll_offset);
    
    let visible_messages: Vec<String> = log.messages
        .iter()
        .skip(start)
        .take(end - start)
        .cloned()
        .collect();
    
    **text = visible_messages.join("\n");
}

pub fn message_scroll_system(
    mut log: ResMut<MessageLog>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    let max_scroll = log.messages.len().saturating_sub(VISIBLE_LINES);
    
    if keys.just_pressed(KeyCode::PageUp) {
        log.scroll_offset = (log.scroll_offset + 1).min(max_scroll);
    }
    if keys.just_pressed(KeyCode::PageDown) {
        log.scroll_offset = log.scroll_offset.saturating_sub(1);
    }
}


// Macro for sending messages
#[macro_export]
macro_rules! punk {
    ($log:expr, $($arg:tt)*) => {
        $log.add(format!($($arg)*));
    };
}