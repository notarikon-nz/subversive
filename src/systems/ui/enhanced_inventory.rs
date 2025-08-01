// src/systems/ui/enhanced_inventory.rs - Grid-based inventory system
use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use crate::systems::ui::*;
use std::collections::HashMap;
use crate::systems::cursor::{CursorEntity};

// === CORE INVENTORY COMPONENTS ===

#[derive(Component, Clone)]
pub struct InventoryItem {
    pub id: String,
    pub name: String,
    pub item_type: ItemType,
    pub rarity: ItemRarity,
    pub quantity: u32,
    pub max_stack: u32,
    pub stats: ItemStats,
    pub description: String,
    pub icon_path: String,
    pub weight: f32,
    pub value: u32,
    pub is_favorited: bool,
    pub is_locked: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ItemType {
    Weapon(WeaponType),
    Attachment(AttachmentSlot),
    Consumable,
    Material,
    Tool(ToolType),
    Cybernetic(CyberneticType),
    Intel,
    Currency,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ItemRarity {
    Common,    // White
    Uncommon,  // Green
    Rare,      // Blue
    Epic,      // Purple
    Legendary, // Orange
    Unique,    // Red
}

#[derive(Debug, Clone, Default)]
pub struct ItemStats {
    pub damage: i16,
    pub accuracy: i16,
    pub range: i16,
    pub reload_speed: i16,
    pub armor: i16,
    pub stealth: i16,
    pub hacking: i16,
}

// === GRID INVENTORY SYSTEM ===

#[derive(Resource)]
pub struct InventoryGrid {
    pub width: usize,
    pub height: usize,
    pub slots: Vec<Vec<Option<InventorySlot>>>,
    pub selected_slot: Option<(usize, usize)>,
    pub dragging_item: Option<DraggedItem>,
    pub filter: InventoryFilter,
    pub sort_mode: SortMode,
    pub tab: InventoryTab,
}

#[derive(Clone)]
pub struct InventorySlot {
    pub item: InventoryItem,
    pub position: (usize, usize),
    pub size: (usize, usize), // Multi-slot items
}

#[derive(Clone)]
pub struct DraggedItem {
    pub item: InventoryItem,
    pub origin: (usize, usize),
    pub cursor_offset: egui::Vec2,
}

#[derive(Debug, Clone, Default)]
pub struct InventoryFilter {
    pub text: String,
    pub item_type: Option<ItemType>,
    pub rarity: Option<ItemRarity>,
    pub show_favorites_only: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SortMode {
    Name,
    Type,
    Rarity,
    Value,
    Weight,
    RecentlyAcquired,
}

#[derive(Debug, Clone, PartialEq)]
pub enum InventoryTab {
    All,
    Weapons,
    Gear,
    Consumables,
    Materials,
    Intel,
}

// === LOADOUT SYSTEM ===

#[derive(Resource, Default)]
pub struct LoadoutManager {
    pub presets: HashMap<String, LoadoutPreset>,
    pub current_preset: Option<String>,
    pub quick_slots: [Option<String>; 4], // F1-F4 quick loadouts
}

#[derive(Clone)]
pub struct LoadoutPreset {
    pub name: String,
    pub weapon: Option<WeaponConfig>,
    pub tools: Vec<ToolType>,
    pub cybernetics: Vec<CyberneticType>,
    pub consumables: HashMap<String, u32>,
}

// === COMPARISON SYSTEM ===

#[derive(Default)]
pub struct ItemComparison {
    pub comparing_item: Option<InventoryItem>,
    pub equipped_item: Option<InventoryItem>,
    pub stat_differences: ItemStats,
}

// === ENHANCED INVENTORY UI SYSTEM ===

pub fn enhanced_inventory_system(
    mut contexts: EguiContexts,
    mut inventory_state: ResMut<InventoryState>,
    mut inventory_grid: ResMut<InventoryGrid>,
    mut loadout_manager: ResMut<LoadoutManager>,
    agent_query: Query<(&Inventory, &WeaponState), With<Agent>>,
    selection: Res<SelectionState>, // ADD: Get current selection
    input: Res<ButtonInput<KeyCode>>,
    mut audio_events: EventWriter<AudioEvent>,
    mut local_last_toggle: Local<bool>, // Track our toggle state locally
    mut cursor_query: Query<&mut Visibility, With<CursorEntity>>, // Control cursor visibility
) {
    // Toggle inventory with debouncing
    let toggle_pressed = input.just_pressed(KeyCode::KeyI);

    if toggle_pressed && !*local_last_toggle {
        inventory_state.ui_open = !inventory_state.ui_open;
        *local_last_toggle = true;

        // FIXED: Update selected agent when opening inventory
        if inventory_state.ui_open {
            inventory_state.selected_agent = selection.selected.first().copied();

            audio_events.write(AudioEvent {
                sound: AudioType::CursorInteract,
                volume: 0.3,
            });
        }
    } else if !toggle_pressed {
        *local_last_toggle = false;
    }

    // FIXED: Hide custom cursor when inventory is open to prevent z-index issues
    if let Ok(mut cursor_visibility) = cursor_query.single_mut() {
        *cursor_visibility = if inventory_state.ui_open {
            Visibility::Hidden
        } else {
            Visibility::Visible
        };
    }

    if !inventory_state.ui_open {
        return;
    }

    // FIXED: Ensure we have an agent selected - fallback to first selected agent
    if inventory_state.selected_agent.is_none() && !selection.selected.is_empty() {
        inventory_state.selected_agent = selection.selected.first().copied();
    }

    if let Ok(ctx) = contexts.ctx_mut() {

        // FIXED: Force inventory window to be on top with proper size constraints
        egui::Window::new("AGENT INVENTORY")
            .resizable(true) // Allow resizing so users can adjust if needed
            .default_size([900.0, 650.0]) // Slightly larger default size
            .min_size([700.0, 500.0]) // Minimum size to ensure usability
            .max_size([1200.0, 800.0]) // Maximum size to prevent screen overflow
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0]) // Center the window
            .order(egui::Order::Foreground) // Force to front
            .show(ctx, |ui| {
                // Add padding around the content
                ui.spacing_mut().item_spacing = egui::vec2(8.0, 6.0);
                ui.spacing_mut().button_padding = egui::vec2(8.0, 4.0);

                render_inventory_ui(
                    ui,
                    &mut inventory_grid,
                    &mut loadout_manager,
                    &agent_query,
                    &inventory_state,
                    &mut audio_events,
                );
            });

        // Item comparison tooltip - also force to front
        if let Some(comparing) = &inventory_grid.dragging_item {
            render_comparison_tooltip(ctx, comparing);
        }
    }
}


fn render_inventory_ui(
    ui: &mut egui::Ui,
    inventory_grid: &mut InventoryGrid,
    loadout_manager: &mut LoadoutManager,
    agent_query: &Query<(&Inventory, &WeaponState), With<Agent>>,
    inventory_state: &InventoryState,
    audio_events: &mut EventWriter<AudioEvent>,
) {
    // FIXED: Use available_rect to constrain layout to window size
    let available_size = ui.available_rect_before_wrap().size();

    ui.allocate_ui_with_layout(
        available_size,
        egui::Layout::top_down(egui::Align::Min),
        |ui| {
            // Top section - tabs and filters (fixed height)
            ui.allocate_ui_with_layout(
                egui::vec2(available_size.x, 80.0),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    render_inventory_tabs(ui, inventory_grid);
                    ui.add_space(5.0);
                    render_filter_bar(ui, inventory_grid);
                }
            );

            ui.add_space(10.0);

            // Main content area - use remaining space
            let remaining_height = available_size.y - 120.0; // Reserve space for bottom panel

            ui.allocate_ui_with_layout(
                egui::vec2(available_size.x, remaining_height),
                egui::Layout::left_to_right(egui::Align::TOP),
                |ui| {
                    // Left panel - Inventory grid (60% of width)
                    let grid_width = available_size.x * 0.6;
                    ui.allocate_ui_with_layout(
                        egui::vec2(grid_width, remaining_height),
                        egui::Layout::top_down(egui::Align::Min),
                        |ui| {
                            ui.heading("INVENTORY GRID");
                            ui.add_space(5.0);
                            render_inventory_grid_constrained(ui, inventory_grid, audio_events);
                        }
                    );

                    ui.separator();

                    // Right panel - Stats and loadouts (40% of width)
                    let stats_width = available_size.x * 0.35;
                    ui.allocate_ui_with_layout(
                        egui::vec2(stats_width, remaining_height),
                        egui::Layout::top_down(egui::Align::Min),
                        |ui| {
                            if let Some(agent) = inventory_state.selected_agent {
                                if let Ok((inventory, weapon_state)) = agent_query.get(agent) {
                                    render_agent_stats_panel(ui, inventory, weapon_state);
                                    ui.add_space(10.0);
                                    render_loadout_panel(ui, loadout_manager, inventory);
                                }
                            } else {
                                ui.label("No agent selected");
                            }
                        }
                    );
                }
            );

            // Bottom panel - Action bar (fixed height)
            ui.add_space(5.0);
            ui.separator();
            render_action_bar(ui, inventory_grid, audio_events);
        }
    );
}

fn render_inventory_tabs(ui: &mut egui::Ui, inventory_grid: &mut InventoryGrid) {
    ui.horizontal(|ui| {
        let tabs = [
            (InventoryTab::All, "ALL"),
            (InventoryTab::Weapons, "WEAPONS"),
            (InventoryTab::Gear, "GEAR"),
            (InventoryTab::Consumables, "CONSUMABLES"),
            (InventoryTab::Materials, "MATERIALS"),
            (InventoryTab::Intel, "INTEL"),
        ];

        for (tab, label) in tabs {
            if ui.selectable_label(inventory_grid.tab == tab, label).clicked() {
                inventory_grid.tab = tab;
            }
        }
    });
}

fn render_filter_bar(ui: &mut egui::Ui, inventory_grid: &mut InventoryGrid) {
    ui.horizontal(|ui| {
        // Search section
        ui.label("SEARCH:");
        ui.add_sized([120.0, 20.0], egui::TextEdit::singleline(&mut inventory_grid.filter.text));

        if ui.button("⭐").clicked() {
            inventory_grid.filter.show_favorites_only = !inventory_grid.filter.show_favorites_only;
        }

        ui.separator();

        // Sort section - more compact
        ui.label("SORT:");
        egui::ComboBox::from_id_salt("sort_combo")
            .width(80.0)
            .selected_text(format!("{:?}", inventory_grid.sort_mode))
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut inventory_grid.sort_mode, SortMode::Name, "Name");
                ui.selectable_value(&mut inventory_grid.sort_mode, SortMode::Type, "Type");
                ui.selectable_value(&mut inventory_grid.sort_mode, SortMode::Rarity, "Rarity");
                ui.selectable_value(&mut inventory_grid.sort_mode, SortMode::Value, "Value");
            });

        if ui.button("SORT").clicked() {
            sort_inventory(inventory_grid);
        }
    });
}

fn render_inventory_grid_constrained(
    ui: &mut egui::Ui,
    inventory_grid: &mut InventoryGrid,
    audio_events: &mut EventWriter<AudioEvent>,
) {
    let available_rect = ui.available_rect_before_wrap();
    let available_size = available_rect.size();

    // Calculate slot size based on available space
    let padding = 10.0;
    let spacing = 2.0;
    let max_width = available_size.x - padding * 2.0;
    let max_height = available_size.y - padding * 2.0;

    // Calculate optimal slot size to fit the grid
    let total_h_spacing = (inventory_grid.width - 1) as f32 * spacing;
    let total_v_spacing = (inventory_grid.height - 1) as f32 * spacing;

    let slot_width = (max_width - total_h_spacing) / inventory_grid.width as f32;
    let slot_height = (max_height - total_v_spacing) / inventory_grid.height as f32;

    // Use smaller dimension to keep slots square
    let slot_size = slot_width.min(slot_height).max(32.0).min(64.0); // Clamp between 32-64px

    // Center the grid within available space
    let grid_width = inventory_grid.width as f32 * slot_size + total_h_spacing;
    let grid_height = inventory_grid.height as f32 * slot_size + total_v_spacing;

    let start_x = (available_size.x - grid_width) * 0.5;
    let start_y = padding;

    // Use a scroll area if grid is still too large
    egui::ScrollArea::vertical()
        .max_height(max_height)
        .show(ui, |ui| {
            ui.allocate_ui_at_rect(
                egui::Rect::from_min_size(
                    available_rect.min + egui::vec2(start_x, start_y),
                    egui::vec2(grid_width, grid_height)
                ),
                |ui| {
                    for y in 0..inventory_grid.height {
                        for x in 0..inventory_grid.width {
                            let slot_pos = egui::pos2(
                                x as f32 * (slot_size + spacing),
                                y as f32 * (slot_size + spacing)
                            );

                            let slot_rect = egui::Rect::from_min_size(
                                slot_pos,
                                egui::vec2(slot_size, slot_size)
                            );

                            render_inventory_slot(ui, inventory_grid, (x, y), slot_rect, audio_events);
                        }
                    }
                }
            );
        });
}

fn render_inventory_slot(
    ui: &mut egui::Ui,
    inventory_grid: &mut InventoryGrid,
    position: (usize, usize),
    rect: egui::Rect,
    audio_events: &mut EventWriter<AudioEvent>,
) {
    let (x, y) = position;
    let slot = &inventory_grid.slots[y][x];

    // Background color based on content
    let bg_color = if slot.is_some() {
        egui::Color32::from_gray(40)
    } else {
        egui::Color32::from_gray(20)
    };

    // Border color for selection
    let border_color = if inventory_grid.selected_slot == Some(position) {
        egui::Color32::YELLOW
    } else {
        egui::Color32::from_gray(60)
    };

    ui.painter().rect(rect, 2.0, bg_color, egui::Stroke::new(1.0, border_color), egui::StrokeKind::Outside);

    // Render item if present
    if let Some(slot_item) = slot {
        render_item_in_slot(ui, &slot_item.item, rect);

        // Handle interaction
        let response = ui.allocate_rect(rect, egui::Sense::click_and_drag());

        if response.clicked() {
            inventory_grid.selected_slot = Some(position);
            audio_events.write(AudioEvent {
                sound: AudioType::CursorTarget,
                volume: 0.2,
            });
        }

        if response.drag_started() {
            inventory_grid.dragging_item = Some(DraggedItem {
                item: slot_item.item.clone(),
                origin: position,
                cursor_offset: response.interact_pointer_pos().unwrap() - rect.min,
            });
        }

        // Tooltip on hover
        if response.hovered() {
            render_item_tooltip(ui, &slot_item.item);
        }
    } else {
        // Empty slot - handle drop
        let response = ui.allocate_rect(rect, egui::Sense::click());

        if response.clicked() {
            inventory_grid.selected_slot = Some(position);
        }
    }

    // Handle drop
    handle_slot_drop(ui, inventory_grid, position, rect);
}

fn render_item_in_slot(ui: &mut egui::Ui, item: &InventoryItem, rect: egui::Rect) {
    // Rarity border
    let rarity_color = match item.rarity {
        ItemRarity::Common => egui::Color32::WHITE,
        ItemRarity::Uncommon => egui::Color32::GREEN,
        ItemRarity::Rare => egui::Color32::BLUE,
        ItemRarity::Epic => egui::Color32::from_rgb(128, 0, 128),
        ItemRarity::Legendary => egui::Color32::from_rgb(255, 165, 0),
        ItemRarity::Unique => egui::Color32::RED,
    };

    ui.painter().rect_stroke(rect, 2.0, egui::Stroke::new(2.0, rarity_color), egui::StrokeKind::Outside);

    // Item icon (placeholder)
    let icon_rect = rect.shrink(4.0);
    ui.painter().rect_filled(icon_rect, 1.0, egui::Color32::from_gray(100));

    // Item quantity if stackable
    if item.quantity > 1 {
        let text_pos = egui::pos2(rect.max.x - 12.0, rect.max.y - 12.0);
        ui.painter().text(
            text_pos,
            egui::Align2::RIGHT_BOTTOM,
            item.quantity.to_string(),
            egui::FontId::monospace(10.0),
            egui::Color32::WHITE,
        );
    }

    // Favorite star
    if item.is_favorited {
        let star_pos = egui::pos2(rect.min.x + 2.0, rect.min.y + 2.0);
        ui.painter().text(
            star_pos,
            egui::Align2::LEFT_TOP,
            "⭐",
            egui::FontId::monospace(8.0),
            egui::Color32::YELLOW,
        );
    }

    // Lock icon
    if item.is_locked {
        let lock_pos = egui::pos2(rect.max.x - 8.0, rect.min.y + 2.0);
        ui.painter().text(
            lock_pos,
            egui::Align2::RIGHT_TOP,
            "🔒",
            egui::FontId::monospace(8.0),
            egui::Color32::WHITE,
        );
    }
}

fn render_item_tooltip(ui: &mut egui::Ui, item: &InventoryItem) {

    // PLACEHOLDER
    // Use our existing implementation
    /*
    ui.ctx().show_tooltip_at_pointer(|ui| {
        ui.label(format!(
            "{}\n{:?} - {:?}\n{}\nValue: {} | Weight: {:.1}",
            item.name,
            item.item_type,
            item.rarity,
            item.description,
            item.value,
            item.weight
        ));
    });
    */
}

fn render_comparison_tooltip(ctx: &egui::Context, dragged_item: &DraggedItem) {
    egui::Window::new("Item Comparison")
        .movable(false)
        .resizable(false)
        .title_bar(false)
        .show(ctx, |ui| {
            ui.label(format!("Comparing: {}", dragged_item.item.name));
            // Add stat comparison here
        });
}

fn render_agent_stats_panel(
    ui: &mut egui::Ui,
    inventory: &Inventory,
    weapon_state: &WeaponState,
) {
    ui.heading("AGENT STATUS");

    ui.label(format!("Credits: {}", inventory.currency));

    if let Some(weapon) = &inventory.equipped_weapon {
        ui.separator();
        ui.label("EQUIPPED WEAPON:");
        ui.label(format!("{:?}", weapon.base_weapon));
        ui.label(format!("Ammo: {}/{}", weapon_state.current_ammo, weapon_state.max_ammo));

        let stats = weapon.stats();
        if stats.accuracy != 0 || stats.range != 0 {
            ui.label(format!("Mods: Acc{:+} Rng{:+}", stats.accuracy, stats.range));
        }
    }
}

fn render_loadout_panel(
    ui: &mut egui::Ui,
    loadout_manager: &mut LoadoutManager,
    inventory: &Inventory,
) {
    ui.heading("LOADOUTS");

    ui.horizontal(|ui| {
        if ui.button("SAVE CURRENT").clicked() {
            save_current_loadout(loadout_manager, inventory);
        }

        if ui.button("QUICK EQUIP").clicked() {
            // Implement quick equip logic
        }
    });

    // Show saved presets
    for (name, _preset) in &loadout_manager.presets {
        ui.horizontal(|ui| {
            if ui.button("LOAD").clicked() {
                loadout_manager.current_preset = Some(name.clone());
            }
            ui.label(name);
        });
    }
}

fn render_action_bar(
    ui: &mut egui::Ui,
    inventory_grid: &mut InventoryGrid,
    audio_events: &mut EventWriter<AudioEvent>,
) {
    ui.horizontal(|ui| {
        if ui.button("SORT ALL").clicked() {
            sort_inventory(inventory_grid);
            audio_events.write(AudioEvent {
                sound: AudioType::CursorInteract,
                volume: 0.3,
            });
        }

        if ui.button("CLEAR FILTERS").clicked() {
            inventory_grid.filter = InventoryFilter::default();
        }

        ui.separator();

        ui.label("Weight: 45.2/100.0 kg");

        ui.separator();

        if let Some(selected) = inventory_grid.selected_slot {
            ui.label(format!("Selected: ({}, {})", selected.0, selected.1));
        }
    });
}

// === HELPER FUNCTIONS ===

fn handle_slot_drop(
    ui: &mut egui::Ui,
    inventory_grid: &mut InventoryGrid,
    position: (usize, usize),
    rect: egui::Rect,
) {
    if ui.input(|i| i.pointer.any_released()) {
        // Extract the dragged item data first, before any mutable operations
        let dragged_item_data = inventory_grid.dragging_item.as_ref().map(|dragged| {
            (dragged.item.clone(), dragged.origin)
        });

        if let Some((item, origin)) = dragged_item_data {
            if rect.contains(ui.input(|i| i.pointer.interact_pos().unwrap_or_default())) {
                // Attempt to place item
                if can_place_item_at(inventory_grid, &item, position) {
                    place_item(inventory_grid, item, position);
                    remove_item(inventory_grid, origin);
                }
            }
        }
        inventory_grid.dragging_item = None;
    }
}

fn can_place_item_at(
    inventory_grid: &InventoryGrid,
    _item: &InventoryItem,
    position: (usize, usize),
) -> bool {
    let (x, y) = position;
    x < inventory_grid.width && y < inventory_grid.height && inventory_grid.slots[y][x].is_none()
}

fn place_item(inventory_grid: &mut InventoryGrid, item: InventoryItem, position: (usize, usize)) {
    let (x, y) = position;
    inventory_grid.slots[y][x] = Some(InventorySlot {
        item,
        position,
        size: (1, 1), // Single slot for now
    });
}

fn remove_item(inventory_grid: &mut InventoryGrid, position: (usize, usize)) {
    let (x, y) = position;
    inventory_grid.slots[y][x] = None;
}

fn sort_inventory(inventory_grid: &mut InventoryGrid) {
    // Collect all items
    let mut items = Vec::new();
    for row in &mut inventory_grid.slots {
        for slot in row {
            if let Some(slot_item) = slot.take() {
                items.push(slot_item.item);
            }
        }
    }

    // Sort based on current mode
    match inventory_grid.sort_mode {
        SortMode::Name => items.sort_by(|a, b| a.name.cmp(&b.name)),
        SortMode::Type => items.sort_by_key(|item| format!("{:?}", item.item_type)),
        SortMode::Rarity => items.sort_by_key(|item| item.rarity as u8),
        SortMode::Value => items.sort_by(|a, b| b.value.cmp(&a.value)),
        SortMode::Weight => items.sort_by(|a, b| a.weight.partial_cmp(&b.weight).unwrap()),
        SortMode::RecentlyAcquired => {}, // TODO: Implement timestamp tracking
    }

    // Place items back in grid
    let mut item_index = 0;
    for y in 0..inventory_grid.height {
        for x in 0..inventory_grid.width {
            if item_index < items.len() {
                inventory_grid.slots[y][x] = Some(InventorySlot {
                    item: items[item_index].clone(),
                    position: (x, y),
                    size: (1, 1),
                });
                item_index += 1;
            }
        }
    }
}

fn save_current_loadout(loadout_manager: &mut LoadoutManager, inventory: &Inventory) {
    let preset = LoadoutPreset {
        name: format!("Loadout {}", loadout_manager.presets.len() + 1),
        weapon: inventory.equipped_weapon.clone(),
        tools: inventory.equipped_tools.clone(),
        cybernetics: inventory.cybernetics.clone(),
        consumables: HashMap::new(), // TODO: Add consumables to inventory
    };

    loadout_manager.presets.insert(preset.name.clone(), preset);
}

// === STARTUP SYSTEM ===

pub fn setup_enhanced_inventory(mut commands: Commands) {
    let inventory_grid = InventoryGrid {
        width: 10,
        height: 8,
        slots: vec![vec![None; 10]; 8],
        selected_slot: None,
        dragging_item: None,
        filter: InventoryFilter::default(),
        sort_mode: SortMode::Type,
        tab: InventoryTab::All,
    };

    commands.insert_resource(inventory_grid);
    commands.insert_resource(LoadoutManager::default());
}

impl Default for InventoryGrid {
    fn default() -> Self {
        Self {
            width: 10,
            height: 8,
            slots: vec![vec![None; 10]; 8],
            selected_slot: None,
            dragging_item: None,
            filter: InventoryFilter::default(),
            sort_mode: SortMode::Type,
            tab: InventoryTab::All,
        }
    }
}

impl Default for SortMode {
    fn default() -> Self {
        SortMode::Type
    }
}

impl Default for InventoryTab {
    fn default() -> Self {
        InventoryTab::All
    }
}