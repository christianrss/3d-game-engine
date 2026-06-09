//! Matemática 3D usada pelo jogo e pela renderização.

pub mod ray;

/// Re-exporta tipos do `glam` com nomes familiares.
pub use glam::{Mat4, Quat, Vec2, Vec3, Vec4};

pub use ray::ray_sphere;
