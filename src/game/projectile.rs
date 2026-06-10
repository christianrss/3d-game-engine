//! Projéteis balísticos — arrasto do ar, vento e gravidade.

use crate::assets::sample_desert_height;
use crate::game::score::Score;
use crate::game::ecosystem::{CreatureKind, Ecosystem};
use crate::game::world::GameWorld;
use crate::math::Vec3;

pub const BULLET_SPEED: f32 = 180.0;
pub const GRAVITY: f32 = 9.81;
pub const AIR_DRAG: f32 = 0.00012;
pub const WIND: Vec3 = Vec3::new(2.2, 0.0, -0.9);
pub const BULLET_RADIUS: f32 = 0.14;
pub const MAX_LIFETIME: f32 = 5.0;

#[derive(Debug, Clone)]
pub struct Projectile {
    pub pos: Vec3,
    pub vel: Vec3,
    pub age: f32,
    pub damage: f32,
    pub radius: f32,
    pub trail: Vec<Vec3>,
}

#[derive(Debug, Clone, Copy)]
pub struct ProjectileParams {
    pub speed: f32,
    pub damage: f32,
    pub radius: f32,
}

#[derive(Default)]
pub struct ProjectileSystem {
    pub active: Vec<Projectile>,
    pub trajectory: Vec<Vec3>,
}

impl ProjectileSystem {
    pub fn spawn(&mut self, origin: Vec3, direction: Vec3, params: ProjectileParams) {
        let vel = direction.normalize() * params.speed;
        self.active.push(Projectile {
            pos: origin,
            vel,
            age: 0.0,
            damage: params.damage,
            radius: params.radius,
            trail: vec![origin],
        });
    }

    pub fn update_trajectory_preview(&mut self, origin: Vec3, direction: Vec3, speed: f32) {
        self.trajectory = compute_ballistic_arc(origin, direction, speed, GRAVITY, 56, 0.035);
    }

    pub fn update(
        &mut self,
        dt: f32,
        world: &mut GameWorld,
        score: &mut Score,
        eco: &mut Ecosystem,
    ) -> (Vec<Vec3>, Vec<(u32, CreatureKind)>) {
        let mut hit_positions = Vec::new();
        let mut kills = Vec::new();

        for bullet in &mut self.active {
            bullet.age += dt;

            let speed = bullet.vel.length();
            if speed > 0.01 {
                let drag = bullet.vel.normalize() * (-AIR_DRAG * speed * speed);
                bullet.vel += (Vec3::NEG_Y * GRAVITY + drag + WIND * 0.15) * dt;
            } else {
                bullet.vel.y -= GRAVITY * dt;
            }
            bullet.pos += bullet.vel * dt;

            if bullet.trail.last().map(|p| p.distance(bullet.pos)).unwrap_or(1.0) > 0.12 {
                bullet.trail.push(bullet.pos);
                if bullet.trail.len() > 28 {
                    bullet.trail.remove(0);
                }
            }

            let ground = sample_desert_height(bullet.pos.x, bullet.pos.z) + 0.08;
            if bullet.pos.y < ground {
                bullet.age = MAX_LIFETIME;
                score.register_miss();
                continue;
            }

            if let Some((idx, points, hit_pos)) = find_target_hit(world, bullet.pos) {
                let target_id = world.targets[idx].id;
                world.targets[idx].alive = false;
                world.remove_target_drawables(target_id);
                score.register_hit(points);
                hit_positions.push(hit_pos);
                bullet.age = MAX_LIFETIME;
                continue;
            }

            if let Some((id, hit_pos, kind)) = eco.damage_at(bullet.pos, bullet.radius, bullet.damage)
            {
                hit_positions.push(hit_pos);
                kills.push((id, kind));
                bullet.age = MAX_LIFETIME;
            }
        }

        self.active.retain(|p| p.age < MAX_LIFETIME);
        (hit_positions, kills)
    }
}

fn find_target_hit(world: &GameWorld, pos: Vec3) -> Option<(usize, u32, Vec3)> {
    let mut best: Option<(usize, f32, u32)> = None;
    for (i, target) in world.targets.iter().enumerate() {
        if !target.alive {
            continue;
        }
        let dist = pos.distance(target.position);
            if dist <= target.radius + 0.14 {
            match best {
                None => best = Some((i, dist, target.points)),
                Some((_, d, _)) if dist < d => best = Some((i, dist, target.points)),
                _ => {}
            }
        }
    }
    best.map(|(i, _, points)| (i, points, world.targets[i].position))
}

pub fn compute_ballistic_arc(
    origin: Vec3,
    direction: Vec3,
    speed: f32,
    gravity: f32,
    steps: usize,
    dt: f32,
) -> Vec<Vec3> {
    let mut points = Vec::with_capacity(steps);
    let mut pos = origin;
    let mut vel = direction.normalize() * speed;
    for _ in 0..steps {
        points.push(pos);
        let sp = vel.length();
        if sp > 0.01 {
            let drag = vel.normalize() * (-AIR_DRAG * sp * sp);
            vel += (Vec3::NEG_Y * gravity + drag + WIND * 0.15) * dt;
        } else {
            vel.y -= gravity * dt;
        }
        pos += vel * dt;
        let ground = sample_desert_height(pos.x, pos.z) + 0.05;
        if pos.y < ground {
            points.push(Vec3::new(pos.x, ground, pos.z));
            break;
        }
    }
    points
}
