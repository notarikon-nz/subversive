// src/core/attachments.rs
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::core::WeaponType;

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
        
        Self {
            base_weapon: weapon,
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
        
        // Load from JSON files
        if let Ok(content) = std::fs::read_to_string("assets/attachments/tier1.json") {
            if let Ok(attachments) = serde_json::from_str::<HashMap<String, WeaponAttachment>>(&content) {
                for (id, mut attachment) in attachments {
                    attachment.id = id.clone();
                    db.attachments.insert(id, attachment);
                }
            }
        }
        
        // Fallback: Create some basic attachments if file loading fails
        if db.attachments.is_empty() {
            db.create_default_attachments();
        }
        
        db
    }
    
    fn create_default_attachments(&mut self) {
        let attachments = vec![
            WeaponAttachment {
                id: "red_dot".to_string(),
                name: "Red Dot Sight".to_string(),
                slot: AttachmentSlot::Sight,
                rarity: AttachmentRarity::Common,
                stats: AttachmentStats { accuracy: 2, ..Default::default() },
            },
            WeaponAttachment {
                id: "suppressor".to_string(),
                name: "Sound Suppressor".to_string(),
                slot: AttachmentSlot::Barrel,
                rarity: AttachmentRarity::Common,
                stats: AttachmentStats { range: -2, noise: -5, ..Default::default() },
            },
            WeaponAttachment {
                id: "extended_mag".to_string(),
                name: "Extended Magazine".to_string(),
                slot: AttachmentSlot::Magazine,
                rarity: AttachmentRarity::Rare,
                stats: AttachmentStats { reload_speed: -2, ammo_capacity: 3, ..Default::default() },
            },
            WeaponAttachment {
                id: "bipod".to_string(),
                name: "Bipod".to_string(),
                slot: AttachmentSlot::Grip,
                rarity: AttachmentRarity::Rare,
                stats: AttachmentStats { accuracy: 3, reload_speed: -1, ..Default::default() },
            },
        ];
        
        for attachment in attachments {
            self.attachments.insert(attachment.id.clone(), attachment);
        }
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