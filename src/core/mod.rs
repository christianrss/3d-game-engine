//! Camada core compartilhada entre jogos — ECS, física, plugins.

pub mod ecs;
pub mod game_plugin;
pub mod physics;

pub use ecs::{Component, EcsWorld, Entity};
pub use game_plugin::{GameContext, GamePlugin};
pub use physics::{RigidBody, SphereCollider};
