// src/core/attachments.rs - Streamlined attachment system
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::core::{WeaponType, WeaponBehavior};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum AttachmentSlot {
    Barrel,
    Sight,
    Magazine,
    Grip,
    Stock,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AttachmentStats {
    pub accuracy: i8,
    pub range: i8,
    pub noise: i8,
    pub reload_speed: i8,
    pub ammo_capacity: i8,
}

impl AttachmentStats {
    pub fn apply(&self, base: &mut AttachmentStats) {
        base.accuracy += self.accuracy;
        base.range += self.range;
        base.noise += self.noise;
        base.reload_speed += self.reload_speed;
        base.ammo_capacity += self.ammo_capacity;
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeaponAttachment {
    pub id: String,
    pub name: String,
    pub slot: AttachmentSlot,
    pub stats: AttachmentStats,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeaponConfig {
    pub base_weapon: WeaponType,
    pub attachments: HashMap<AttachmentSlot, WeaponAttachment>,
    pub behavior: WeaponBehavior,
}

impl WeaponConfig {
    pub fn new(weapon: WeaponType) -> Self {
        Self {
            behavior: WeaponBehavior::for_weapon_type(&weapon),
            base_weapon: weapon,
            attachments: HashMap::new(),
        }
    }

    pub fn attach(&mut self, attachment: WeaponAttachment) -> Option<WeaponAttachment> {
        self.attachments.insert(attachment.slot.clone(), attachment)
    }

    pub fn detach(&mut self, slot: &AttachmentSlot) -> Option<WeaponAttachment> {
        self.attachments.remove(slot)
    }

    pub fn stats(&self) -> AttachmentStats {
        let mut total = AttachmentStats::default();
        for attachment in self.attachments.values() {
            attachment.stats.apply(&mut total);
        }
        total
    }

    // Legacy compatibility methods
    pub fn calculate_total_stats(&self) -> AttachmentStats {
        self.stats()
    }

    pub fn get_effective_range(&self) -> f32 {
        let base_range = self.behavior.preferred_range;
        let stats = self.stats();
        base_range * (1.0 + stats.range as f32 * 0.1)
    }

    pub fn supported_slots(&self) -> Vec<AttachmentSlot> {
        match self.base_weapon {
            WeaponType::Pistol => vec![AttachmentSlot::Sight, AttachmentSlot::Barrel],
            WeaponType::Shotgun => vec![AttachmentSlot::Stock, AttachmentSlot::Barrel],
            WeaponType::Rifle => vec![
                AttachmentSlot::Sight,
                AttachmentSlot::Barrel,
                AttachmentSlot::Magazine,
                AttachmentSlot::Stock,
            ],
            WeaponType::Minigun => vec![AttachmentSlot::Sight, AttachmentSlot::Grip],
            WeaponType::Flamethrower => vec![AttachmentSlot::Grip],
            WeaponType::GrenadeLauncher => vec![AttachmentSlot::Grip],
            WeaponType::RocketLauncher => vec![AttachmentSlot::Grip],
            WeaponType::LaserRifle => vec![AttachmentSlot::Grip],
            WeaponType::PlasmaGun => vec![AttachmentSlot::Grip],
        }
    }
}

#[derive(Component)]
pub struct EquippedWeapon {
    pub config: WeaponConfig,
}

#[derive(Resource, Default, Deserialize)]
pub struct AttachmentDatabase {
    pub attachments: HashMap<String, WeaponAttachment>,
}

impl AttachmentDatabase {
    pub fn load() -> Self {
        let paths = ["data/attachments/tier1.json", "data/attachments/tier2.json", "data/attachments/tier3.json"];
        let mut db = Self::default();

        for path in paths {
            if let Ok(content) = std::fs::read_to_string(path) {
                if let Ok(attachments) = serde_json::from_str::<HashMap<String, WeaponAttachment>>(&content) {
                    for (id, mut attachment) in attachments {
                        attachment.id = id.clone();
                        db.attachments.insert(id, attachment);
                    }
                }
            }
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

#[derive(Resource, Default)]
pub struct UnlockedAttachments {
    pub attachments: std::collections::HashSet<String>,
}

// ===== MAIN.RS =====

pub fn setup_attachments(mut commands: Commands) {
    let attachment_db = AttachmentDatabase::load();
    let mut unlocked = UnlockedAttachments::default();
    unlocked.attachments.insert("red_dot".to_string());
    unlocked.attachments.insert("iron_sights".to_string());
    unlocked.attachments.insert("tactical_grip".to_string());

    commands.insert_resource(attachment_db);
    commands.insert_resource(unlocked);
}