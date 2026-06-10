//! Viewmodel FPS — rifle tático + braço/mão (malha detalhada procedural).

use crate::graphics::{Color, Mesh, Vertex};
use crate::math::Vec3;

/// Rifle estilo M4 + antebraço e mão direita (fallback procedural).
pub fn build_fps_viewmodel() -> Mesh {
    let mut parts = arm_parts();
    parts.extend(gun_parts());
    merge(&parts)
}

/// Apenas braço/mão — mesclado com modelo glTF da arma.
pub fn build_fps_arm() -> Mesh {
    merge(&arm_parts())
}

fn arm_parts() -> Vec<(Mesh, Vec3)> {
    let mut parts = Vec::new();
    parts.push((
        rounded_box(0.07, 0.06, 0.32, Color::rgb(0.72, 0.55, 0.42), 6),
        Vec3::new(0.12, -0.18, -0.15),
    ));
    parts.push((
        rounded_box(0.09, 0.05, 0.1, Color::rgb(0.68, 0.52, 0.4), 6),
        Vec3::new(0.1, -0.2, -0.02),
    ));
    parts
}

fn gun_parts() -> Vec<(Mesh, Vec3)> {
    let mut parts = Vec::new();
    parts.push((
        rounded_box(0.05, 0.1, 0.22, Color::rgb(0.22, 0.23, 0.25), 8),
        Vec3::new(0.08, -0.12, -0.28),
    ));
    // Cano
    parts.push((
        cylinder_y(0.018, 0.42, Color::rgb(0.18, 0.19, 0.2), 12),
        Vec3::new(0.08, -0.08, -0.55),
    ));
    // Guarda-mão
    parts.push((
        rounded_box(0.045, 0.055, 0.18, Color::rgb(0.28, 0.2, 0.14), 6),
        Vec3::new(0.08, -0.1, -0.42),
    ));
    // Coronha
    parts.push((
        rounded_box(0.04, 0.11, 0.2, Color::rgb(0.32, 0.2, 0.1), 6),
        Vec3::new(0.06, -0.1, -0.08),
    ));
    // Carregador
    parts.push((
        rounded_box(0.035, 0.14, 0.06, Color::rgb(0.2, 0.21, 0.22), 6),
        Vec3::new(0.08, -0.2, -0.25),
    ));
    // Mira traseira
    parts.push((
        rounded_box(0.02, 0.03, 0.02, Color::rgb(0.15, 0.15, 0.16), 4),
        Vec3::new(0.08, -0.04, -0.2),
    ));
    // Empunhadura (grip)
    parts.push((
        rounded_box(0.035, 0.1, 0.05, Color::rgb(0.25, 0.16, 0.1), 6),
        Vec3::new(0.08, -0.2, -0.18),
    ));

    parts
}

fn merge(parts: &[(Mesh, Vec3)]) -> Mesh {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    for (mesh, off) in parts {
        let base = vertices.len() as u32;
        for v in &mesh.vertices {
            let mut nv = *v;
            nv.position[0] += off.x;
            nv.position[1] += off.y;
            nv.position[2] += off.z;
            vertices.push(nv);
        }
        indices.extend(mesh.indices.iter().map(|i| base + i));
    }
    Mesh { vertices, indices }
}

fn rounded_box(w: f32, h: f32, d: f32, color: Color, segs: u32) -> Mesh {
    cylinder_y(w.min(h) * 0.5, d, color, segs.max(4))
}

fn cylinder_y(radius: f32, height: f32, color: Color, segments: u32) -> Mesh {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    let hh = height / 2.0;
    for i in 0..=segments {
        let a = i as f32 / segments as f32 * std::f32::consts::TAU;
        let x = a.cos() * radius;
        let z = a.sin() * radius;
        let n = [a.cos(), 0.0, a.sin()];
        vertices.push(Vertex::new([x, -hh, z], n, [0.0, 0.0], color));
        vertices.push(Vertex::new([x, hh, z], n, [1.0, 1.0], color));
    }
    for i in 0..segments {
        let a = i * 2;
        indices.extend_from_slice(&[a, a + 1, a + 2, a + 2, a + 1, a + 3]);
    }
    Mesh { vertices, indices }
}
