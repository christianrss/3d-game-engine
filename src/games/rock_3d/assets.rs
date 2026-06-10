//! Assets exclusivos do Rock 3D.

use crate::assets::{load_pack, AssetPack, ROCK_3D_PACK, Rock3DAssetPack};

pub const ROCK_3D_ID: &str = ROCK_3D_PACK;

pub struct Rock3DAssets;

impl AssetPack for Rock3DAssets {
    fn id(&self) -> &'static str {
        ROCK_3D_ID
    }

    fn load(&self) -> Result<crate::assets::AssetLibrary, String> {
        Rock3DAssetPack.load()
    }
}

pub fn load_assets() -> Result<crate::assets::AssetLibrary, String> {
    load_pack(ROCK_3D_ID)
}
