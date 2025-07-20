// src/systems/ui/hub/mod.rs - Main hub coordination
use bevy::prelude::*;
use crate::core::*;

// Re-export all tab modules
pub mod global_map;
pub mod research;
pub mod agents;
pub mod manufacture;
pub mod missions;

pub use global_map::*;
pub use research::*;
pub use agents::*;
pub use manufacture::*;
pub use missions::*;

#[derive(Component)]
pub struct HubScreen;

#[derive(Resource, Default)]
pub struct HubState {
    pub active_tab: HubTab,
    pub selected_region: usize,
    pub selected_research_project: usize,
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

// Main hub coordination system
pub fn hub_system(
    mut commands: Commands,
    mut next_state: ResMut<NextState<GameState>>,
    mut global_data: ResMut<GlobalData>,
    mut hub_state: ResMut<HubState>,
    mut manufacture_state: ResMut<ManufactureState>,
    mut research_progress: ResMut<ResearchProgress>,  // ADD
    research_db: Res<ResearchDatabase>,               // ADD
    mut unlocked: ResMut<UnlockedAttachments>, // ADD
    input: Res<ButtonInput<KeyCode>>,
    screen_query: Query<Entity, With<HubScreen>>,
    attachment_db: Res<AttachmentDatabase>,
    agent_query: Query<&mut Inventory, With<Agent>>,
) {
    // Global tab switching with Q/E
    let mut tab_changed = false;
    
    if input.just_pressed(KeyCode::KeyQ) {
        hub_state.active_tab = hub_state.active_tab.previous();
        tab_changed = true;
    }
    
    if input.just_pressed(KeyCode::KeyE) {
        hub_state.active_tab = hub_state.active_tab.next();
        tab_changed = true;
    }
    
    if tab_changed {
        rebuild_hub(&mut commands, &screen_query, &global_data, &hub_state, &manufacture_state, &research_progress, &research_db, &attachment_db, &unlocked);
    }

    // Delegate input handling to appropriate tab
    // Spaced out for easier debugging
    let mut needs_rebuild = match hub_state.active_tab {
        HubTab::GlobalMap => global_map::handle_input(
            &input, 
            &mut global_data, 
            &mut hub_state
        ),
        HubTab::Research => research::handle_input(
            &input, 
            &mut global_data, 
            &mut research_progress, 
            &research_db, 
            &mut unlocked,
            &mut hub_state.selected_research_project
        ),
        HubTab::Agents => agents::handle_input(
            &input, 
            &mut hub_state
        ),
        HubTab::Manufacture => manufacture::handle_input(
            &input, 
            &mut hub_state, 
            &mut manufacture_state, 
            &mut global_data, 
            agent_query, 
            &attachment_db
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

    // Create/rebuild UI if needed
    if screen_query.is_empty() || needs_rebuild {
        rebuild_hub(&mut commands, &screen_query, &global_data, &hub_state, &manufacture_state, &research_progress, &research_db, &attachment_db, &unlocked);
    }
}

fn rebuild_hub(
    commands: &mut Commands,
    screen_query: &Query<Entity, With<HubScreen>>,
    global_data: &GlobalData,
    hub_state: &HubState,
    manufacture_state: &ManufactureState,
    research_progress: &ResearchProgress,  // ADD
    research_db: &ResearchDatabase,        // ADD    
    attachment_db: &AttachmentDatabase,
    unlocked: &UnlockedAttachments,
) {
    for entity in screen_query.iter() {
        commands.entity(entity).despawn_recursive();
    }
    create_hub_ui(commands, global_data, hub_state, manufacture_state, 
        research_progress,
        research_db,
        attachment_db, unlocked);
}

fn create_hub_ui(
    commands: &mut Commands, 
    global_data: &GlobalData, 
    hub_state: &HubState,
    manufacture_state: &ManufactureState,
    research_progress: &ResearchProgress,
    research_db: &ResearchDatabase,
    attachment_db: &AttachmentDatabase,
    unlocked: &UnlockedAttachments,
) {
    commands.spawn((
        NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                ..default()
            },
            background_color: Color::srgb(0.1, 0.1, 0.2).into(),
            ..default()
        },
        HubScreen,
    )).with_children(|parent| {
        // Fixed header
        create_header(parent, global_data);
        
        // Fixed tab bar
        create_tab_bar(parent, hub_state.active_tab);
        
        // Content area that scrolls
        match hub_state.active_tab {
            HubTab::GlobalMap => global_map::create_content(parent, global_data, hub_state),
            HubTab::Research => research::create_content(
                parent, 
                global_data, 
                &research_progress, 
                &research_db,
                hub_state.selected_research_project
            ),
            HubTab::Agents => agents::create_content(parent, global_data),
            HubTab::Manufacture => manufacture::create_content(parent, global_data, manufacture_state, attachment_db, unlocked),
            HubTab::Missions => missions::create_content(parent, global_data, hub_state),
        }
        
        // Fixed footer
        create_footer(parent, hub_state.active_tab);
    });
}

fn create_header(parent: &mut ChildSpawnerCommands, global_data: &GlobalData) {
    parent.spawn(NodeBundle {
        style: Style {
            width: Val::Percent(100.0),
            height: Val::Px(80.0),
            flex_shrink: 0.0, // Don't allow shrinking
            justify_content: JustifyContent::SpaceBetween,
            align_items: AlignItems::Center,
            padding: UiRect::all(Val::Px(20.0)),
            ..default()
        },
        background_color: Color::srgb(0.15, 0.15, 0.25).into(),
        ..default()
    }).with_children(|header| {
        header.spawn(TextBundle::from_section(
            "SUBVERSIVE",
            TextStyle { font_size: 32.0, color: Color::WHITE, ..default() }
        ));
        
        header.spawn(NodeBundle {
            style: Style {
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(30.0),
                ..default()
            },
            ..default()
        }).with_children(|info| {
            info.spawn(TextBundle::from_section(
                format!("Day {}", global_data.current_day),
                TextStyle { font_size: 18.0, color: Color::WHITE, ..default() }
            ));
            info.spawn(TextBundle::from_section(
                format!("Credits: {}", global_data.credits),
                TextStyle { font_size: 18.0, color: Color::srgb(0.8, 0.8, 0.2), ..default() }
            ));
        });
    });
}

fn create_tab_bar(parent: &mut ChildSpawnerCommands, active_tab: HubTab) {
    parent.spawn(NodeBundle {
        style: Style {
            width: Val::Percent(100.0),
            height: Val::Px(50.0),
            flex_shrink: 0.0, // Don't allow shrinking
            flex_direction: FlexDirection::Row,
            ..default()
        },
        background_color: Color::srgb(0.08, 0.08, 0.15).into(),
        ..default()
    }).with_children(|tabs| {
        let tab_configs = [
            (HubTab::GlobalMap, "GLOBAL MAP"),
            (HubTab::Research, "RESEARCH"),
            (HubTab::Agents, "AGENTS"),
            (HubTab::Manufacture, "MANUFACTURE"),
            (HubTab::Missions, "MISSIONS"),
        ];
        
        for (tab, title) in tab_configs {
            let is_active = tab == active_tab;
            let bg_color = if is_active {
                Color::srgb(0.2, 0.6, 0.8)
            } else {
                Color::srgb(0.12, 0.12, 0.2)
            };
            let text_color = if is_active { Color::WHITE } else { Color::srgb(0.7, 0.7, 0.7) };
            
            tabs.spawn(NodeBundle {
                style: Style {
                    width: Val::Percent(20.0),
                    height: Val::Percent(100.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                background_color: bg_color.into(),
                ..default()
            }).with_children(|tab_button| {
                tab_button.spawn(TextBundle::from_section(
                    title,
                    TextStyle { font_size: 14.0, color: text_color, ..default() }
                ));
            });
        }
    });
}

fn create_footer(parent: &mut ChildSpawnerCommands, active_tab: HubTab) {
    parent.spawn(NodeBundle {
        style: Style {
            width: Val::Percent(100.0),
            height: Val::Px(60.0),
            flex_shrink: 0.0, // Don't allow shrinking
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            padding: UiRect::all(Val::Px(10.0)),
            ..default()
        },
        background_color: Color::srgb(0.08, 0.08, 0.15).into(),
        ..default()
    }).with_children(|footer| {
        let controls = match active_tab {
            HubTab::GlobalMap => "UP/DOWN: Select | W: Wait Day | ENTER: Mission | Q/E: Switch Tabs",
            HubTab::Research => "Navigation: Arrow Keys | Purchase: ENTER | Q/E: Switch Tabs",
            HubTab::Agents => "Select: Arrow Keys | Modify: ENTER | Q/E: Switch Tabs",
            HubTab::Manufacture => "1-3: Agent | ↑↓: Slots | ←→: Attachments | ENTER: Attach/Detach | Q/E: Switch Tabs",
            HubTab::Missions => "Launch: ENTER | Q/E: Switch Tabs | ESC: Quit",
        };
        
        footer.spawn(TextBundle::from_section(
            controls,
            TextStyle { font_size: 14.0, color: Color::srgb(0.7, 0.7, 0.7), ..default() }
        ));
    });
}

