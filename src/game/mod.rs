//! # Lógica do Jogo
//!
//! Independente do backend gráfico — funciona com OpenGL, Vulkan ou DirectX.

mod desert;
mod input;
mod particles;
mod player;
mod projectile;
mod scene;
mod score;
mod shooting;
mod viewmodel;
pub use input::InputState;
pub use player::Player;
pub use scene::SceneBuilder;
pub use score::Score;
pub use particles::ParticleSystem;
pub use projectile::ProjectileSystem;
pub use shooting::try_shoot;
pub use viewmodel::ViewModelAnimator;
pub mod world;
pub use world::{Drawable, GameWorld, Target};
// Target export kept for API; fields simplified in world.rs
