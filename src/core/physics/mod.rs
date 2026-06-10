//! Física reutilizável com unidades SI.

mod collision;
mod constants;
mod rigid_body;

pub use collision::{resolve_sphere_aabb, resolve_sphere_plane, CollisionEvent, CollisionKind};
pub use constants::*;
pub use rigid_body::{RigidBody, SphereCollider};
