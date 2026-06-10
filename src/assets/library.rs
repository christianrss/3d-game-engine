//! Container de assets carregados — populado por [`AssetPack`](crate::assets::pack::AssetPack).

use crate::assets::loader::ModelAsset;
use crate::assets::pack::{load_pack, AssetPack, DESERT_SHOOTER_PACK};
use crate::assets::packs::DesertAssetPack;
use crate::graphics::TextureData;
use crate::math::Vec3;
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
    pub rock_albedo: TextureData,
    pub rock_normal: TextureData,
    pub rock_rough: TextureData,
    pub root: PathBuf,
}

impl AssetLibrary {
    /// Carrega o pack do Desert Shooter (compatibilidade).
    pub fn load() -> Result<Self, String> {
        DesertAssetPack.load()
    }

    /// Carrega um pack por id (`desert-shooter`, `rock-3d`, `studio`).
    pub fn load_pack(id: &str) -> Result<Self, String> {
        load_pack(id)
    }

    pub fn get_model(&self, id: &str) -> Option<&ModelAsset> {
        self.models.get(id)
    }
}

/// Id padrão do pack legado.
pub const DEFAULT_PACK: &str = DESERT_SHOOTER_PACK;
