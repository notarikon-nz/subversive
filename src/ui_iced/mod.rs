// ui_iced/mod.rs - Core Iced UI system
use bevy::prelude::*;
use iced::{
    widget::{button, column, container, row, text, scrollable, progress_bar, slider, Column, Row},
    Element, Length, Color, Theme, application, Subscription,
};
use std::sync::{Arc, Mutex};
use crate::core::*;
use crate::systems::ui::hub::{HubTab};

pub mod integration;
pub mod inventory;
pub mod research;
pub mod singapore_map;
pub mod sync_systems;

// Shared state bridge between Bevy and Iced
#[derive(Resource, Clone)]
pub struct IcedUIBridge {
    pub state: Arc<Mutex<UISharedState>>,
}

#[derive(Default)]
pub struct UISharedState {
    pub game_state: GameState,
    pub global_data: GlobalData,
    pub inventory_open: bool,
    pub selected_agent: Option<Entity>,
    pub hub_tab: HubTab,
    pub menu_index: usize,
    pub post_mission: Option<PostMissionResults>,
}

// Main Iced App
pub struct SubversiveUI {
    bridge: Arc<Mutex<UISharedState>>,
    active_screen: Screen,
}

#[derive(Debug, Clone)]
pub enum Screen {
    MainMenu,
    Hub,
    Mission,
    PostMission,
    Settings,
}

#[derive(Debug, Clone)]
pub enum Message {
    MainMenu(MenuMsg),
    Hub(HubMsg),
    Mission(MissionMsg),
    PostMission(PostMsg),
    Navigate(Screen),
}

#[derive(Debug, Clone)]
pub enum MenuMsg {
    Select(usize),
    Continue,
    NewGame,
    Settings,
    Quit,
}

#[derive(Debug, Clone)]
pub enum HubMsg {
    TabChanged(HubTab),
    WaitDay,
    SelectCity(String),
    LaunchMission,
    Research(ResearchMsg),
    Territory(TerritoryMsg),
}

#[derive(Debug, Clone)]
pub enum ResearchMsg {
    StartProject(String),
    AssignScientist(Entity, String),
    SetPriority(String, ResearchPriority),
}

#[derive(Debug, Clone)]
pub enum TerritoryMsg {
    SetTaxRate(String, f32),
}

#[derive(Debug, Clone)]
pub enum MissionMsg {
    ToggleInventory,
    SelectSlot(usize, usize),
    DragItem(usize, usize),
    DropItem(usize, usize),
    Pause,
}

#[derive(Debug, Clone)]
pub enum PostMsg {
    Continue,
    Quit,
}

impl application::Application for SubversiveUI {
    type Message = Message;
    type Theme = Theme;
    type Executor = iced::executor::Default;
    type Flags = Arc<Mutex<UISharedState>>;

    fn new(bridge: Self::Flags) -> (Self, Command<Message>) {
        (
            Self {
                bridge: bridge.clone(),
                active_screen: Screen::MainMenu,
            },
            Command::none()
        )
    }

    fn title(&self) -> String {
        "SUBVERSIVE".to_string()
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::Navigate(screen) => {
                self.active_screen = screen;
                Command::none()
            }
            Message::MainMenu(msg) => self.handle_menu(msg),
            Message::Hub(msg) => self.handle_hub(msg),
            Message::Mission(msg) => self.handle_mission(msg),
            Message::PostMission(msg) => self.handle_post_mission(msg),
        }
    }

    fn view(&self) -> Element<Message> {
        let state = self.bridge.lock().unwrap();
        
        match self.active_screen {
            Screen::MainMenu => self.view_main_menu(&state),
            Screen::Hub => self.view_hub(&state),
            Screen::Mission => self.view_mission(&state),
            Screen::PostMission => self.view_post_mission(&state),
            Screen::Settings => self.view_settings(&state),
        }
    }

    fn theme(&self) -> Theme {
        cyberpunk_theme()
    }
    
    fn subscription(&self) -> Subscription<Message> {
        Subscription::none()
    }
}

impl SubversiveUI {
    fn handle_menu(&mut self, msg: MenuMsg) -> Command<Message> {
        let mut state = self.bridge.lock().unwrap();
        
        match msg {
            MenuMsg::Continue => {
                if let Some((data, _, _)) = crate::systems::save::load_game() {
                    state.global_data = data;
                    drop(state);
                    self.active_screen = Screen::Hub;
                }
            }
            MenuMsg::NewGame => {
                state.global_data = GlobalData::default();
                drop(state);
                self.active_screen = Screen::Hub;
            }
            MenuMsg::Settings => self.active_screen = Screen::Settings,
            MenuMsg::Quit => std::process::exit(0),
            _ => {}
        }
        Command::none()
    }

    fn handle_hub(&mut self, msg: HubMsg) -> Command<Message> {
        let mut state = self.bridge.lock().unwrap();
        
        match msg {
            HubMsg::TabChanged(tab) => state.hub_tab = tab,
            HubMsg::WaitDay => state.global_data.current_day += 1,
            HubMsg::LaunchMission => {
                drop(state);
                self.active_screen = Screen::Mission;
            }
            _ => {}
        }
        Command::none()
    }

    fn handle_mission(&mut self, msg: MissionMsg) -> Command<Message> {
        let mut state = self.bridge.lock().unwrap();
        
        match msg {
            MissionMsg::ToggleInventory => state.inventory_open = !state.inventory_open,
            _ => {}
        }
        Command::none()
    }

    fn handle_post_mission(&mut self, msg: PostMsg) -> Command<Message> {
        match msg {
            PostMsg::Continue => self.active_screen = Screen::Hub,
            PostMsg::Quit => std::process::exit(0),
        }
        Command::none()
    }

    // View functions
    fn view_main_menu(&self, state: &UISharedState) -> Element<Message> {
        let has_save = crate::systems::save::save_game_exists();
        
        let mut buttons = column![].spacing(10);
        
        if has_save {
            buttons = buttons.push(menu_button("Continue", MenuMsg::Continue));
        }
        
        buttons = buttons
            .push(menu_button("New Game", MenuMsg::NewGame))
            .push(menu_button("Settings", MenuMsg::Settings))
            .push(menu_button("Quit", MenuMsg::Quit));

        container(
            column![
                text("SUBVERSIVE").size(48),
                buttons
            ]
            .spacing(50)
            .align_items(iced::Alignment::Center)
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x()
        .center_y()
        .into()
    }

    fn view_hub(&self, state: &UISharedState) -> Element<Message> {
        let tabs = row![
            tab_button("Map", HubTab::GlobalMap, state.hub_tab),
            tab_button("Territory", HubTab::Territory, state.hub_tab),
            tab_button("Research", HubTab::Research, state.hub_tab),
            tab_button("Agents", HubTab::Agents, state.hub_tab),
            tab_button("Gear", HubTab::Manufacture, state.hub_tab),
            tab_button("Mission", HubTab::Missions, state.hub_tab),
        ]
        .spacing(5);

        let content = match state.hub_tab {
            HubTab::GlobalMap => view_global_map(state),
            HubTab::Territory => view_territory(state),
            HubTab::Research => view_research(state),
            HubTab::Agents => view_agents(state),
            HubTab::Manufacture => view_manufacture(state),
            HubTab::Missions => view_missions(state),
        };

        column![
            tabs,
            scrollable(content).height(Length::Fill)
        ]
        .into()
    }

    fn view_mission(&self, state: &UISharedState) -> Element<Message> {
        if state.inventory_open {
            view_inventory(state)
        } else {
            container(
                text("Mission in progress...")
                    .size(24)
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()
        }
    }

    fn view_post_mission(&self, state: &UISharedState) -> Element<Message> {
        if let Some(results) = &state.post_mission {
            let title = if results.success {
                text("MISSION SUCCESS").color(Color::from_rgb(0.0, 1.0, 0.0))
            } else {
                text("MISSION FAILED").color(Color::from_rgb(1.0, 0.0, 0.0))
            };

            column![
                title.size(32),
                text(format!("Time: {:.1}s", results.time_taken)),
                text(format!("Enemies: {}", results.enemies_killed)),
                text(format!("Credits: ${}", results.credits_earned)),
                row![
                    button("Continue").on_press(Message::PostMission(PostMsg::Continue)),
                    button("Quit").on_press(Message::PostMission(PostMsg::Quit))
                ].spacing(10)
            ]
            .spacing(20)
            .into()
        } else {
            text("Loading results...").into()
        }
    }

    fn view_settings(&self, _state: &UISharedState) -> Element<Message> {
        column![
            text("Settings").size(32),
            text("Coming soon..."),
            button("Back").on_press(Message::Navigate(Screen::MainMenu))
        ]
        .spacing(20)
        .into()
    }
}

// Helper functions
fn menu_button(label: &str, msg: MenuMsg) -> Element<Message> {
    button(text(label).size(24))
        .width(200)
        .height(40)
        .on_press(Message::MainMenu(msg))
        .into()
}

fn tab_button(label: &str, tab: HubTab, current: HubTab) -> Element<Message> {
    let btn = button(text(label).size(16));
    
    if tab == current {
        btn.style(iced::theme::Button::Primary)
    } else {
        btn
    }
    .on_press(Message::Hub(HubMsg::TabChanged(tab)))
    .into()
}

fn cyberpunk_theme() -> Theme {
    // Custom theme matching the original cyberpunk colors
    Theme::custom("cyberpunk".to_string(), iced::theme::Palette {
        background: Color::from_rgb(0.03, 0.03, 0.05),
        text: Color::from_rgb(0.99, 1.0, 0.32),
        primary: Color::from_rgb(0.99, 1.0, 0.32),
        success: Color::from_rgb(0.0, 1.0, 1.0),
        danger: Color::from_rgb(1.0, 0.0, 0.59),
    })
}

// Hub view components (kept minimal)
fn view_global_map(state: &UISharedState) -> Element<Message> {
    column![
        text("GLOBAL OPERATIONS MAP").size(24),
        text(format!("Day: {}", state.global_data.current_day)),
        button("Wait Day").on_press(Message::Hub(HubMsg::WaitDay)),
    ]
    .spacing(10)
    .into()
}

fn view_territory(state: &UISharedState) -> Element<Message> {
    text("Territory Control").into()
}

fn view_research(state: &UISharedState) -> Element<Message> {
    column![
        text("RESEARCH & DEVELOPMENT").size(24),
        text(format!("Credits: ${}", state.global_data.credits)),
    ]
    .spacing(10)
    .into()
}

fn view_agents(state: &UISharedState) -> Element<Message> {
    text("Agent Management").into()
}

fn view_manufacture(state: &UISharedState) -> Element<Message> {
    text("Weapon Manufacture").into()
}

fn view_missions(state: &UISharedState) -> Element<Message> {
    column![
        text("MISSION BRIEFING").size(24),
        button("Launch Mission").on_press(Message::Hub(HubMsg::LaunchMission))
    ]
    .spacing(10)
    .into()
}

fn view_inventory(state: &UISharedState) -> Element<Message> {
    let grid = (0..8).map(|y| {
        row((0..10).map(|x| {
            button(text("□"))
                .width(32)
                .height(32)
                .on_press(Message::Mission(MissionMsg::SelectSlot(x, y)))
        }))
        .spacing(2)
        .into()
    });

    column![
        text("INVENTORY").size(24),
        Column::with_children(grid).spacing(2),
        button("Close").on_press(Message::Mission(MissionMsg::ToggleInventory))
    ]
    .spacing(10)
    .into()
}

// Bevy integration system
pub fn sync_iced_ui(
    bridge: Res<IcedUIBridge>,
    game_state: Res<State<GameState>>,
    global_data: Res<GlobalData>,
    inventory_state: Res<InventoryState>,
    selection: Res<SelectionState>,
) {
    let mut state = bridge.state.lock().unwrap();
    state.game_state = **game_state;
    state.global_data = global_data.clone();
    state.inventory_open = inventory_state.ui_open;
    state.selected_agent = selection.selected.first().copied();
}
