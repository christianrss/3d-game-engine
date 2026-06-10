//! Trait de pack de assets — cada jogo carrega só o que precisa.

use crate::assets::library::AssetLibrary;
use crate::assets::packs::{DesertAssetPack, Rock3DAssetPack, StudioAssetPack};

pub const DESERT_SHOOTER_PACK: &str = "desert-shooter";
pub const ROCK_3D_PACK: &str = "rock-3d";
pub const STUDIO_PACK: &str = "studio";

pub trait AssetPack: Send + Sync {
    fn id(&self) -> &'static str;
    fn load(&self) -> Result<AssetLibrary, String>;
}

pub fn load_pack(id: &str) -> Result<AssetLibrary, String> {
    match id {
        DESERT_SHOOTER_PACK => DesertAssetPack.load(),
        ROCK_3D_PACK => Rock3DAssetPack.load(),
        STUDIO_PACK => StudioAssetPack.load(),
        _ => Err(format!("Asset pack desconhecido: {id}")),
    }
}
