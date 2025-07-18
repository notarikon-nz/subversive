use bevy::prelude::*;
use leafwing_input_manager::prelude::*;

pub mod events;
pub use events::*;

// === STATES ===
#[derive(States, Debug, Clone, PartialEq, Eq, Hash)]
pub enum GameState {
    Mission,
}

impl Default for GameState {
    fn default() -> Self { GameState::Mission }
}

// === RESOURCES ===
#[derive(Resource)]
pub struct GameMode {
    pub paused: bool,
    pub targeting: Option<TargetingMode>,
}

impl Default for GameMode {
    fn default() -> Self {
        Self { paused: false, targeting: None }
    }
}

#[derive(Debug)]
pub enum TargetingMode {
    Neurovector { agent: Entity },
    Combat { agent: Entity },
}

#[derive(Resource, Default)]
pub struct SelectionState {
    pub selected: Vec<Entity>,
}

#[derive(Resource)]
pub struct MissionData {
    pub timer: f32,
    pub alert_level: AlertLevel,
}

impl Default for MissionData {
    fn default() -> Self {
        Self { timer: 0.0, alert_level: AlertLevel::Green }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum AlertLevel {
    Green,
    Yellow,
    Orange,
    Red,
}

// === INPUT ===
#[derive(Actionlike, PartialEq, Eq, Hash, Clone, Copy, Debug, Reflect)]
pub enum PlayerAction {
    Pause,
    Select,
    Move,
    Neurovector,
    Combat,
    Interact,
    Inventory,
}

// === CORE COMPONENTS ===
#[derive(Component)]
pub struct Agent;

#[derive(Component)]
pub struct Civilian;

#[derive(Component)]
pub struct Enemy;

#[derive(Component)]
pub struct Health(pub f32);

#[derive(Component)]
pub struct Dead;

#[derive(Component)]
pub struct MovementSpeed(pub f32);

#[derive(Component)]
pub struct Controllable;

#[derive(Component)]
pub struct Selected;

#[derive(Component)]
pub struct Selectable {
    pub radius: f32,
}

#[derive(Component)]
pub struct NeurovectorTarget;

#[derive(Component)]
pub struct NeurovectorControlled {
    pub controller: Entity,
}

#[derive(Component)]
pub struct MoveTarget {
    pub position: Vec2,
}

// === COMPLEX COMPONENTS ===
#[derive(Component)]
pub struct Vision {
    pub range: f32,
    pub angle: f32,
    pub direction: Vec2,
}

impl Vision {
    pub fn new(range: f32, angle_degrees: f32) -> Self {
        Self {
            range,
            angle: angle_degrees.to_radians(),
            direction: Vec2::X,
        }
    }
}

#[derive(Component)]
pub struct NeurovectorCapability {
    pub range: f32,
    pub max_targets: u8,
    pub cooldown: f32,
    pub current_cooldown: f32,
    pub controlled: Vec<Entity>,
}

impl Default for NeurovectorCapability {
    fn default() -> Self {
        Self {
            range: 200.0,
            max_targets: 3,
            cooldown: 5.0,
            current_cooldown: 0.0,
            controlled: Vec::new(),
        }
    }
}

#[derive(Component)]
pub struct Patrol {
    pub points: Vec<Vec2>,
    pub current_index: usize,
}

impl Patrol {
    pub fn new(points: Vec<Vec2>) -> Self {
        Self { points, current_index: 0 }
    }
    
    pub fn current_target(&self) -> Option<Vec2> {
        self.points.get(self.current_index).copied()
    }
    
    pub fn advance(&mut self) {
        if !self.points.is_empty() {
            self.current_index = (self.current_index + 1) % self.points.len();
        }
    }
}

#[derive(Component)]
pub struct Terminal {
    pub terminal_type: TerminalType,
    pub range: f32,
    pub accessed: bool,
}

#[derive(Debug, Clone)]
pub enum TerminalType {
    Objective,
    Equipment,
    Intel,
}

#[derive(Component)]
pub struct InventoryUI;

#[derive(Resource)]
pub struct InventoryState {
    pub ui_open: bool,
    pub selected_agent: Option<Entity>,
}

impl Default for InventoryState {
    fn default() -> Self {
        Self {
            ui_open: false,
            selected_agent: None,
        }
    }
}

#[derive(Component, Default)]
pub struct Inventory {
    pub weapons: Vec<WeaponType>,
    pub tools: Vec<ToolType>,
    pub currency: u32,
    pub equipped_weapon: Option<WeaponType>,
    pub equipped_tools: Vec<ToolType>,
    pub cybernetics: Vec<CyberneticType>,
    pub intel_documents: Vec<String>,
}

impl Inventory {
    pub fn add_weapon(&mut self, weapon: WeaponType) {
        if self.equipped_weapon.is_none() {
            self.equipped_weapon = Some(weapon.clone());
        }
        self.weapons.push(weapon);
    }
    
    pub fn add_currency(&mut self, amount: u32) {
        self.currency += amount;
    }
    
    pub fn add_tool(&mut self, tool: ToolType) {
        if self.equipped_tools.len() < 2 {
            self.equipped_tools.push(tool.clone());
        }
        self.tools.push(tool);
    }
    
    pub fn add_cybernetic(&mut self, cybernetic: CyberneticType) {
        self.cybernetics.push(cybernetic);
    }
    
    pub fn add_intel(&mut self, document: String) {
        self.intel_documents.push(document);
    }
}

#[derive(Debug, Clone)]
pub enum CyberneticType {
    Neurovector,
    CombatEnhancer,
    StealthModule,
    TechInterface,
}

#[derive(Debug, Clone)]
pub enum WeaponType {
    Pistol,
    Rifle,
    Minigun,
    Flamethrower,
}

#[derive(Debug, Clone)]
pub enum ToolType {
    Hacker,
    Scanner,
    Lockpick,
    MedKit,
}

// === UTILITY FUNCTIONS ===
pub fn get_world_mouse_position(
    windows: &Query<&Window>,
    cameras: &Query<(&Camera, &GlobalTransform)>,
) -> Option<Vec2> {
    let window = windows.get_single().ok()?;
    let (camera, camera_transform) = cameras.get_single().ok()?;
    
    window.cursor_position()
        .and_then(|cursor| camera.viewport_to_world_2d(camera_transform, cursor))
}