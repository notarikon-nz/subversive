// src/core/scene_cache.rs - New file for scene caching
use bevy::prelude::*;
use std::collections::HashMap;
use std::time::SystemTime;
use crate::systems::scenes::SceneData;

#[derive(Resource)]
pub struct SceneCache {
    scenes: HashMap<String, CachedScene>,
    cache_hits: u32,
    cache_misses: u32,
}

#[derive(Clone)]
struct CachedScene {
    data: SceneData,
    last_modified: SystemTime,
    file_path: String,
}

impl Default for SceneCache {
    fn default() -> Self {
        Self {
            scenes: HashMap::new(),
            cache_hits: 0,
            cache_misses: 0,
        }
    }
}

impl SceneCache {
    /// Get a scene from cache, loading/reloading if necessary
    pub fn get_scene(&mut self, scene_name: &str) -> Option<&SceneData> {
        let file_path = format!("scenes/{}.json", scene_name);
        
        // Check if file exists
        let file_metadata = match std::fs::metadata(&file_path) {
            Ok(metadata) => metadata,
            Err(_) => {
                warn!("Scene file not found: {}", file_path);
                return None;
            }
        };
        
        let file_modified = file_metadata.modified().unwrap_or(SystemTime::UNIX_EPOCH);
        
        // Check if we need to load/reload
        let needs_reload = match self.scenes.get(scene_name) {
            Some(cached) => file_modified > cached.last_modified,
            None => true,
        };
        
        if needs_reload {
            match self.load_scene_from_file(scene_name, &file_path, file_modified) {
                Some(scene_data) => {
                    self.cache_misses += 1;
                    info!("Scene '{}' loaded/reloaded (cache misses: {})", scene_name, self.cache_misses);
                }
                None => return None,
            }
        } else {
            self.cache_hits += 1;
        }
        
        self.scenes.get(scene_name).map(|cached| &cached.data)
    }
    
    /// Force reload a specific scene (useful for development)
    pub fn reload_scene(&mut self, scene_name: &str) -> bool {
        let file_path = format!("scenes/{}.json", scene_name);
        
        match std::fs::metadata(&file_path) {
            Ok(metadata) => {
                let file_modified = metadata.modified().unwrap_or(SystemTime::UNIX_EPOCH);
                self.load_scene_from_file(scene_name, &file_path, file_modified).is_some()
            }
            Err(_) => false,
        }
    }
    
    /// Clear entire cache (useful for development)
    pub fn clear_cache(&mut self) {
        self.scenes.clear();
        self.cache_hits = 0;
        self.cache_misses = 0;
        info!("Scene cache cleared");
    }
    
    /// Get cache statistics
    pub fn get_stats(&self) -> (u32, u32, f32) {
        let total = self.cache_hits + self.cache_misses;
        let hit_rate = if total > 0 { 
            self.cache_hits as f32 / total as f32 
        } else { 
            0.0 
        };
        (self.cache_hits, self.cache_misses, hit_rate)
    }
    
    /// Preload common scenes at startup
    pub fn preload_scenes(&mut self, scene_names: &[&str]) {
        for &scene_name in scene_names {
            self.get_scene(scene_name);
        }
        info!("Preloaded {} scenes", scene_names.len());
    }
    
    fn load_scene_from_file(&mut self, scene_name: &str, file_path: &str, file_modified: SystemTime) -> Option<SceneData> {
        match std::fs::read_to_string(file_path) {
            Ok(content) => {
                match serde_json::from_str::<SceneData>(&content) {
                    Ok(scene_data) => {
                        let cached_scene = CachedScene {
                            data: scene_data.clone(),
                            last_modified: file_modified,
                            file_path: file_path.to_string(),
                        };
                        
                        self.scenes.insert(scene_name.to_string(), cached_scene);
                        Some(scene_data)
                    }
                    Err(e) => {
                        error!("Failed to parse scene {}: {}", scene_name, e);
                        None
                    }
                }
            }
            Err(e) => {
                error!("Failed to read scene file {}: {}", file_path, e);
                None
            }
        }
    }
}

// Development helper system
pub fn scene_cache_debug_system(
    cache: Res<SceneCache>,
    input: Res<ButtonInput<KeyCode>>,
) {
    if input.just_pressed(KeyCode::F9) {
        let (hits, misses, hit_rate) = cache.get_stats();
        info!("Scene Cache Stats - Hits: {}, Misses: {}, Hit Rate: {:.1}%", 
              hits, misses, hit_rate * 100.0);
    }
}