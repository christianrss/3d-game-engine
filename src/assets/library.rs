//! Biblioteca de assets — modelos procedural HQ + texturas PBR Poly Haven.

use crate::assets::gltf_loader::{
    load_gltf_viewmodel, merge_meshes, offset_muzzle, viewmodel_gun_candidates,
};
use crate::assets::loader::{load_texture, ModelAsset};
use crate::assets::procedural::{generate_boulder, generate_dead_tree, generate_shooting_target};
use crate::assets::viewmodel::{build_fps_arm, build_fps_viewmodel};
use crate::math::Vec3;
use crate::assets::terrain::generate_desert_terrain;
use crate::graphics::TextureData;
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug)]
pub struct AssetLibrary {
    pub models: HashMap<String, ModelAsset>,
    pub viewmodel: ModelAsset,
    pub viewmodel_muzzle: Vec3,
    pub terrain: ModelAsset,
    pub sand_albedo: TextureData,
    pub sand_normal: TextureData,
    pub sand_rough: TextureData,
    pub sand_ao: TextureData,
    pub root: PathBuf,
}

impl AssetLibrary {
    pub fn load() -> Result<Self, String> {
        let root = PathBuf::from("assets");
        let tex_dir = root.join("textures");

        let mut models = HashMap::new();

        let boulders = [
            ("boulder_a", 1u32, 2.2, 3),
            ("boulder_b", 2, 1.8, 3),
            ("boulder_c", 3, 1.4, 2),
            ("boulder_d", 4, 2.8, 4),
            ("boulder_e", 5, 1.0, 2),
            ("boulder_f", 6, 1.6, 3),
        ];
        for (id, seed, _scale_hint, subdiv) in boulders {
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

        for (id, seed) in [("dead_tree_a", 10u32), ("dead_tree_b", 11)] {
            models.insert(
                id.to_string(),
                ModelAsset {
                    name: id.into(),
                    mesh: generate_dead_tree(seed, 3.5),
                    texture_path: None,
                    tiling: 1.0,
                },
            );
        }

        models.insert(
            "target".into(),
            ModelAsset {
                name: "target".into(),
                mesh: generate_shooting_target(),
                texture_path: None,
                tiling: 1.0,
            },
        );

        let (viewmodel, viewmodel_muzzle) = load_viewmodel_asset(&root);

        let terrain_mesh = generate_desert_terrain(256, 220.0, 6.0, 60.0);
        let terrain = ModelAsset {
            name: "terrain".into(),
            mesh: terrain_mesh,
            texture_path: None,
            tiling: 60.0,
        };

        let sand_albedo = load_texture(tex_dir.join("sand_diff.jpg"))?;
        let sand_normal = load_texture(tex_dir.join("sand_normal.jpg"))?;
        let sand_rough = if tex_dir.join("sand_rough.jpg").exists() {
            load_texture(tex_dir.join("sand_rough.jpg"))?
        } else {
            generate_roughness_from_albedo(&sand_albedo)
        };
        let sand_ao = generate_ao_from_albedo(&sand_albedo);

        log::info!(
            "Texturas PBR: {}x{} (Poly Haven / procedural)",
            sand_albedo.width,
            sand_albedo.height
        );

        Ok(Self {
            models,
            viewmodel,
            viewmodel_muzzle,
            terrain,
            sand_albedo,
            sand_normal,
            sand_rough,
            sand_ao,
            root,
        })
    }

    pub fn get_model(&self, id: &str) -> Option<&ModelAsset> {
        self.models.get(id)
    }
}

fn load_viewmodel_asset(root: &PathBuf) -> (ModelAsset, Vec3) {
    for path in viewmodel_gun_candidates(root) {
        if is_avocado_sample(&path) {
            log::warn!("Ignorando sample Avocado em {}", path.display());
            continue;
        }
        match load_gltf_viewmodel(&path, "viewmodel") {
            Ok(gun) => {
                let arm = build_fps_arm();
                let mesh = merge_meshes(&[gun.mesh, arm]);
                let muzzle = offset_muzzle(gun.muzzle_local, Vec3::ZERO);
                log::info!("Viewmodel: glTF {} + braço procedural", path.display());
                return (
                    ModelAsset {
                        name: "viewmodel".into(),
                        mesh,
                        texture_path: gun.texture_path,
                        tiling: 1.0,
                    },
                    muzzle,
                );
            }
            Err(e) => log::warn!("glTF {}: {e}", path.display()),
        }
    }

    log::info!("Viewmodel procedural (nenhum glTF válido em assets/gun/)");
    (
        ModelAsset {
            name: "viewmodel".into(),
            mesh: build_fps_viewmodel(),
            texture_path: None,
            tiling: 1.0,
        },
        Vec3::new(0.08, -0.08, -0.78),
    )
}

fn is_avocado_sample(path: &std::path::Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| e.eq_ignore_ascii_case("gltf"))
        .unwrap_or(false)
        && std::fs::read_to_string(path)
            .map(|s| s.contains("Avocado"))
            .unwrap_or(false)
}

fn generate_roughness_from_albedo(albedo: &TextureData) -> TextureData {
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

fn generate_ao_from_albedo(albedo: &TextureData) -> TextureData {
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
