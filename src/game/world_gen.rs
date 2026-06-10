//! Geracao procedural do mega-deserto — oasis, riachos, vales, montanhas.

use crate::assets::sample_desert_height;
use crate::game::physics::{CollisionWorld, WORLD_HALF};
use crate::game::world::{Drawable, GameWorld};
use crate::graphics::DrawMaterial;
use crate::math::{Quat, Vec3};

pub const OASIS_POSITIONS: &[(f32, f32)] = &[
    (-38.0, -52.0),
    (120.0, 80.0),
    (-200.0, 150.0),
    (300.0, -180.0),
    (-30.0, -35.0),
];

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

    fn chance(&mut self, p: f32) -> bool {
        self.range(0.0, 1.0) < p
    }
}

pub struct WorldGenResult {
    pub collision: CollisionWorld,
}

/// Nivel da agua — minimo local menos profundidade do basin.
pub fn oasis_water_level(ox: f32, oz: f32) -> f32 {
    let mut min_h = f32::MAX;
    for dx in [-8.0_f32, -4.0, 0.0, 4.0, 8.0] {
        for dz in [-8.0, -4.0, 0.0, 4.0, 8.0] {
            min_h = min_h.min(sample_desert_height(ox + dx, oz + dz));
        }
    }
    min_h - 1.2
}

pub fn populate_mega_desert(world: &mut GameWorld) -> WorldGenResult {
    let mut rng = Rng::new(0xDE5E_0042);
    let mut collision = CollisionWorld::default();

    world.add_drawable(Drawable {
        model_id: "terrain".into(),
        position: Vec3::ZERO,
        rotation: Quat::IDENTITY,
        scale: Vec3::ONE,
        material: DrawMaterial::Terrain { tiling: 140.0 },
        target_id: None,
    });

    for &(ox, oz) in OASIS_POSITIONS {
        spawn_oasis(world, ox, oz, &mut rng, &mut collision);
    }

    spawn_streams(world, &mut rng);
    spawn_valleys_and_mountains(world, &mut rng, &mut collision);
    spawn_narrow_passages(world, &mut rng, &mut collision);

    let pyramid_spots = [
        (280.0, -320.0),
        (-450.0, 380.0),
        (520.0, 420.0),
        (-380.0, -520.0),
        (0.0, 600.0),
    ];
    for &(px, pz) in &pyramid_spots {
        let y = sample_desert_height(px, pz);
        world.add_drawable(Drawable {
            model_id: "pyramid".into(),
            position: Vec3::new(px, y, pz),
            rotation: Quat::from_rotation_y(rng.range(0.0, 0.5)),
            scale: Vec3::splat(rng.range(0.8, 1.4)),
            material: DrawMaterial::rock(),
            target_id: None,
        });
        collision.add_sphere(px, pz, 8.0);
    }

    let rock_models = [
        "rock_scan_a", "rock_scan_b", "rock_scan_c", "boulder_a", "boulder_b", "boulder_c",
    ];
    for _ in 0..140 {
        let x = rng.range(-WORLD_HALF * 0.9, WORLD_HALF * 0.9);
        let z = rng.range(-WORLD_HALF * 0.9, WORLD_HALF * 0.9);
        if x.abs() < 20.0 && z.abs() < 20.0 {
            continue;
        }
        let model = rock_models[(rng.next_u32() as usize) % rock_models.len()];
        let scale = rng.range(0.6, 1.8);
        place_prop(world, model, x, z, scale, DrawMaterial::rock(), &mut rng);
        collision.add_sphere(x, z, 1.2 * scale);
    }

    for _ in 0..28 {
        let x = rng.range(-WORLD_HALF * 0.85, WORLD_HALF * 0.85);
        let z = rng.range(-WORLD_HALF * 0.85, WORLD_HALF * 0.85);
        let model = if rng.chance(0.5) {
            "dead_tree_a"
        } else {
            "dead_tree_b"
        };
        place_prop(world, model, x, z, rng.range(0.8, 1.3), DrawMaterial::wood(), &mut rng);
        collision.add_sphere(x, z, 0.8);
    }

    WorldGenResult { collision }
}

fn spawn_oasis(
    world: &mut GameWorld,
    ox: f32,
    oz: f32,
    rng: &mut Rng,
    collision: &mut CollisionWorld,
) {
    let water_y = oasis_water_level(ox, oz);
    world.add_drawable(Drawable {
        model_id: "oasis_water".into(),
        position: Vec3::new(ox, water_y, oz),
        rotation: Quat::IDENTITY,
        scale: Vec3::ONE,
        material: DrawMaterial::Water,
        target_id: None,
    });

    for i in 0..18 {
        let a = i as f32 * 0.72 + rng.range(0.0, 0.4);
        let r = rng.range(6.0, 16.0);
        let gx = ox + a.cos() * r;
        let gz = oz + a.sin() * r;
        place_prop(world, "grass_clump", gx, gz, rng.range(0.9, 1.4), DrawMaterial::foliage(), rng);
    }

    for i in 0..5 {
        let a = i as f32 * 1.26;
        let r = rng.range(4.0, 11.0);
        let px = ox + a.cos() * r;
        let pz = oz + a.sin() * r;
        place_prop(
            world,
            "palm_tree",
            px,
            pz,
            rng.range(0.85, 1.25),
            DrawMaterial::foliage(),
            rng,
        );
        collision.add_sphere(px, pz, 0.5);
    }

    let well_x = ox + 10.0;
    let well_z = oz - 6.0;
    place_prop(world, "well", well_x, well_z, 1.0, DrawMaterial::rock(), rng);
    collision.add_sphere(well_x, well_z, 1.0);
}

fn spawn_streams(world: &mut GameWorld, rng: &mut Rng) {
    let pairs = [
        (OASIS_POSITIONS[0], OASIS_POSITIONS[4]),
        (OASIS_POSITIONS[1], OASIS_POSITIONS[3]),
        (OASIS_POSITIONS[2], OASIS_POSITIONS[0]),
    ];
    for (a, b) in pairs {
        let steps = 12;
        for s in 0..steps {
            let t = s as f32 / steps as f32;
            let x = a.0 + (b.0 - a.0) * t + rng.range(-3.0, 3.0);
            let z = a.1 + (b.1 - a.1) * t + rng.range(-3.0, 3.0);
            let wy = oasis_water_level(x, z) + 0.05;
            world.add_drawable(Drawable {
                model_id: "stream_water".into(),
                position: Vec3::new(x, wy, z),
                rotation: Quat::from_rotation_y(rng.range(0.0, std::f32::consts::TAU)),
                scale: Vec3::new(rng.range(0.8, 1.2), 1.0, rng.range(2.5, 4.0)),
                material: DrawMaterial::Water,
                target_id: None,
            });
        }
    }
}

fn spawn_valleys_and_mountains(
    world: &mut GameWorld,
    rng: &mut Rng,
    collision: &mut CollisionWorld,
) {
    let mountain_spots = [
        (180.0, -220.0),
        (-320.0, -280.0),
        (420.0, 310.0),
        (-480.0, 120.0),
        (60.0, 380.0),
    ];
    for &(mx, mz) in &mountain_spots {
        let h = sample_desert_height(mx, mz);
        if h < 8.0 {
            continue;
        }
        let scale = rng.range(1.0, 1.8);
        place_prop(
            world,
            "mountain_rock",
            mx,
            mz,
            scale,
            DrawMaterial::rock(),
            rng,
        );
        collision.add_sphere(mx, mz, 4.5 * scale);
        if rng.chance(0.4) {
            let fx = mx + rng.range(-8.0, 8.0);
            let fz = mz + rng.range(-8.0, 8.0);
            let fy = sample_desert_height(fx, fz) + rng.range(6.0, 14.0);
            world.add_drawable(Drawable {
                model_id: "stream_water".into(),
                position: Vec3::new(fx, fy, fz),
                rotation: Quat::IDENTITY,
                scale: Vec3::new(1.5, 3.0, 1.5),
                material: DrawMaterial::Water,
                target_id: None,
            });
        }
    }
}

fn spawn_narrow_passages(
    world: &mut GameWorld,
    rng: &mut Rng,
    collision: &mut CollisionWorld,
) {
    let passages = [(-120.0, 60.0, 0.0_f32), (200.0, -90.0, 1.2), (-280.0, -140.0, 2.4)];
    for (cx, cz, ang) in passages {
        for i in 0..8 {
            let along = (i as f32 - 3.5) * 5.0;
            for side in [-1.0_f32, 1.0] {
                let x = cx + along * ang.cos() - side * 4.5 * ang.sin();
                let z = cz + along * ang.sin() + side * 4.5 * ang.cos();
                let model = if rng.chance(0.5) {
                    "boulder_d"
                } else {
                    "boulder_e"
                };
                place_prop(world, model, x, z, rng.range(1.2, 2.0), DrawMaterial::rock(), rng);
                collision.add_sphere(x, z, 2.0);
            }
        }
    }
}

fn place_prop(
    world: &mut GameWorld,
    model: &str,
    x: f32,
    z: f32,
    scale: f32,
    mat: DrawMaterial,
    rng: &mut Rng,
) {
    let y = sample_desert_height(x, z);
    world.add_drawable(Drawable {
        model_id: model.into(),
        position: Vec3::new(x, y, z),
        rotation: Quat::from_rotation_y(rng.range(0.0, std::f32::consts::TAU)),
        scale: Vec3::splat(scale),
        material: mat,
        target_id: None,
    });
}

pub fn random_spawn_points(count: usize, min_dist: f32) -> Vec<(f32, f32)> {
    let mut rng = Rng::new(0xFA09_A099);
    let mut out = Vec::with_capacity(count);
    for _ in 0..count * 4 {
        if out.len() >= count {
            break;
        }
        let x = rng.range(-WORLD_HALF * 0.8, WORLD_HALF * 0.8);
        let z = rng.range(-WORLD_HALF * 0.8, WORLD_HALF * 0.8);
        if out.iter().all(|&(ox, oz)| {
            let dx = x - ox;
            let dz = z - oz;
            (dx * dx + dz * dz) > min_dist * min_dist
        }) {
            out.push((x, z));
        }
    }
    out
}
