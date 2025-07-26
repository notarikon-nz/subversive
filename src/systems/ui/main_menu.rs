use bevy::prelude::*;
use crate::core::*;
use crate::systems::save::save_game_exists;

#[derive(Component)]
pub struct MainMenuUI;

#[derive(Component)]
pub struct MenuOption {
    pub option_type: MenuOptionType,
    pub index: usize,
}

#[derive(PartialEq, Clone, Copy)]
pub enum MenuOptionType {
    Continue,
    NewGame,
    Settings,
    Credits,
    Quit,
}

#[derive(Resource)]
pub struct MainMenuState {
    pub selected_index: usize,
    pub has_save: bool,
}

impl Default for MainMenuState {
    fn default() -> Self {
        Self {
            selected_index: 0,
            has_save: false,
        }
    }
}

pub fn setup_main_menu(
    mut commands: Commands,
    mut menu_state: ResMut<MainMenuState>,
) {
    // Check for save game
    menu_state.has_save = save_game_exists();
    menu_state.selected_index = 0;

    // Main container
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        BackgroundColor(Color::srgb(0.05, 0.05, 0.1)),
        MainMenuUI,
    )).with_children(|parent| {
        // Logo placeholder
        parent.spawn((
            Node {
                width: Val::Px(400.0),
                height: Val::Px(150.0),
                margin: UiRect::bottom(Val::Px(50.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.2, 0.2, 0.3, 0.3)),
        )).with_children(|logo| {
            logo.spawn((
                Text::new("SUBVERSIVE"),
                TextFont {
                    font_size: 48.0,
                    ..default()
                },
                TextColor(Color::srgb(0.8, 0.2, 0.2)),
            ));
        });

        // Menu options container
        parent.spawn((
            Node {
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(10.0),
                ..default()
            },
        )).with_children(|menu| {
            let mut index = 0;

            // Continue (conditional)
            if menu_state.has_save {
                spawn_menu_option(menu, "Continue", MenuOptionType::Continue, index, index == menu_state.selected_index);
                index += 1;
            }

            // New Game
            spawn_menu_option(menu, "New Game", MenuOptionType::NewGame, index, index == menu_state.selected_index);
            index += 1;

            // Settings
            spawn_menu_option(menu, "Settings", MenuOptionType::Settings, index, index == menu_state.selected_index);
            index += 1;

            // Credits
            spawn_menu_option(menu, "Credits", MenuOptionType::Credits, index, index == menu_state.selected_index);
            index += 1;

            // Quit
            spawn_menu_option(menu, "Quit Game", MenuOptionType::Quit, index, index == menu_state.selected_index);
        });

        // Version number
        parent.spawn((
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(10.0),
                left: Val::Px(10.0),
                ..default()
            },
        )).with_children(|version| {
            version.spawn((
                Text::new("v0.1.0"),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::srgb(0.5, 0.5, 0.5)),
            ));
        });
    });
}

fn spawn_menu_option(
    parent: &mut ChildSpawnerCommands,
    text: &str,
    option_type: MenuOptionType,
    index: usize,
    is_selected: bool,
) {
    let color = if is_selected {
        Color::srgb(0.8, 0.8, 0.2)
    } else {
        Color::WHITE
    };

    parent.spawn((
        Button,
        Node {
            padding: UiRect::all(Val::Px(15.0)),
            justify_content: JustifyContent::Center,
            ..default()
        },
        BackgroundColor(Color::NONE),
        MenuOption { option_type, index },
    )).with_children(|button| {
        button.spawn((
            Text::new(text),
            TextFont {
                font_size: 24.0,
                ..default()
            },
            TextColor(color),
        ));
    });
}

pub fn menu_input_system(
    input: Res<ButtonInput<KeyCode>>,
    mut menu_state: ResMut<MainMenuState>,
    mut next_state: ResMut<NextState<GameState>>,
    options: Query<&MenuOption>,
    mut app_exit: EventWriter<bevy::app::AppExit>,
    mut global_data: ResMut<GlobalData>,
) {
    let option_count = options.iter().count();
    if option_count == 0 { return; }

    // Handle keyboard navigation
    if input.just_pressed(KeyCode::KeyW) || input.just_pressed(KeyCode::ArrowUp) {
        if menu_state.selected_index > 0 {
            menu_state.selected_index -= 1;
        } else {
            menu_state.selected_index = option_count - 1;
        }
    }

    if input.just_pressed(KeyCode::KeyS) || input.just_pressed(KeyCode::ArrowDown) {
        menu_state.selected_index = (menu_state.selected_index + 1) % option_count;
    }

    // Jump to quit
    if input.just_pressed(KeyCode::Escape) {
        menu_state.selected_index = option_count - 1;
    }

    // Select option
    if input.just_pressed(KeyCode::Space) || input.just_pressed(KeyCode::Enter) {
        if let Some(option) = options.iter().find(|o| o.index == menu_state.selected_index) {
            execute_menu_option(option.option_type, &mut next_state, &mut app_exit, &mut global_data);
        }
    }
}

pub fn menu_mouse_system(
    mut menu_state: ResMut<MainMenuState>,
    mut next_state: ResMut<NextState<GameState>>,
    interaction_query: Query<(&Interaction, &MenuOption), (Changed<Interaction>, With<Button>)>,
    mut app_exit: EventWriter<bevy::app::AppExit>,
    mut global_data: ResMut<GlobalData>,
) {
    // Handle hover - simplified approach using Bevy's built-in Interaction
    for (interaction, option) in interaction_query.iter() {
        match interaction {
            Interaction::Hovered => {
                menu_state.selected_index = option.index;
            }
            Interaction::Pressed => {
                execute_menu_option(option.option_type, &mut next_state, &mut app_exit, &mut global_data);
            }
            Interaction::None => {}
        }
    }
}

fn execute_menu_option(
    option_type: MenuOptionType,
    next_state: &mut NextState<GameState>,
    app_exit: &mut EventWriter<bevy::app::AppExit>,
    global_data: &mut GlobalData,
) {
    match option_type {
        MenuOptionType::Continue => {
            // Load save game
            if let Some(loaded_data) = crate::systems::save::load_game() {
                *global_data = loaded_data;
                next_state.set(GameState::GlobalMap);
            }
        },
        MenuOptionType::NewGame => {
            // Reset to default
            *global_data = GlobalData::default();
            crate::systems::save::save_game(global_data);
            next_state.set(GameState::GlobalMap); // Or GameState::FoundCorp when implemented
        },
        MenuOptionType::Settings => {
            next_state.set(GameState::Settings);
        },
        MenuOptionType::Credits => {
            next_state.set(GameState::Credits);
        },
        MenuOptionType::Quit => {
            app_exit.write(bevy::app::AppExit::Success);
        },
    }
}

pub fn update_menu_visuals(
    menu_state: Res<MainMenuState>,
    mut options: Query<(&MenuOption, &Children)>,
    mut texts: Query<&mut TextColor>,
) {
    if !menu_state.is_changed() { return; }

    for (option, children) in options.iter_mut() {
        let is_selected = option.index == menu_state.selected_index;
        let color = if is_selected {
            Color::srgb(0.8, 0.8, 0.2)
        } else {
            Color::WHITE
        };

        for child in children.iter() {  // Note the & before child
            if let Ok(mut text_color) = texts.get_mut(child) {
                text_color.0 = color;
            }
        }
    }
}

pub fn cleanup_main_menu(
    mut commands: Commands,
    menu_ui: Query<Entity, With<MainMenuUI>>,
) {
    for entity in menu_ui.iter() {
        commands.entity(entity).insert(MarkedForDespawn);
    }
}