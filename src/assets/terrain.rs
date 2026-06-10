//! Terreno do deserto — heightmap procedural + UV para textura de areia.

use crate::graphics::{Color, Mesh, Vertex};
use noise::{NoiseFn, Perlin};

const TERRAIN_SEED: u32 = 42;
const HEIGHT_SCALE: f32 = 4.0;

fn height_at(perlin: &Perlin, wx: f32, wz: f32) -> f32 {
    let nx = wx * 0.02;
    let nz = wz * 0.02;
    perlin.get([nx as f64, nz as f64]) as f32 * HEIGHT_SCALE
        + perlin.get([nx as f64 * 2.5, nz as f64 * 2.5]) as f32 * HEIGHT_SCALE * 0.3
}

/// Altura do terreno em um ponto do mundo (para posicionar props e jogador).
pub fn sample_desert_height(wx: f32, wz: f32) -> f32 {
    height_at(&Perlin::new(TERRAIN_SEED), wx, wz)
}

/// Gera terreno ondulado estilo dunas com coordenadas UV para tiling de textura.
pub fn generate_desert_terrain(
    grid: u32,
    world_size: f32,
    height_scale: f32,
    uv_repeat: f32,
) -> Mesh {
    let perlin = Perlin::new(TERRAIN_SEED);
    let grid = grid.max(8);
    let half = world_size / 2.0;
    let step = world_size / grid as f32;

    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    for z in 0..=grid {
        for x in 0..=grid {
            let wx = -half + x as f32 * step;
            let wz = -half + z as f32 * step;

            let nx = wx * 0.02;
            let nz = wz * 0.02;
            let h = height_at(&perlin, wx, wz) * (height_scale / HEIGHT_SCALE);

            let eps = 0.5;
            let hx =
                perlin.get([(nx + eps) as f64, nz as f64]) as f32 - perlin.get([(nx - eps) as f64, nz as f64]) as f32;
            let hz =
                perlin.get([nx as f64, (nz + eps) as f64]) as f32 - perlin.get([nx as f64, (nz - eps) as f64]) as f32;
            let normal = glam::Vec3::new(-hx, 2.0, -hz).normalize();

            let u = (x as f32 / grid as f32) * uv_repeat;
            let v = (z as f32 / grid as f32) * uv_repeat;

            vertices.push(Vertex::new(
                [wx, h, wz],
                normal.to_array(),
                [u, v],
                Color::WHITE,
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
