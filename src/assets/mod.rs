//! # Assets — Modelos e texturas open source
//!
//! Cada jogo registra um [`AssetPack`](pack::AssetPack) com modelos e texturas próprios.
//! Texturas: [Poly Haven](https://polyhaven.com) (CC0) · Modelos: scans glTF CC0.

mod common;
mod creatures;
mod gltf_loader;
mod gpu_cache;
mod library;
mod loader;
mod pack;
mod packs;
mod procedural;
mod props;
mod terrain;
mod viewmodel;
mod water;

pub use common::{asset_root, load_pbr_textures, PbrTextures};
pub use gpu_cache::GpuAssetCache;
pub use library::{AssetLibrary, DEFAULT_PACK};
pub use loader::{load_texture, load_texture_from_path, mesh_foot_offset, ModelAsset};
pub use pack::{
    load_pack, AssetPack, DESERT_SHOOTER_PACK, ROCK_3D_PACK, STUDIO_PACK,
};
pub use packs::{DesertAssetPack, Rock3DAssetPack, StudioAssetPack};
pub use terrain::{
    generate_desert_terrain, sample_desert_height, sample_quarry_height, TERRAIN_VISUAL_SCALE,
    QUARRY_HEIGHT_SCALE,
};
