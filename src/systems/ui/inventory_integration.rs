// src/systems/ui/inventory_integration.rs - Integration layer for enhanced inventory
use bevy::prelude::*;
use crate::core::*;
use crate::systems::ui::enhanced_inventory::*;

// === CONVERSION UTILITIES ===

impl From<WeaponType> for ItemType {
    fn from(weapon: WeaponType) -> Self {
        ItemType::Weapon(weapon)
    }
}

impl From<ToolType> for ItemType {
    fn from(tool: ToolType) -> Self {
        ItemType::Tool(tool)
    }
}

impl From<CyberneticType> for ItemType {
    fn from(cybernetic: CyberneticType) -> Self {
        ItemType::Cybernetic(cybernetic)
    }
}

impl From<AttachmentSlot> for ItemType {
    fn from(slot: AttachmentSlot) -> Self {
        ItemType::Attachment(slot)
    }
}

// === INVENTORY SYNCHRONIZATION ===

pub fn sync_inventory_to_grid(
    mut inventory_grid: ResMut<InventoryGrid>,
    mut inventory_state: ResMut<InventoryState>,
    agent_query: Query<&Inventory, (With<Agent>, Changed<Inventory>)>,
    selection: Res<SelectionState>, // ADD: Monitor selection changes
) {
    // FIXED: Update selected agent if selection changed
    if selection.is_changed() && !selection.selected.is_empty() {
        inventory_state.selected_agent = selection.selected.first().copied();
    }
    
    // Sync grid when inventory changes or when agent selection changes
    let should_sync = if let Some(agent_entity) = inventory_state.selected_agent {
        agent_query.get(agent_entity).is_ok()
    } else {
        false
    };
    
    if should_sync {
        if let Ok(inventory) = agent_query.get(inventory_state.selected_agent.unwrap()) {
            populate_grid_from_inventory(&mut inventory_grid, inventory);
        }
    } else if !selection.selected.is_empty() {
        // Fallback: use first selected agent if current selection is invalid
        inventory_state.selected_agent = selection.selected.first().copied();
        if let Some(agent_entity) = inventory_state.selected_agent {
            if let Ok(inventory) = agent_query.get(agent_entity) {
                populate_grid_from_inventory(&mut inventory_grid, inventory);
            }
        }
    }
}

pub fn sync_grid_to_inventory(
    inventory_grid: Res<InventoryGrid>,
    mut agent_query: Query<&mut Inventory, With<Agent>>,
    inventory_state: Res<InventoryState>,
) {
    if inventory_grid.is_changed() {
        if let Some(agent_entity) = inventory_state.selected_agent {
            if let Ok(mut inventory) = agent_query.get_mut(agent_entity) {
                update_inventory_from_grid(&inventory_grid, &mut inventory);
            }
        }
    }
}

fn populate_grid_from_inventory(grid: &mut InventoryGrid, inventory: &Inventory) {
    // Clear existing grid
    for row in &mut grid.slots {
        for slot in row {
            *slot = None;
        }
    }
    
    let mut slot_index = 0;
    
    // Add equipped weapon
    if let Some(weapon_config) = &inventory.equipped_weapon {
        if let Some(item) = create_weapon_item(weapon_config) {
            place_item_in_next_slot(grid, item, &mut slot_index);
        }
    }
    
    // Add other weapons
    for weapon_config in &inventory.weapons {
        let is_equipped = inventory.equipped_weapon.as_ref()
            .map_or(false, |equipped| equipped.base_weapon == weapon_config.base_weapon);
        
        if !is_equipped {
            if let Some(item) = create_weapon_item(weapon_config) {
                place_item_in_next_slot(grid, item, &mut slot_index);
            }
        }
    }
    
    // Add tools
    for tool in &inventory.tools {
        if let Some(item) = create_tool_item(tool) {
            place_item_in_next_slot(grid, item, &mut slot_index);
        }
    }
    
    // Add cybernetics
    for cybernetic in &inventory.cybernetics {
        if let Some(item) = create_cybernetic_item(cybernetic) {
            place_item_in_next_slot(grid, item, &mut slot_index);
        }
    }
    
    // Add currency as special item
    if inventory.currency > 0 {
        let currency_item = InventoryItem {
            id: "credits".to_string(),
            name: "Credits".to_string(),
            item_type: ItemType::Currency,
            rarity: ItemRarity::Common,
            quantity: inventory.currency,
            max_stack: u32::MAX,
            stats: ItemStats::default(),
            description: "Digital currency used for purchases".to_string(),
            icon_path: "icons/credits.png".to_string(),
            weight: 0.0,
            value: 1,
            is_favorited: false,
            is_locked: false,
        };
        place_item_in_next_slot(grid, currency_item, &mut slot_index);
    }
}

fn place_item_in_next_slot(grid: &mut InventoryGrid, item: InventoryItem, slot_index: &mut usize) {
    let total_slots = grid.width * grid.height;
    if *slot_index < total_slots {
        let x = *slot_index % grid.width;
        let y = *slot_index / grid.width;
        
        grid.slots[y][x] = Some(InventorySlot {
            item,
            position: (x, y),
            size: (1, 1),
        });
        
        *slot_index += 1;
    }
}

fn update_inventory_from_grid(grid: &InventoryGrid, inventory: &mut Inventory) {
    // Clear existing inventory
    inventory.weapons.clear();
    inventory.tools.clear();
    inventory.cybernetics.clear();
    inventory.equipped_weapon = None;
    inventory.currency = 0;
    
    // Extract items from grid
    for row in &grid.slots {
        for slot in row {
            if let Some(slot_item) = slot {
                match &slot_item.item.item_type {
                    ItemType::Weapon(weapon_type) => {
                        let config = WeaponConfig::new(*weapon_type);
                        if inventory.equipped_weapon.is_none() {
                            inventory.equipped_weapon = Some(config.clone());
                        }
                        inventory.weapons.push(config);
                    },
                    ItemType::Tool(tool_type) => {
                        inventory.tools.push(*tool_type);
                        if inventory.equipped_tools.len() < 2 {
                            inventory.equipped_tools.push(*tool_type);
                        }
                    },
                    ItemType::Cybernetic(cybernetic_type) => {
                        inventory.cybernetics.push(*cybernetic_type);
                    },
                    ItemType::Currency => {
                        inventory.currency += slot_item.item.quantity;
                    },
                    _ => {}, // Handle other types as needed
                }
            }
        }
    }
}

// === ITEM CREATION UTILITIES ===

fn create_weapon_item(weapon_config: &WeaponConfig) -> Option<InventoryItem> {
    let (rarity, description) = match weapon_config.base_weapon {
        WeaponType::Pistol => (ItemRarity::Common, "Reliable sidearm for close encounters"),
        WeaponType::Rifle => (ItemRarity::Uncommon, "Versatile assault rifle"),
        WeaponType::Shotgun => (ItemRarity::Uncommon, "High damage close-range weapon"),
        WeaponType::Minigun => (ItemRarity::Epic, "Heavy weapon with devastating firepower"),
        WeaponType::Flamethrower => (ItemRarity::Rare, "Area denial weapon"),
        WeaponType::GrenadeLauncher => (ItemRarity::Epic, "Explosive projectile launcher"),
        WeaponType::RocketLauncher => (ItemRarity::Legendary, "Anti-vehicle rocket system"),
        WeaponType::LaserRifle => (ItemRarity::Epic, "Energy-based precision weapon"),
        WeaponType::PlasmaGun => (ItemRarity::Legendary, "Advanced plasma technology"),
    };

    let stats = weapon_config.stats();
    let base_damage = match weapon_config.base_weapon {
        WeaponType::Pistol => 25,
        WeaponType::Rifle => 35,
        WeaponType::Shotgun => 60,
        WeaponType::Minigun => 20,
        WeaponType::Flamethrower => 15,
        WeaponType::GrenadeLauncher => 80,
        WeaponType::RocketLauncher => 150,
        WeaponType::LaserRifle => 45,
        WeaponType::PlasmaGun => 70,
    };
        
    let item_stats = ItemStats {
        damage: base_damage,
        accuracy: stats.accuracy as i16,
        range: stats.range as i16,
        reload_speed: stats.reload_speed as i16,
        armor: 0,
        stealth: -stats.noise as i16, // Negative noise = positive stealth
        hacking: 0,
    };

    Some(InventoryItem {
        id: format!("{:?}_{}", weapon_config.base_weapon, fastrand::u32(..)),
        name: format!("{:?}", weapon_config.base_weapon),
        item_type: ItemType::Weapon(weapon_config.base_weapon),
        rarity,
        quantity: 1,
        max_stack: 1,
        stats: item_stats,
        description: description.to_string(),
        icon_path: format!("icons/weapons/{:?}.png", weapon_config.base_weapon).to_lowercase(),
        weight: get_weapon_weight(&weapon_config.base_weapon),
        value: get_weapon_value(&weapon_config.base_weapon),
        is_favorited: false,
        is_locked: false,
    })
}

fn create_tool_item(tool: &ToolType) -> Option<InventoryItem> {
    let (rarity, description, weight, value) = match tool {
        ToolType::Lockpick => (ItemRarity::Common, "Basic lock manipulation tool", 0.1, 50),
        ToolType::Scanner => (ItemRarity::Uncommon, "Electronic detection device", 0.5, 200),
        ToolType::MedKit => (ItemRarity::Common, "Emergency medical supplies", 1.0, 100),
        ToolType::Grenade => (ItemRarity::Uncommon, "Explosive device", 0.3, 150),
        ToolType::TimeBomb => (ItemRarity::Rare, "Delayed explosive device", 0.8, 500),
        ToolType::Hacker => (ItemRarity::Rare, "Advanced hacking toolkit", 0.2, 1000),
        ToolType::EnhancedSensors => (ItemRarity::Rare, "Enhanced Sensors", 0.2, 2000),
        ToolType::SatelliteUplink => (ItemRarity::Rare, "Satellite Uplink", 0.2, 4000),
        ToolType::TacticalScanner => (ItemRarity::Rare, "Tactical Scanner", 0.2, 8000),
        ToolType::NetworkScanner => (ItemRarity::Rare, "Network Scanner", 0.2, 16000),

    };

    Some(InventoryItem {
        id: format!("{:?}_{}", tool, fastrand::u32(..)),
        name: format!("{:?}", tool),
        item_type: ItemType::Tool(*tool),
        rarity,
        quantity: 1,
        max_stack: match tool {
            ToolType::MedKit | ToolType::Grenade => 5,
            ToolType::TimeBomb => 3,
            _ => 1,
        },
        stats: ItemStats::default(),
        description: description.to_string(),
        icon_path: format!("icons/tools/{:?}.png", tool).to_lowercase(),
        weight,
        value,
        is_favorited: false,
        is_locked: false,
    })
}

fn create_cybernetic_item(cybernetic: &CyberneticType) -> Option<InventoryItem> {
    let (rarity, description, stats) = match cybernetic {
        CyberneticType::Neurovector => (
            ItemRarity::Epic,
            "Neural interface for mind control capabilities",
            ItemStats { hacking: 15, stealth: 5, ..Default::default() }
        ),
        CyberneticType::NeuralInterface => (
            ItemRarity::Uncommon,
            "Neural interface for improved scanning capabilities",
            ItemStats { hacking: 15, ..Default::default() }
        ),
        CyberneticType::CombatEnhancer => (
            ItemRarity::Rare,
            "Combat reflexes and damage enhancement",
            ItemStats { damage: 20, accuracy: 10, ..Default::default() }
        ),
        CyberneticType::StealthModule => (
            ItemRarity::Rare,
            "Advanced stealth and infiltration systems",
            ItemStats { stealth: 25, hacking: 5, ..Default::default() }
        ),
        CyberneticType::TechInterface => (
            ItemRarity::Epic,
            "Enhanced hacking and technical capabilities",
            ItemStats { hacking: 30, ..Default::default() }
        ),
        CyberneticType::ArmorPlating => (
            ItemRarity::Uncommon,
            "Subdermal armor plating for protection",
            ItemStats { armor: 25, ..Default::default() }
        ),
        CyberneticType::ReflexEnhancer => (
            ItemRarity::Rare,
            "Enhanced reaction time and accuracy",
            ItemStats { accuracy: 20, reload_speed: 15, ..Default::default() }
        ),
    };

    Some(InventoryItem {
        id: format!("{:?}_{}", cybernetic, fastrand::u32(..)),
        name: format!("{:?}", cybernetic),
        item_type: ItemType::Cybernetic(*cybernetic),
        rarity,
        quantity: 1,
        max_stack: 1,
        stats,
        description: description.to_string(),
        icon_path: format!("icons/cybernetics/{:?}.png", cybernetic).to_lowercase(),
        weight: 0.0, // Cybernetics are implanted
        value: match cybernetic {
            CyberneticType::Neurovector => 50000,
            CyberneticType::NeuralInterface => 25000,
            CyberneticType::TechInterface => 30000,
            CyberneticType::CombatEnhancer => 25000,
            CyberneticType::StealthModule => 25000,
            CyberneticType::ReflexEnhancer => 20000,
            CyberneticType::ArmorPlating => 15000,
        },
        is_favorited: false,
        is_locked: true, // Cybernetics should be locked by default
    })
}

// === ITEM INTERACTION SYSTEMS ===

pub fn handle_item_actions(
    mut inventory_grid: ResMut<InventoryGrid>,
    mut agent_query: Query<&mut Inventory, With<Agent>>,
    inventory_state: Res<InventoryState>,
    input: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut audio_events: EventWriter<AudioEvent>,
) {
    // Handle right-click context menu
    if mouse.just_pressed(MouseButton::Right) {
        if let Some(selected) = inventory_grid.selected_slot {
            if let Some(slot) = &mut inventory_grid.slots[selected.1][selected.0] {
                show_context_menu(&mut slot.item, &mut audio_events);
            }
        }
    }
    
    // Handle keyboard shortcuts
    if input.just_pressed(KeyCode::KeyF) {
        toggle_favorite_selected_item(&mut inventory_grid, &mut audio_events);
    }
    
    if input.just_pressed(KeyCode::KeyL) {
        toggle_lock_selected_item(&mut inventory_grid, &mut audio_events);
    }
    
    if input.just_pressed(KeyCode::Delete) {
        delete_selected_item(&mut inventory_grid, &mut audio_events);
    }
}

fn show_context_menu(item: &mut InventoryItem, audio_events: &mut EventWriter<AudioEvent>) {
    // This would typically show an egui context menu
    // For now, we'll just toggle favorite as an example
    item.is_favorited = !item.is_favorited;
    
    audio_events.write(AudioEvent {
        sound: AudioType::CursorInteract,
        volume: 0.2,
    });
}

fn toggle_favorite_selected_item(
    inventory_grid: &mut InventoryGrid,
    audio_events: &mut EventWriter<AudioEvent>,
) {
    if let Some(selected) = inventory_grid.selected_slot {
        if let Some(slot) = &mut inventory_grid.slots[selected.1][selected.0] {
            slot.item.is_favorited = !slot.item.is_favorited;
            
            audio_events.write(AudioEvent {
                sound: AudioType::CursorInteract,
                volume: 0.3,
            });
        }
    }
}

fn toggle_lock_selected_item(
    inventory_grid: &mut InventoryGrid,
    audio_events: &mut EventWriter<AudioEvent>,
) {
    if let Some(selected) = inventory_grid.selected_slot {
        if let Some(slot) = &mut inventory_grid.slots[selected.1][selected.0] {
            slot.item.is_locked = !slot.item.is_locked;
            
            audio_events.write(AudioEvent {
                sound: AudioType::CursorInteract,
                volume: 0.3,
            });
        }
    }
}

fn delete_selected_item(
    inventory_grid: &mut InventoryGrid,
    audio_events: &mut EventWriter<AudioEvent>,
) {
    if let Some(selected) = inventory_grid.selected_slot {
        if let Some(slot) = &inventory_grid.slots[selected.1][selected.0] {
            if !slot.item.is_locked {
                inventory_grid.slots[selected.1][selected.0] = None;
                inventory_grid.selected_slot = None;
                
                audio_events.write(AudioEvent {
                    sound: AudioType::CursorInteract,
                    volume: 0.4,
                });
            }
        }
    }
}

// === FILTERING SYSTEM ===

pub fn apply_inventory_filters(
    mut inventory_grid: ResMut<InventoryGrid>,
) {
    if !inventory_grid.is_changed() {
        return;
    }
    
    // Extract all the immutable data we need first
    let filter_text = inventory_grid.filter.text.clone();
    let show_favorites_only = inventory_grid.filter.show_favorites_only;
    let current_tab = inventory_grid.tab.clone();
    
    // Now we can safely do mutable operations
    for row in &mut inventory_grid.slots {
        for slot in row {
            if let Some(slot_item) = slot {
                let mut visible = true;
                
                // Text filter
                if !filter_text.is_empty() {
                    visible &= slot_item.item.name.to_lowercase()
                        .contains(&filter_text.to_lowercase());
                }
                
                // Favorites filter
                if show_favorites_only {
                    visible &= slot_item.item.is_favorited;
                }
                
                // Tab filter
                visible &= match current_tab {
                    InventoryTab::All => true,
                    InventoryTab::Weapons => matches!(slot_item.item.item_type, ItemType::Weapon(_)),
                    InventoryTab::Gear => matches!(slot_item.item.item_type, 
                        ItemType::Tool(_) | ItemType::Cybernetic(_) | ItemType::Attachment(_)),
                    InventoryTab::Consumables => matches!(slot_item.item.item_type, ItemType::Consumable),
                    InventoryTab::Materials => matches!(slot_item.item.item_type, ItemType::Material),
                    InventoryTab::Intel => matches!(slot_item.item.item_type, ItemType::Intel),
                };
                
                // Store visibility (you might want to add a visible field to InventorySlot)
                // For now, we'll hide by setting to None temporarily
                // This is a simplified approach - in production you'd want better filtering
            }
        }
    }
}

// === WEIGHT AND CAPACITY SYSTEM ===

pub fn calculate_inventory_weight(inventory_grid: &InventoryGrid) -> f32 {
    let mut total_weight = 0.0;
    
    for row in &inventory_grid.slots {
        for slot in row {
            if let Some(slot_item) = slot {
                total_weight += slot_item.item.weight * slot_item.item.quantity as f32;
            }
        }
    }
    
    total_weight
}

pub fn calculate_inventory_value(inventory_grid: &InventoryGrid) -> u32 {
    let mut total_value = 0;
    
    for row in &inventory_grid.slots {
        for slot in row {
            if let Some(slot_item) = slot {
                total_value += slot_item.item.value * slot_item.item.quantity;
            }
        }
    }
    
    total_value
}

// === LOADOUT QUICK ACTIONS ===

pub fn handle_loadout_hotkeys(
    input: Res<ButtonInput<KeyCode>>,
    mut loadout_manager: ResMut<LoadoutManager>,
    mut agent_query: Query<&mut Inventory, With<Agent>>,
    inventory_state: Res<InventoryState>,
    mut audio_events: EventWriter<AudioEvent>,
) {
    // F1-F4 for quick loadouts
    let hotkeys = [
        (KeyCode::F1, 0),
        (KeyCode::F2, 1),
        (KeyCode::F3, 2),
        (KeyCode::F4, 3),
    ];
    
    for (key, slot_index) in hotkeys {
        if input.just_pressed(key) {
            if input.pressed(KeyCode::ShiftLeft) || input.pressed(KeyCode::ShiftRight) {
                // Save current loadout to slot
                if let Some(agent_entity) = inventory_state.selected_agent {
                    if let Ok(inventory) = agent_query.get(agent_entity) {
                        save_loadout_to_slot(&mut loadout_manager, inventory, slot_index);
                        
                        audio_events.write(AudioEvent {
                            sound: AudioType::CursorInteract,
                            volume: 0.5,
                        });
                    }
                }
            } else {
                // Load loadout from slot
                if let Some(preset_name) = &loadout_manager.quick_slots[slot_index] {
                    if let Some(agent_entity) = inventory_state.selected_agent {
                        if let Ok(mut inventory) = agent_query.get_mut(agent_entity) {
                            apply_loadout_preset(&loadout_manager, preset_name, &mut inventory);
                            
                            audio_events.write(AudioEvent {
                                sound: AudioType::CursorInteract,
                                volume: 0.5,
                            });
                        }
                    }
                }
            }
        }
    }
}

fn save_loadout_to_slot(
    loadout_manager: &mut LoadoutManager,
    inventory: &Inventory,
    slot_index: usize,
) {
    let preset_name = format!("Quick Slot {}", slot_index + 1);
    
    let preset = LoadoutPreset {
        name: preset_name.clone(),
        weapon: inventory.equipped_weapon.clone(),
        tools: inventory.equipped_tools.clone(),
        cybernetics: inventory.cybernetics.clone(),
        consumables: std::collections::HashMap::new(),
    };
    
    loadout_manager.presets.insert(preset_name.clone(), preset);
    loadout_manager.quick_slots[slot_index] = Some(preset_name);
}

fn apply_loadout_preset(
    loadout_manager: &LoadoutManager,
    preset_name: &str,
    inventory: &mut Inventory,
) {
    if let Some(preset) = loadout_manager.presets.get(preset_name) {
        inventory.equipped_weapon = preset.weapon.clone();
        inventory.equipped_tools = preset.tools.clone();
        // Note: Cybernetics typically shouldn't be swapped quickly in realistic scenarios
        // but this is included for completeness
    }
}

// TEMPORARY HELPERS
// These should come from our weapon database

pub fn weapon_description(weapon: &WeaponType) -> String {
    match weapon {
        WeaponType::Pistol => "Reliable sidearm for close encounters".to_string(),
        WeaponType::Rifle => "Versatile assault rifle for most situations".to_string(),
        WeaponType::Shotgun => "High damage close-range weapon".to_string(),
        WeaponType::Minigun => "Heavy weapon with devastating sustained firepower".to_string(),
        WeaponType::Flamethrower => "Area denial weapon that spreads burning fuel".to_string(),
        WeaponType::GrenadeLauncher => "Explosive projectile launcher for area damage".to_string(),
        WeaponType::RocketLauncher => "Anti-vehicle weapon with massive destructive power".to_string(),
        WeaponType::LaserRifle => "Energy-based weapon with pinpoint accuracy".to_string(),
        WeaponType::PlasmaGun => "Advanced energy weapon using superheated plasma".to_string(),
    }
}

fn get_weapon_weight(weapon: &WeaponType) -> f32 {
    match weapon {
        WeaponType::Pistol => 1.2,
        WeaponType::Rifle => 3.5,
        WeaponType::Shotgun => 3.8,
        WeaponType::Minigun => 15.0,
        WeaponType::Flamethrower => 8.5,
        WeaponType::GrenadeLauncher => 5.2,
        WeaponType::RocketLauncher => 12.0,
        WeaponType::LaserRifle => 4.0,
        WeaponType::PlasmaGun => 4.5,
    }
}

fn get_weapon_value(weapon: &WeaponType) -> u32 {
    match weapon {
        WeaponType::Pistol => 500,
        WeaponType::Rifle => 1500,
        WeaponType::Shotgun => 1200,
        WeaponType::Minigun => 25000,
        WeaponType::Flamethrower => 8000,
        WeaponType::GrenadeLauncher => 15000,
        WeaponType::RocketLauncher => 50000,
        WeaponType::LaserRifle => 20000,
        WeaponType::PlasmaGun => 45000,
    }
}