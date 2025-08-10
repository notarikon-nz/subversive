// src/systems/profiling.rs - Minimal working profiling for Bevy 0.16

use bevy::prelude::*;
use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use std::collections::VecDeque;
use bevy_egui::{egui, EguiContexts};

// === RESOURCES ===

#[derive(Resource)]
pub struct ProfilingConfig {
    pub enabled: bool,
    pub show_overlay: bool,
    pub log_interval: f32,
}

impl Default for ProfilingConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            show_overlay: false,
            log_interval: 1.0,
        }
    }
}

#[derive(Clone, Resource, Default, PartialEq)]
pub struct PerformanceMetrics {
    pub fps: f32,
    pub frame_time_ms: f32,
    pub frame_time_history: VecDeque<f32>,
    pub total_entities: usize,
    pub agent_count: usize,
    pub enemy_count: usize,
    pub civilian_count: usize,

    pub physics_ms:f32,
    pub depth_sort_ms: f32,
    pub ai_ms: f32,
    pub pathfinding_ms: f32,
}

// === PLUGIN ===

pub struct ProfilingPlugin;

impl Plugin for ProfilingPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(FrameTimeDiagnosticsPlugin::default())
            .init_resource::<ProfilingConfig>()
            .init_resource::<PerformanceMetrics>()
            .add_systems(Update, (
                collect_metrics,
                toggle_overlay,
                render_overlay,
            ));
    }
}

// === SYSTEMS ===

fn collect_metrics(
    mut metrics: ResMut<PerformanceMetrics>,
    diagnostics: Res<DiagnosticsStore>,
    agents: Query<&crate::core::Agent>,
    enemies: Query<&crate::core::Enemy>,
    civilians: Query<&crate::core::Civilian>,
) {
    // Get FPS from diagnostics
    if let Some(fps) = diagnostics
        .get(&FrameTimeDiagnosticsPlugin::FPS)
        .and_then(|fps| fps.smoothed())
    {
        metrics.fps = fps as f32;
        metrics.frame_time_ms = if fps > 0.0 { 1000.0 / fps as f32 } else { 0.0 };
        let metrics_clone = metrics.clone();

        metrics.frame_time_history.push_back(metrics_clone.frame_time_ms);
        if metrics.frame_time_history.len() > 60 {
            metrics.frame_time_history.pop_front();
        }
    }
    
    // Count entities
    metrics.agent_count = agents.iter().count();
    metrics.enemy_count = enemies.iter().count();
    metrics.civilian_count = civilians.iter().count();
    metrics.total_entities = metrics.agent_count + metrics.enemy_count + metrics.civilian_count;
}

fn toggle_overlay(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut config: ResMut<ProfilingConfig>,
) {
    if keyboard.just_pressed(KeyCode::F12) {
        config.show_overlay = !config.show_overlay;
        info!("Profiling overlay: {}", if config.show_overlay { "ON" } else { "OFF" });
    }
}

fn render_overlay(
    mut contexts: EguiContexts,
    metrics: Res<PerformanceMetrics>,
    config: Res<ProfilingConfig>,
) {
    if !config.show_overlay {
        return;
    }
    
    if let Ok(ctx) = contexts.ctx_mut() {
    
        egui::Window::new("Performance")
            .default_pos(egui::pos2(10.0, 10.0))
            .show(ctx, |ui| {
                ui.heading("Performance Metrics");
                
                // Frame stats
                ui.label(format!("FPS: {:.1}", metrics.fps));
                ui.label(format!("Frame Time: {:.2}ms", metrics.frame_time_ms));
                
                // Entity counts
                ui.separator();
                ui.label(format!("Total Entities: {}", metrics.total_entities));
                ui.label(format!("  Agents: {}", metrics.agent_count));
                ui.label(format!("  Enemies: {}", metrics.enemy_count));
                ui.label(format!("  Civilians: {}", metrics.civilian_count));
                
                // Performance warning
                if metrics.frame_time_ms > 16.67 {
                    ui.colored_label(egui::Color32::YELLOW, "⚠ Below 60 FPS");
                }
                if metrics.frame_time_ms > 33.33 {
                    ui.colored_label(egui::Color32::RED, "⚠ Below 30 FPS");
                }
            });
    }
}