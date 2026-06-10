//! Mundo da pedreira — identidade visual do Rock 3D (sem oásis/fauna do deserto).

use crate::assets::sample_quarry_height;
use crate::game::{CollisionWorld, StaticCollider};
use crate::games::rock_3d::ground::snap_to_quarry_ground_heuristic;
use crate::game::world::{Drawable, GameWorld};
use crate::graphics::DrawMaterial;
use crate::games::rock_3d::maps::MapConfig;
use crate::math::{Quat, Vec3};

struct Rng {
    state: u32,
}

impl Rng {
    fn new(seed: u32) -> Self {
        Self { state: seed }
    }

    fn next_u32(&mut self) -> u32 {
        self.state = self.state.wrapping_mul(1664525).wrapping_add(1013904223);
        self.state
    }

    fn range(&mut self, min: f32, max: f32) -> f32 {
        min + (self.next_u32() as f32 / u32::MAX as f32) * (max - min)
    }
}

/// Constrói o ambiente de arremesso conforme o mapa ativo.
pub fn build_map_world(world: &mut GameWorld, map: &MapConfig) -> CollisionWorld {
    match map.kind {
        crate::games::rock_3d::maps::MapKind::Quarry => build_quarry(world, map),
        _ => build_quarry(world, map),
    }
}

fn build_quarry(world: &mut GameWorld, map: &MapConfig) -> CollisionWorld {
    let mut rng = Rng::new(0xB0C3_3001);
    let mut collision = CollisionWorld::default();
    let half = map.world_half;

    world.add_drawable(Drawable {
        model_id: "terrain".into(),
        position: Vec3::ZERO,
        rotation: Quat::IDENTITY,
        scale: Vec3::ONE,
        material: DrawMaterial::Terrain { tiling: 90.0 },
        target_id: None,
    });

    let rock_models = [
        "rock_scan_a",
        "rock_scan_b",
        "rock_scan_c",
        "boulder_a",
        "boulder_b",
        "boulder_d",
    ];

    for _ in 0..72 {
        let x = rng.range(-half * 0.75, half * 0.75);
        let z = rng.range(-half * 0.75, half * 0.75);
        if x.abs() < 12.0 && z > -8.0 {
            continue;
        }
        let model = rock_models[(rng.next_u32() as usize) % rock_models.len()];
        let scale = rng.range(0.5, 1.6);
        place_rock(world, model, x, z, scale, &mut rng);
        add_quarry_sphere(&mut collision, x, z, 1.0 * scale);
    }

    for ring in 0..3 {
        let radius = half * (0.55 + ring as f32 * 0.12);
        let count = 10 + ring * 4;
        for i in 0..count {
            let a = (i as f32 / count as f32) * std::f32::consts::TAU;
            let x = a.cos() * radius;
            let z = a.sin() * radius;
            let model = if ring == 0 { "rock_scan_b" } else { "boulder_d" };
            let scale = rng.range(1.4, 2.4);
            place_rock(world, model, x, z, scale, &mut rng);
            add_quarry_sphere(&mut collision, x, z, 2.2 * scale);
        }
    }

    for i in 0..6 {
        let x = rng.range(-40.0, 40.0);
        let z = rng.range(-half * 0.5, -20.0);
        place_rock(world, "target", x, z, 1.2, &mut rng);
    }

    collision
}

fn add_quarry_sphere(collision: &mut CollisionWorld, x: f32, z: f32, radius: f32) {
    let y = sample_quarry_height(x, z);
    collision.static_bodies.push(StaticCollider {
        pos: Vec3::new(x, y + radius * 0.5, z),
        radius,
    });
}

fn place_rock(
    world: &mut GameWorld,
    model: &str,
    x: f32,
    z: f32,
    scale: f32,
    rng: &mut Rng,
) {
    let pos = snap_to_quarry_ground_heuristic(x, z, model, scale);
    world.add_drawable(Drawable {
        model_id: model.into(),
        position: pos,
        rotation: Quat::from_rotation_y(rng.range(0.0, std::f32::consts::TAU)),
        scale: Vec3::splat(scale),
        material: if model == "target" {
            DrawMaterial::metal()
        } else {
            DrawMaterial::rock()
        },
        target_id: None,
    });
}
