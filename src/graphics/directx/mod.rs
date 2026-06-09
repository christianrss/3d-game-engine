//! # Backend DirectX 11
//!
//! Implementação direta com a API Direct3D 11 via crate `windows`.
//! Disponível apenas no Windows.

mod renderer;

pub use renderer::DirectX11Renderer;
