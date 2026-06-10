//! Gera meshes coloridas por forma — cada objeto usa sua cor correta.

use crate::graphics::primitives::{cube, cylinder, plane, sphere};
use crate::graphics::{Color, Mesh};

/// Cria uma mesh primitiva com a cor indicada.
pub fn shape_mesh(shape: &str, color: Color) -> Mesh {
    match shape {
        "plane" => plane(1.0, color),
        "sphere" => sphere(1.0, color, 20, 16),
        "cylinder" => cylinder(1.0, 1.0, color, 20),
        "cube" => cube(color),
        _ => sphere(1.0, color, 16, 12),
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
