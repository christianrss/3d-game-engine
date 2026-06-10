//! Fisica de colisao — terreno, blocos, props e entidades.

use crate::assets::sample_desert_height;
use crate::game::building::{BlockGrid, BlockKey, BlockKind, GRID, BLOCK_HEIGHT};
use crate::math::Vec3;

pub const WORLD_HALF: f32 = 1024.0;
pub const PLAYER_RADIUS: f32 = 0.42;
pub const PLAYER_EYE: f32 = 1.7;

#[derive(Debug, Clone)]
pub struct StaticCollider {
    pub pos: Vec3,
    pub radius: f32,
}

#[derive(Debug, Default)]
pub struct CollisionWorld {
    pub static_bodies: Vec<StaticCollider>,
}

impl CollisionWorld {
    pub fn add_sphere(&mut self, x: f32, z: f32, radius: f32) {
        let y = sample_desert_height(x, z);
        self.static_bodies.push(StaticCollider {
            pos: Vec3::new(x, y + radius * 0.5, z),
            radius,
        });
    }

    /// Corrige posicao apos movimento — nao reaplica velocidade.
    pub fn resolve_player(&self, pos: &mut Vec3, blocks: &BlockGrid) {
        let feet_y = pos.y - PLAYER_EYE;

        for _ in 0..2 {
            for body in self.nearby(pos.x, pos.z, 14.0) {
                push_sphere(pos, PLAYER_RADIUS, body.pos, body.radius);
            }
            push_blocks(pos, PLAYER_RADIUS, feet_y, blocks);
        }

        snap_to_ground(pos);
    }

    pub fn move_creature(
        &self,
        pos: &mut Vec3,
        wish_vel: Vec3,
        dt: f32,
        radius: f32,
        blocks: &BlockGrid,
    ) {
        let mut next = *pos + wish_vel * dt;
        next.x = next.x.clamp(-WORLD_HALF + radius, WORLD_HALF - radius);
        next.z = next.z.clamp(-WORLD_HALF + radius, WORLD_HALF - radius);

        for body in self.nearby(next.x, next.z, 10.0) {
            push_sphere(&mut next, radius, body.pos, body.radius);
        }
        let body_y = next.y;
        push_blocks(&mut next, radius, body_y, blocks);
        next.y = sample_desert_height(next.x, next.z);
        *pos = next;
    }

    pub fn separate_entities(positions: &mut [Vec3], radii: &[f32], alive: &[bool]) {
        let n = positions.len();
        for i in 0..n {
            if !alive[i] {
                continue;
            }
            for j in (i + 1)..n {
                if !alive[j] {
                    continue;
                }
                let delta = positions[i] - positions[j];
                let dist_sq = delta.x * delta.x + delta.z * delta.z;
                let min = radii[i] + radii[j];
                if dist_sq < min * min && dist_sq > 0.0001 {
                    let dist = dist_sq.sqrt();
                    let push = delta * ((min - dist) / dist) * 0.5;
                    positions[i] += push;
                    positions[j] -= push;
                }
            }
        }
    }

    fn nearby<'a>(&'a self, x: f32, z: f32, range: f32) -> impl Iterator<Item = &'a StaticCollider> {
        let r2 = range * range;
        self.static_bodies.iter().filter(move |b| {
            let dx = b.pos.x - x;
            let dz = b.pos.z - z;
            dx * dx + dz * dz <= (range + b.radius).powi(2).min(r2 + b.radius * b.radius * 4.0)
        })
    }
}

pub fn snap_to_ground(pos: &mut Vec3) {
    let ground = sample_desert_height(pos.x, pos.z);
    pos.y = ground + PLAYER_EYE;
}

fn push_sphere(pos: &mut Vec3, radius: f32, center: Vec3, center_r: f32) {
    let delta = Vec3::new(pos.x - center.x, 0.0, pos.z - center.z);
    let dist = delta.length();
    let min = radius + center_r;
    if dist < min && dist > 0.001 {
        let push = delta.normalize() * (min - dist);
        pos.x += push.x;
        pos.z += push.z;
    }
}

fn push_blocks(pos: &mut Vec3, radius: f32, body_y: f32, blocks: &BlockGrid) {
    let gx = (pos.x / GRID).round() as i32;
    let gz = (pos.z / GRID).round() as i32;
    for (&key, &block) in &blocks.cells {
        if (key.x - gx).abs() > 1 || (key.z - gz).abs() > 1 {
            continue;
        }
        let (center, _, scale) = BlockGrid::world_transform(key, block);
        let (half_w, half_h) = match block.kind {
            BlockKind::Fence => (0.5, 1.1),
            BlockKind::Wall | BlockKind::WoodWall => (scale.x * 0.5, scale.y),
            BlockKind::Dirt | BlockKind::Stone => (BLOCK_HEIGHT * 0.5, BLOCK_HEIGHT),
        };
        if body_y < center.y + half_h && body_y > center.y - half_h - 0.5 {
            push_aabb_xz(pos, radius, center, half_w);
        }
    }
}

fn push_aabb_xz(pos: &mut Vec3, radius: f32, center: Vec3, half: f32) {
    let dx = pos.x - center.x;
    let dz = pos.z - center.z;
    let ox = half + radius - dx.abs();
    let oz = half + radius - dz.abs();
    if ox > 0.0 && oz > 0.0 {
        if ox < oz {
            pos.x += if dx > 0.0 { ox } else { -ox };
        } else {
            pos.z += if dz > 0.0 { oz } else { -oz };
        }
    }
}
