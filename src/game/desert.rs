//! Gera o mapa do deserto na [`GameWorld`].

use crate::game::world::{Drawable, GameWorld};
use crate::graphics::Color;
use crate::math::Vec3;

const DUNES: [(f32, f32, f32); 8] = [
    (25.0, -0.5, -30.0),
    (-40.0, -0.5, -50.0),
    (60.0, -0.5, -20.0),
    (-20.0, -0.5, -70.0),
    (45.0, -0.5, -80.0),
    (-55.0, -0.5, -25.0),
    (10.0, -0.5, -90.0),
    (-35.0, -0.5, -60.0),
];

const CACTI: [(f32, f32); 12] = [
    (8.0, -12.0),
    (-15.0, -18.0),
    (22.0, -25.0),
    (-8.0, -35.0),
    (30.0, -40.0),
    (-25.0, -45.0),
    (5.0, -55.0),
    (-30.0, -15.0),
    (18.0, -65.0),
    (-12.0, -75.0),
    (35.0, -10.0),
    (-40.0, -35.0),
];

const ROCKS: [(f32, f32); 15] = [
    (3.0, -8.0),
    (-6.0, -14.0),
    (14.0, -22.0),
    (-10.0, -28.0),
    (20.0, -32.0),
    (-16.0, -38.0),
    (7.0, -42.0),
    (-22.0, -48.0),
    (26.0, -52.0),
    (-5.0, -62.0),
    (11.0, -68.0),
    (-28.0, -12.0),
    (32.0, -18.0),
    (-14.0, -72.0),
    (17.0, -78.0),
];

/// Popula o mundo com chão, dunas, cactos e rochas.
pub fn build_desert(world: &mut GameWorld, ground_size: f32) {
    // Chão
    world.add_drawable(Drawable {
        mesh_name: "plane".into(),
        position: Vec3::new(0.0, 0.0, -ground_size / 2.0),
        scale: Vec3::new(ground_size, 1.0, ground_size),
        color: Color::SAND,
    });

    // Dunas (esferas achatadas)
    for (i, (x, y, z)) in DUNES.iter().enumerate() {
        let s = 4.0 + i as f32 * 0.7;
        world.add_drawable(Drawable {
            mesh_name: "sphere".into(),
            position: Vec3::new(*x, *y, *z),
            scale: Vec3::new(s * 2.0, s * 0.4, s * 1.5),
            color: Color::DUNE,
        });
    }

    // Cactos
    for (x, z) in CACTI {
        world.add_drawable(Drawable {
            mesh_name: "cylinder".into(),
            position: Vec3::new(x, 0.75, z),
            scale: Vec3::new(0.3, 1.5, 0.3),
            color: Color::CACTUS,
        });
    }

    // Rochas
    for (i, (x, z)) in ROCKS.iter().enumerate() {
        let s = 0.3 + (i % 4) as f32 * 0.15;
        world.add_drawable(Drawable {
            mesh_name: "sphere".into(),
            position: Vec3::new(*x, s * 0.4, *z),
            scale: Vec3::new(s * 1.3, s * 0.6, s),
            color: Color::ROCK,
        });
    }
}
