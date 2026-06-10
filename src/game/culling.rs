//! Culling por distancia e frustum — nao renderiza o mundo inteiro de uma vez.

use crate::math::Vec3;

pub const SHADOW_DIST: f32 = 55.0;
pub const REFLECT_DIST: f32 = 45.0;
pub const RADAR_RANGE: f32 = 90.0;
pub const BEACON_RANGE: f32 = 45.0;
pub const AI_FULL_DIST: f32 = 90.0;
pub const AI_SLEEP_DIST: f32 = 220.0;

pub fn max_draw_distance(model_id: &str) -> f32 {
    match model_id {
        "terrain" => f32::INFINITY,
        "pyramid" => 420.0,
        "oasis_water" => 110.0,
        "ufo" => 480.0,
        "mirage" => 100.0,
        "sheep" | "camel" | "goat" | "hermit" | "et" | "snake" | "scorpion" | "lion" | "dog" => {
            190.0
        }
        "bird" => 120.0,
        "grass_clump" | "palm_tree" => 85.0,
        "well" | "mountain_rock" | "desert_castle" => 320.0,
        "desert_cabin" | "desert_house" | "desert_market" | "desert_tower" | "desert_caravan" => {
            180.0
        }
        "npc_vendor" | "npc_soldier" | "npc_caravan" | "npc_hunter" | "npc_builder" | "npc_citizen" => {
            120.0
        }
        "stream_water" => 90.0,
        "target" | "fence_post" | "dirt_block" | "stone_block" | "stone_wall" | "wood_wall" => {
            140.0
        }
        _ => 150.0,
    }
}

pub fn dist_sq(a: Vec3, b: Vec3) -> f32 {
    let dx = a.x - b.x;
    let dz = a.z - b.z;
    let dy = a.y - b.y;
    dx * dx + dy * dy + dz * dz
}

pub fn dist_sq_xz(a: Vec3, b: Vec3) -> f32 {
    let dx = a.x - b.x;
    let dz = a.z - b.z;
    dx * dx + dz * dz
}

/// Visivel para draw principal.
pub fn should_draw(pos: Vec3, cam_pos: Vec3, cam_forward: Vec3, model_id: &str) -> bool {
    let to = pos - cam_pos;
    let d2 = to.length_squared();
    if d2 < 9.0 {
        return true;
    }
    let max_d = max_draw_distance(model_id);
    if !max_d.is_finite() {
        return true;
    }
    if d2 > max_d * max_d {
        return false;
    }
    let dist = d2.sqrt();
    to.dot(cam_forward) / dist > -0.15
}

pub fn should_shadow(pos: Vec3, cam_pos: Vec3, model_id: &str) -> bool {
    if model_id == "terrain" || model_id == "mirage" || model_id == "ufo" {
        return false;
    }
    dist_sq_xz(pos, cam_pos) < SHADOW_DIST * SHADOW_DIST
}

pub fn should_reflect(pos: Vec3, cam_pos: Vec3) -> bool {
    dist_sq_xz(pos, cam_pos) < REFLECT_DIST * REFLECT_DIST
}

pub fn in_ai_range(pos: Vec3, player_pos: Vec3) -> bool {
    dist_sq_xz(pos, player_pos) < AI_FULL_DIST * AI_FULL_DIST
}

pub fn ai_sleeping(pos: Vec3, player_pos: Vec3) -> bool {
    dist_sq_xz(pos, player_pos) > AI_SLEEP_DIST * AI_SLEEP_DIST
}
