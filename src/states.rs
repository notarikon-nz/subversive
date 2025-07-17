use bevy::prelude::*;

#[derive(States, Debug, Clone, PartialEq, Eq, Hash)]
pub enum GameState {
    MainMenu,
    GlobalMap,
    Mission,
    MissionBriefing,
    PostMission,
}

impl Default for GameState {
    fn default() -> Self {
        GameState::Mission // Start directly in mission for rapid testing
    }
}

#[derive(States, Debug, Clone, PartialEq, Eq, Hash)]
pub enum MissionState {
    Planning,
    Active,
    Paused,
    Failed,
    Complete,
}

impl Default for MissionState {
    fn default() -> Self {
        MissionState::Active
    }
}

#[derive(States, Debug, Clone, PartialEq, Eq, Hash)]
pub enum PauseState {
    ViewOnly,
    IssueOrders,
    ReviewIntel,
}

impl Default for PauseState {
    fn default() -> Self {
        PauseState::IssueOrders
    }
}