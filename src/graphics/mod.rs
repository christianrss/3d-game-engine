//! # Camada Gráfica
//!
//! Implementa renderização 3D do zero com três backends que compartilham
//! a mesma interface [`GfxBackend`].

mod backend;
mod camera;
pub mod mesh_factory;
mod primitives;
pub mod renderer;
mod shaders;
mod types;

#[cfg(feature = "opengl")]
pub mod opengl;
#[cfg(feature = "vulkan")]
pub mod vulkan;
#[cfg(all(feature = "directx", target_os = "windows"))]
pub mod directx;

pub use backend::{BackendKind, GfxBackend};
pub use camera::Camera;
pub use primitives::*;
pub use renderer::{GfxRenderer, HudState, MeshCache};
pub use types::{Color, DrawMaterial, GpuMesh, GpuTexture, Mesh, ParticleDraw, TextureData, Vertex};
