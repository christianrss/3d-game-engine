//! ECS leve — sem dependências externas.

mod component;
mod entity;
mod world;

pub use component::Component;
pub use entity::Entity;
pub use world::EcsWorld;
