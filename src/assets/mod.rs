//! # Assets — Modelos e texturas open source
//!
//! Modelos: [Kenney Nature Kit](https://kenney.nl/assets/nature-kit) (CC0)
//! Texturas: [Poly Haven](https://polyhaven.com) (CC0)

mod gltf_loader;
mod gpu_cache;
mod library;
mod loader;
mod procedural;
mod terrain;
mod viewmodel;

pub use gpu_cache::GpuAssetCache;
pub use library::AssetLibrary;
pub use loader::{load_texture, load_texture_from_path, ModelAsset};
pub use terrain::{generate_desert_terrain, sample_desert_height};
