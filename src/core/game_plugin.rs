//! Trait para registrar jogos na engine.

/// Contexto compartilhado passado ao plugin de jogo.
pub struct GameContext {
    pub window_title: String,
    pub width: u32,
    pub height: u32,
}

impl Default for GameContext {
    fn default() -> Self {
        Self {
            window_title: "Game".into(),
            width: 1280,
            height: 720,
        }
    }
}

/// Interface que cada jogo implementa para integrar com a engine.
pub trait GamePlugin: Send {
    fn name(&self) -> &str;
    fn window_title(&self) -> &str;
    /// Id do [`AssetPack`](crate::assets::AssetPack) usado por este jogo.
    fn asset_pack_id(&self) -> &str;
    fn run(&self, ctx: GameContext);
}
