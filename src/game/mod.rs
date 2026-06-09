//! # Lógica do Jogo
//!
//! Independente do backend gráfico — funciona com OpenGL, Vulkan ou DirectX.

mod desert;
mod input;
mod player;
mod scene;
mod score;
mod shooting;
pub use input::InputState;
pub use player::Player;
pub use scene::SceneBuilder;
pub use score::Score;
pub use shooting::try_shoot;
pub mod world;
pub use world::{Drawable, GameWorld, Target};
