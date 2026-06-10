//! Carrega modelos OBJ (Kenney) e texturas PNG/JPG (Poly Haven).

use crate::graphics::{Color, Mesh, TextureData, Vertex};
use image::GenericImage;
use std::path::Path;

/// Modelo 3D carregado da CPU.
#[derive(Debug, Clone)]
pub struct ModelAsset {
    pub name: String,
    pub mesh: Mesh,
    /// Caminho da textura albedo (se houver)
    pub texture_path: Option<String>,
    pub tiling: f32,
}

/// Carrega um arquivo OBJ com cores do material MTL (Kenney).
pub fn load_obj(path: impl AsRef<Path>, name: &str, scale: f32) -> Result<ModelAsset, String> {
    let path = path.as_ref();
    let (models, materials) = tobj::load_obj(
        path,
        &tobj::LoadOptions {
            triangulate: true,
            single_index: true,
            ..Default::default()
        },
    )
    .map_err(|e| format!("OBJ {}: {e}", path.display()))?;
    let materials = materials.map_err(|e| format!("MTL {}: {e}", path.display()))?;

    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    let mut texture_path = None;

    for model in &models {
        let mesh = &model.mesh;
        let base = vertices.len() as u32;

        let color = if let Some(mat_id) = model.mesh.material_id {
            materials
                .get(mat_id)
                .map(|m| {
                    if let Some(tex) = &m.diffuse_texture {
                        if texture_path.is_none() {
                            let tex_path = path.parent().unwrap().join(tex);
                            if tex_path.exists() {
                                texture_path = Some(tex_path.to_string_lossy().to_string());
                            }
                        }
                    }
                    let diffuse = m.diffuse.unwrap_or([1.0, 1.0, 1.0]);
                    color_from_tobj(&diffuse)
                })
                .unwrap_or(Color::WHITE)
        } else {
            Color::WHITE
        };

        for i in 0..mesh.positions.len() / 3 {
            let pos = [
                mesh.positions[i * 3] * scale,
                mesh.positions[i * 3 + 1] * scale,
                mesh.positions[i * 3 + 2] * scale,
            ];
            let normal = if mesh.normals.len() >= (i + 1) * 3 {
                [
                    mesh.normals[i * 3],
                    mesh.normals[i * 3 + 1],
                    mesh.normals[i * 3 + 2],
                ]
            } else {
                [0.0, 1.0, 0.0]
            };
            let uv = if mesh.texcoords.len() >= (i + 1) * 2 {
                [mesh.texcoords[i * 2], mesh.texcoords[i * 2 + 1]]
            } else {
                [0.0, 0.0]
            };

            vertices.push(Vertex::new(pos, normal, uv, color));
        }

        indices.extend(mesh.indices.iter().map(|&idx| base + idx));
    }

    Ok(ModelAsset {
        name: name.to_string(),
        mesh: Mesh { vertices, indices },
        texture_path,
        tiling: 1.0,
    })
}

fn color_from_tobj(diffuse: &[f32; 3]) -> Color {
    Color::rgb(diffuse[0], diffuse[1], diffuse[2])
}

/// Carrega textura de disco (RGBA8).
pub fn load_texture_from_path(path: impl AsRef<Path>) -> Result<image::RgbaImage, String> {
    let path = path.as_ref();
    image::open(path).map_err(|e| e.to_string()).map(|img| img.to_rgba8())
}

pub fn load_texture(path: impl AsRef<Path>) -> Result<TextureData, String> {
    let path = path.as_ref();
    if path.exists() {
        let img = load_texture_from_path(path)?;
        let (w, h) = img.dimensions();
        return Ok(TextureData {
            width: w,
            height: h,
            pixels: img.into_raw(),
        });
    }
    log::warn!("Textura ausente ({}), usando procedural", path.display());
    Ok(generate_procedural_sand(512))
}

/// Areia procedural quando download CC0 não está disponível.
pub fn generate_procedural_sand(size: u32) -> TextureData {
    let mut pixels = vec![0u8; (size * size * 4) as usize];
    for z in 0..size {
        for x in 0..size {
            let fx = x as f32 / size as f32;
            let fz = z as f32 / size as f32;
            let n = ((fx * 47.0).sin() * (fz * 31.0).cos() * 0.5 + 0.5) * 0.15;
            let r = (0.82 + n) * 255.0;
            let g = (0.68 + n * 0.6) * 255.0;
            let b = (0.38 + n * 0.3) * 255.0;
            let i = ((z * size + x) * 4) as usize;
            pixels[i] = r as u8;
            pixels[i + 1] = g as u8;
            pixels[i + 2] = b as u8;
            pixels[i + 3] = 255;
        }
    }
    TextureData {
        width: size,
        height: size,
        pixels,
    }
}

pub fn generate_flat_normal(size: u32) -> TextureData {
    let mut pixels = vec![0u8; (size * size * 4) as usize];
    for i in (0..pixels.len()).step_by(4) {
        pixels[i] = 128;
        pixels[i + 1] = 128;
        pixels[i + 2] = 255;
        pixels[i + 3] = 255;
    }
    TextureData {
        width: size,
        height: size,
        pixels,
    }
}

