// src/systems/ui/hub/manufacture.rs - Complex manufacture tab separated out
use bevy::prelude::*;
use crate::core::*;

pub fn handle_input(
    input: &ButtonInput<KeyCode>,
    hub_state: &mut HubState,
    manufacture_state: &mut ManufactureState,
    global_data: &mut GlobalData,
    mut agent_query: Query<&mut Inventory, With<Agent>>,
    attachment_db: &AttachmentDatabase,
) -> bool {
    let mut needs_rebuild = false;
    
    // Navigate agents with 1-3 keys
    if input.just_pressed(KeyCode::Digit1) {
        manufacture_state.selected_agent_idx = 0;
        manufacture_state.selected_weapon_idx = 0;
        manufacture_state.selected_slot = None;
        manufacture_state.selected_attachments.clear();
        needs_rebuild = true;
        info!("Selected Agent 1");
    }
    if input.just_pressed(KeyCode::Digit2) {
        manufacture_state.selected_agent_idx = 1;
        manufacture_state.selected_weapon_idx = 0;
        manufacture_state.selected_slot = None;
        manufacture_state.selected_attachments.clear();
        needs_rebuild = true;
        info!("Selected Agent 2");
    }
    if input.just_pressed(KeyCode::Digit3) {
        manufacture_state.selected_agent_idx = 2;
        manufacture_state.selected_weapon_idx = 0;
        manufacture_state.selected_slot = None;
        manufacture_state.selected_attachments.clear();
        needs_rebuild = true;
        info!("Selected Agent 3");
    }
    
    // Navigate weapon slots with UP/DOWN
    if input.just_pressed(KeyCode::ArrowUp) || input.just_pressed(KeyCode::ArrowDown) {
        cycle_selection(manufacture_state, input.just_pressed(KeyCode::ArrowDown));
        needs_rebuild = true;
    }
    
    // Navigate attachments within slot with LEFT/RIGHT
    if input.just_pressed(KeyCode::ArrowLeft) || input.just_pressed(KeyCode::ArrowRight) {
        cycle_attachment_selection(manufacture_state, input.just_pressed(KeyCode::ArrowRight), attachment_db, &UnlockedAttachments::default());
        needs_rebuild = true;
    }
    
    // Attach/Detach with Enter
    if input.just_pressed(KeyCode::Enter) {
        execute_attachment_action(manufacture_state, global_data, &mut agent_query, attachment_db);
        needs_rebuild = true;
        info!("Processing attachment action");
    }
    
    // Back to agents with Backspace
    if input.just_pressed(KeyCode::Backspace) {
        hub_state.active_tab = super::HubTab::Agents;
        needs_rebuild = true;
    }
    
    needs_rebuild
}

pub fn create_content(
    parent: &mut ChildBuilder, 
    global_data: &GlobalData,
    manufacture_state: &ManufactureState,
    attachment_db: &AttachmentDatabase,
    unlocked: &UnlockedAttachments,
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
        content.spawn(TextBundle::from_section(
            "WEAPON MANUFACTURE",
            TextStyle { font_size: 24.0, color: Color::srgb(0.8, 0.6, 0.2), ..default() }
        ));
        
        // Agent selection display
        content.spawn(NodeBundle {
            style: Style {
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(20.0),
                margin: UiRect::top(Val::Px(10.0)),
                ..default()
            },
            ..default()
        }).with_children(|agents| {
            for i in 0..3 {
                let is_selected = i == manufacture_state.selected_agent_idx;
                let color = if is_selected { Color::srgb(0.2, 0.8, 0.2) } else { Color::srgb(0.6, 0.6, 0.6) };
                let prefix = if is_selected { "> " } else { "  " };
                
                agents.spawn(TextBundle::from_section(
                    format!("{}Agent {} (Lv{})", prefix, i + 1, global_data.agent_levels[i]),
                    TextStyle { font_size: 16.0, color, ..default() }
                ));
            }
        });
        
        // Weapon slots display
        content.spawn(NodeBundle {
            style: Style {
                flex_direction: FlexDirection::Column,
                margin: UiRect::top(Val::Px(20.0)),
                padding: UiRect::all(Val::Px(10.0)),
                row_gap: Val::Px(8.0),
                ..default()
            },
            background_color: Color::srgba(0.2, 0.2, 0.3, 0.3).into(),
            ..default()
        }).with_children(|weapon_panel| {
            weapon_panel.spawn(TextBundle::from_section(
                "CURRENT WEAPON: Rifle", // TODO: Get from agent inventory
                TextStyle { font_size: 18.0, color: Color::WHITE, ..default() }
            ));
            
            let slots = vec![
                ("Sight", AttachmentSlot::Sight),
                ("Barrel", AttachmentSlot::Barrel),
                ("Magazine", AttachmentSlot::Magazine),
                ("Grip", AttachmentSlot::Grip),
                ("Stock", AttachmentSlot::Stock),
            ];
            
            for (slot_name, slot) in slots {
                let is_selected = manufacture_state.selected_slot.as_ref() == Some(&slot);
                let color = if is_selected { Color::srgb(0.8, 0.8, 0.2) } else { Color::WHITE };
                let prefix = if is_selected { "> " } else { "  " };
                
                weapon_panel.spawn(TextBundle::from_section(
                    format!("{}{}: None equipped", prefix, slot_name),
                    TextStyle { font_size: 14.0, color, ..default() }
                ));
            }
        });
        
        // Available attachments for selected slot
        if let Some(selected_slot) = &manufacture_state.selected_slot {
            content.spawn(NodeBundle {
                style: Style {
                    flex_direction: FlexDirection::Column,
                    margin: UiRect::top(Val::Px(20.0)),
                    padding: UiRect::all(Val::Px(10.0)),
                    row_gap: Val::Px(5.0),
                    ..default()
                },
                background_color: Color::srgba(0.3, 0.2, 0.2, 0.3).into(),
                ..default()
            }).with_children(|attachments_panel| {
                attachments_panel.spawn(TextBundle::from_section(
                    format!("AVAILABLE {:?} ATTACHMENTS:", selected_slot),
                    TextStyle { font_size: 16.0, color: Color::srgb(0.8, 0.6, 0.2), ..default() }
                ));
                
                let available_attachments = attachment_db.get_by_slot(selected_slot);
                let mut found_any = false;
                
                for attachment in available_attachments {
                    if unlocked.attachments.contains(&attachment.id) {
                        found_any = true;
                        
                        let is_selected = manufacture_state.selected_attachments.get(selected_slot) == Some(&attachment.id);
                        let base_color = match attachment.rarity {
                            AttachmentRarity::Common => Color::srgb(0.8, 0.8, 0.8),
                            AttachmentRarity::Rare => Color::srgb(0.6, 0.6, 1.0),
                            AttachmentRarity::Epic => Color::srgb(1.0, 0.6, 1.0),
                        };
                        let color = if is_selected { 
                            Color::srgb(1.0, 1.0, 0.2) // Bright yellow when selected
                        } else { 
                            base_color 
                        };
                        let prefix = if is_selected { "> " } else { "  " };
                        
                        attachments_panel.spawn(TextBundle::from_section(
                            format!("{}• {} (Acc{:+} Rng{:+} Noise{:+})", 
                                    prefix,
                                    attachment.name,
                                    attachment.stats.accuracy,
                                    attachment.stats.range,
                                    attachment.stats.noise),
                            TextStyle { font_size: 12.0, color, ..default() }
                        ));
                    }
                }
                
                if !found_any {
                    attachments_panel.spawn(TextBundle::from_section(
                        "No unlocked attachments for this slot",
                        TextStyle { font_size: 12.0, color: Color::srgb(0.6, 0.6, 0.6), ..default() }
                    ));
                }
            });
        }
        
        // Controls help
        content.spawn(TextBundle::from_section(
            "\n1-3: Select Agent | ↑↓: Navigate Slots | ←→: Select Attachment | ENTER: Attach/Detach",
            TextStyle { font_size: 12.0, color: Color::srgb(0.7, 0.7, 0.7), ..default() }
        ));
        
        content.spawn(TextBundle::from_section(
            format!("Credits: {}", global_data.credits),
            TextStyle { font_size: 14.0, color: Color::srgb(0.8, 0.8, 0.2), ..default() }
        ));
    });
}

fn cycle_selection(manufacture_state: &mut ManufactureState, forward: bool) {
    let slots = vec![
        AttachmentSlot::Sight,
        AttachmentSlot::Barrel, 
        AttachmentSlot::Magazine,
        AttachmentSlot::Grip,
        AttachmentSlot::Stock,
    ];
    
    let current_idx = if let Some(slot) = &manufacture_state.selected_slot {
        slots.iter().position(|s| s == slot).unwrap_or(0)
    } else {
        0
    };
    
    let new_idx = if forward {
        (current_idx + 1) % slots.len()
    } else {
        if current_idx == 0 { slots.len() - 1 } else { current_idx - 1 }
    };
    
    manufacture_state.selected_slot = Some(slots[new_idx].clone());
    info!("Selected slot: {:?}", slots[new_idx]);
}

fn cycle_attachment_selection(
    manufacture_state: &mut ManufactureState,
    forward: bool,
    attachment_db: &AttachmentDatabase,
    unlocked: &UnlockedAttachments,
) {
    let Some(selected_slot) = &manufacture_state.selected_slot else { return; };
    
    let available: Vec<String> = attachment_db.get_by_slot(selected_slot)
        .iter()
        .filter(|att| unlocked.attachments.contains(&att.id))
        .map(|att| att.id.clone())
        .collect();
    
    if available.is_empty() { return; }
    
    let current_idx = if let Some(att_id) = manufacture_state.selected_attachments.get(selected_slot) {
        available.iter().position(|id| id == att_id).unwrap_or(0)
    } else {
        0
    };
    
    let new_idx = if forward {
        (current_idx + 1) % available.len()
    } else {
        if current_idx == 0 { available.len() - 1 } else { current_idx - 1 }
    };
    
    manufacture_state.selected_attachments.insert(selected_slot.clone(), available[new_idx].clone());
    info!("Selected {} for {:?} slot", available[new_idx], selected_slot);
}

fn execute_attachment_action(
    manufacture_state: &mut ManufactureState,
    global_data: &mut GlobalData,
    agent_query: &mut Query<&mut Inventory, With<Agent>>,
    attachment_db: &AttachmentDatabase,
) {
    let Some(selected_slot) = &manufacture_state.selected_slot else { return; };
    
    let mut inventories: Vec<_> = agent_query.iter_mut().collect();
    let Some(inventory) = inventories.get_mut(manufacture_state.selected_agent_idx) else { return; };
    
    let Some(weapon_config) = inventory.weapons.get_mut(0) else { return; };
    
    let current_attachment_name = weapon_config.attachments.get(selected_slot)
        .and_then(|opt| opt.as_ref())
        .map(|att| att.name.clone());
    
    let mut config_changed = false;
    
    if let Some(current_name) = current_attachment_name {
        // DETACH
        weapon_config.detach(selected_slot);
        
        let refund = 0; // TODO: Calculate actual refund
        global_data.credits += refund;
        
        info!("Detached {} from {:?} slot", current_name, selected_slot);
        manufacture_state.selected_attachments.remove(selected_slot);
        config_changed = true;
        
    } else if let Some(attachment_id) = manufacture_state.selected_attachments.get(selected_slot) {
        // ATTACH
        if let Some(attachment) = attachment_db.get(attachment_id) {
            let cost = 0; // TODO: Calculate actual cost
            
            if global_data.credits >= cost {
                weapon_config.attach(attachment.clone());
                global_data.credits -= cost;
                
                info!("Attached {} to {:?} slot", attachment.name, selected_slot);
                config_changed = true;
            } else {
                info!("Insufficient credits to attach {}", attachment.name);
            }
        }
    }
    
    // Save configuration to GlobalData if changed
    if config_changed {
        let loadout = AgentLoadout {
            weapon_configs: inventory.weapons.clone(),
            equipped_weapon_idx: 0,
            tools: inventory.tools.clone(),
            cybernetics: inventory.cybernetics.clone(),
        };
        
        global_data.save_agent_loadout(manufacture_state.selected_agent_idx, loadout);
        
        // Update equipped weapon in current inventory
        if let Some(weapon_config) = inventory.weapons.get(0) {
            inventory.equipped_weapon = Some(weapon_config.clone());
        }
    }
}