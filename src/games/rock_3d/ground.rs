//! Ancoragem de props no terreno da pedreira.

use crate::assets::sample_quarry_height;
use crate::math::Vec3;

/// Elevação Y para assentar a base do modelo no chão (fallback sem malha).
pub fn model_ground_lift(model_id: &str, scale: f32) -> f32 {
    if model_id.starts_with("rock_scan_") {
        return 0.0;
    }
    match model_id {
        "boulder_small" | "boulder_medium" | "rock_hand" => 0.0,
        "target" => 0.55 * scale,
        "boulder_a" | "boulder_b" | "boulder_c" | "boulder_d" | "boulder_e" | "boulder_f" => {
            1.05 * scale
        }
        "dead_tree_a" | "dead_tree_b" => 0.0,
        _ => 0.9 * scale,
    }
}

pub fn snap_to_quarry_ground(x: f32, z: f32, foot_offset: f32, scale: f32) -> Vec3 {
    let ground = sample_quarry_height(x, z);
    Vec3::new(x, ground + foot_offset * scale, z)
}

pub fn snap_to_quarry_ground_heuristic(x: f32, z: f32, model_id: &str, scale: f32) -> Vec3 {
    snap_to_quarry_ground(x, z, model_ground_lift(model_id, scale) / scale.max(0.001), scale)
}

pub fn snap_target_position(x: f32, z: f32, height: f32) -> Vec3 {
    let ground = sample_quarry_height(x, z);
    Vec3::new(x, ground + height, z)
}
