// src/systems/ui/hub/mod.rs - Updated with agent state integration
use bevy::prelude::*;
use crate::core::*;
use crate::systems::*;
use serde::{Deserialize, Serialize};

// Re-export all tab modules
pub mod global_map;
pub mod research;
pub mod agents;
pub mod manufacture;
pub mod missions;

pub use agents::*;
pub use global_map::*;

#[derive(Component)]
pub struct HubScreen;

#[derive(Resource, Default)]
pub struct HubState {
    pub active_tab: HubTab,
    pub selected_region: usize,
    pub selected_research_project: usize,
}

#[derive(Debug, Default, Clone, Resource, Serialize, Deserialize)]
pub struct CyberneticsDatabase {
    pub cybernetics: Vec<CyberneticUpgrade>,
}

impl CyberneticsDatabase {
    pub fn load() -> Self {
        match std::fs::read_to_string("data/cybernetics.json") {
            Ok(content) => {
                match serde_json::from_str::<CyberneticsDatabase>(&content) {  // Add type annotation
                    Ok(data) => {
                        info!("Loaded {} cybernetics from data/cybernetics.json", data.cybernetics.len());
                        data
                    },
                    Err(e) => {
                        error!("Failed to parse cybernetics.json: {}", e);
                        Self { cybernetics: Vec::new() }
                    }
                }
            },
            Err(e) => {
                error!("Failed to load data/cybernetics.json: {}", e);
                Self { cybernetics: Vec::new() }
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum HubTab {
    #[default]
    GlobalMap,
    Research,
    Agents,
    Manufacture,
    Missions,
}

impl HubTab {
    pub fn next(self) -> Self {
        match self {
            Self::GlobalMap => Self::Research,
            Self::Research => Self::Agents,
            Self::Agents => Self::Manufacture,
            Self::Manufacture => Self::Missions,
            Self::Missions => Self::GlobalMap,
        }
    }
    
    pub fn previous(self) -> Self {
        match self {
            Self::GlobalMap => Self::Missions,
            Self::Research => Self::GlobalMap,
            Self::Agents => Self::Research,
            Self::Manufacture => Self::Agents,
            Self::Missions => Self::Manufacture,
        }
    }
}

pub fn reset_hub_to_global_map(mut hub_state: ResMut<HubState>) {
    hub_state.active_tab = HubTab::GlobalMap;
}

// Group related resources together
#[derive(Resource)]
pub struct HubDatabases {
    pub research_db: ResearchDatabase,
    pub cybernetics_db: CyberneticsDatabase,
    pub attachment_db: AttachmentDatabase,
    pub cities_db: CitiesDatabase,
}
impl Default for HubDatabases {
    fn default() -> Self {
        Self {
            research_db: ResearchDatabase::default(),
            cybernetics_db: CyberneticsDatabase::default(),
            attachment_db: AttachmentDatabase::default(),
            cities_db: CitiesDatabase::default(),
        }
    }
}


#[derive(Resource)]
pub struct HubStates {
    pub hub_state: HubState,
    pub manufacture_state: ManufactureState,
    pub agent_state: AgentManagementState,
    pub map_state: GlobalMapState,
}

impl Default for HubStates {
    fn default() -> Self {
        Self {
            hub_state: HubState::default(),
            agent_state: AgentManagementState::default(),
            map_state: GlobalMapState::default(),
            manufacture_state: ManufactureState::default(),
        }
    }
}

#[derive(Resource)]
pub struct HubProgress {
    pub research_progress: ResearchProgress,
    pub cities_progress: CitiesProgress,
    pub unlocked: UnlockedAttachments,
}

impl Default for HubProgress {
    fn default() -> Self {
        Self {
            research_progress: ResearchProgress::default(),
            cities_progress: CitiesProgress::default(),
            unlocked: UnlockedAttachments::default(),
        }
    }
}

// Main hub coordination system - now under 16 parameters
pub fn hub_system(
    mut commands: Commands,
    mut next_state: ResMut<NextState<GameState>>,
    mut global_data: ResMut<GlobalData>,
    mut hub_states: ResMut<HubStates>,
    mut hub_progress: ResMut<HubProgress>,
    hub_databases: Res<HubDatabases>,
    input: Res<ButtonInput<KeyCode>>,
    screen_query: Query<Entity, With<HubScreen>>,
    agent_query: Query<&mut Inventory, With<Agent>>,
    fonts: Option<Res<GameFonts>>,
    windows: Query<&Window>,
    cameras: Query<(&Camera, &GlobalTransform)>,
    mouse: Res<ButtonInput<MouseButton>>,
    city_query: Query<(Entity, &Transform, &InteractiveCity)>,
) {
    // Global tab switching with Q/E
    let mut tab_changed = false;
    
    if input.just_pressed(KeyCode::KeyQ) {
        hub_states.hub_state.active_tab = hub_states.hub_state.active_tab.previous();
        tab_changed = true;
    }
    
    if input.just_pressed(KeyCode::KeyE) {
        hub_states.hub_state.active_tab = hub_states.hub_state.active_tab.next();
        tab_changed = true;
    }
    
    if tab_changed {
        let fonts_ref = fonts.as_ref().map(|f| f.as_ref());
        rebuild_hub(
            &mut commands, 
            &screen_query, 
            &global_data, 
            &hub_states, 
            &hub_progress, 
            &hub_databases, 
            fonts_ref);
    }

    let HubStates {
        hub_state,
        agent_state,
        map_state,
        manufacture_state,
        ..
    } = &mut *hub_states;

    let HubProgress {
        cities_progress,
        unlocked,
        research_progress,
        ..
    } = &mut *hub_progress;

    // Delegate input handling to appropriate tab
    let needs_rebuild = match hub_state.active_tab {
        HubTab::GlobalMap => global_map::handle_input(
            &input, 
            &mut global_data, 
            hub_state,
            &hub_databases.cities_db,
            cities_progress,
            map_state,
            &windows,
            &cameras,
            &mouse,
            &city_query,
        ),
        HubTab::Research => research::handle_input(
            &input, 
            &mut global_data, 
            research_progress, 
            &hub_databases.research_db, 
            unlocked,
            &mut hub_states.hub_state.selected_research_project
        ),
        HubTab::Agents => agents::handle_input(
            &input, 
            hub_state,
            agent_state,
            &mut global_data,
            &hub_databases.cybernetics_db.cybernetics
        ),
        HubTab::Manufacture => manufacture::handle_input(
            &input, 
            hub_state, 
            manufacture_state, 
            &mut global_data, 
            agent_query, 
            &hub_databases.attachment_db
        ),
        HubTab::Missions => missions::handle_input(
            &input, 
            &mut commands, 
            &mut next_state, 
            &global_data
        ),
    };

    // Global escape
    if input.just_pressed(KeyCode::Escape) {
        std::process::exit(0);
    }

    if screen_query.is_empty() || needs_rebuild {
        let fonts_ref = fonts.as_ref().map(|f| f.as_ref());
        rebuild_hub(&mut commands, &screen_query, &global_data, &hub_states, 
                   &hub_progress, &hub_databases, fonts_ref);  
    }
}

fn rebuild_hub(
    commands: &mut Commands,
    screen_query: &Query<Entity, With<HubScreen>>,
    global_data: &GlobalData,
    hub_states: &HubStates,
    hub_progress: &HubProgress,
    hub_databases: &HubDatabases,
    fonts: Option<&GameFonts>,
) {
    for entity in screen_query.iter() {
        commands.safe_despawn(entity);
    }
    create_hub_ui(commands, global_data, hub_states, hub_progress, 
                  hub_databases, fonts);
}

fn create_hub_ui(
    commands: &mut Commands, 
    global_data: &GlobalData, 
    hub_states: &HubStates,
    hub_progress: &HubProgress,
    hub_databases: &HubDatabases,
    fonts: Option<&GameFonts>,
) {
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            ..default()
        },
        BackgroundColor(Color::srgb(0.1, 0.1, 0.2)),
        HubScreen,
    )).with_children(|parent| {
        create_header(parent, global_data, fonts);
        create_tab_bar(parent, hub_states.hub_state.active_tab, fonts);
        
        match hub_states.hub_state.active_tab {
            HubTab::GlobalMap => global_map::create_content(
                parent, 
                global_data, 
                &hub_states.hub_state, 
                &hub_databases.cities_db, 
                &hub_progress.cities_progress, 
                &mut hub_states.map_state.clone() // Clone for immutable access
            ),
            HubTab::Research => research::create_content(
                parent, 
                global_data, 
                &hub_progress.research_progress, 
                &hub_databases.research_db,
                hub_states.hub_state.selected_research_project
            ),
            HubTab::Agents => agents::create_content(
                parent, 
                global_data, 
                &hub_states.agent_state,
                &hub_databases.cybernetics_db.cybernetics,
            ),
            HubTab::Manufacture => manufacture::create_content(
                parent, 
                global_data, 
                &hub_states.manufacture_state, 
                &hub_databases.attachment_db, 
                &hub_progress.unlocked
            ),
            HubTab::Missions => missions::create_content(parent, global_data, &hub_states.hub_state),
        }
        
        create_footer(parent, hub_states.hub_state.active_tab, fonts); 
    });
}

fn create_header(
    parent: &mut ChildSpawnerCommands, 
    global_data: &GlobalData,
    fonts: Option<&GameFonts>,  
) {
    parent.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Px(80.0),
            flex_shrink: 0.0,
            justify_content: JustifyContent::SpaceBetween,
            align_items: AlignItems::Center,
            padding: UiRect::all(Val::Px(20.0)),
            ..default()
        },
        BackgroundColor(Color::srgb(0.15, 0.15, 0.25)),
    )).with_children(|header| {
        // Title with custom font
        if let Some(fonts) = fonts {
            let (text, font, color) = create_text_with_font(
                "SUBVERSIVE",
                fonts.main_font.clone(), 
                32.0,
                Color::WHITE,
            );
            header.spawn((text, font, color));
        } else {
            // Fallback
            header.spawn((
                Text::new("SUBVERSIVE"),
                TextFont { font_size: 32.0, ..default() },
                TextColor(Color::WHITE),
            ));
        }
        
        header.spawn(Node {
            flex_direction: FlexDirection::Row,
            column_gap: Val::Px(30.0),
            ..default()
        }).with_children(|info| {
            // Day counter with custom font
            if let Some(fonts) = fonts {
                let (day_text, day_font, day_color) = create_text_with_font(
                    &format!("Day {}", global_data.current_day),
                    fonts.ui_font.clone(),
                    18.0,
                    Color::WHITE,
                );
                info.spawn((day_text, day_font, day_color));
                
                let (credits_text, credits_font, credits_color) = create_text_with_font(
                    &format!("Credits: {}", global_data.credits),
                    fonts.ui_font.clone(),
                    18.0,
                    Color::srgb(0.8, 0.8, 0.2),
                );
                info.spawn((credits_text, credits_font, credits_color));
            } else {
                // Fallback
                info.spawn((
                    Text::new(format!("Day {}", global_data.current_day)),
                    TextFont { font_size: 18.0, ..default() },
                    TextColor(Color::WHITE),
                ));
                info.spawn((
                    Text::new(format!("Credits: {}", global_data.credits)),
                    TextFont { font_size: 18.0, ..default() },
                    TextColor(Color::srgb(0.8, 0.8, 0.2)),
                ));
            }
        });
    });
}

fn create_tab_bar(
    parent: &mut ChildSpawnerCommands, 
    active_tab: HubTab,
    fonts: Option<&GameFonts>,
) {
    parent.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Px(50.0),
            flex_shrink: 0.0,
            flex_direction: FlexDirection::Row,
            ..default()
        },
        BackgroundColor(Color::srgb(0.08, 0.08, 0.15)),
    )).with_children(|tabs| {
        let tab_configs = [
            (HubTab::GlobalMap, "GLOBAL MAP"),
            (HubTab::Research, "RESEARCH"),
            (HubTab::Agents, "AGENTS"),
            (HubTab::Manufacture, "MANUFACTURE"),
            (HubTab::Missions, "MISSIONS"),
        ];
        
        // Q
        tabs.spawn((
            Node {
                width: Val::Percent(5.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgb(0.12, 0.12, 0.2)),
        )).with_children(|tab_button| {
            tab_button.spawn((
                Text::new("Q"),
                TextFont { font_size: 14.0, ..default() },
                TextColor(Color::WHITE),
            ));
        });

        // MAIN TABS
        for (tab, title) in tab_configs {
            let is_active = tab == active_tab;
            let bg_color = if is_active { Color::srgb(0.2, 0.6, 0.8) } else { Color::srgb(0.12, 0.12, 0.2) };
            let text_color = if is_active { Color::WHITE } else { Color::srgb(0.7, 0.7, 0.7) };
            tabs.spawn((
                Node {
                    width: Val::Percent(18.0),
                    height: Val::Percent(100.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                BackgroundColor(bg_color),
            )).with_children(|tab_button| {
                if let Some(fonts) = fonts {
                    let (text, font, color) = create_text_with_font(
                        title,
                        fonts.ui_font.clone(),  
                        14.0,
                        text_color,
                    );
                    tab_button.spawn((text, font, color));
                } else {
                    tab_button.spawn((
                        Text::new(title),
                        TextFont { font_size: 14.0, ..default() },
                        TextColor(text_color),
                    ));
                }
            });
        }
        // E
        tabs.spawn((
            Node {
                width: Val::Percent(5.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgb(0.12, 0.12, 0.2)),
        )).with_children(|tab_button| {
            tab_button.spawn((
                Text::new("E"),
                TextFont { font_size: 14.0, ..default() },
                TextColor(Color::WHITE),
            ));
        });        
    });
}

fn create_footer(
    parent: &mut ChildSpawnerCommands, 
    active_tab: HubTab,
    fonts: Option<&GameFonts>
) {
    parent.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Px(60.0),
            flex_shrink: 0.0,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            padding: UiRect::all(Val::Px(10.0)),
            ..default()
        },
        BackgroundColor(Color::srgb(0.08, 0.08, 0.15)),
    )).with_children(|footer| {
        let controls = match active_tab {
            HubTab::GlobalMap => "UP/DOWN: Select | W: Wait Day | ENTER: Mission | Q/E: Switch Tabs",
            HubTab::Research => "Navigation: Arrow Keys | Purchase: ENTER | Q/E: Switch Tabs",
            HubTab::Agents => "←→: Agent | 1-3: View | ↑↓: Navigate | ENTER: Install | Q/E: Switch Tabs",
            HubTab::Manufacture => "1-3: Agent | ↑↓: Slots | ←→: Attachments | ENTER: Attach/Detach | Q/E: Switch Tabs",
            HubTab::Missions => "Launch: ENTER | Q/E: Switch Tabs | ESC: Quit",
        };
        
        footer.spawn((
            Text::new(controls),
            TextFont { font_size: 14.0, ..default() },
            TextColor(Color::srgb(0.7, 0.7, 0.7)),
        ));
    });
}