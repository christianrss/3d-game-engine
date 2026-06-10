//! # Rock 3D — Jogo competitivo de arremesso de pedras.
//!
//! Mecânicas profundas de física, progressão e múltiplos modos de jogo.

pub mod ai;
pub mod app;
pub mod assets;
pub mod audio;
pub mod camera;
pub mod ground;
pub mod maps;
pub mod modes;
pub mod physics;
pub mod procedural;
pub mod progression;
pub mod replay;
pub mod save;
pub mod scoring;
pub mod state;
pub mod stones;
pub mod targets;
pub mod throw;
pub mod ui;
pub mod viewmodel;
pub mod weather;
pub mod world;

pub use app::Rock3DApp;
pub use assets::{load_assets as load_rock_assets, Rock3DAssets, ROCK_3D_ID};
pub use state::Rock3DState;

use crate::core::{GameContext, GamePlugin};

/// Plugin para integração com a engine.
pub struct Rock3DPlugin;

impl GamePlugin for Rock3DPlugin {
    fn name(&self) -> &str {
        "Rock 3D"
    }

    fn window_title(&self) -> &str {
        "Rock 3D — Competitive Stone Throwing"
    }

    fn asset_pack_id(&self) -> &str {
        ROCK_3D_ID
    }

    fn run(&self, ctx: GameContext) {
        Rock3DApp::new()
            .with_title(ctx.window_title)
            .with_size(ctx.width, ctx.height)
            .run();
    }
}
