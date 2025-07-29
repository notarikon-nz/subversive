// src/systems/ui/hub/manufacture.rs - egui version
use bevy::prelude::*;
use bevy_egui::egui;
use crate::core::*;

pub fn show_manufacture(
    ui: &mut egui::Ui,
    global_data: &mut GlobalData,
    attachment_db: &AttachmentDatabase,
    unlocked: &UnlockedAttachments,
    agent_query: &mut Query<&mut Inventory, With<Agent>>,
    input: &ButtonInput<KeyCode>,
) {
    ui.heading("WEAPON MANUFACTURE");
    
    ui.separator();
    
    // Agent selection
    let mut selected_agent = 0; // In real implementation, store this in state
    ui.horizontal(|ui| {
        for i in 0..3 {
            let text = format!("Agent {} (Lv{})", i + 1, global_data.agent_levels[i]);
            if ui.selectable_label(selected_agent == i, text).clicked() {
                selected_agent = i;
            }
        }
    });
    
    ui.separator();
    
    // Current weapon display
    ui.group(|ui| {
        let loadout = global_data.get_agent_loadout(selected_agent);
        if let Some(weapon_config) = loadout.weapon_configs.get(loadout.equipped_weapon_idx) {
        
            ui.heading(format!("CURRENT WEAPON: {:?}", weapon_config.base_weapon));
            ui.separator();
            
            // Weapon stats
            let stats = weapon_config.stats();
            ui.horizontal(|ui| {
                if stats.accuracy != 0 {
                    let color = if stats.accuracy > 0 { egui::Color32::GREEN } else { egui::Color32::RED };
                    ui.colored_label(color, format!("Accuracy: {:+}", stats.accuracy));
                }
                if stats.range != 0 {
                    let color = if stats.range > 0 { egui::Color32::GREEN } else { egui::Color32::RED };
                    ui.colored_label(color, format!("Range: {:+}", stats.range));
                }
                if stats.noise != 0 {
                    let color = if stats.noise < 0 { egui::Color32::GREEN } else { egui::Color32::RED };
                    ui.colored_label(color, format!("Noise: {:+}", stats.noise));
                }
            });
            
            ui.separator();
            
            // Attachment slots
            let slots = [
                ("Sight", AttachmentSlot::Sight),
                ("Barrel", AttachmentSlot::Barrel),
                ("Magazine", AttachmentSlot::Magazine),
                ("Grip", AttachmentSlot::Grip),
                ("Stock", AttachmentSlot::Stock),
            ];
            
            let attachments = weapon_config.attachments.clone();

            for (slot_name, slot) in slots {
                ui.horizontal(|ui| {
                    ui.label(format!("{}:", slot_name));
                    
                    if let Some(attachment) = attachments.get(&slot) {
                        ui.colored_label(egui::Color32::GREEN, &attachment.name);
                        if ui.small_button("Remove").clicked() {
                            // Remove attachment logic
                            remove_attachment(global_data, selected_agent, &slot, agent_query);
                        }
                    } else {
                        ui.weak("None equipped");
                        
                        // Show available attachments for this slot
                        egui::ComboBox::from_id_salt(format!("{:?}", slot))
                            .selected_text("Select attachment...")
                            .show_ui(ui, |ui| {
                                let available_attachments = attachment_db.get_by_slot(&slot);
                                
                                for attachment in available_attachments {
                                    if unlocked.attachments.contains(&attachment.id) {
                                        let response = ui.selectable_label(false, &attachment.name);
                                        if response.clicked() {
                                            attach_attachment(global_data, selected_agent, attachment.clone(), agent_query);
                                        }
                                        
                                        // Show stats on hover
                                        if response.hovered() {
                                            response.on_hover_ui(|ui| {
                                                ui.label(&attachment.name);
                                                ui.label(format!("Accuracy: {:+}", attachment.stats.accuracy));
                                                ui.label(format!("Range: {:+}", attachment.stats.range));
                                                ui.label(format!("Noise: {:+}", attachment.stats.noise));
                                            });
                                        }
                                    }
                                }
                            });
                    }
                });
            }
        
        } else {
            ui.group(|ui| {
                ui.colored_label(egui::Color32::RED, "No weapon equipped");
            });
        }
    });
    
    ui.separator();
    
    // Available attachments by category
    ui.collapsing("AVAILABLE ATTACHMENTS", |ui| {
        let categories = [
            AttachmentSlot::Sight,
            AttachmentSlot::Barrel,
            AttachmentSlot::Magazine,
            AttachmentSlot::Grip,
            AttachmentSlot::Stock,
        ];
        
        for category in categories {
            ui.collapsing(format!("{:?} Attachments", category), |ui| {
                let available_attachments = attachment_db.get_by_slot(&category);
                let mut found_any = false;
                
                for attachment in available_attachments {
                    if unlocked.attachments.contains(&attachment.id) {
                        found_any = true;
                        
                        ui.group(|ui| {
                            ui.horizontal(|ui| {
                                ui.label(&attachment.name);
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    if ui.small_button("Equip").clicked() {
                                        attach_attachment(global_data, selected_agent, attachment.clone(), agent_query);
                                    }
                                });
                            });
                            
                            ui.horizontal(|ui| {
                                if attachment.stats.accuracy != 0 {
                                    let color = if attachment.stats.accuracy > 0 { egui::Color32::GREEN } else { egui::Color32::RED };
                                    ui.colored_label(color, format!("Acc: {:+}", attachment.stats.accuracy));
                                }
                                if attachment.stats.range != 0 {
                                    let color = if attachment.stats.range > 0 { egui::Color32::GREEN } else { egui::Color32::RED };
                                    ui.colored_label(color, format!("Rng: {:+}", attachment.stats.range));
                                }
                                if attachment.stats.noise != 0 {
                                    let color = if attachment.stats.noise < 0 { egui::Color32::GREEN } else { egui::Color32::RED };
                                    ui.colored_label(color, format!("Noise: {:+}", attachment.stats.noise));
                                }
                            });
                        });
                    }
                }
                
                if !found_any {
                    ui.weak("No unlocked attachments for this slot");
                }
            });
        }
    });
    
    ui.separator();
    ui.colored_label(egui::Color32::YELLOW, format!("Credits: {}", global_data.credits));
}

fn attach_attachment(
    global_data: &mut GlobalData,
    agent_idx: usize,
    attachment: WeaponAttachment,
    agent_query: &mut Query<&mut Inventory, With<Agent>>,
) {
    let mut inventories: Vec<_> = agent_query.iter_mut().collect();
    let Some(inventory) = inventories.get_mut(agent_idx) else { return; };
    
    let Some(weapon_config) = inventory.weapons.get_mut(0) else { return; };
    
    // Attach the new attachment
    weapon_config.attach(attachment);
    
    // Update the loadout in global data
    let loadout = AgentLoadout {
        weapon_configs: inventory.weapons.clone(),
        equipped_weapon_idx: 0,
        tools: inventory.tools.clone(),
        cybernetics: inventory.cybernetics.clone(),
    };
    
    global_data.save_agent_loadout(agent_idx, loadout);
    
    // Update equipped weapon
    if let Some(weapon_config) = inventory.weapons.get(0) {
        inventory.equipped_weapon = Some(weapon_config.clone());
    }
}

fn remove_attachment(
    global_data: &mut GlobalData,
    agent_idx: usize,
    slot: &AttachmentSlot,
    agent_query: &mut Query<&mut Inventory, With<Agent>>,
) {
    let mut inventories: Vec<_> = agent_query.iter_mut().collect();
    let Some(inventory) = inventories.get_mut(agent_idx) else { return; };
    
    let Some(weapon_config) = inventory.weapons.get_mut(0) else { return; };
    
    // Remove the attachment
    weapon_config.detach(slot);
    
    // Update the loadout in global data
    let loadout = AgentLoadout {
        weapon_configs: inventory.weapons.clone(),
        equipped_weapon_idx: 0,
        tools: inventory.tools.clone(),
        cybernetics: inventory.cybernetics.clone(),
    };
    
    global_data.save_agent_loadout(agent_idx, loadout);
    
    // Update equipped weapon
    if let Some(weapon_config) = inventory.weapons.get(0) {
        inventory.equipped_weapon = Some(weapon_config.clone());
    }
}