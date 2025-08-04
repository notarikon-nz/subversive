// src/systems/ui/loading_system.rs
use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use std::collections::HashMap;
use std::path::Path;
use crate::core::*;

#[derive(Resource, Default)]
pub struct LoadingProgress {
    pub progress: f32,
    pub current_task: String,
    pub completed_tasks: u32,
    pub total_tasks: u32,
}

#[derive(Resource, Default, Clone)]
pub struct SvgCache {
    pub textures: HashMap<String, Handle<Image>>,
}

pub fn loading_system(
    mut commands: Commands,
    mut loading_progress: Local<Option<LoadingProgress>>,
    mut svg_cache: Local<Option<SvgCache>>,
    mut contexts: EguiContexts,
    mut images: ResMut<Assets<Image>>,
    windows: Query<&Window>,
    mut next_state: ResMut<NextState<GameState>>,
    mut startup_frames: Local<u32>,
) {
    // Wait a few frames for egui to initialize
    *startup_frames += 1;
    if *startup_frames < 3 {
        return;
    }
    // Initialize on first run
    if loading_progress.is_none() {
        *loading_progress = Some(LoadingProgress {
            total_tasks: 6,
            current_task: "Initializing...".to_string(),
            ..default()
        });
        *svg_cache = Some(SvgCache::default());
    }

    let progress = loading_progress.as_mut().unwrap();
    let cache = svg_cache.as_mut().unwrap();
    
    let Ok(window) = windows.single() else { return; };
    let resolution = Vec2::new(window.width(), window.height());

    // Process loading tasks
    if progress.completed_tasks < progress.total_tasks {
        match progress.completed_tasks {
            0 => load_ui_svg("vectors/interface.svg", "interface", cache, &mut images, resolution, progress),
            1 => load_ui_svg("vectors/buttons.svg", "buttons", cache, &mut images, resolution, progress),
            2 => load_ui_svg("vectors/icons.svg", "icons", cache, &mut images, resolution, progress),
            3 => load_ui_svg("vectors/panels.svg", "panels", cache, &mut images, resolution, progress),
            4 => load_ui_svg("vectors/hud.svg", "hud", cache, &mut images, resolution, progress),
            5 => {
                progress.current_task = "Finalizing...".to_string();
                progress.completed_tasks += 1;
            }
            _ => {}
        }
        progress.progress = progress.completed_tasks as f32 / progress.total_tasks as f32;
    }

    // Render loading screen
    if let Ok(ctx) = contexts.ctx_mut() {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(300.0); // Fixed value instead of window height calculation
                
                ui.heading("SUBVERSIVE");
                ui.add_space(30.0);
                
                let bar = egui::ProgressBar::new(progress.progress)
                    .desired_width(400.0)
                    .text(&progress.current_task);
                ui.add(bar);
                
                ui.add_space(15.0);
                ui.label(format!("Loading assets... {}/{}", progress.completed_tasks, progress.total_tasks));
            });
        });
    }
    // Complete loading
    if progress.progress >= 1.0 {
        commands.insert_resource(cache.clone());
        next_state.set(GameState::MainMenu);
    }
}

fn load_ui_svg(
    path: &str,
    key: &str,
    cache: &mut SvgCache,
    images: &mut ResMut<Assets<Image>>,
    resolution: Vec2,
    progress: &mut LoadingProgress,
) {
    progress.current_task = format!("Loading {}...", key);
    
    let full_path = format!("assets/{}", path);
    let svg_path = Path::new(&full_path);
    
    // Use nsvg's correct API
    match nsvg::parse_file(svg_path, nsvg::Units::Pixel, 96.0) {
        Ok(svg_image) => {
            let scale = calc_scale(&svg_image, resolution);
            match svg_image.rasterize(scale) {
                Ok(raster_image) => {
                    let image = raster_to_bevy_image(raster_image, scale);
                    cache.textures.insert(key.to_string(), images.add(image));
                    info!("Loaded SVG: {} (scale: {:.2})", key, scale);
                }
                Err(e) => {
                    warn!("Failed to rasterize SVG {}: {:?}", key, e);
                    cache.textures.insert(key.to_string(), images.add(fallback_image()));
                }
            }
        }
        Err(e) => {
            warn!("Failed to load SVG: {} - {:?}, using fallback", path, e);
            cache.textures.insert(key.to_string(), images.add(fallback_image()));
        }
    }
    
    progress.completed_tasks += 1;
}

fn calc_scale(svg_image: &nsvg::SvgImage, resolution: Vec2) -> f32 {
    let svg_width = svg_image.width();
    let svg_height = svg_image.height();
    
    // Calculate scale based on target resolution vs design resolution
    let base_scale = (resolution.x / 1280.0).min(resolution.y / 720.0);
    base_scale.clamp(0.5, 3.0)
}

fn raster_to_bevy_image(raster_image: nsvg::image::RgbaImage, scale: f32) -> Image {
    let (width, height) = raster_image.dimensions();
    let data = raster_image.into_raw();
    
    Image::new(
        bevy::render::render_resource::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        bevy::render::render_resource::TextureDimension::D2,
        data,
        bevy::render::render_resource::TextureFormat::Rgba8UnormSrgb,
        bevy::render::render_asset::RenderAssetUsages::RENDER_WORLD,
    )
}

fn fallback_image() -> Image {
    let mut data = Vec::with_capacity(64 * 64 * 4);
    for _ in 0..64 * 64 {
        data.extend_from_slice(&[0, 255, 255, 255]); // RGBA cyan
    }
    Image::new(
        bevy::render::render_resource::Extent3d {
            width: 64,
            height: 64,
            depth_or_array_layers: 1,
        },
        bevy::render::render_resource::TextureDimension::D2,
        data,
        bevy::render::render_resource::TextureFormat::Rgba8UnormSrgb,
        bevy::render::render_asset::RenderAssetUsages::RENDER_WORLD,
    )
}

impl SvgCache {
    pub fn get(&self, key: &str) -> Option<&Handle<Image>> {
        self.textures.get(key)
    }
}

// ===== LEGACY =====
pub fn old_loading_system(
    mut next_state: ResMut<NextState<GameState>>,
    mut frame_count: Local<u32>,
) {
    *frame_count += 1;
    if *frame_count > 10 {  // Wait 10 frames
        next_state.set(GameState::MainMenu);
    }
}