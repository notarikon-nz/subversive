// src/systems/ui/hub.rs - New unified hub screen replacing global map
use bevy::prelude::*;
use crate::core::*;

#[derive(Component)]
pub struct HubScreen;

#[derive(Resource, Default)]
pub struct HubState {
    pub active_tab: HubTab,
    pub selected_region: usize, // Carried between tabs
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum HubTab {
    #[default]
    GlobalMap,
    Research,
    Agents,
    Missions,
}

impl HubTab {
    pub fn from_keycode(key: KeyCode) -> Option<Self> {
        match key {
            KeyCode::Digit1 => Some(Self::GlobalMap),
            KeyCode::Digit2 => Some(Self::Research),
            KeyCode::Digit3 => Some(Self::Agents),
            KeyCode::Digit4 => Some(Self::Missions),
            _ => None,
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
    mut hub_state: ResMut<HubState>,
    input: Res<ButtonInput<KeyCode>>,
    screen_query: Query<Entity, With<HubScreen>>,
) {
    // Tab switching with number keys
    if let Some(tab) = input.get_just_pressed().find_map(|&key| HubTab::from_keycode(key)) {
        hub_state.active_tab = tab;
        rebuild_hub(&mut commands, &screen_query, &global_data, &hub_state);
    }

    // Handle tab-specific input
    match hub_state.active_tab {
        HubTab::GlobalMap => handle_global_map_input(&mut global_data, &mut hub_state, &input, &mut commands, &screen_query),
        HubTab::Research => handle_research_input(&input),
        HubTab::Agents => handle_agents_input(&input),
        HubTab::Missions => handle_missions_input(&input, &mut commands, &mut next_state, &global_data),
    }

    // Global controls
    if input.just_pressed(KeyCode::Escape) {
        std::process::exit(0);
    }

    // Create UI if it doesn't exist
    if screen_query.is_empty() {
        create_hub_ui(&mut commands, &global_data, &hub_state);
    }
}


fn handle_global_map_input(
    global_data: &mut GlobalData,
    hub_state: &mut HubState,
    input: &ButtonInput<KeyCode>,
    commands: &mut Commands,
    screen_query: &Query<Entity, With<HubScreen>>,
) {
    let mut needs_rebuild = false;

    // Region selection
    if input.just_pressed(KeyCode::ArrowUp) && hub_state.selected_region > 0 {
        hub_state.selected_region -= 1;
        global_data.selected_region = hub_state.selected_region;
        needs_rebuild = true;
    }
    if input.just_pressed(KeyCode::ArrowDown) && hub_state.selected_region < global_data.regions.len() - 1 {
        hub_state.selected_region += 1;
        global_data.selected_region = hub_state.selected_region;
        needs_rebuild = true;
    }

    // Wait a day
    if input.just_pressed(KeyCode::KeyW) {
        global_data.current_day += 1;
        let current_day = global_data.current_day;
        for region in &mut global_data.regions {
            region.update_alert(current_day);
        }
        needs_rebuild = true;
        info!("Waited 1 day. Current day: {}", current_day);
    }

    // Click region to go to missions tab
    if input.just_pressed(KeyCode::Enter) {
        hub_state.active_tab = HubTab::Missions;
        needs_rebuild = true;
        info!("Switching to Missions tab for region: {}", global_data.regions[hub_state.selected_region].name);
    }

    if needs_rebuild {
        rebuild_hub(commands, screen_query, global_data, hub_state);
    }
}

fn handle_research_input(_input: &ButtonInput<KeyCode>) {
    // TODO: Implement research tree navigation
    // - Arrow keys to navigate tech tree
    // - Enter to purchase research
    // - Research dependencies and progress tracking
}

fn handle_agents_input(_input: &ButtonInput<KeyCode>) {
    // TODO: Implement agent management
    // - Arrow keys to select agents
    // - Enter to modify equipment
    // - Save/Load squad presets
    // - Agent recovery status display
}

fn handle_missions_input(
    input: &ButtonInput<KeyCode>,
    commands: &mut Commands,
    next_state: &mut NextState<GameState>,
    global_data: &GlobalData,
) {
    // Launch mission
    if input.just_pressed(KeyCode::Enter) {
        let ready_agents = (0..3).filter(|&i| global_data.agent_recovery[i] <= global_data.current_day).count();
        if ready_agents > 0 {
            commands.insert_resource(ShouldRestart);
            next_state.set(GameState::Mission);
            info!("Launching mission in {} with {} agents", 
                  global_data.regions[global_data.selected_region].name, ready_agents);
        } else {
            info!("No agents ready for deployment!");
        }
    }
}

fn rebuild_hub(
    commands: &mut Commands,
    screen_query: &Query<Entity, With<HubScreen>>,
    global_data: &GlobalData,
    hub_state: &HubState,
) {
    for entity in screen_query.iter() {
        commands.entity(entity).despawn_recursive();
    }
    create_hub_ui(commands, global_data, hub_state);
}

fn create_hub_ui(commands: &mut Commands, global_data: &GlobalData, hub_state: &HubState) {
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
        // Header with title and universal info
        create_header(parent, global_data);
        
        // Tab bar
        create_tab_bar(parent, hub_state.active_tab);
        
        // Content area based on active tab
        create_tab_content(parent, hub_state.active_tab, global_data, hub_state);
        
        // Footer with controls
        create_footer(parent, hub_state.active_tab);
    });
}

fn create_header(parent: &mut ChildBuilder, global_data: &GlobalData) {
    parent.spawn(NodeBundle {
        style: Style {
            width: Val::Percent(100.0),
            height: Val::Px(80.0),
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

fn create_tab_bar(parent: &mut ChildBuilder, active_tab: HubTab) {
    parent.spawn(NodeBundle {
        style: Style {
            width: Val::Percent(100.0),
            height: Val::Px(50.0),
            flex_direction: FlexDirection::Row,
            ..default()
        },
        background_color: Color::srgb(0.08, 0.08, 0.15).into(),
        ..default()
    }).with_children(|tabs| {
        let tab_configs = [
            (HubTab::GlobalMap, "1. GLOBAL MAP", "World overview"),
            (HubTab::Research, "2. RESEARCH", "Tech development"),
            (HubTab::Agents, "3. AGENTS", "Squad management"),
            (HubTab::Missions, "4. MISSIONS", "Mission briefing"),
        ];
        
        for (tab, title, _description) in tab_configs {
            let is_active = tab == active_tab;
            let bg_color = if is_active {
                Color::srgb(0.2, 0.6, 0.8)
            } else {
                Color::srgb(0.12, 0.12, 0.2)
            };
            let text_color = if is_active { Color::WHITE } else { Color::srgb(0.7, 0.7, 0.7) };
            
            tabs.spawn(NodeBundle {
                style: Style {
                    width: Val::Percent(25.0),
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
                    TextStyle { font_size: 16.0, color: text_color, ..default() }
                ));
            });
        }
    });
}

fn create_tab_content(
    parent: &mut ChildBuilder, 
    active_tab: HubTab, 
    global_data: &GlobalData, 
    hub_state: &HubState
) {
    parent.spawn(NodeBundle {
        style: Style {
            width: Val::Percent(100.0),
            flex_grow: 1.0,
            padding: UiRect::all(Val::Px(20.0)),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(15.0),
            ..default()
        },
        ..default()
    }).with_children(|content| {
        match active_tab {
            HubTab::GlobalMap => create_global_map_content(content, global_data, hub_state),
            HubTab::Research => create_research_content(content, global_data),
            HubTab::Agents => create_agents_content(content, global_data),
            HubTab::Missions => create_missions_content(content, global_data, hub_state),
        }
    });
}

fn create_global_map_content(parent: &mut ChildBuilder, global_data: &GlobalData, hub_state: &HubState) {
    // Agent status overview
    parent.spawn(TextBundle::from_section(
        "AGENT STATUS:",
        TextStyle { font_size: 20.0, color: Color::WHITE, ..default() }
    ));
    
    for i in 0..3 {
        let level = global_data.agent_levels[i];
        let is_recovering = global_data.agent_recovery[i] > global_data.current_day;
        let recovery_days = if is_recovering { 
            global_data.agent_recovery[i] - global_data.current_day 
        } else { 0 };
        
        let color = if is_recovering { Color::srgb(0.5, 0.5, 0.5) } else { Color::srgb(0.2, 0.8, 0.2) };
        let status = if is_recovering {
            format!("Agent {}: Level {} - RECOVERING ({} days)", i + 1, level, recovery_days)
        } else {
            format!("Agent {}: Level {} - READY", i + 1, level)
        };
        
        parent.spawn(TextBundle::from_section(
            status,
            TextStyle { font_size: 16.0, color, ..default() }
        ));
    }
    
    // World regions
    parent.spawn(TextBundle::from_section(
        "\nWORLD REGIONS:",
        TextStyle { font_size: 20.0, color: Color::WHITE, ..default() }
    ));
    
    for (i, region) in global_data.regions.iter().enumerate() {
        let is_selected = i == hub_state.selected_region;
        let color = if is_selected { Color::srgb(0.2, 0.8, 0.2) } else { Color::WHITE };
        let prefix = if is_selected { "> " } else { "  " };
        
        parent.spawn(TextBundle::from_section(
            format!("{}{} (Threat: {}, Alert: {:?})", 
                    prefix, region.name, region.threat_level, region.alert_level),
            TextStyle { font_size: 18.0, color, ..default() }
        ));
    }
}

fn create_research_content(parent: &mut ChildBuilder, global_data: &GlobalData) {
    parent.spawn(TextBundle::from_section(
        "RESEARCH & DEVELOPMENT",
        TextStyle { font_size: 24.0, color: Color::srgb(0.8, 0.8, 0.2), ..default() }
    ));
    
    parent.spawn(TextBundle::from_section(
        "TODO: Implement research tree",
        TextStyle { font_size: 16.0, color: Color::srgb(0.6, 0.6, 0.6), ..default() }
    ));
    
    // TODO: Research tree implementation
    // - Visual tech tree with branching paths
    // - Weapon research: Pistol → Rifle → Minigun → Flamethrower
    // - Cybernetics research: Basic → Advanced → Experimental
    // - Tool research: Hacker → Scanner → Advanced tools
    // - Dependencies clearly shown with lines/arrows
    // - Progress indicators: "Next unlock in X missions"
    // - Cost in credits, research points, or mission requirements
    // - Unlocked items automatically available in Agent tab
    
    parent.spawn(TextBundle::from_section(
        format!("Available Credits: {}", global_data.credits),
        TextStyle { font_size: 16.0, color: Color::WHITE, ..default() }
    ));
}

fn create_agents_content(parent: &mut ChildBuilder, global_data: &GlobalData) {
    parent.spawn(TextBundle::from_section(
        "AGENT MANAGEMENT",
        TextStyle { font_size: 24.0, color: Color::srgb(0.2, 0.8, 0.2), ..default() }
    ));
    
    parent.spawn(TextBundle::from_section(
        "TODO: Implement squad management",
        TextStyle { font_size: 16.0, color: Color::srgb(0.6, 0.6, 0.6), ..default() }
    ));
    
    // TODO: Agent management implementation
    // - 3-agent squad display with individual stats
    // - Equipment assignment per agent (weapons, tools, cybernetics)
    // - Visual equipment slots with drag-drop or selection
    // - Agent progression: Level, Experience, Specializations
    // - Recovery timers and injury status
    // - Squad preset system: Save/Load configurations
    //   * "Stealth Squad" - silenced weapons, scanners, stealth mods
    //   * "Assault Team" - heavy weapons, armor, combat mods
    //   * "Tech Specialists" - hacking tools, advanced cybernetics
    // - Equipment availability based on research unlocks
    // - Agent customization: Names, appearance (if desired)
    
    for i in 0..3 {
        let level = global_data.agent_levels[i];
        let exp = global_data.agent_experience[i];
        let next_level_exp = experience_for_level(level + 1);
        
        parent.spawn(TextBundle::from_section(
            format!("Agent {}: Level {} ({}/{} XP)", i + 1, level, exp, next_level_exp),
            TextStyle { font_size: 16.0, color: Color::WHITE, ..default() }
        ));
    }
}

fn create_missions_content(parent: &mut ChildBuilder, global_data: &GlobalData, hub_state: &HubState) {
    let region = &global_data.regions[hub_state.selected_region];
    
    parent.spawn(TextBundle::from_section(
        format!("MISSION BRIEFING: {}", region.name),
        TextStyle { font_size: 24.0, color: Color::srgb(0.8, 0.2, 0.2), ..default() }
    ));
    
    // Mission intel
    parent.spawn(TextBundle::from_section(
        format!("Threat Level: {} | Alert Status: {:?}", region.threat_level, region.alert_level),
        TextStyle { font_size: 18.0, color: Color::WHITE, ..default() }
    ));
    
    parent.spawn(TextBundle::from_section(
        "TODO: Implement detailed mission briefing",
        TextStyle { font_size: 16.0, color: Color::srgb(0.6, 0.6, 0.6), ..default() }
    ));
    
    // TODO: Mission briefing implementation
    // - Threat intelligence based on region and alert level
    //   * "Expected enemies: 2-4 Corporate Guards"
    //   * "Equipment spotted: Pistols, Light Armor"
    //   * "Patrol patterns: Regular, 2-minute intervals"
    //   * "Civilian density: High (avoid casualties)"
    // - Mission objectives with difficulty ratings
    //   * Primary: Access Corporate Terminal (Required)
    //   * Secondary: Extract research data (Bonus credits)
    //   * Optional: No civilian casualties (Bonus XP)
    // - Environmental hazards/advantages
    //   * "Security cameras in main lobby"
    //   * "Back entrance available"
    //   * "Power grid vulnerable to EMP"
    // - Squad readiness assessment
    //   * Agent status (ready/recovering)
    //   * Equipment check (missing critical items?)
    //   * Recommended squad composition for mission type
    // - Risk/Reward breakdown
    //   * Base credits: 500-800
    //   * Stealth bonus: +200
    //   * Speed bonus: +100
    //   * Risk factors: Alert level penalties
    
    // Squad readiness check
    let ready_agents = (0..3).filter(|&i| global_data.agent_recovery[i] <= global_data.current_day).count();
    
    if ready_agents > 0 {
        parent.spawn(TextBundle::from_section(
            format!("\nSquad Status: {} agents ready for deployment", ready_agents),
            TextStyle { font_size: 16.0, color: Color::srgb(0.2, 0.8, 0.2), ..default() }
        ));
    } else {
        parent.spawn(TextBundle::from_section(
            "\nSquad Status: No agents available (all recovering)",
            TextStyle { font_size: 16.0, color: Color::srgb(0.8, 0.2, 0.2), ..default() }
        ));
    }
}

fn create_footer(parent: &mut ChildBuilder, active_tab: HubTab) {
    parent.spawn(NodeBundle {
        style: Style {
            width: Val::Percent(100.0),
            height: Val::Px(60.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            padding: UiRect::all(Val::Px(10.0)),
            ..default()
        },
        background_color: Color::srgb(0.08, 0.08, 0.15).into(),
        ..default()
    }).with_children(|footer| {
        let controls = match active_tab {
            HubTab::GlobalMap => "UP/DOWN: Select Region | W: Wait Day | ENTER: View Mission | F5: Save | ESC: Quit",
            HubTab::Research => "Navigation: Arrow Keys | Purchase: ENTER | 1-4: Switch Tabs | ESC: Quit",
            HubTab::Agents => "Select Agent: Arrow Keys | Modify: ENTER | Save/Load Preset: S/L | 1-4: Switch Tabs",
            HubTab::Missions => "Launch Mission: ENTER | 1-4: Switch Tabs | ESC: Quit",
        };
        
        footer.spawn(TextBundle::from_section(
            controls,
            TextStyle { font_size: 14.0, color: Color::srgb(0.7, 0.7, 0.7), ..default() }
        ));
    });
}