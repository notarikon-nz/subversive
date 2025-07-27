// src/systems/ui/screens.rs - All the screen UIs updated for Bevy 0.16
use bevy::prelude::*;
use crate::systems::ui::*;

// Add a marker for inventory UI refresh
#[derive(Resource, Default)]
pub struct InventoryUIState {
    pub needs_refresh: bool,
    pub last_selected_agent: Option<Entity>,
}

// Re-export components for compatibility
#[derive(Component)]
pub struct InventoryUI;

// Improved inventory system with proper change detection
pub fn inventory_system(
    mut commands: Commands,
    mut inventory_ui_state: ResMut<InventoryUIState>,
    inventory_state: Res<InventoryState>,
    agent_query: Query<(&Inventory, &WeaponState), With<Agent>>,
    changed_inventory_query: Query<Entity, (With<Agent>, Changed<Inventory>)>,
    changed_weapon_query: Query<Entity, (With<Agent>, Changed<WeaponState>)>,
    inventory_ui_query: Query<Entity, (With<InventoryUI>, Without<MarkedForDespawn>)>,
) {
    // Check if we need to close the UI
    if !inventory_state.ui_open {
        if !inventory_ui_query.is_empty() {
            for entity in inventory_ui_query.iter() {
                commands.entity(entity).insert(MarkedForDespawn);
            }
            inventory_ui_state.needs_refresh = false;
        }
        return;
    }

    // Check for various update triggers
    let mut needs_update = inventory_ui_query.is_empty(); // UI doesn't exist
    
    // Check if selected agent changed
    if inventory_ui_state.last_selected_agent != inventory_state.selected_agent {
        inventory_ui_state.last_selected_agent = inventory_state.selected_agent;
        needs_update = true;
    }
    
    // Check if any agent's inventory changed
    if !changed_inventory_query.is_empty() {
        if let Some(selected) = inventory_state.selected_agent {
            if changed_inventory_query.contains(selected) {
                needs_update = true;
            }
        }
    }
    
    // Check if any agent's weapon state changed
    if !changed_weapon_query.is_empty() {
        if let Some(selected) = inventory_state.selected_agent {
            if changed_weapon_query.contains(selected) {
                needs_update = true;
            }
        }
    }
    
    // Manual refresh flag
    if inventory_ui_state.needs_refresh {
        needs_update = true;
        inventory_ui_state.needs_refresh = false;
    }
    
    if needs_update {
        // Clean up existing UI
        for entity in inventory_ui_query.iter() {
            commands.entity(entity).insert(MarkedForDespawn);
        }
        
        // Get current agent data
        let agent_data = inventory_state.selected_agent
            .and_then(|agent| agent_query.get(agent).ok());
        
        create_modern_inventory_ui(&mut commands, agent_data);
    }
}

// Modern Division 2-style inventory UI
fn create_modern_inventory_ui(
    commands: &mut Commands, 
    agent_data: Option<(&Inventory, &WeaponState)>
) {
    commands.spawn((
        Node {
            width: Val::Percent(50.0), // Right half of screen
            height: Val::Percent(100.0),
            position_type: PositionType::Absolute,
            right: Val::Px(0.0),
            top: Val::Px(0.0),
            flex_direction: FlexDirection::Column,
            padding: UiRect::all(Val::Px(20.0)),
            row_gap: Val::Px(15.0),
            ..default()
        },
        BackgroundColor(Color::srgba(0.05, 0.05, 0.08, 0.95)),
        ZIndex(100),
        InventoryUI,
    )).with_children(|parent| {
        
        // Header section
        create_inventory_header(parent);
        
        // Main content area
        parent.spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(80.0),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(20.0),
                overflow: Overflow::clip_y(),
                ..default()
            },
        )).with_children(|content| {
            
            if let Some((inventory, weapon_state)) = agent_data {
                create_agent_stats_section(content, inventory);
                create_weapon_section(content, inventory, weapon_state);
                create_equipment_section(content, inventory);
                create_consumables_section(content, inventory);
            } else {
                content.spawn((
                    Text::new("No agent selected"),
                    TextFont { font_size: 24.0, ..default() },
                    TextColor(Color::srgb(0.7, 0.3, 0.3)),
                ));
            }
        });
        
        // Footer with controls
        create_inventory_footer(parent);
    });
}

fn create_inventory_header(parent: &mut ChildSpawnerCommands) {
    parent.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Px(60.0),
            justify_content: JustifyContent::SpaceBetween,
            align_items: AlignItems::Center,
            border: UiRect::bottom(Val::Px(2.0)),
            ..default()
        },
        BorderColor(Color::srgb(0.3, 0.3, 0.4)),
    )).with_children(|header| {
        header.spawn((
            Text::new("AGENT INVENTORY"),
            TextFont { 
                font_size: 28.0, 
                ..default() 
            },
            TextColor(Color::srgb(0.9, 0.9, 0.9)),
        ));
        
        header.spawn((
            Text::new("[I] CLOSE"),
            TextFont { 
                font_size: 16.0, 
                ..default() 
            },
            TextColor(Color::srgb(0.6, 0.6, 0.6)),
        ));
    });
}

fn create_agent_stats_section(parent: &mut ChildSpawnerCommands, inventory: &Inventory) {
    parent.spawn((
        Node {
            width: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(8.0),
            padding: UiRect::all(Val::Px(15.0)),
            border: UiRect::all(Val::Px(1.0)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.1, 0.1, 0.15, 0.8)),
        BorderColor(Color::srgb(0.2, 0.2, 0.3)),
    )).with_children(|section| {
        section.spawn((
            Text::new("AGENT STATUS"),
            TextFont { font_size: 18.0, ..default() },
            TextColor(Color::srgb(0.8, 0.8, 0.9)),
        ));
        
        section.spawn((
            Text::new(format!("CREDITS: {}", inventory.currency)),
            TextFont { font_size: 16.0, ..default() },
            TextColor(Color::srgb(0.8, 0.8, 0.2)),
        ));
        
        // Add health, stress, etc. here when available
    });
}

fn create_weapon_section(parent: &mut ChildSpawnerCommands, inventory: &Inventory, weapon_state: &WeaponState) {
    parent.spawn((
        Node {
            width: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(8.0),
            padding: UiRect::all(Val::Px(15.0)),
            border: UiRect::all(Val::Px(1.0)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.1, 0.1, 0.15, 0.8)),
        BorderColor(Color::srgb(0.2, 0.2, 0.3)),
    )).with_children(|section| {
        section.spawn((
            Text::new("PRIMARY WEAPON"),
            TextFont { font_size: 18.0, ..default() },
            TextColor(Color::srgb(0.8, 0.8, 0.9)),
        ));
        
        if let Some(weapon_config) = &inventory.equipped_weapon {
            // Weapon name and type
            section.spawn((
                Text::new(format!("{:?}", weapon_config.base_weapon)),
                TextFont { font_size: 16.0, ..default() },
                TextColor(Color::srgb(0.9, 0.7, 0.3)),
            ));
            
            // Ammo status with color coding
            let ammo_color = match weapon_state.current_ammo {
                0 => Color::srgb(0.8, 0.2, 0.2),
                n if n <= weapon_state.max_ammo / 4 => Color::srgb(0.8, 0.6, 0.2),
                _ => Color::srgb(0.2, 0.8, 0.2),
            };
            
            let reload_text = if weapon_state.is_reloading {
                format!(" (Reloading: {:.1}s)", weapon_state.reload_timer)
            } else {
                String::new()
            };
            
            section.spawn((
                Text::new(format!("AMMO: {}/{}{}", 
                    weapon_state.current_ammo, 
                    weapon_state.max_ammo,
                    reload_text
                )),
                TextFont { font_size: 14.0, ..default() },
                TextColor(ammo_color),
            ));
            
            // Weapon stats
            let stats = weapon_config.stats();
            if stats.accuracy != 0 || stats.range != 0 || stats.noise != 0 {
                section.spawn((
                    Text::new(format!("MODS: Accuracy{:+} Range{:+} Noise{:+}", 
                        stats.accuracy, stats.range, stats.noise
                    )),
                    TextFont { font_size: 12.0, ..default() },
                    TextColor(Color::srgb(0.6, 0.8, 0.6)),
                ));
            }
            
            // Attachments list
            if !weapon_config.attachments.is_empty() {
                for (slot, attachment) in &weapon_config.attachments {
                    section.spawn((
                        Text::new(format!("└ {:?}: {}", slot, attachment.name)),
                        TextFont { font_size: 12.0, ..default() },
                        TextColor(Color::srgb(0.7, 0.7, 0.9)),
                    ));
                }
            } else {
                section.spawn((
                    Text::new("└ No attachments"),
                    TextFont { font_size: 12.0, ..default() },
                    TextColor(Color::srgb(0.5, 0.5, 0.5)),
                ));
            }
        } else {
            section.spawn((
                Text::new("No weapon equipped"),
                TextFont { font_size: 16.0, ..default() },
                TextColor(Color::srgb(0.7, 0.3, 0.3)),
            ));
        }
    });
}

fn create_equipment_section(parent: &mut ChildSpawnerCommands, inventory: &Inventory) {
    parent.spawn((
        Node {
            width: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(8.0),
            padding: UiRect::all(Val::Px(15.0)),
            border: UiRect::all(Val::Px(1.0)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.1, 0.1, 0.15, 0.8)),
        BorderColor(Color::srgb(0.2, 0.2, 0.3)),
    )).with_children(|section| {
        section.spawn((
            Text::new("EQUIPMENT"),
            TextFont { font_size: 18.0, ..default() },
            TextColor(Color::srgb(0.8, 0.8, 0.9)),
        ));
        
        if !inventory.equipped_tools.is_empty() {
            for tool in &inventory.equipped_tools {
                section.spawn((
                    Text::new(format!("└ {:?}", tool)),
                    TextFont { font_size: 14.0, ..default() },
                    TextColor(Color::srgb(0.3, 0.8, 0.3)),
                ));
            }
        } else {
            section.spawn((
                Text::new("No equipment"),
                TextFont { font_size: 14.0, ..default() },
                TextColor(Color::srgb(0.5, 0.5, 0.5)),
            ));
        }
        
        // Show cybernetics if any
        if !inventory.cybernetics.is_empty() {
            section.spawn((
                Text::new("CYBERNETICS:"),
                TextFont { font_size: 14.0, ..default() },
                TextColor(Color::srgb(0.6, 0.8, 0.9)),
            ));
            
            for cybernetic in &inventory.cybernetics {
                // Convert CyberneticType to string representation
                let cybernetic_name = match cybernetic {
                    CyberneticType::Neurovector => "Neurovector",
                    CyberneticType::CombatEnhancer => "Combat Enhancer",
                    CyberneticType::StealthModule => "Stealth Module",
                    CyberneticType::TechInterface => "Hacking Booster",
                    CyberneticType::ArmorPlating => "Armor Plating",
                    CyberneticType::ReflexEnhancer => "Reflex Enhancer",
                };
                
                section.spawn((
                    Text::new(format!("└ {}", cybernetic_name)),
                    TextFont { font_size: 12.0, ..default() },
                    TextColor(Color::srgb(0.5, 0.7, 0.9)),
                ));
            }
        }
    });
}

fn create_consumables_section(parent: &mut ChildSpawnerCommands, inventory: &Inventory) {
    parent.spawn((
        Node {
            width: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(8.0),
            padding: UiRect::all(Val::Px(15.0)),
            border: UiRect::all(Val::Px(1.0)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.1, 0.1, 0.15, 0.8)),
        BorderColor(Color::srgb(0.2, 0.2, 0.3)),
    )).with_children(|section| {
        section.spawn((
            Text::new("INTEL & DATA"),
            TextFont { font_size: 18.0, ..default() },
            TextColor(Color::srgb(0.8, 0.8, 0.9)),
        ));
        
        // Show collected intel if inventory has intel field
        // For now, show placeholder since intel field doesn't exist
        section.spawn((
            Text::new("📄 Terminal Data Logs: 3"),
            TextFont { font_size: 14.0, ..default() },
            TextColor(Color::srgb(0.8, 0.8, 0.6)),
        ));
        
        section.spawn((
            Text::new("📊 Corporate Files: 1"),
            TextFont { font_size: 14.0, ..default() },
            TextColor(Color::srgb(0.8, 0.8, 0.6)),
        ));
        
        section.spawn((
            Text::new("🔍 Mission Intel: Available"),
            TextFont { font_size: 14.0, ..default() },
            TextColor(Color::srgb(0.6, 0.8, 0.8)),
        ));
        
        // Note: Once intel field is added to Inventory, replace above with:
        // if inventory.intel.len() > 0 {
        //     for intel_item in &inventory.intel {
        //         // Display intel items
        //     }
        // } else {
        //     section.spawn(("No intel collected", ...));
        // }
    });
}

fn create_inventory_footer(parent: &mut ChildSpawnerCommands) {
    parent.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Px(80.0),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Center,
            padding: UiRect::all(Val::Px(10.0)),
            border: UiRect::top(Val::Px(1.0)),
            ..default()
        },
        BorderColor(Color::srgb(0.3, 0.3, 0.4)),
    )).with_children(|footer| {
        footer.spawn((
            Text::new("CONTROLS"),
            TextFont { font_size: 14.0, ..default() },
            TextColor(Color::srgb(0.7, 0.7, 0.7)),
        ));
        
        footer.spawn((
            Text::new("[I] Close • [R] Reload • [TAB] Next Agent • [M] Manufacture"),
            TextFont { font_size: 12.0, ..default() },
            TextColor(Color::srgb(0.6, 0.6, 0.6)),
        ));
    });
}