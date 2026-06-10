//! Jogos construídos sobre a engine — cada um com assets e mundo próprios.

pub mod desert_shooter;
pub mod rock_3d;

pub use desert_shooter::{DesertShooterPlugin, DESERT_SHOOTER_ID};
pub use rock_3d::{Rock3DPlugin, ROCK_3D_ID};
