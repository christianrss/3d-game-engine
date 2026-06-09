//! # Backend OpenGL 3.3 Core
//!
//! Implementação direta usando a API OpenGL via crate `gl`.
//! Fluxo: criar contexto → compilar shaders → VAO/VBO → draw calls.

pub mod context;
pub mod renderer;

pub use context::GlContext;
pub use renderer::{OpenGLError, OpenGLRenderer};
