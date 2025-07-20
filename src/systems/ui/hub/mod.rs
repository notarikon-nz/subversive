// src/systems/ui/hub/mod.rs - Updated with agent state integration
use bevy::prelude::*;
use crate::core::*;
use serde::{Deserialize, Serialize};

// Re-export all tab modules
pub mod global_map;
pub mod research;
pub mod agents;
pub mod manufacture;
pub mod missions;

pub use agents::*;

#[derive(Component)]
pub struct HubScreen;

#[derive(Resource, Default)]
pub struct HubState {
    pub active_tab: HubTab,
    pub selected_region: usize,
    pub selected_research_project: usize,
}

#[derive(Resource, Serialize, Deserialize)]
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

// Main hub coordination system
pub fn hub_system(
    mut commands: Commands,
    mut next_state: ResMut<NextState<GameState>>,
    mut global_data: ResMut<GlobalData>,
    mut hub_state: ResMut<HubState>,
    mut manufacture_state: ResMut<ManufactureState>,
    mut agent_state: ResMut<AgentManagementState>,
    mut research_progress: ResMut<ResearchProgress>,
    research_db: Res<ResearchDatabase>,
    cybernetics_db: Res<CyberneticsDatabase>,
    mut unlocked: ResMut<UnlockedAttachments>,
    input: Res<ButtonInput<KeyCode>>,
    screen_query: Query<Entity, With<HubScreen>>,
    attachment_db: Res<AttachmentDatabase>,
    agent_query: Query<&mut Inventory, With<Agent>>,
    fonts: Option<Res<GameFonts>>,  // ADD THIS
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
        let fonts_ref = fonts.as_ref().map(|f| f.as_ref());
        rebuild_hub(
            &mut commands, 
            &screen_query, 
            &global_data, 
            &hub_state, 
            &manufacture_state, 
            &agent_state, 
            &research_progress, 
            &research_db, 
            &attachment_db, 
            &unlocked, 
            &cybernetics_db, 
            fonts_ref);
    }

    // Delegate input handling to appropriate tab
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
            &mut hub_state,
            &mut agent_state,
            &mut global_data,
            &cybernetics_db.cybernetics
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

    if screen_query.is_empty() || needs_rebuild {
        let fonts_ref = fonts.as_ref().map(|f| f.as_ref());
        rebuild_hub(&mut commands, &screen_query, &global_data, &hub_state, 
                   &manufacture_state, &agent_state, &research_progress, 
                   &research_db, &attachment_db, &unlocked, &cybernetics_db,
                   fonts_ref);  
    }
}

fn rebuild_hub(
    commands: &mut Commands,
    screen_query: &Query<Entity, With<HubScreen>>,
    global_data: &GlobalData,
    hub_state: &HubState,
    manufacture_state: &ManufactureState,
    agent_state: &AgentManagementState,
    research_progress: &ResearchProgress,
    research_db: &ResearchDatabase,
    attachment_db: &AttachmentDatabase,
    unlocked: &UnlockedAttachments,
    cybernetics_db: &CyberneticsDatabase,
    fonts: Option<&GameFonts>,
) {
    for entity in screen_query.iter() {
        commands.safe_despawn(entity);
    }
    create_hub_ui(commands, global_data, hub_state, manufacture_state, agent_state,
        research_progress, research_db, attachment_db, unlocked, cybernetics_db, fonts);
}

fn create_hub_ui(
    commands: &mut Commands, 
    global_data: &GlobalData, 
    hub_state: &HubState,
    manufacture_state: &ManufactureState,
    agent_state: &AgentManagementState,
    research_progress: &ResearchProgress,
    research_db: &ResearchDatabase,
    attachment_db: &AttachmentDatabase,
    unlocked: &UnlockedAttachments,
    cybernetics_db: &CyberneticsDatabase,
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
        create_tab_bar(parent, hub_state.active_tab, fonts);
        
        match hub_state.active_tab {
            HubTab::GlobalMap => global_map::create_content(parent, global_data, hub_state),
            HubTab::Research => research::create_content(
                parent, 
                global_data, 
                &research_progress, 
                &research_db,
                hub_state.selected_research_project
            ),
            HubTab::Agents => agents::create_content(
                parent, 
                global_data, 
                agent_state,
                &cybernetics_db.cybernetics,
                // fonts  // You can add this parameter later
            ),
            HubTab::Manufacture => manufacture::create_content(parent, global_data, manufacture_state, attachment_db, unlocked),
            HubTab::Missions => missions::create_content(parent, global_data, hub_state),
        }
        
        create_footer(parent, hub_state.active_tab, fonts); 
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
        
        for (tab, title) in tab_configs {
            let is_active = tab == active_tab;
            let bg_color = if is_active {
                Color::srgb(0.2, 0.6, 0.8)
            } else {
                Color::srgb(0.12, 0.12, 0.2)
            };
            let text_color = if is_active { 
                Color::WHITE 
            } else { 
                Color::srgb(0.7, 0.7, 0.7) 
            };
            
            tabs.spawn((
                Node {
                    width: Val::Percent(20.0),
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