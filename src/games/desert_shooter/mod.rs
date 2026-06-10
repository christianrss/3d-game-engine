//! # Desert Shooter — FPS no mega-deserto com construção e simulação.

pub mod world;

use crate::assets::{load_pack, AssetPack, DESERT_SHOOTER_PACK, DesertAssetPack};
use crate::core::{GameContext, GamePlugin};
use crate::engine::EngineApp;
use crate::game::SceneBuilder;

pub use world::build_desert_world;

pub const DESERT_SHOOTER_ID: &str = DESERT_SHOOTER_PACK;

/// Pack de assets exclusivo deste jogo.
pub struct DesertShooterAssets;

impl AssetPack for DesertShooterAssets {
    fn id(&self) -> &'static str {
        DESERT_SHOOTER_ID
    }

    fn load(&self) -> Result<crate::assets::AssetLibrary, String> {
        DesertAssetPack.load()
    }
}

/// Plugin de integração com a engine.
pub struct DesertShooterPlugin;

impl GamePlugin for DesertShooterPlugin {
    fn name(&self) -> &str {
        "Desert Shooter"
    }

    fn window_title(&self) -> &str {
        "Desert Shooter"
    }

    fn asset_pack_id(&self) -> &str {
        DESERT_SHOOTER_ID
    }

    fn run(&self, ctx: GameContext) {
        EngineApp::new()
            .with_window_title(ctx.window_title)
            .with_scene(SceneBuilder::new().with_desert_map())
            .run();
    }
}

/// Carrega assets do Desert Shooter.
pub fn load_assets() -> Result<crate::assets::AssetLibrary, String> {
    load_pack(DESERT_SHOOTER_ID)
}
