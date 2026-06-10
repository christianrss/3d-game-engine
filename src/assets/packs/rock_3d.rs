//! Assets do Rock 3D — pedras, alvos, braço FPS; sem fauna/deserto/construção.

use crate::assets::common::{
    asset_root, ensure_rock_scan_fallbacks, insert_model, load_gltf_prop_id, load_pbr_textures,
    load_procedural_boulders, load_rock_scans, quarry_terrain_mesh,
};
use crate::assets::library::AssetLibrary;
use crate::assets::loader::ModelAsset;
use crate::assets::pack::AssetPack;
use crate::assets::procedural::generate_shooting_target;
use crate::assets::viewmodel::build_fps_arm;
use crate::math::Vec3;
use std::collections::HashMap;

pub struct Rock3DAssetPack;

impl AssetPack for Rock3DAssetPack {
    fn id(&self) -> &'static str {
        crate::assets::pack::ROCK_3D_PACK
    }

    fn load(&self) -> Result<AssetLibrary, String> {
        let root = asset_root();
        let mut models = HashMap::new();

        load_rock_scans(&root, &mut models);
        load_procedural_boulders(&mut models);
        ensure_rock_scan_fallbacks(&mut models);

        for (id, path, size, fallback) in [
            ("boulder_small", "models/boulder_01.gltf", 0.34f32, "boulder_e"),
            ("boulder_medium", "models/namaqualand_boulder_02.gltf", 0.72f32, "boulder_b"),
            ("rock_hand", "models/boulder_01.gltf", 0.16f32, "boulder_e"),
        ] {
            load_gltf_prop_id(&root, &mut models, id, path, size, fallback);
        }

        insert_model(&mut models, "fps_arm", build_fps_arm());
        insert_model(&mut models, "target", generate_shooting_target());

        let pbr = load_pbr_textures(&root)?;
        let fps_arm_mesh = models
            .get("fps_arm")
            .cloned()
            .unwrap_or_else(|| ModelAsset {
                name: "fps_arm".into(),
                mesh: build_fps_arm(),
                texture_path: None,
                tiling: 1.0,
            });

        log::info!(
            "Rock 3D pack — {} modelos (pedras + alvos + braço)",
            models.len()
        );

        Ok(AssetLibrary {
            models,
            viewmodel: fps_arm_mesh,
            viewmodel_muzzle: Vec3::new(0.1, -0.1, -0.3),
            terrain: quarry_terrain_mesh(),
            sand_albedo: pbr.sand_albedo,
            sand_normal: pbr.sand_normal,
            sand_rough: pbr.sand_rough,
            sand_ao: pbr.sand_ao,
            rock_albedo: pbr.rock_albedo,
            rock_normal: pbr.rock_normal,
            rock_rough: pbr.rock_rough,
            root,
        })
    }
}
