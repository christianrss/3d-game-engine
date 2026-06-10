//! # Desert Shooter Engine — Renderização Nativa
//!
//! Game engine 3D **sem frameworks** (sem Bevy, Unity, etc.).
//! A renderização é implementada do zero com três backends:
//!
//! | Backend   | API              | Plataforma        |
//! |-----------|------------------|-------------------|
//! | OpenGL    | OpenGL 3.3 Core  | Win / Linux / Mac |
//! | Vulkan    | Vulkan 1.2       | Win / Linux       |
//! | DirectX   | Direct3D 11      | Windows           |
//!
//! ## Arquitetura
//!
//! ```text
//! ┌──────────────────────────────────────────────────────────┐
//! │  EngineApp          loop principal (winit)               │
//! ├──────────────────────────────────────────────────────────┤
//! │  GfxBackend trait   interface comum aos 3 backends     │
//! │    ├─ OpenGLRenderer                                     │
//! │    ├─ VulkanRenderer                                     │
//! │    └─ DirectX11Renderer                                  │
//! ├──────────────────────────────────────────────────────────┤
//! │  GameWorld          jogador, alvos, mapa, pontuação      │
//! │  SceneBuilder       constrói a cena declarativamente    │
//! └──────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Escolher o backend
//!
//! ```bash
//! # OpenGL (padrão)
//! cargo run
//!
//! # Vulkan
//! cargo run --features vulkan --no-default-features
//! set GFX_BACKEND=vulkan   # Windows
//!
//! # DirectX 11 (só Windows)
//! cargo run --features directx --no-default-features
//! set GFX_BACKEND=directx
//! ```

pub mod assets;
pub mod audio;
pub mod core;
pub mod editor;
pub mod engine;
pub mod game;
pub mod games;
pub mod graphics;
pub mod math;
pub mod scripting;

pub mod prelude {
    pub use crate::core::{EcsWorld, Entity, GamePlugin};
    pub use crate::editor::EngineStudio;
    pub use crate::engine::EngineApp;
    pub use crate::scripting::LuaRuntime;
    pub use crate::game::SceneBuilder;
    pub use crate::games::{DesertShooterPlugin, Rock3DPlugin};
    pub use crate::graphics::{BackendKind, Camera, Color, GfxRenderer, Vertex};
    pub use crate::math::{ray_sphere, Vec3};
}
