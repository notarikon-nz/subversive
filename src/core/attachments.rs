// src/core/attachments.rs
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::core::WeaponType;
use crate::core::*;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum AttachmentSlot {
    Barrel,
    Sight, 
    Magazine,
    Grip,
    Stock,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AttachmentRarity {
    Common,
    Rare,
    Epic,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttachmentStats {
    pub accuracy: i8,
    pub range: i8,
    pub noise: i8,
    pub reload_speed: i8,
    pub ammo_capacity: i8,
}

impl Default for AttachmentStats {
    fn default() -> Self {
        Self {
            accuracy: 0,
            range: 0,
            noise: 0,
            reload_speed: 0,
            ammo_capacity: 0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeaponAttachment {
    pub id: String,
    pub name: String,
    pub slot: AttachmentSlot,
    pub rarity: AttachmentRarity,
    pub stats: AttachmentStats,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeaponConfig {
    pub base_weapon: WeaponType,
    pub supported_slots: Vec<AttachmentSlot>,
    pub attachments: HashMap<AttachmentSlot, Option<WeaponAttachment>>,
    pub behavior: WeaponBehavior,
}

impl WeaponConfig {
    pub fn new(weapon: WeaponType) -> Self {
        let supported_slots = match weapon {
            WeaponType::Pistol => vec![AttachmentSlot::Sight, AttachmentSlot::Barrel],
            WeaponType::Rifle => vec![
                AttachmentSlot::Sight, 
                AttachmentSlot::Barrel, 
                AttachmentSlot::Magazine,
                AttachmentSlot::Stock,
            ],
            WeaponType::Minigun => vec![AttachmentSlot::Sight, AttachmentSlot::Grip],
            WeaponType::Flamethrower => vec![AttachmentSlot::Grip],
        };
        
        let mut attachments = HashMap::new();
        for slot in &supported_slots {
            attachments.insert(slot.clone(), None);
        }
        
        let behavior = WeaponBehavior::for_weapon_type(&weapon);
        
        Self {
            base_weapon: weapon,
            behavior,
            supported_slots,
            attachments,
        }
    }
    
    pub fn attach(&mut self, attachment: WeaponAttachment) -> Option<WeaponAttachment> {
        if self.supported_slots.contains(&attachment.slot) {
            self.attachments.insert(attachment.slot.clone(), Some(attachment.clone()));
            None // Successfully attached
        } else {
            Some(attachment) // Return it back if slot not supported
        }
    }
    
    pub fn detach(&mut self, slot: &AttachmentSlot) -> Option<WeaponAttachment> {
        if let Some(current) = self.attachments.get_mut(slot) {
            current.take()
        } else {
            None
        }
    }
    
    // weapon behavior
    pub fn get_effective_range(&self) -> f32 {
        let base_range = self.behavior.preferred_range;
        let stats = self.calculate_total_stats();
        base_range * (1.0 + stats.range as f32 * 0.1)
    }

    pub fn calculate_total_stats(&self) -> AttachmentStats {
        let mut total = AttachmentStats::default();
        
        for attachment_opt in self.attachments.values() {
            if let Some(attachment) = attachment_opt {
                total.accuracy += attachment.stats.accuracy;
                total.range += attachment.stats.range;
                total.noise += attachment.stats.noise;
                total.reload_speed += attachment.stats.reload_speed;
                total.ammo_capacity += attachment.stats.ammo_capacity;
            }
        }
        
        total
    }
}

#[derive(Component)]
pub struct EquippedWeapon {
    pub config: WeaponConfig,
}

#[derive(Resource, Default)]
pub struct UnlockedAttachments {
    pub attachments: std::collections::HashSet<String>,
}

#[derive(Resource, Default)]
pub struct AttachmentDatabase {
    pub attachments: HashMap<String, WeaponAttachment>,
}

impl AttachmentDatabase {
    pub fn load() -> Self {
        let mut db = Self::default();
        
        // Load attachment files from data directory
        let attachment_files = [
            "data/attachments/tier1.json",
            "data/attachments/tier2.json", 
            "data/attachments/tier3.json"
        ];
        
        for file_path in attachment_files {
            match std::fs::read_to_string(file_path) {
                Ok(content) => {
                    match serde_json::from_str::<HashMap<String, WeaponAttachment>>(&content) {
                        Ok(attachments) => {
                            for (id, mut attachment) in attachments {
                                attachment.id = id.clone();
                                db.attachments.insert(id, attachment);
                            }
                            info!("Loaded {} attachments from {}", db.attachments.len(), file_path);
                        },
                        Err(e) => error!("Failed to parse {}: {}", file_path, e),
                    }
                },
                Err(e) => error!("Failed to load {}: {}", file_path, e),
            }
        }
        
        if db.attachments.is_empty() {
            error!("No attachments loaded! Game may not function properly.");
            error!("Create attachment files in data/attachments/ directory.");
        }
        
        db
    }
    
    pub fn get(&self, id: &str) -> Option<&WeaponAttachment> {
        self.attachments.get(id)
    }
    
    pub fn get_by_slot(&self, slot: &AttachmentSlot) -> Vec<&WeaponAttachment> {
        self.attachments.values()
            .filter(|att| att.slot == *slot)
            .collect()
    }
}