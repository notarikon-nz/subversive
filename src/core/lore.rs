// src/core/lore.rs - Efficient lore storage and retrieval system
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::core::*;

// === LORE ENTRY TYPES ===
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoreEntry {
    pub id: String,
    pub title: String,
    pub content: String,
    pub category: LoreCategory,
    pub discovered: bool,
    pub source: String, // Where this was found (e.g., "Terminal A-7", "Encrypted Email")
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum LoreCategory {
    CorpMemo,
    PersonalLog,
    NewsReport,
    TechnicalDoc,
    Conversation,
    Research,
    History,
    Mission,
}

impl LoreCategory {
    pub fn icon(&self) -> &'static str {
        match self {
            LoreCategory::CorpMemo => "📄",
            LoreCategory::PersonalLog => "📔",
            LoreCategory::NewsReport => "📰",
            LoreCategory::TechnicalDoc => "🔧",
            LoreCategory::Conversation => "💬",
            LoreCategory::Research => "🔬",
            LoreCategory::History => "📚",
            LoreCategory::Mission => "🎯",
        }
    }
    
    pub fn color(&self) -> Color {
        match self {
            LoreCategory::CorpMemo => Color::srgb(0.8, 0.2, 0.2),
            LoreCategory::PersonalLog => Color::srgb(0.2, 0.8, 0.2),
            LoreCategory::NewsReport => Color::srgb(0.2, 0.2, 0.8),
            LoreCategory::TechnicalDoc => Color::srgb(0.8, 0.8, 0.2),
            LoreCategory::Conversation => Color::srgb(0.8, 0.2, 0.8),
            LoreCategory::Research => Color::srgb(0.2, 0.8, 0.8),
            LoreCategory::History => Color::srgb(0.6, 0.4, 0.2),
            LoreCategory::Mission => Color::srgb(0.8, 0.6, 0.2),
        }
    }
}

// === LORE DATABASE ===
#[derive(Resource, Serialize, Deserialize)]
pub struct LoreDatabase {
    pub entries: HashMap<String, LoreEntry>,
    pub discovered_count: usize,
}

impl Default for LoreDatabase {
    fn default() -> Self {
        Self {
            entries: HashMap::new(),
            discovered_count: 0,
        }
    }
}

impl LoreDatabase {
    pub fn load() -> Self {
        match std::fs::read_to_string("data/lore.json") {
            Ok(content) => {
                match serde_json::from_str::<LoreDatabase>(&content) {
                    Ok(mut db) => {
                        db.discovered_count = db.entries.values().filter(|e| e.discovered).count();
                        info!("Loaded {} lore entries ({} discovered)", db.entries.len(), db.discovered_count);
                        db
                    },
                    Err(e) => {
                        error!("Failed to parse lore.json: {}", e);
                        Self::default()
                    }
                }
            },
            Err(_) => {
                warn!("No lore.json found, creating empty database");
                Self::default()
            }
        }
    }
    
    pub fn save(&self) {
        if let Ok(json) = serde_json::to_string_pretty(self) {
            if std::fs::write("data/lore.json", json).is_ok() {
                info!("Saved lore database with {} entries", self.entries.len());
            }
        }
    }
    
    pub fn discover_entry(&mut self, id: &str) -> bool {
        if let Some(entry) = self.entries.get_mut(id) {
            if !entry.discovered {
                entry.discovered = true;
                self.discovered_count += 1;
                info!("Discovered lore: {}", entry.title);
                return true;
            }
        }
        false
    }
    
    pub fn get_entry(&self, id: &str) -> Option<&LoreEntry> {
        self.entries.get(id).filter(|e| e.discovered)
    }
    
    pub fn get_entries_by_category(&self, category: LoreCategory) -> Vec<&LoreEntry> {
        self.entries.values()
            .filter(|e| e.discovered && e.category == category)
            .collect()
    }
    
    pub fn get_all_discovered(&self) -> Vec<&LoreEntry> {
        self.entries.values()
            .filter(|e| e.discovered)
            .collect()
    }
    
    pub fn add_entry(&mut self, entry: LoreEntry) {
        self.entries.insert(entry.id.clone(), entry);
    }
}

// === LORE SOURCE COMPONENT ===
#[derive(Component)]
pub struct LoreSource {
    pub lore_ids: Vec<String>,
    pub requires_hacking: bool,
    pub access_time: f32, // Time to read/access
    pub one_time_use: bool,
    pub accessed: bool,
}

impl LoreSource {
    pub fn new(lore_ids: Vec<String>) -> Self {
        Self {
            lore_ids,
            requires_hacking: false,
            access_time: 2.0,
            one_time_use: false,
            accessed: false,
        }
    }
    
    pub fn hackable(mut self) -> Self {
        self.requires_hacking = true;
        self.access_time = 5.0;
        self
    }
    
    pub fn quick_read(mut self) -> Self {
        self.access_time = 1.0;
        self
    }
    
    pub fn one_time(mut self) -> Self {
        self.one_time_use = true;
        self
    }
}

// === LORE INTERACTION SYSTEM ===
#[derive(Event)]
pub struct LoreAccessEvent {
    pub agent: Entity,
    pub source: Entity,
}

pub fn lore_interaction_system(
    mut lore_events: EventReader<LoreAccessEvent>,
    mut lore_db: ResMut<LoreDatabase>,
    mut lore_sources: Query<&mut LoreSource>,
    mut commands: Commands,
    mut audio_events: EventWriter<AudioEvent>,
) {
    for event in lore_events.read() {
        if let Ok(mut lore_source) = lore_sources.get_mut(event.source) {
            if lore_source.accessed && lore_source.one_time_use {
                continue; // Already accessed
            }
            
            // Discover all lore entries from this source
            let mut discovered_any = false;
            for lore_id in &lore_source.lore_ids {
                if lore_db.discover_entry(lore_id) {
                    discovered_any = true;
                }
            }
            
            if discovered_any {
                lore_source.accessed = true;
                
                // Play discovery sound
                audio_events.write(AudioEvent {
                    sound: AudioType::TerminalAccess,
                    volume: 0.4,
                });
                
                // Show notification
                spawn_lore_notification(&mut commands, &lore_source.lore_ids[0], &lore_db);
            }
        }
    }
}

fn spawn_lore_notification(commands: &mut Commands, lore_id: &str, lore_db: &LoreDatabase) {
    if let Some(entry) = lore_db.get_entry(lore_id) {
        commands.spawn((
            Text::new(format!("📖 Discovered: {}", entry.title)),
            TextFont { font_size: 16.0, ..default() },
            TextColor(entry.category.color()),
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(100.0),
                right: Val::Px(20.0),
                padding: UiRect::all(Val::Px(10.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.8)),
            ZIndex(1000),
            LoreNotification { lifetime: 3.0 },
        ));
    }
}

#[derive(Component)]
pub struct LoreNotification {
    pub lifetime: f32,
}

pub fn lore_notification_system(
    mut commands: Commands,
    mut notifications: Query<(Entity, &mut LoreNotification)>,
    time: Res<Time>,
) {
    for (entity, mut notification) in notifications.iter_mut() {
        notification.lifetime -= time.delta_secs();
        if notification.lifetime <= 0.0 {
            commands.entity(entity).insert(MarkedForDespawn);
        }
    }
}

// === HELPER FUNCTIONS ===
pub fn create_sample_lore_entries() -> Vec<LoreEntry> {
    vec![
        LoreEntry {
            id: "corp_memo_001".to_string(),
            title: "Project Neurovector Status".to_string(),
            content: "TO: All Department Heads\nFROM: Dr. Sarah Chen, R&D\n\nProject Neurovector has entered Phase 3 testing. Initial trials show promising results for civilian crowd control applications. However, we're seeing unexpected side effects in prolonged exposure scenarios.\n\nRecommend proceeding with caution. The board wants results, but we can't afford another incident like the Tokyo facility.\n\n- Dr. Chen".to_string(),
            category: LoreCategory::CorpMemo,
            discovered: false,
            source: "Terminal A-7".to_string(),
        },
        LoreEntry {
            id: "personal_log_002".to_string(),
            title: "Marcus - Day 14".to_string(),
            content: "They think I don't know what they're doing in Sub-Level 3. The screaming stopped two days ago. That's not a good sign.\n\nFound Sarah's research notes in the trash. The neural interface isn't just for crowd control - they're trying to create permanent compliance. Turn people into puppets.\n\nI need to get this information out. If they catch me...".to_string(),
            category: LoreCategory::PersonalLog,
            discovered: false,
            source: "Hidden Drive".to_string(),
        },
        LoreEntry {
            id: "news_report_003".to_string(),
            title: "Underground Violence Escalates".to_string(),
            content: "Neo-Tokyo News Network - Reports of coordinated attacks on corporate facilities continue to rise. Authorities suspect the involvement of the shadowy organization known only as 'The Resistance.'\n\nCorporate Security Chief Williams stated: 'These terrorists threaten the stability that our citizens depend on. We will not negotiate with extremists.'\n\nMeanwhile, civilian casualties from police response continue to mount...".to_string(),
            category: LoreCategory::NewsReport,
            discovered: false,
            source: "News Terminal".to_string(),
        },
    ]
}

// === INTEGRATION WITH EXISTING SYSTEMS ===
pub fn add_lore_to_terminal(commands: &mut Commands, entity: Entity, lore_ids: Vec<String>) {
    commands.entity(entity).insert(LoreSource::new(lore_ids));
}

pub fn add_hackable_lore(commands: &mut Commands, entity: Entity, lore_ids: Vec<String>) {
    commands.entity(entity).insert(LoreSource::new(lore_ids).hackable());
}