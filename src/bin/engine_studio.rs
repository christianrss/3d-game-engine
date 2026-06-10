//! Engine Studio — editor de cenas estilo Unity.
//!
//! ```bash
//! cargo run --bin engine-studio
//! ```
//!
//! Atalhos: F5 Play | F6 Stop | S Salvar | N Novo cubo | ↑↓ Selecionar | Botão direito orbitar

use desert_shooter_engine::prelude::*;

fn main() {
    EngineStudio::new()
        .with_scene_path("scenes/default.scene.json")
        .run();
}
