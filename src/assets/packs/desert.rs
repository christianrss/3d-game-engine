//! Assets do Desert Shooter — mundo aberto, construção, fauna, arma FPS.

use crate::assets::common::{
    asset_root, desert_terrain_mesh, ensure_rock_scan_fallbacks, insert_model,
    load_gltf_prop_id, load_pbr_textures, load_procedural_boulders, load_rock_scans,
};
use crate::assets::gltf_loader::{
    load_gltf_viewmodel, merge_meshes, offset_muzzle, viewmodel_gun_candidates,
};
use crate::assets::library::AssetLibrary;
use crate::assets::loader::ModelAsset;
use crate::assets::pack::AssetPack;
use crate::assets::creatures::{generate_fence_post, generate_sheep, generate_stone_wall, generate_wood_wall};
use crate::assets::props::{
    generate_bird, generate_camel, generate_desert_cabin, generate_desert_caravan,
    generate_desert_castle, generate_desert_house, generate_desert_market, generate_desert_tower,
    generate_dog, generate_et, generate_goat, generate_grass_clump, generate_hermit, generate_lion,
    generate_mirage, generate_mountain_rock, generate_npc_builder, generate_npc_caravan,
    generate_npc_citizen, generate_npc_hunter, generate_npc_soldier, generate_npc_vendor,
    generate_palm_tree, generate_pyramid, generate_scorpion, generate_snake, generate_ufo,
    generate_well,
};
use crate::assets::procedural::{generate_dead_tree, generate_shooting_target};
use crate::assets::viewmodel::{build_fps_arm, build_fps_viewmodel};
use crate::assets::water::generate_water_plane;
use crate::math::Vec3;
use std::collections::HashMap;
use std::path::PathBuf;

pub struct DesertAssetPack;

impl AssetPack for DesertAssetPack {
    fn id(&self) -> &'static str {
        crate::assets::pack::DESERT_SHOOTER_PACK
    }

    fn load(&self) -> Result<AssetLibrary, String> {
        let root = asset_root();
        let mut models = HashMap::new();

        load_rock_scans(&root, &mut models);
        load_procedural_boulders(&mut models);
        ensure_rock_scan_fallbacks(&mut models);

        // Props glTF para construção (substitui blocos Minecraft)
        load_gltf_prop_id(&root, &mut models, "rock_prop_s", "models/boulder_01.gltf", 1.0, "boulder_e");
        load_gltf_prop_id(
            &root,
            &mut models,
            "rock_prop_m",
            "models/namaqualand_boulder_02.gltf",
            1.4,
            "boulder_b",
        );
        load_gltf_prop_id(
            &root,
            &mut models,
            "rock_wall",
            "models/coast_rocks_01.gltf",
            2.2,
            "stone_wall",
        );
        load_gltf_prop_id(
            &root,
            &mut models,
            "sand_pile",
            "models/boulder_01.gltf",
            0.9,
            "boulder_c",
        );

        for (id, seed) in [("dead_tree_a", 10u32), ("dead_tree_b", 11)] {
            insert_model(&mut models, id, generate_dead_tree(seed, 3.5));
        }
        insert_model(&mut models, "target", generate_shooting_target());
        insert_model(&mut models, "sheep", generate_sheep());
        insert_model(&mut models, "fence_post", generate_fence_post());
        insert_model(&mut models, "stone_wall", generate_stone_wall());
        insert_model(&mut models, "wood_wall", generate_wood_wall());

        for (id, mesh_fn) in [
            ("camel", generate_camel as fn() -> _),
            ("goat", generate_goat),
            ("snake", generate_snake),
            ("scorpion", generate_scorpion),
            ("hermit", generate_hermit),
            ("et", generate_et),
            ("bird", generate_bird),
            ("lion", generate_lion),
            ("dog", generate_dog),
            ("pyramid", generate_pyramid),
            ("ufo", generate_ufo),
            ("mirage", generate_mirage),
            ("grass_clump", generate_grass_clump),
            ("palm_tree", generate_palm_tree),
            ("well", generate_well),
            ("mountain_rock", generate_mountain_rock),
            ("desert_cabin", generate_desert_cabin),
            ("desert_house", generate_desert_house),
            ("desert_market", generate_desert_market),
            ("desert_castle", generate_desert_castle),
            ("desert_tower", generate_desert_tower),
            ("desert_caravan", generate_desert_caravan),
            ("npc_vendor", generate_npc_vendor),
            ("npc_soldier", generate_npc_soldier),
            ("npc_caravan", generate_npc_caravan),
            ("npc_hunter", generate_npc_hunter),
            ("npc_builder", generate_npc_builder),
            ("npc_citizen", generate_npc_citizen),
        ] {
            insert_model(&mut models, id, mesh_fn());
        }

        models.insert(
            "oasis_water".into(),
            ModelAsset {
                name: "oasis_water".into(),
                mesh: generate_water_plane(26.0, 48),
                texture_path: None,
                tiling: 4.0,
            },
        );
        models.insert(
            "stream_water".into(),
            ModelAsset {
                name: "stream_water".into(),
                mesh: generate_water_plane(6.0, 16),
                texture_path: None,
                tiling: 2.0,
            },
        );

        let (viewmodel, viewmodel_muzzle) = load_desert_viewmodel(&root);
        let pbr = load_pbr_textures(&root)?;

        log::info!(
            "Desert Shooter pack — {} modelos, viewmodel + terreno mega-deserto",
            models.len()
        );

        Ok(AssetLibrary {
            models,
            viewmodel,
            viewmodel_muzzle,
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

fn load_desert_viewmodel(root: &PathBuf) -> (ModelAsset, Vec3) {
    for path in viewmodel_gun_candidates(root) {
        if is_avocado_sample(&path) {
            continue;
        }
        if let Ok(gun) = load_gltf_viewmodel(&path, "viewmodel") {
            let arm = build_fps_arm();
            let mesh = merge_meshes(&[gun.mesh, arm]);
            let muzzle = offset_muzzle(gun.muzzle_local, Vec3::ZERO);
            log::info!("Desert viewmodel: glTF {} + braço", path.display());
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
    }
    log::info!("Desert viewmodel procedural");
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
