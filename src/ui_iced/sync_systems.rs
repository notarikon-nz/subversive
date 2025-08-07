// ui_iced/sync_systems.rs - State synchronization between Bevy and Iced

use bevy::prelude::*;
use crate::core::*;
use crate::systems::*;
use crate::ui_iced::{IcedUIBridge, UISharedState};
use crate::systems::ui::hub::{HubState};

// Main synchronization system - runs every frame
pub fn sync_game_to_ui(
    bridge: Res<IcedUIBridge>,
    game_state: Res<State<GameState>>,
    global_data: Res<GlobalData>,
    inventory_state: Res<InventoryState>,
    selection: Res<SelectionState>,
    post_mission: Option<Res<PostMissionResults>>,
    mission_data: Res<MissionData>,
    research_progress: Res<ResearchProgress>,
    territory_manager: Res<TerritoryManager>,
    cities_progress: Res<CitiesProgress>,
    hub_state: Res<HubState>,
    menu_state: Res<MainMenuState>,
) {
    // Only sync if something changed
    if !game_state.is_changed() 
        && !global_data.is_changed() 
        && !inventory_state.is_changed()
        && !selection.is_changed()
        && !research_progress.is_changed()
        && !territory_manager.is_changed() {
        return;
    }

    // Lock and update shared state
    if let Ok(mut state) = bridge.state.try_lock() {
        // Core state
        state.game_state = **game_state;
        state.global_data = global_data.clone();
        state.inventory_open = inventory_state.ui_open;
        state.selected_agent = selection.selected.first().copied();
        
        // Hub state
        state.hub_tab = hub_state.active_tab;
        
        // Menu state
        state.menu_index = menu_state.selected_index;
        
        // Post mission results
        state.post_mission = post_mission.map(|r| r.clone());
        
        // Extended state for complex UIs
        state.research_progress = research_progress.clone();
        state.territory_manager = territory_manager.clone();
        state.cities_progress = cities_progress.clone();
        state.mission_data = mission_data.clone();
    }
}

// Sync UI actions back to Bevy
pub fn sync_ui_to_game(
    bridge: Res<IcedUIBridge>,
    mut next_state: ResMut<NextState<GameState>>,
    mut global_data: ResMut<GlobalData>,
    mut inventory_state: ResMut<InventoryState>,
    mut hub_state: ResMut<HubState>,
    mut selection: ResMut<SelectionState>,
    mut commands: Commands,
    mut research_events: EventWriter<ResearchStartEvent>,
    mut territory_events: EventWriter<TerritoryControlEvent>,
) {
    if let Ok(state) = bridge.state.try_lock() {
        // Check for pending UI actions
        if let Some(actions) = state.pending_actions.clone() {
            for action in actions {
                handle_ui_action(
                    action,
                    &mut next_state,
                    &mut global_data,
                    &mut inventory_state,
                    &mut hub_state,
                    &mut selection,
                    &mut commands,
                    &mut research_events,
                    &mut territory_events,
                );
            }
            
            // Clear processed actions
            drop(state);
            if let Ok(mut state) = bridge.state.try_lock() {
                state.pending_actions.clear();
            }
        }
    }
}

// Extended shared state with action queue
#[derive(Default, Clone)]
pub struct UISharedStateExtended {
    // ... existing fields ...
    pub research_progress: ResearchProgress,
    pub territory_manager: TerritoryManager,
    pub cities_progress: CitiesProgress,
    pub mission_data: MissionData,
    
    // Action queue for UI->Game communication
    pub pending_actions: Vec<UIAction>,
}

#[derive(Clone, Debug)]
pub enum UIAction {
    ChangeGameState(GameState),
    StartResearch(String),
    LaunchMission(String),
    WaitDay,
    SaveGame,
    LoadGame,
    SelectAgent(Entity),
    ToggleInventory,
    MoveItem { from: (usize, usize), to: (usize, usize) },
    SetTaxRate { city: String, rate: f32 },
}

fn handle_ui_action(
    action: UIAction,
    next_state: &mut ResMut<NextState<GameState>>,
    global_data: &mut ResMut<GlobalData>,
    inventory_state: &mut ResMut<InventoryState>,
    hub_state: &mut ResMut<HubState>,
    selection: &mut ResMut<SelectionState>,
    commands: &mut Commands,
    research_events: &mut EventWriter<ResearchStartEvent>,
    territory_events: &mut EventWriter<TerritoryControlEvent>,
) {
    match action {
        UIAction::ChangeGameState(state) => {
            next_state.set(state);
        }
        UIAction::StartResearch(project_id) => {
            research_events.send(ResearchStartEvent { project_id });
        }
        UIAction::LaunchMission(city_id) => {
            commands.insert_resource(MissionLaunchData {
                city_id: city_id.clone(),
                region_id: global_data.selected_region,
            });
            commands.insert_resource(ShouldRestart);
            next_state.set(GameState::Mission);
        }
        UIAction::WaitDay => {
            global_data.current_day += 1;
            territory_events.send(TerritoryControlEvent::DayPassed);
        }
        UIAction::SaveGame => {
            // Trigger save
            commands.insert_resource(TriggerSave);
        }
        UIAction::LoadGame => {
            if let Some((data, territory, progression)) = crate::systems::save::load_game() {
                **global_data = data;
                // Update other systems...
                next_state.set(GameState::GlobalMap);
            }
        }
        UIAction::SelectAgent(entity) => {
            selection.selected.clear();
            selection.selected.push(entity);
        }
        UIAction::ToggleInventory => {
            inventory_state.ui_open = !inventory_state.ui_open;
        }
        UIAction::MoveItem { from, to } => {
            // Handle inventory movement
            // This would interact with your inventory grid system
        }
        UIAction::SetTaxRate { city, rate } => {
            territory_events.send(TerritoryControlEvent::SetTaxRate { city, rate });
        }
    }
}

// Helper resources for triggering actions
#[derive(Resource)]
pub struct TriggerSave;

#[derive(Event)]
pub struct ResearchStartEvent {
    pub project_id: String,
}

// In your Iced UI code, queue actions like this:
impl SubversiveUI {
    fn queue_action(&mut self, action: UIAction) {
        if let Ok(mut state) = self.bridge.lock() {
            state.pending_actions.push(action);
        }
    }
    
    // Example in handle_hub:
    fn handle_hub(&mut self, msg: HubMsg) -> Command<Message> {
        match msg {
            HubMsg::LaunchMission => {
                self.queue_action(UIAction::LaunchMission(
                    self.get_selected_city().to_string()
                ));
            }
            HubMsg::Research(ResearchMsg::StartProject(id)) => {
                self.queue_action(UIAction::StartResearch(id));
            }
            // etc...
        }
        Command::none()
    }
}

// Setup in main.rs:
pub fn setup_ui_sync_systems(app: &mut App) {
    app
        // Add sync systems
        .add_systems(Update, (
            sync_game_to_ui,
            sync_ui_to_game,
        ).chain())
        
        // Add event types
        .add_event::<ResearchStartEvent>()
        
        // Add to your existing systems
        .add_systems(Update, (
            handle_research_start_events,
            handle_save_trigger,
        ).run_if(resource_exists::<IcedUIBridge>));
}

fn handle_research_start_events(
    mut events: EventReader<ResearchStartEvent>,
    mut research_progress: ResMut<ResearchProgress>,
    research_db: Res<ResearchDatabase>,
    mut global_data: ResMut<GlobalData>,
) {
    for event in events.read() {
        if let Some(project) = research_db.get_project(&event.project_id) {
            if global_data.credits >= project.cost {
                global_data.credits -= project.cost;
                start_research_project(
                    &event.project_id,
                    None, // Would find best scientist
                    &mut global_data,
                    &mut research_progress,
                    &research_db,
                );
            }
        }
    }
}

fn handle_save_trigger(
    trigger: Option<Res<TriggerSave>>,
    mut commands: Commands,
    global_data: Res<GlobalData>,
    research_progress: Res<ResearchProgress>,
    territory_manager: Res<TerritoryManager>,
    progression_tracker: Res<CampaignProgressionTracker>,
) {
    if trigger.is_some() {
        crate::systems::save::save_game_complete(
            &global_data,
            &research_progress,
            &territory_manager,
            &progression_tracker,
        );
        commands.remove_resource::<TriggerSave>();
    }
}

// Example: Inventory sync for the grid system
pub fn sync_inventory_grid(
    bridge: Res<IcedUIBridge>,
    mut inventory_grid: ResMut<InventoryGrid>,
    agent_query: Query<&Inventory, With<Agent>>,
) {
    if let Ok(state) = bridge.state.try_lock() {
        if let Some(agent) = state.selected_agent {
            if let Ok(inventory) = agent_query.get(agent) {
                // Only update if inventory changed
                if inventory.is_changed() {
                    populate_grid_from_inventory(&mut inventory_grid, inventory);
                }
            }
        }
    }
}

fn populate_grid_from_inventory(grid: &mut InventoryGrid, inventory: &Inventory) {
    // Implementation from your inventory_integration.rs
}