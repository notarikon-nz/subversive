// src/systems/ui/hub/manufacture.rs - Simplified using UIBuilder
use bevy::prelude::*;
use crate::core::*;
use crate::systems::ui::builder::*;

pub fn handle_input(
    input: &ButtonInput<KeyCode>,
    hub_state: &mut super::HubState,
    manufacture_state: &mut ManufactureState,
    global_data: &mut GlobalData,
    mut agent_query: Query<&mut Inventory, With<Agent>>,
    attachment_db: &AttachmentDatabase,
) -> bool {
    let mut needs_rebuild = false;
    
    if input.just_pressed(KeyCode::Digit1) {
        manufacture_state.selected_agent_idx = 0;
        manufacture_state.selected_weapon_idx = 0;
        manufacture_state.selected_slot = None;
        manufacture_state.selected_attachments.clear();
        needs_rebuild = true;
    }
    if input.just_pressed(KeyCode::Digit2) {
        manufacture_state.selected_agent_idx = 1;
        manufacture_state.selected_weapon_idx = 0;
        manufacture_state.selected_slot = None;
        manufacture_state.selected_attachments.clear();
        needs_rebuild = true;
    }
    if input.just_pressed(KeyCode::Digit3) {
        manufacture_state.selected_agent_idx = 2;
        manufacture_state.selected_weapon_idx = 0;
        manufacture_state.selected_slot = None;
        manufacture_state.selected_attachments.clear();
        needs_rebuild = true;
    }
    
    if input.just_pressed(KeyCode::ArrowUp) || input.just_pressed(KeyCode::ArrowDown) {
        cycle_selection(manufacture_state, input.just_pressed(KeyCode::ArrowDown));
        needs_rebuild = true;
    }
    
    if input.just_pressed(KeyCode::ArrowLeft) || input.just_pressed(KeyCode::ArrowRight) {
        cycle_attachment_selection(manufacture_state, input.just_pressed(KeyCode::ArrowRight), attachment_db, &UnlockedAttachments::default());
        needs_rebuild = true;
    }
    
    if input.just_pressed(KeyCode::Enter) {
        execute_attachment_action(manufacture_state, global_data, &mut agent_query, attachment_db);
        needs_rebuild = true;
    }
    
    if input.just_pressed(KeyCode::Backspace) {
        hub_state.active_tab = super::HubTab::Agents;
        needs_rebuild = true;
    }
    
    needs_rebuild
}

pub fn create_content(
    parent: &mut ChildSpawnerCommands, 
    global_data: &GlobalData,
    manufacture_state: &ManufactureState,
    attachment_db: &AttachmentDatabase,
    unlocked: &UnlockedAttachments,
) {
    parent.spawn(UIBuilder::content_area()).with_children(|content| {
        content.spawn(UIBuilder::title("WEAPON MANUFACTURE"));
        
        // Agent selection
        content.spawn(UIBuilder::row(20.0)).with_children(|agents| {
            for i in 0..3 {
                let is_selected = i == manufacture_state.selected_agent_idx;
                let color = if is_selected { Color::srgb(0.2, 0.8, 0.2) } else { Color::srgb(0.6, 0.6, 0.6) };
                let text = UIBuilder::selection_item(is_selected, "", &format!("Agent {} (Lv{})", i + 1, global_data.agent_levels[i]));
                agents.spawn(UIBuilder::text(&text, 16.0, color));
            }
        });
        
        // Weapon slots
        let (panel_node, panel_bg) = UIBuilder::section_panel();
        content.spawn((panel_node, BackgroundColor(Color::srgba(0.2, 0.2, 0.3, 0.3)))).with_children(|weapon_panel| {
            weapon_panel.spawn(UIBuilder::subtitle("CURRENT WEAPON: Rifle"));
            
            let slots = [
                ("Sight", AttachmentSlot::Sight),
                ("Barrel", AttachmentSlot::Barrel),
                ("Magazine", AttachmentSlot::Magazine),
                ("Grip", AttachmentSlot::Grip),
                ("Stock", AttachmentSlot::Stock),
            ];
            
            for (slot_name, slot) in slots {
                let is_selected = manufacture_state.selected_slot.as_ref() == Some(&slot);
                let color = if is_selected { Color::srgb(0.8, 0.8, 0.2) } else { Color::WHITE };
                let text = UIBuilder::selection_item(is_selected, "", &format!("{}: None equipped", slot_name));
                weapon_panel.spawn(UIBuilder::text(&text, 14.0, color));
            }
        });
        
        // Available attachments
        if let Some(selected_slot) = &manufacture_state.selected_slot {
            let (panel_node, panel_bg) = UIBuilder::section_panel();
            content.spawn((panel_node, BackgroundColor(Color::srgba(0.3, 0.2, 0.2, 0.3)))).with_children(|attachments_panel| {
                attachments_panel.spawn(UIBuilder::text(&format!("AVAILABLE {:?} ATTACHMENTS:", selected_slot), 16.0, Color::srgb(0.8, 0.6, 0.2)));
                
                let available_attachments = attachment_db.get_by_slot(selected_slot);
                let mut found_any = false;
                
                for attachment in available_attachments {
                    if unlocked.attachments.contains(&attachment.id) {
                        found_any = true;
                        
                        let is_selected = manufacture_state.selected_attachments.get(selected_slot) == Some(&attachment.id);
                        let color = if is_selected { 
                            Color::srgb(1.0, 1.0, 0.2)
                        } else { 
                            match attachment.rarity {
                                AttachmentRarity::Common => Color::srgb(0.8, 0.8, 0.8),
                                AttachmentRarity::Rare => Color::srgb(0.6, 0.6, 1.0),
                                AttachmentRarity::Epic => Color::srgb(1.0, 0.6, 1.0),
                            }
                        };
                        
                        let text = UIBuilder::selection_item(
                            is_selected,
                            "• ",
                            &format!("{} (Acc{:+} Rng{:+} Noise{:+})", 
                                    attachment.name,
                                    attachment.stats.accuracy,
                                    attachment.stats.range,
                                    attachment.stats.noise)
                        );
                        
                        attachments_panel.spawn(UIBuilder::text(&text, 12.0, color));
                    }
                }
                
                if !found_any {
                    attachments_panel.spawn(UIBuilder::text("No unlocked attachments for this slot", 12.0, Color::srgb(0.6, 0.6, 0.6)));
                }
            });
        }
        
        content.spawn(UIBuilder::nav_controls("1-3: Agent | ↑↓: Slots | ←→: Attachments | ENTER: Modify"));
        content.spawn(UIBuilder::text(&UIBuilder::credits_display(global_data.credits), 14.0, Color::srgb(0.8, 0.8, 0.2)));
    });
}

fn cycle_selection(manufacture_state: &mut ManufactureState, forward: bool) {
    let slots = [AttachmentSlot::Sight, AttachmentSlot::Barrel, AttachmentSlot::Magazine, AttachmentSlot::Grip, AttachmentSlot::Stock];
    
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
        weapon_config.detach(selected_slot);
        info!("Detached {} from {:?} slot", current_name, selected_slot);
        manufacture_state.selected_attachments.remove(selected_slot);
        config_changed = true;
    } else if let Some(attachment_id) = manufacture_state.selected_attachments.get(selected_slot) {
        if let Some(attachment) = attachment_db.get(attachment_id) {
            weapon_config.attach(attachment.clone());
            info!("Attached {} to {:?} slot", attachment.name, selected_slot);
            config_changed = true;
        }
    }
    
    if config_changed {
        let loadout = AgentLoadout {
            weapon_configs: inventory.weapons.clone(),
            equipped_weapon_idx: 0,
            tools: inventory.tools.clone(),
            cybernetics: inventory.cybernetics.clone(),
        };
        
        global_data.save_agent_loadout(manufacture_state.selected_agent_idx, loadout);
        
        if let Some(weapon_config) = inventory.weapons.get(0) {
            inventory.equipped_weapon = Some(weapon_config.clone());
        }
    }
}