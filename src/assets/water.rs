//! Malha de água — lago oásis com alta resolução de grid.

use crate::graphics::{Color, Mesh, Vertex};

/// Plano de água subdividido para ondas no vertex shader.
pub fn generate_water_plane(size: f32, grid: u32) -> Mesh {
    let half = size / 2.0;
    let step = size / grid as f32;
    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    for z in 0..=grid {
        for x in 0..=grid {
            let wx = -half + x as f32 * step;
            let wz = -half + z as f32 * step;
            let u = x as f32 / grid as f32;
            let v = z as f32 / grid as f32;
            vertices.push(Vertex::new(
                [wx, 0.0, wz],
                [0.0, 1.0, 0.0],
                [u * 4.0, v * 4.0],
                Color::rgb(0.15, 0.45, 0.65),
            ));
        }
    }

    for z in 0..grid {
        for x in 0..grid {
            let i = z * (grid + 1) + x;
            indices.extend_from_slice(&[i, i + grid + 1, i + 1, i + 1, i + grid + 1, i + grid + 2]);
        }
    }

    Mesh { vertices, indices }
}
