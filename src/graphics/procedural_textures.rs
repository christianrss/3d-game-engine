//! Texturas procedurais geradas na CPU.

use crate::graphics::TextureData;

/// Normal map RGBA a partir de ruído — detalhe para primitivas sem textura.
pub fn generate_noise_normal(size: u32, seed: u32) -> TextureData {
    let mut height = vec![0.0f32; (size * size) as usize];
    for z in 0..size {
        for x in 0..size {
            let fx = x as f32 / size as f32 * 6.0 + seed as f32 * 0.17;
            let fz = z as f32 / size as f32 * 6.0 + seed as f32 * 0.23;
            let h = (fx * 1.7 + fz * 2.3).sin() * 0.35
                + (fx * 4.1 - fz * 3.7).cos() * 0.18
                + (fx * 9.3 + fz * 8.1).sin() * 0.08;
            height[(z * size + x) as usize] = h;
        }
    }

    let mut pixels = vec![0u8; (size * size * 4) as usize];
    let eps = 1.0 / size as f32;

    for z in 0..size {
        for x in 0..size {
            let h = |dx: i32, dz: i32| -> f32 {
                let nx = (x as i32 + dx).clamp(0, size as i32 - 1) as u32;
                let nz = (z as i32 + dz).clamp(0, size as i32 - 1) as u32;
                height[(nz * size + nx) as usize]
            };
            let dx = (h(1, 0) - h(-1, 0)) / (2.0 * eps);
            let dz = (h(0, 1) - h(0, -1)) / (2.0 * eps);
            let n = [-dx, 1.0, -dz];
            let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt().max(0.0001);
            let i = ((z * size + x) * 4) as usize;
            pixels[i] = ((n[0] / len * 0.5 + 0.5) * 255.0) as u8;
            pixels[i + 1] = ((n[1] / len * 0.5 + 0.5) * 255.0) as u8;
            pixels[i + 2] = ((n[2] / len * 0.5 + 0.5) * 255.0) as u8;
            pixels[i + 3] = 255;
        }
    }

    TextureData {
        width: size,
        height: size,
        pixels,
    }
}
