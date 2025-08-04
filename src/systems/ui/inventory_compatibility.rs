// src/systems/ui/inventory_compatibility.rs - Ensures backward compatibility
use bevy::prelude::*;
use crate::core::*;
use crate::systems::ui::enhanced_inventory::*;
use crate::systems::ui::inventory_integration::{weapon_description};
// === BACKWARD COMPATIBILITY FOR EXISTING INVENTORY COMPONENT ===

impl From<&Inventory> for Vec<InventoryItem> {
    fn from(inventory: &Inventory) -> Self {
        let mut items = Vec::new();

        // Convert weapons
        for weapon_config in &inventory.weapons {
            if let Some(item) = create_weapon_item_compat(weapon_config) {
                items.push(item);
            }
        }

        // Convert tools
        for tool in &inventory.tools {
            if let Some(item) = create_tool_item_compat(tool) {
                items.push(item);
            }
        }

        // Convert cybernetics
        for cybernetic in &inventory.cybernetics {
            if let Some(item) = create_cybernetic_item_compat(cybernetic) {
                items.push(item);
            }
        }

        // Convert access cards to items
        for inventory_item in &inventory.items {
            if let Some(item) = create_special_item(inventory_item) {
                items.push(item);
            }
        }

        items
    }
}

// Helper functions for compatibility
fn create_weapon_item_compat(weapon_config: &WeaponConfig) -> Option<InventoryItem> {

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

    let base_stats = ItemStats {
        damage: base_damage,
        accuracy: 0,
        range: weapon_config.behavior.preferred_range as i16,
        reload_speed: 0,
        armor: 0,
        stealth: 0,
        hacking: 0,
    };

    let rarity = determine_weapon_rarity(&weapon_config.base_weapon);

    Some(InventoryItem {
        id: format!("{:?}_{}", weapon_config.base_weapon, fastrand::u32(..)),
        name: weapon_name_display(&weapon_config.base_weapon),
        item_type: ItemType::Weapon(weapon_config.base_weapon),
        rarity,
        quantity: 1,
        max_stack: 1,
        stats: base_stats,
        description: weapon_description(&weapon_config.base_weapon),
        icon_path: format!("weapons/{:?}.png", weapon_config.base_weapon).to_lowercase(),
        weight: weapon_weight(&weapon_config.base_weapon),
        value: weapon_market_value(&weapon_config.base_weapon),
        is_favorited: false,
        is_locked: false,
    })
}

fn create_tool_item_compat(tool: &ToolType) -> Option<InventoryItem> {
    let (name, desc, rarity, weight, value, max_stack) = match tool {
        ToolType::Lockpick => ("Lockpick", "Electronic lock manipulation device", ItemRarity::Common, 0.1, 75, 1),
        ToolType::Scanner => ("Scanner", "Multi-spectrum detection scanner", ItemRarity::Uncommon, 0.5, 300, 1),
        ToolType::MedKit => ("Medical Kit", "Emergency field medical supplies", ItemRarity::Common, 0.8, 120, 3),
        ToolType::Grenade => ("Fragmentation Grenade", "High-explosive fragmentation device", ItemRarity::Uncommon, 0.4, 200, 5),
        ToolType::TimeBomb => ("Time Bomb", "Delayed detonation explosive", ItemRarity::Rare, 1.2, 800, 2),
        ToolType::Hacker => ("Hacking Device", "Electronic warfare toolkit", ItemRarity::Epic, 0.3, 2500, 1),
        ToolType::AdvancedHacker => ("Hacking Device", "Advanced electronic warfare toolkit", ItemRarity::Epic, 0.3, 3500, 1),
        ToolType::EnhancedSensors => ("Enhanced Sensors", "...", ItemRarity::Common, 0.3, 500, 1),
        ToolType::SatelliteUplink => ("Satellite Uplink", "...", ItemRarity::Uncommon, 0.3, 1000, 1),
        ToolType::TacticalScanner => ("Tactical Scanner", "...", ItemRarity::Rare, 0.3, 2000, 1),
        ToolType::NetworkScanner => ("Network Scanner", "...", ItemRarity::Epic, 0.3, 4500, 1),
        ToolType::DroneSwarm => ("Drone Swarm", "Creates a swarm of drones", ItemRarity::Common, 0.1, 100, 1),
        ToolType::QuantumComm => ("Quantum Communicator", "Non-interceptible quantum communication device", ItemRarity::Common, 0.1, 100, 1),
        _ => ("Unknown Device", "An unknown technological device", ItemRarity::Common, 0.1, 100, 1),
    };

    Some(InventoryItem {
        id: format!("{:?}_{}", tool, fastrand::u32(..)),
        name: name.to_string(),
        item_type: ItemType::Tool(*tool),
        rarity,
        quantity: 1,
        max_stack,
        stats: tool_stats(tool),
        description: desc.to_string(),
        icon_path: format!("tools/{:?}.png", tool).to_lowercase(),
        weight,
        value,
        is_favorited: false,
        is_locked: false,
    })
}

fn create_cybernetic_item_compat(cybernetic: &CyberneticType) -> Option<InventoryItem> {
    let (name, desc, stats, value) = match cybernetic {
        CyberneticType::Neurovector => (
            "Neurovector Implant",
            "Direct neural interface for remote consciousness manipulation",
            ItemStats { hacking: 25, stealth: 10, ..Default::default() },
            75000
        ),
        CyberneticType::NeuralInterface => (
            "Neural Scanner Interface",
            "Neural scanning interface for remote infrastructure scanning",
            ItemStats { hacking: 25, stealth: 10, ..Default::default() },
            75000
        ),
        CyberneticType::CombatEnhancer => (
            "Combat Enhancement Suite",
            "Integrated combat reflexes and damage amplification",
            ItemStats { damage: 30, accuracy: 15, ..Default::default() },
            45000
        ),
        CyberneticType::StealthModule => (
            "Stealth Infiltration Module",
            "Advanced cloaking and noise suppression systems",
            ItemStats { stealth: 35, hacking: 10, ..Default::default() },
            55000
        ),
        CyberneticType::TechInterface => (
            "Technical Interface Package",
            "Enhanced hacking capabilities and system integration",
            ItemStats { hacking: 40, ..Default::default() },
            65000
        ),
        CyberneticType::ArmorPlating => (
            "Subdermal Armor Plating",
            "Kevlar-titanium composite protection layer",
            ItemStats { armor: 40, ..Default::default() },
            25000
        ),
        CyberneticType::ReflexEnhancer => (
            "Reflex Enhancement System",
            "Neural acceleration for improved reaction times",
            ItemStats { accuracy: 25, reload_speed: 20, ..Default::default() },
            35000
        ),
        CyberneticType::OpticalCamo => (
            "Optical Camoflague",
            "Advanced optical cloaking system",
            ItemStats { stealth: 40, hacking: 10, ..Default::default() },
            35000
        ),
    };

    Some(InventoryItem {
        id: format!("{:?}_{}", cybernetic, fastrand::u32(..)),
        name: name.to_string(),
        item_type: ItemType::Cybernetic(*cybernetic),
        rarity: ItemRarity::Epic, // All cybernetics are epic
        quantity: 1,
        max_stack: 1,
        stats,
        description: desc.to_string(),
        icon_path: format!("cybernetics/{:?}.png", cybernetic).to_lowercase(),
        weight: 0.0, // Implanted items have no carry weight
        value,
        is_favorited: false,
        is_locked: true, // Cybernetics should be locked by default
    })
}

fn create_special_item(inventory_item: &crate::core::components::OriginalInventoryItem) -> Option<InventoryItem> {
    match inventory_item {
        crate::core::components::OriginalInventoryItem::AccessCard { level, card_type } => {
            Some(InventoryItem {
                id: format!("access_card_{}_{:?}", level, card_type),
                name: format!("Access Card Lv.{}", level),
                item_type: ItemType::Material,
                rarity: match level {
                    1..=2 => ItemRarity::Common,
                    3..=4 => ItemRarity::Uncommon,
                    5..=6 => ItemRarity::Rare,
                    _ => ItemRarity::Epic,
                },
                quantity: 1,
                max_stack: 10,
                stats: ItemStats::default(),
                description: format!("{:?} access authorization card", card_type),
                icon_path: "items/access_card.png".to_string(),
                weight: 0.01,
                value: *level as u32 * 100,
                is_favorited: false,
                is_locked: false,
            })
        },
        crate::core::components::OriginalInventoryItem::Keycard { access_level, facility_id } => {
            Some(InventoryItem {
                id: format!("keycard_{}_{}", facility_id, access_level),
                name: format!("Keycard: {}", facility_id),
                item_type: ItemType::Material,
                rarity: ItemRarity::Uncommon,
                quantity: 1,
                max_stack: 5,
                stats: ItemStats::default(),
                description: format!("Facility keycard for {}", facility_id),
                icon_path: "items/keycard.png".to_string(),
                weight: 0.01,
                value: *access_level as u32 * 200,
                is_favorited: false,
                is_locked: false,
            })
        }
    }
}

// === UTILITY FUNCTIONS ===

fn determine_weapon_rarity(weapon: &WeaponType) -> ItemRarity {
    match weapon {
        WeaponType::Pistol => ItemRarity::Common,
        WeaponType::Rifle | WeaponType::Shotgun => ItemRarity::Uncommon,
        WeaponType::Flamethrower | WeaponType::LaserRifle => ItemRarity::Rare,
        WeaponType::Minigun | WeaponType::GrenadeLauncher => ItemRarity::Epic,
        WeaponType::RocketLauncher | WeaponType::PlasmaGun => ItemRarity::Legendary,
    }
}

fn weapon_name_display(weapon: &WeaponType) -> String {
    match weapon {
        WeaponType::Pistol => "Combat Pistol".to_string(),
        WeaponType::Rifle => "Assault Rifle".to_string(),
        WeaponType::Shotgun => "Combat Shotgun".to_string(),
        WeaponType::Minigun => "Heavy weapon with devastating sustained firepower".to_string(),
        WeaponType::Flamethrower => "Area denial weapon that spreads burning fuel".to_string(),
        WeaponType::GrenadeLauncher => "Explosive projectile launcher for area damage".to_string(),
        WeaponType::RocketLauncher => "Anti-vehicle weapon with massive destructive power".to_string(),
        WeaponType::LaserRifle => "Energy-based weapon with pinpoint accuracy".to_string(),
        WeaponType::PlasmaGun => "Advanced energy weapon using superheated plasma".to_string(),
    }
}

fn weapon_weight(weapon: &WeaponType) -> f32 {
    match weapon {
        WeaponType::Pistol => 1.2,
        WeaponType::Rifle => 3.5,
        WeaponType::Shotgun => 3.8,
        WeaponType::Minigun => 18.5,
        WeaponType::Flamethrower => 12.0,
        WeaponType::GrenadeLauncher => 6.5,
        WeaponType::RocketLauncher => 15.2,
        WeaponType::LaserRifle => 4.2,
        WeaponType::PlasmaGun => 5.8,
    }
}

fn weapon_market_value(weapon: &WeaponType) -> u32 {
    match weapon {
        WeaponType::Pistol => 800,
        WeaponType::Rifle => 2200,
        WeaponType::Shotgun => 1800,
        WeaponType::Minigun => 45000,
        WeaponType::Flamethrower => 12000,
        WeaponType::GrenadeLauncher => 25000,
        WeaponType::RocketLauncher => 75000,
        WeaponType::LaserRifle => 35000,
        WeaponType::PlasmaGun => 85000,
    }
}

fn tool_stats(tool: &ToolType) -> ItemStats {
    match tool {
        ToolType::Lockpick => ItemStats { hacking: 5, ..Default::default() },
        ToolType::Scanner => ItemStats { hacking: 10, stealth: 5, ..Default::default() },
        ToolType::MedKit => ItemStats::default(),
        ToolType::Grenade => ItemStats { damage: 75, ..Default::default() },
        ToolType::TimeBomb => ItemStats { damage: 150, ..Default::default() },
        ToolType::Hacker => ItemStats { hacking: 20, ..Default::default() },
        ToolType::AdvancedHacker => ItemStats { hacking: 25, ..Default::default() },
        ToolType::EnhancedSensors => ItemStats {hacking: 30, ..Default::default() },
        ToolType::SatelliteUplink => ItemStats {hacking: 40, ..Default::default() },
        ToolType::TacticalScanner => ItemStats {hacking: 50, ..Default::default() },
        ToolType::NetworkScanner => ItemStats {hacking: 60, ..Default::default() },
        ToolType::DroneSwarm => ItemStats {hacking: 10, ..Default::default() },
        ToolType::QuantumComm => ItemStats {hacking: 10, ..Default::default() },
        _ => ItemStats {hacking: 10, ..Default::default() },
    }
}

// === INVENTORY MIGRATION SYSTEM ===

pub fn migrate_old_inventory_system(
    mut commands: Commands,
    mut inventory_grid: ResMut<InventoryGrid>,
    agent_query: Query<(Entity, &Inventory), (With<Agent>, Changed<Inventory>)>,
    inventory_state: Res<InventoryState>,
) {
    // This system helps migrate from the old inventory format to the new grid system
    // Run only when inventory changes to avoid performance issues

    for (entity, inventory) in agent_query.iter() {
        if Some(entity) == inventory_state.selected_agent {
            migrate_inventory_to_grid(&mut inventory_grid, inventory);
        }
    }
}

fn migrate_inventory_to_grid(grid: &mut InventoryGrid, inventory: &Inventory) {
    // Clear grid
    for row in &mut grid.slots {
        for slot in row {
            *slot = None;
        }
    }

    let items: Vec<InventoryItem> = inventory.into();
    let mut slot_index = 0;

    // Place items in grid
    for item in items {
        if slot_index < grid.width * grid.height {
            let x = slot_index % grid.width;
            let y = slot_index / grid.width;

            grid.slots[y][x] = Some(InventorySlot {
                item,
                position: (x, y),
                size: (1, 1),
            });

            slot_index += 1;
        }
    }
}

// === ENHANCED ITEM CREATION WITH ATTACHMENTS ===

pub fn create_weapon_with_attachments(weapon_config: &WeaponConfig) -> InventoryItem {
    let mut base_item = create_weapon_item_compat(weapon_config).unwrap();

    // Modify stats based on attachments
    let attachment_stats = weapon_config.stats();
    base_item.stats.accuracy += attachment_stats.accuracy as i16;
    base_item.stats.range += attachment_stats.range as i16;
    base_item.stats.reload_speed += attachment_stats.reload_speed as i16;

    // Update name to show attachments
    if !weapon_config.attachments.is_empty() {
        let attachment_count = weapon_config.attachments.len();
        base_item.name = format!("{} (+{})", base_item.name, attachment_count);

        // Upgrade rarity if heavily modified
        if attachment_count >= 3 {
            base_item.rarity = match base_item.rarity {
                ItemRarity::Common => ItemRarity::Uncommon,
                ItemRarity::Uncommon => ItemRarity::Rare,
                ItemRarity::Rare => ItemRarity::Epic,
                _ => base_item.rarity,
            };
        }

        // Increase value based on attachments
        base_item.value += attachment_count as u32 * 500;
    }

    base_item
}

// === SMART SORTING ALGORITHMS ===

pub fn smart_sort_inventory(grid: &mut InventoryGrid, sort_mode: SortMode) {
    let mut items = Vec::new();

    // Collect all items
    for row in &mut grid.slots {
        for slot in row {
            if let Some(slot_item) = slot.take() {
                items.push(slot_item.item);
            }
        }
    }

    // Apply sophisticated sorting
    match sort_mode {
        SortMode::Name => {
            items.sort_by(|a, b| a.name.cmp(&b.name));
        },
        SortMode::Type => {
            // Sort by type first, then by rarity within type
            items.sort_by(|a, b| {
                let type_order = type_sort_priority(&a.item_type).cmp(&type_sort_priority(&b.item_type));
                if type_order == std::cmp::Ordering::Equal {
                    (b.rarity as u8).cmp(&(a.rarity as u8)) // Higher rarity first
                } else {
                    type_order
                }
            });
        },
        SortMode::Rarity => {
            items.sort_by(|a, b| {
                let rarity_cmp = (b.rarity as u8).cmp(&(a.rarity as u8));
                if rarity_cmp == std::cmp::Ordering::Equal {
                    b.value.cmp(&a.value) // Higher value first if same rarity
                } else {
                    rarity_cmp
                }
            });
        },
        SortMode::Value => {
            items.sort_by(|a, b| b.value.cmp(&a.value));
        },
        SortMode::Weight => {
            items.sort_by(|a, b| a.weight.partial_cmp(&b.weight).unwrap_or(std::cmp::Ordering::Equal));
        },
        SortMode::RecentlyAcquired => {
            // For now, sort by ID (which includes timestamp)
            // In production, you'd track acquisition_time
            items.sort_by(|a, b| b.id.cmp(&a.id));
        },
    }

    // Special sorting: Favorites and locked items always on top
    items.sort_by(|a, b| {
        if a.is_favorited && !b.is_favorited {
            std::cmp::Ordering::Less
        } else if !a.is_favorited && b.is_favorited {
            std::cmp::Ordering::Greater
        } else if a.is_locked && !b.is_locked {
            std::cmp::Ordering::Less
        } else if !a.is_locked && b.is_locked {
            std::cmp::Ordering::Greater
        } else {
            std::cmp::Ordering::Equal
        }
    });

    // Place items back in grid
    let mut item_index = 0;
    for y in 0..grid.height {
        for x in 0..grid.width {
            if item_index < items.len() {
                grid.slots[y][x] = Some(InventorySlot {
                    item: items[item_index].clone(),
                    position: (x, y),
                    size: (1, 1),
                });
                item_index += 1;
            }
        }
    }
}

fn type_sort_priority(item_type: &ItemType) -> u8 {
    match item_type {
        ItemType::Weapon(_) => 0,
        ItemType::Attachment(_) => 1,
        ItemType::Tool(_) => 2,
        ItemType::Cybernetic(_) => 3,
        ItemType::Consumable => 4,
        ItemType::Material => 5,
        ItemType::Intel => 6,
        ItemType::Currency => 7,
    }
}

// === PERFORMANCE OPTIMIZATION ===

#[derive(Resource, Default)]
pub struct InventoryCache {
    pub last_inventory_hash: u64,
    pub cached_items: Vec<InventoryItem>,
    pub dirty: bool,
}

pub fn optimize_inventory_updates(
    mut cache: ResMut<InventoryCache>,
    agent_query: Query<&Inventory, (With<Agent>, Changed<Inventory>)>,
    inventory_state: Res<InventoryState>,
) {
    // Only update when inventory actually changes
    if let Some(agent_entity) = inventory_state.selected_agent {
        if let Ok(inventory) = agent_query.get(agent_entity) {
            // Simple hash of inventory state
            let current_hash = calculate_inventory_hash(inventory);

            if cache.last_inventory_hash != current_hash {
                cache.cached_items = inventory.into();
                cache.last_inventory_hash = current_hash;
                cache.dirty = true;
            }
        }
    }
}

fn calculate_inventory_hash(inventory: &Inventory) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();

    // Hash key inventory components
    inventory.currency.hash(&mut hasher);
    inventory.weapons.len().hash(&mut hasher);
    inventory.tools.len().hash(&mut hasher);
    inventory.cybernetics.len().hash(&mut hasher);

    // Hash equipped weapon if present
    if let Some(weapon) = &inventory.equipped_weapon {
        format!("{:?}", weapon.base_weapon).hash(&mut hasher);
        weapon.attachments.len().hash(&mut hasher);
    }

    hasher.finish()
}

// === ERROR HANDLING AND VALIDATION ===

pub fn validate_inventory_integrity(
    inventory_grid: Res<InventoryGrid>,
    mut commands: Commands,
) {
    // Validate grid integrity
    let mut total_items = 0;
    let mut duplicate_positions = std::collections::HashSet::new();

    for (y, row) in inventory_grid.slots.iter().enumerate() {
        for (x, slot) in row.iter().enumerate() {
            if let Some(slot_item) = slot {
                total_items += 1;

                // Check for position consistency
                if slot_item.position != (x, y) {
                    warn!("Inventory slot position mismatch at ({}, {})", x, y);
                }

                // Check for duplicates
                let pos_key = format!("{}_{}", x, y);
                if !duplicate_positions.insert(pos_key) {
                    error!("Duplicate item found at position ({}, {})", x, y);
                }

                // Validate item data
                if slot_item.item.quantity == 0 {
                    warn!("Item with zero quantity found: {}", slot_item.item.name);
                }

                if slot_item.item.quantity > slot_item.item.max_stack {
                    warn!("Item exceeds max stack size: {} ({})",
                          slot_item.item.name, slot_item.item.quantity);
                }
            }
        }
    }

    info!("Inventory validation complete: {} items found", total_items);
}