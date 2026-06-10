//! Utilitários compartilhados entre packs de assets por jogo.

use crate::assets::gltf_loader::load_gltf_prop;
use crate::assets::loader::{load_texture, ModelAsset};
use crate::assets::procedural::generate_boulder;
use crate::assets::terrain::generate_desert_terrain;
use crate::graphics::TextureData;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub struct PbrTextures {
    pub sand_albedo: TextureData,
    pub sand_normal: TextureData,
    pub sand_rough: TextureData,
    pub sand_ao: TextureData,
    pub rock_albedo: TextureData,
    pub rock_normal: TextureData,
    pub rock_rough: TextureData,
}

pub fn asset_root() -> PathBuf {
    PathBuf::from("assets")
}

pub fn load_pbr_textures(root: &Path) -> Result<PbrTextures, String> {
    let tex_dir = root.join("textures");
    let sand_albedo = load_texture(tex_dir.join("sand_diff.jpg"))?;
    let sand_normal = load_texture(tex_dir.join("sand_normal.jpg"))?;
    let sand_rough = if tex_dir.join("sand_rough.jpg").exists() {
        load_texture(tex_dir.join("sand_rough.jpg"))?
    } else {
        generate_roughness_from_albedo(&sand_albedo)
    };
    let sand_ao = generate_ao_from_albedo(&sand_albedo);

    let rock_dir = tex_dir.join("rock");
    let rock_albedo = load_texture(rock_dir.join("rock_diff.jpg"))?;
    let rock_normal = load_texture(rock_dir.join("rock_normal.jpg"))?;
    let rock_rough = load_texture(rock_dir.join("rock_rough.jpg"))?;

    log::info!(
        "PBR areia {}x{} + rocha {}x{}",
        sand_albedo.width,
        sand_albedo.height,
        rock_albedo.width,
        rock_albedo.height
    );

    Ok(PbrTextures {
        sand_albedo,
        sand_normal,
        sand_rough,
        sand_ao,
        rock_albedo,
        rock_normal,
        rock_rough,
    })
}

pub fn load_rock_scans(root: &Path, models: &mut HashMap<String, ModelAsset>) {
    for (id, path, size) in [
        ("rock_scan_a", "models/boulder_01.gltf", 2.6f32),
        ("rock_scan_b", "models/coast_rocks_01.gltf", 3.8f32),
        ("rock_scan_c", "models/namaqualand_boulder_02.gltf", 2.4f32),
    ] {
        let p = root.join(path);
        if p.exists() {
            match load_gltf_prop(&p, size) {
                Ok(m) => {
                    models.insert(id.into(), m);
                }
                Err(e) => log::warn!("Scan {id}: {e}"),
            }
        }
    }
}

pub fn load_procedural_boulders(models: &mut HashMap<String, ModelAsset>) {
    for (id, seed, subdiv) in [
        ("boulder_a", 1u32, 4),
        ("boulder_b", 2, 4),
        ("boulder_c", 3, 3),
        ("boulder_d", 4, 5),
        ("boulder_e", 5, 3),
        ("boulder_f", 6, 4),
    ] {
        models.insert(
            id.to_string(),
            ModelAsset {
                name: id.into(),
                mesh: generate_boulder(seed, 1.0, subdiv),
                texture_path: None,
                tiling: 1.0,
            },
        );
    }
}

pub fn ensure_rock_scan_fallbacks(models: &mut HashMap<String, ModelAsset>) {
    for (scan, fallback) in [
        ("rock_scan_a", "boulder_a"),
        ("rock_scan_b", "boulder_b"),
        ("rock_scan_c", "boulder_c"),
    ] {
        if !models.contains_key(scan) {
            if let Some(m) = models.get(fallback).cloned() {
                models.insert(scan.into(), m);
            }
        }
    }
}

pub fn load_gltf_prop_id(
    root: &Path,
    models: &mut HashMap<String, ModelAsset>,
    id: &str,
    path: &str,
    size: f32,
    fallback: &str,
) {
    let p = root.join(path);
    if p.exists() {
        if let Ok(m) = load_gltf_prop(&p, size) {
            models.insert(id.into(), m);
            return;
        }
    }
    if let Some(m) = models.get(fallback).cloned() {
        models.insert(id.into(), m);
    }
}

pub fn desert_terrain_mesh() -> ModelAsset {
    ModelAsset {
        name: "terrain".into(),
        mesh: generate_desert_terrain(320, 2048.0, crate::assets::terrain::TERRAIN_VISUAL_SCALE, 120.0),
        texture_path: None,
        tiling: 60.0,
    }
}

pub fn quarry_terrain_mesh() -> ModelAsset {
    ModelAsset {
        name: "terrain".into(),
        mesh: generate_desert_terrain(192, 1024.0, crate::assets::terrain::TERRAIN_VISUAL_SCALE * 0.45, 28.0),
        texture_path: None,
        tiling: 80.0,
    }
}

pub fn insert_model(models: &mut HashMap<String, ModelAsset>, id: &str, mesh: crate::graphics::Mesh) {
    models.insert(
        id.into(),
        ModelAsset {
            name: id.into(),
            mesh,
            texture_path: None,
            tiling: 1.0,
        },
    );
}

pub fn generate_roughness_from_albedo(albedo: &TextureData) -> TextureData {
    let mut pixels = vec![0u8; albedo.pixels.len()];
    for i in (0..albedo.pixels.len()).step_by(4) {
        let r = albedo.pixels[i] as f32 / 255.0;
        let g = albedo.pixels[i + 1] as f32 / 255.0;
        let b = albedo.pixels[i + 2] as f32 / 255.0;
        let lum = r * 0.299 + g * 0.587 + b * 0.114;
        let rough = (0.75 + lum * 0.2) * 255.0;
        pixels[i] = rough as u8;
        pixels[i + 1] = rough as u8;
        pixels[i + 2] = rough as u8;
        pixels[i + 3] = 255;
    }
    TextureData {
        width: albedo.width,
        height: albedo.height,
        pixels,
    }
}

pub fn generate_ao_from_albedo(albedo: &TextureData) -> TextureData {
    let w = albedo.width as usize;
    let h = albedo.height as usize;
    let mut pixels = vec![200u8; albedo.pixels.len()];
    for y in 1..h - 1 {
        for x in 1..w - 1 {
            let i = (y * w + x) * 4;
            let r = albedo.pixels[i] as f32;
            let g = albedo.pixels[i + 1] as f32;
            let b = albedo.pixels[i + 2] as f32;
            let lum = r + g + b;
            let mut min_diff = 0.0f32;
            for (dx, dy) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
                let ni = ((y as i32 + dy) as usize * w + (x as i32 + dx) as usize) * 4;
                let nr = albedo.pixels[ni] as f32;
                let ng = albedo.pixels[ni + 1] as f32;
                let nb = albedo.pixels[ni + 2] as f32;
                min_diff = min_diff.max((lum - (nr + ng + nb)).abs());
            }
            let ao = (255.0 - min_diff * 0.15).clamp(140.0, 255.0) as u8;
            pixels[i] = ao;
            pixels[i + 1] = ao;
            pixels[i + 2] = ao;
            pixels[i + 3] = 255;
        }
    }
    TextureData {
        width: albedo.width,
        height: albedo.height,
        pixels,
    }
}
