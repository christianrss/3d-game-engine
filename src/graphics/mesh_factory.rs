//! Gera meshes coloridas por forma — cada objeto usa sua cor correta.

use crate::graphics::primitives::{cylinder, plane, rounded_box, sphere};
use crate::graphics::{Color, Mesh};

/// Cria uma mesh primitiva com a cor indicada.
pub fn shape_mesh(shape: &str, color: Color) -> Mesh {
    match shape {
        "plane" => plane(1.0, color),
        "sphere" => sphere(1.0, color, 32, 24),
        "cylinder" => cylinder(1.0, 1.0, color, 32),
        "cube" => rounded_box(1.0, color, 2),
        _ => sphere(1.0, color, 24, 18),
    }
}

/// Chave única para cache GPU (forma + cor).
pub fn mesh_cache_key(shape: &str, color: Color) -> String {
    format!(
        "{}_{}_{}_{}",
        shape,
        (color.r * 255.0) as u8,
        (color.g * 255.0) as u8,
        (color.b * 255.0) as u8
    )
}
