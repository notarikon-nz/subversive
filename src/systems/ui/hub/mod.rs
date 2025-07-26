// src/systems/ui/hub/mod.rs - Simplified and consolidated
use bevy::prelude::*;
use bevy::ecs::system::SystemParam;
use bevy::input::mouse::*;
use crate::core::*;
use crate::systems::ui::builder::*;
use crate::systems::ui::hub::global_map::{InteractiveCity};
use serde::{Deserialize, Serialize};
use crate::systems::ui::hub::missions::{ScrollContainer, ScrollableContent, ScrollbarThumb};

pub mod agents;
pub mod research;
pub mod manufacture;
pub mod missions;
pub mod global_map;

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
        std::fs::read_to_string("data/cybernetics.json")
            .ok()
            .and_then(|content| serde_json::from_str(&content).ok())
            .unwrap_or_else(|| Self { cybernetics: Vec::new() })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum HubTab {
    #[default] GlobalMap, Research, Agents, Manufacture, Missions,
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
            research_db: ResearchDatabase::load(),
            cybernetics_db: CyberneticsDatabase::load(),
            attachment_db: AttachmentDatabase::load(),
            cities_db: CitiesDatabase::load(),
        }
    }
}

// Needs to merge with HubState
#[derive(Resource)]
pub struct HubStates {
    pub hub_state: HubState,
    pub manufacture_state: ManufactureState,
    pub agent_state: agents::AgentManagementState,
    pub map_state: global_map::GlobalMapState,
    pub mission_scroll: f32,  
    pub mission_max_scroll: f32,
}

impl Default for HubStates {
    fn default() -> Self {
        Self {
            hub_state: HubState::default(),
            agent_state: agents::AgentManagementState::default(),
            map_state: global_map::GlobalMapState::default(),
            manufacture_state: ManufactureState::default(),
            mission_scroll: 0.0,
            mission_max_scroll: 500.0,
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

// 
#[derive(Component)]
pub struct TabButton {
    pub tab: HubTab,
}

pub fn reset_hub_to_global_map(mut hub_state: ResMut<HubState>) {
    hub_state.active_tab = HubTab::GlobalMap;
}

// Bypass 16 parameter Bevy limit
#[derive(SystemParam)]
pub struct HubSystemQueries<'w, 's> {
    pub windows: Query<'w, 's, &'static Window>,
    pub cameras: Query<'w, 's, (&'static Camera, &'static GlobalTransform)>,
    pub city_query: Query<'w, 's, (Entity, &'static Transform, &'static InteractiveCity)>,
    pub agent_query: Query<'w, 's, &'static mut Inventory, With<Agent>>,
}

pub fn hub_system(
    mut commands: Commands,
    mut next_state: ResMut<NextState<GameState>>,
    mut global_data: ResMut<GlobalData>,
    mut hub_states: ResMut<HubStates>,
    mut hub_progress: ResMut<HubProgress>,
    hub_databases: Res<HubDatabases>,
    input: Res<ButtonInput<KeyCode>>,
    screen_query: Query<Entity, With<HubScreen>>,
    // agent_query: Query<&mut Inventory, With<Agent>>,
    fonts: Option<Res<GameFonts>>,
    // windows: Query<&Window>,
    // cameras: Query<(&Camera, &GlobalTransform)>,
    mouse: Res<ButtonInput<MouseButton>>,
    // city_query: Query<(Entity, &Transform, &global_map::InteractiveCity)>,
    mut queries: HubSystemQueries,  // This replaces 4 parameters!
    tab_button_query: Query<(&Interaction, &TabButton)>,

    mut scroll_events: EventReader<MouseWheel>,
    mut scroll_params: ParamSet<(
        Query<(Entity, &mut ScrollContainer, &GlobalTransform)>,
        Query<&mut Node, With<ScrollableContent>>,
        Query<&mut Node, With<ScrollbarThumb>>,
    )>,
    mut interaction_query: Query<(&Interaction, &TabButton), (Changed<Interaction>, With<Button>)>,
) {
    let mut tab_changed = false;
    
    if input.just_pressed(KeyCode::KeyQ) {
        hub_states.hub_state.active_tab = hub_states.hub_state.active_tab.previous();
        tab_changed = true;
    }
    
    if input.just_pressed(KeyCode::KeyE) {
        hub_states.hub_state.active_tab = hub_states.hub_state.active_tab.next();
        tab_changed = true;
    }
    
    for (interaction, tab_button) in interaction_query.iter() {
        match interaction {
            Interaction::Pressed => {
                hub_states.hub_state.active_tab = tab_button.tab;
                tab_changed = true;
                info!("Tab clicked: {:?}", tab_button.tab);
            }
            Interaction::Hovered => {
                tab_changed = true;
                // Could add hover effects here if desired
            }
            Interaction::None => {}
        }
    }

    if tab_changed {
        rebuild_hub(&mut commands, &screen_query, &global_data, &hub_states, 
                   &hub_progress, &hub_databases, fonts.as_deref());
    }

    let active_tab = hub_states.hub_state.active_tab;

    // Destructure to get separate mutable references
    let HubStates {
        ref mut hub_state,
        ref mut manufacture_state,
        ref mut agent_state,
        ref mut map_state,
        ref mut mission_scroll,
        ref mut mission_max_scroll,
    } = &mut *hub_states;

    let HubProgress {
        ref mut research_progress,
        ref mut cities_progress,
        ref mut unlocked,
    } = &mut *hub_progress;

    let needs_rebuild = match active_tab {
        HubTab::GlobalMap => global_map::handle_input(
            &input, &mut global_data, hub_state,
            &hub_databases.cities_db, 
            map_state, &queries.windows, &queries.cameras, &mouse, &queries.city_query,
        ),
        HubTab::Research => research::handle_input(
            &input, &mut global_data, research_progress,
            &hub_databases.research_db, unlocked,
            &mut hub_state.selected_research_project
        ),
        HubTab::Agents => agents::handle_input(
            &input, hub_state, agent_state,
            &mut global_data, &hub_databases.cybernetics_db.cybernetics
        ),
        HubTab::Manufacture => manufacture::handle_input(
            &input, hub_state, manufacture_state,
            &mut global_data, queries.agent_query, &hub_databases.attachment_db
        ),
        HubTab::Missions => missions::handle_input(
            &input, &mut commands, &mut next_state,
            &global_data, cities_progress, scroll_events,
            &queries.windows, &queries.cameras,
            scroll_params, &mut hub_states,
        ),    
    };

    // NEED CONFIRMATION WINDOW INSTEAD OF HARD EXIT
    if input.just_pressed(KeyCode::Escape) {
        std::process::exit(0);
    }

    if screen_query.is_empty() || needs_rebuild {
        rebuild_hub(&mut commands, &screen_query, &global_data, &hub_states, 
                   &hub_progress, &hub_databases, fonts.as_deref());
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
        commands.entity(entity).insert(MarkedForDespawn);
    }
    create_hub_ui(commands, global_data, hub_states, hub_progress, hub_databases, fonts);
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

        // create_header(parent, global_data, fonts);
        // create_tab_bar(parent, hub_states.hub_state.active_tab, fonts);
        
        create_merged_header(parent, global_data, hub_states.hub_state.active_tab, &hub_databases.cities_db);

            match hub_states.hub_state.active_tab {
                HubTab::GlobalMap => global_map::create_content(
                    parent, global_data, &hub_states.hub_state, 
                    &hub_databases.cities_db, &hub_progress.cities_progress, 
                    &mut hub_states.map_state.clone()
                ),
                HubTab::Research => research::create_content(
                    parent, global_data, &hub_progress.research_progress, 
                    &hub_databases.research_db, hub_states.hub_state.selected_research_project
                ),
                HubTab::Agents => agents::create_content(
                    parent, global_data, &hub_states.agent_state,
                    &hub_databases.cybernetics_db.cybernetics,
                ),
                HubTab::Manufacture => manufacture::create_content(
                    parent, global_data, &hub_states.manufacture_state, 
                    &hub_databases.attachment_db, &hub_progress.unlocked
                ),
                HubTab::Missions => missions::create_content(
                    parent, global_data, &hub_databases.cities_db, &hub_progress.cities_progress, &hub_states,
                ),
            }

        create_footer(parent, hub_states.hub_state.active_tab); 
    });
}

fn create_merged_header(parent: &mut ChildSpawnerCommands, global_data: &GlobalData, active_tab: HubTab, cities_db: &CitiesDatabase) {
    parent.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Px(60.0),
            flex_shrink: 0.0,
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            padding: UiRect::all(Val::Px(15.0)),
            ..default()
        },
        BackgroundColor(Color::srgb(0.12, 0.12, 0.22)),

    )).with_children(|header| {
        
        // Left 25% - Progress section
        header.spawn((
            Node {
                width: Val::Percent(20.0),
                justify_content: JustifyContent::FlexStart,
                align_items: AlignItems::Center,
                ..default()
            },
        )).with_children(|progress| {
            let accessible_cities = cities_db.get_accessible_cities(&global_data).len();
            let total_cities = cities_db.get_all_cities().len();
            
            progress.spawn(UIBuilder::text(
                &format!("Cities: {}/{}", accessible_cities, total_cities), 
                18.0, 
                Color::srgb(0.8, 0.8, 0.2)
            ));
        });
        


        // Center 50% - Tab bar
        header.spawn((
            Node {
                width: Val::Percent(60.0),
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::Center,    //::SpaceEvenly
                align_items: AlignItems::Center,
                ..default()
            },
        )).with_children(|tabs| {
            
            // Q navigation
            tabs.spawn((
                Node {
                    width: Val::Px(30.0),
                    height: Val::Px(30.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    margin: UiRect::right(Val::Px(10.0)),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.15, 0.15, 0.25)),
            )).with_children(|nav| {
                nav.spawn(UIBuilder::text("Q", 12.0, Color::srgb(0.6, 0.6, 0.6)));
            });
            
            // Main tabs
            let tabs_config = [
                (HubTab::GlobalMap, "Map", "TAB"),
                (HubTab::Research, "Research", "TAB"),
                (HubTab::Agents, "Agents", "TAB"),
                (HubTab::Manufacture, "Gear", "TAB"),
                (HubTab::Missions, "Mission", "TAB"),
            ];
            
            for (tab, title, _key) in tabs_config {
                let is_active = tab == active_tab;

                let bg_color = if is_active { 
                    Color::srgb(0.2, 0.6, 0.8) 
                } else { 
                    Color::srgb(0.18, 0.18, 0.28) 
                };
                let text_color = if is_active { 
                    Color::WHITE 
                } else { 
                    Color::srgb(0.7, 0.7, 0.7) 
                };
                let TAB_FONT_SIZE = if is_active { 
                    12.0
                } else { 
                    12.0
                };                
                
                tabs.spawn((
                    Button, 
                    Node {
                        padding: UiRect::all(Val::Px(15.0)),
                        //width: Val::Px(70.0),   
                        //height: Val::Px(30.0),
                        //justify_content: JustifyContent::Center,
                        //align_items: AlignItems::Center,
                        //margin: UiRect::horizontal(Val::Px(2.0)),
                        ..default()
                    },
                    BackgroundColor(bg_color),
                    TabButton {tab},
                )).with_children(|tab_button| {
                    tab_button.spawn((
                        Text::new(title),
                        TextFont { font_size: TAB_FONT_SIZE, ..default() },
                        TextColor(if is_active {
                            Color::srgb(0.8, 0.8, 0.2)
                        } else {
                            Color::WHITE
                        }),
                    ));
                });
            }
            
            // E navigation
            tabs.spawn((
                Node {
                    width: Val::Px(30.0),
                    height: Val::Px(30.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    margin: UiRect::left(Val::Px(10.0)),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.15, 0.15, 0.25)),
            )).with_children(|nav| {
                nav.spawn(UIBuilder::text("E", 12.0, Color::srgb(0.6, 0.6, 0.6)));
            });
        });
        
        // Right 25% - Day and Credits
        header.spawn((
            Node {
                width: Val::Percent(20.0),
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::FlexEnd,
                align_items: AlignItems::Center,
                column_gap: Val::Px(20.0),
                ..default()
            },
        )).with_children(|info| {
            info.spawn(UIBuilder::text(&format!("Day {}", global_data.current_day), 18.0, Color::WHITE));
            info.spawn(UIBuilder::text(&format!("${}", global_data.credits), 18.0, Color::srgb(0.8, 0.8, 0.2)));
        });
    });
}

fn create_footer(parent: &mut ChildSpawnerCommands, active_tab: HubTab) {
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
            HubTab::GlobalMap => "Click Cities | W: Wait Day | ENTER: Mission | Q/E: Tabs",
            HubTab::Research => "↑↓: Navigate | ENTER: Purchase | Q/E: Tabs",
            HubTab::Agents => "←→: Agent | 1-3: View | ↑↓: Navigate | ENTER: Install | Q/E: Tabs",
            HubTab::Manufacture => "1-3: Agent | ↑↓: Slots | ←→: Attachments | ENTER: Modify | Q/E: Tabs",
            HubTab::Missions => "ENTER: Launch | Q/E: Tabs | ESC: Quit",
        };
        
        footer.spawn(UIBuilder::nav_controls(controls));
    });
}