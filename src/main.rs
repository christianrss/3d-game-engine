//! Desert Shooter — demonstração da engine com renderização nativa.
//!
//! ```bash
//! cargo run                          # OpenGL (padrão)
//! cargo run --features vulkan --no-default-features   # Vulkan
//! cargo run --features directx --no-default-features  # DirectX 11 (Windows)
//! cargo run --features all-backends                   # Todos os backends
//! ```

use desert_shooter_engine::prelude::*;

fn main() {
    let scene = SceneBuilder::new()
        .with_desert_map()
        .with_player_spawn(Vec3::new(0.0, 1.7, 8.0))
        .add_target(Vec3::new(5.0, 0.0, -10.0))
        .add_target(Vec3::new(-8.0, 0.0, -15.0))
        .add_target(Vec3::new(12.0, 0.0, -20.0))
        .add_target(Vec3::new(-15.0, 0.0, -25.0))
        .add_target(Vec3::new(0.0, 0.0, -30.0))
        .add_target_at(20.0, 0.0, -45.0, 250)
        .add_target_at(-25.0, 0.0, -50.0, 250);

    EngineApp::new()
        .with_window_title("Desert Shooter — Renderização Nativa")
        .with_scene(scene)
        .run();
}
