//! Assets mínimos para o Engine Studio (preview de terreno + rochas).

use crate::assets::common::{
    asset_root, desert_terrain_mesh, ensure_rock_scan_fallbacks, insert_model,
    load_pbr_textures, load_procedural_boulders, load_rock_scans,
};
use crate::assets::library::AssetLibrary;
use crate::assets::loader::ModelAsset;
use crate::assets::pack::AssetPack;
use crate::assets::procedural::generate_shooting_target;
use crate::assets::viewmodel::build_fps_viewmodel;
use crate::math::Vec3;
use std::collections::HashMap;

pub struct StudioAssetPack;

impl AssetPack for StudioAssetPack {
    fn id(&self) -> &'static str {
        crate::assets::pack::STUDIO_PACK
    }

    fn load(&self) -> Result<AssetLibrary, String> {
        let root = asset_root();
        let mut models = HashMap::new();

        load_rock_scans(&root, &mut models);
        load_procedural_boulders(&mut models);
        ensure_rock_scan_fallbacks(&mut models);
        insert_model(&mut models, "target", generate_shooting_target());

        let pbr = load_pbr_textures(&root)?;

        Ok(AssetLibrary {
            models,
            viewmodel: ModelAsset {
                name: "viewmodel".into(),
                mesh: build_fps_viewmodel(),
                texture_path: None,
                tiling: 1.0,
            },
            viewmodel_muzzle: Vec3::new(0.08, -0.08, -0.78),
            terrain: desert_terrain_mesh(),
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
