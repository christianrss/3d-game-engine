//! Mapa desértico realista — rochas HQ, árvores secas, alvos metálicos.

use crate::assets::sample_desert_height;
use crate::game::world::{Drawable, GameWorld};
use crate::graphics::DrawMaterial;
use crate::math::{Quat, Vec3};

const BOULDERS: [(&str, f32, f32, f32); 18] = [
    ("boulder_a", 10.0, -14.0, 1.4),
    ("boulder_b", -16.0, -20.0, 1.1),
    ("boulder_c", 24.0, -30.0, 0.9),
    ("boulder_d", -10.0, -36.0, 1.8),
    ("boulder_e", 38.0, -44.0, 0.7),
    ("boulder_f", -32.0, -48.0, 1.2),
    ("boulder_a", 6.0, -58.0, 1.0),
    ("boulder_b", -44.0, -26.0, 1.3),
    ("boulder_c", 20.0, -68.0, 0.85),
    ("boulder_d", -18.0, -78.0, 1.6),
    ("boulder_e", 52.0, -34.0, 0.75),
    ("boulder_f", -58.0, -62.0, 1.15),
    ("boulder_a", -6.0, -42.0, 1.25),
    ("boulder_b", 44.0, -72.0, 1.05),
    ("boulder_c", -50.0, -38.0, 0.95),
    ("boulder_d", 14.0, -82.0, 1.7),
    ("boulder_e", -28.0, -88.0, 0.8),
    ("boulder_f", 60.0, -52.0, 1.3),
];

const TREES: [(&str, f32, f32, f32); 6] = [
    ("dead_tree_a", -45.0, -22.0, 1.2),
    ("dead_tree_b", 48.0, -58.0, 1.0),
    ("dead_tree_a", -68.0, -72.0, 1.1),
    ("dead_tree_b", 65.0, -28.0, 0.9),
    ("dead_tree_a", -22.0, -92.0, 1.15),
    ("dead_tree_b", 30.0, -95.0, 1.05),
];

fn place(world: &mut GameWorld, model: &str, x: f32, z: f32, scale: f32, mat: DrawMaterial) {
    let y = sample_desert_height(x, z);
    let yaw = (x * 0.09 + z * 0.13).sin() * std::f32::consts::PI;
    world.add_drawable(Drawable {
        model_id: model.into(),
        position: Vec3::new(x, y, z),
        rotation: Quat::from_rotation_y(yaw),
        scale: Vec3::splat(scale),
        material: mat,
        target_id: None,
    });
}

pub fn build_desert(world: &mut GameWorld, _ground_size: f32) {
    world.add_drawable(Drawable {
        model_id: "terrain".into(),
        position: Vec3::ZERO,
        rotation: Quat::IDENTITY,
        scale: Vec3::ONE,
        material: DrawMaterial::Terrain { tiling: 60.0 },
        target_id: None,
    });

    for (model, x, z, scale) in BOULDERS {
        place(world, model, x, z, scale, DrawMaterial::rock());
    }
    for (model, x, z, scale) in TREES {
        place(world, model, x, z, scale, DrawMaterial::wood());
    }
}
