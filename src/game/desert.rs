//! Mapa mega-deserto procedural.

use crate::game::physics::CollisionWorld;
use crate::game::world::GameWorld;
use crate::game::world_gen::populate_mega_desert;

pub fn build_desert(world: &mut GameWorld) -> CollisionWorld {
    populate_mega_desert(world).collision
}
