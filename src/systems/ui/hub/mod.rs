// src/systems/ui/hub/mod.rs - Simplified and consolidated
use bevy::prelude::*;
use crate::core::*;
use crate::systems::ui::builder::*;
use serde::{Deserialize, Serialize};

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

#[derive(Resource)]
pub struct HubStates {
    pub hub_state: HubState,
    pub manufacture_state: ManufactureState,
    pub agent_state: agents::AgentManagementState,
    pub map_state: global_map::GlobalMapState,
}

impl Default for HubStates {
    fn default() -> Self {
        Self {
            hub_state: HubState::default(),
            agent_state: agents::AgentManagementState::default(),
            map_state: global_map::GlobalMapState::default(),
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

pub fn reset_hub_to_global_map(mut hub_state: ResMut<HubState>) {
    hub_state.active_tab = HubTab::GlobalMap;
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
    agent_query: Query<&mut Inventory, With<Agent>>,
    fonts: Option<Res<GameFonts>>,
    windows: Query<&Window>,
    cameras: Query<(&Camera, &GlobalTransform)>,
    mouse: Res<ButtonInput<MouseButton>>,
    city_query: Query<(Entity, &Transform, &global_map::InteractiveCity)>,
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
        map_state, &windows, &cameras, &mouse, &city_query,
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
        &mut global_data, agent_query, &hub_databases.attachment_db
    ),
    HubTab::Missions => missions::handle_input(
        &input, &mut commands, &mut next_state, 
        &global_data, cities_progress,
    ),
};

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
        create_header(parent, global_data, fonts);
        create_tab_bar(parent, hub_states.hub_state.active_tab, fonts);
        
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
                parent, global_data, &hub_databases.cities_db, &hub_progress.cities_progress,
            ),
        }
        
        create_footer(parent, hub_states.hub_state.active_tab); 
    });
}

fn create_header(parent: &mut ChildSpawnerCommands, global_data: &GlobalData, fonts: Option<&GameFonts>) {
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
        header.spawn(UIBuilder::text("SUBVERSIVE", 32.0, Color::WHITE));
        
        header.spawn(UIBuilder::row(30.0)).with_children(|info| {
            info.spawn(UIBuilder::text(&format!("Day {}", global_data.current_day), 18.0, Color::WHITE));
            info.spawn(UIBuilder::text(&UIBuilder::credits_display(global_data.credits), 18.0, Color::srgb(0.8, 0.8, 0.2)));
        });
    });
}

fn create_tab_bar(parent: &mut ChildSpawnerCommands, active_tab: HubTab, fonts: Option<&GameFonts>) {
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
        // Q/E navigation indicators
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
            tab_button.spawn(UIBuilder::text("Q", 14.0, Color::WHITE));
        });

        let tabs_config = [
            (HubTab::GlobalMap, "GLOBAL MAP"),
            (HubTab::Research, "RESEARCH"),
            (HubTab::Agents, "AGENTS"),
            (HubTab::Manufacture, "MANUFACTURE"),
            (HubTab::Missions, "MISSIONS"),
        ];

        for (tab, title) in tabs_config {
            let (node, bg, text) = UIBuilder::tab_button(title, tab == active_tab);
            tabs.spawn((node, bg)).with_children(|tab_button| {
                tab_button.spawn(text);
            });
        }

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
            tab_button.spawn(UIBuilder::text("E", 14.0, Color::WHITE));
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