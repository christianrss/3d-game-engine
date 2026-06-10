//! Ovelhas com IA — pastar, fugir, rebanho e morte.

use crate::assets::sample_desert_height;
use crate::game::building::{BlockGrid, FENCE_RADIUS, GRID};
use crate::game::world::{Drawable, GameWorld};
use crate::graphics::DrawMaterial;
use crate::math::{Quat, Vec3};

const SHEEP_SPEED: f32 = 2.2;
const FLEE_SPEED: f32 = 5.5;
const HERD_SPEED: f32 = 3.8;
const SHEEP_HIT_RADIUS: f32 = 0.55;
const WOOL_REGROW_SECS: f32 = 90.0;
const SHEAR_RANGE: f32 = 3.5;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SheepAi {
    Graze,
    Wander { timer: f32 },
    Flee { timer: f32 },
    Herded,
}

#[derive(Debug, Clone)]
pub struct Sheep {
    pub id: u32,
    pub pos: Vec3,
    pub vel: Vec3,
    pub yaw: f32,
    pub ai: SheepAi,
    pub health: f32,
    pub alive: bool,
    pub sheared: bool,
    pub wool_regrow: f32,
}

impl Sheep {
    pub fn spawn(id: u32, x: f32, z: f32) -> Self {
        let y = sample_desert_height(x, z);
        Self {
            id,
            pos: Vec3::new(x, y, z),
            vel: Vec3::ZERO,
            yaw: (x * 0.3 + z * 0.17).sin(),
            ai: SheepAi::Graze,
            health: 100.0,
            alive: true,
            sheared: false,
            wool_regrow: 0.0,
        }
    }

    pub fn wool_scale(&self) -> f32 {
        if self.sheared {
            0.82
        } else {
            1.0
        }
    }

    pub fn hit_center(&self) -> Vec3 {
        self.pos + Vec3::new(0.0, 0.55, 0.0)
    }

    pub fn check_hit(&self, bullet_pos: Vec3, bullet_radius: f32) -> bool {
        self.alive && bullet_pos.distance(self.hit_center()) <= SHEEP_HIT_RADIUS + bullet_radius
    }
}

#[derive(Default)]
pub struct SheepFlock {
    pub sheep: Vec<Sheep>,
    next_id: u32,
    pub herded_count: usize,
}

impl SheepFlock {
    pub fn spawn_herd(&mut self, positions: &[(f32, f32)]) {
        for &(x, z) in positions {
            let id = self.next_id;
            self.next_id += 1;
            self.sheep.push(Sheep::spawn(id, x, z));
        }
    }

    pub fn alive_count(&self) -> usize {
        self.sheep.iter().filter(|s| s.alive).count()
    }

    pub fn update(
        &mut self,
        dt: f32,
        player_pos: Vec3,
        _player_vel: Vec3,
        player_sprinting: bool,
        gunshot_pos: Option<Vec3>,
        blocks: &BlockGrid,
    ) {
        self.herded_count = 0;
        for sheep in &mut self.sheep {
            if !sheep.alive {
                continue;
            }
            if sheep.sheared {
                sheep.wool_regrow += dt;
                if sheep.wool_regrow >= WOOL_REGROW_SECS {
                    sheep.sheared = false;
                    sheep.wool_regrow = 0.0;
                }
            }
            if matches!(sheep.ai, SheepAi::Herded) {
                self.herded_count += 1;
            }

            if let Some(shot) = gunshot_pos {
                if sheep.pos.distance(shot) < 18.0 {
                    sheep.ai = SheepAi::Flee { timer: 3.0 };
                }
            }

            if player_sprinting && sheep.pos.distance(player_pos) < 7.0 {
                sheep.ai = SheepAi::Flee { timer: 2.0 };
            }

            let mut wish_vel = Vec3::ZERO;

            match sheep.ai {
                SheepAi::Graze => {
                    if (sheep.pos.x * 0.7 + sheep.id as f32).sin() > 0.995 {
                        sheep.ai = SheepAi::Wander { timer: 2.0 + (sheep.id % 3) as f32 };
                    }
                }
                SheepAi::Wander { ref mut timer } => {
                    *timer -= dt;
                    let dir = Vec3::new(sheep.yaw.cos(), 0.0, sheep.yaw.sin());
                    wish_vel = dir * SHEEP_SPEED * 0.4;
                    if *timer <= 0.0 {
                        sheep.yaw += ((sheep.id as f32 * 1.7).sin()) * 0.8;
                        sheep.ai = SheepAi::Graze;
                    }
                }
                SheepAi::Flee { ref mut timer } => {
                    *timer -= dt;
                    let away = (sheep.pos - player_pos).normalize_or_zero();
                    wish_vel = away * FLEE_SPEED;
                    if *timer <= 0.0 {
                        sheep.ai = SheepAi::Graze;
                    }
                }
                SheepAi::Herded => {
                    let target = player_pos + Vec3::new(
                        ((sheep.id as f32 * 2.1).sin()) * 2.5,
                        0.0,
                        ((sheep.id as f32 * 1.3).cos()) * 2.5,
                    );
                    let to = target - sheep.pos;
                    let dist = to.length();
                    if dist > 0.5 {
                        wish_vel = to.normalize() * HERD_SPEED.min(dist * 2.0);
                    }
                }
            }

            sheep.vel = wish_vel;
            sheep.pos += sheep.vel * dt;
            resolve_fence(&mut sheep.pos, blocks);
            keep_in_pen(&mut sheep.pos, blocks);

            if sheep.vel.length_squared() > 0.01 {
                sheep.yaw = sheep.vel.x.atan2(sheep.vel.z);
            }

            let ground = sample_desert_height(sheep.pos.x, sheep.pos.z);
            sheep.pos.y = ground;
        }
    }

    pub fn toggle_herd_near(&mut self, player_pos: Vec3, range: f32) -> bool {
        let mut nearest: Option<usize> = None;
        let mut best = range;
        for (i, s) in self.sheep.iter().enumerate() {
            if !s.alive {
                continue;
            }
            let d = s.pos.distance(player_pos);
            if d < best {
                best = d;
                nearest = Some(i);
            }
        }
        if let Some(i) = nearest {
            match self.sheep[i].ai {
                SheepAi::Herded => self.sheep[i].ai = SheepAi::Graze,
                _ => self.sheep[i].ai = SheepAi::Herded,
            }
            return true;
        }
        false
    }

    pub fn try_shear_near(&mut self, player_pos: Vec3) -> Option<(Vec3, u32)> {
        let mut nearest: Option<(usize, f32)> = None;
        for (i, s) in self.sheep.iter().enumerate() {
            if !s.alive || s.sheared {
                continue;
            }
            let d = s.pos.distance(player_pos);
            if d <= SHEAR_RANGE {
                match nearest {
                    None => nearest = Some((i, d)),
                    Some((_, bd)) if d < bd => nearest = Some((i, d)),
                    _ => {}
                }
            }
        }
        if let Some((i, _)) = nearest {
            self.sheep[i].sheared = true;
            self.sheep[i].wool_regrow = 0.0;
            let wool = 1 + (self.sheep[i].id % 3);
            let pos = self.sheep[i].hit_center();
            return Some((pos, wool));
        }
        None
    }

    pub fn release_all_herd(&mut self) {
        for s in &mut self.sheep {
            if s.alive && matches!(s.ai, SheepAi::Herded) {
                s.ai = SheepAi::Graze;
            }
        }
    }

    pub fn damage_at(&mut self, pos: Vec3, radius: f32) -> Option<(u32, Vec3)> {
        let mut best: Option<(usize, f32)> = None;
        for (i, s) in self.sheep.iter().enumerate() {
            if !s.alive {
                continue;
            }
            let d = pos.distance(s.hit_center());
            if d <= SHEEP_HIT_RADIUS + radius {
                match best {
                    None => best = Some((i, d)),
                    Some((_, bd)) if d < bd => best = Some((i, d)),
                    _ => {}
                }
            }
        }
        if let Some((i, _)) = best {
            self.sheep[i].health -= 100.0;
            if self.sheep[i].health <= 0.0 {
                self.sheep[i].alive = false;
                let hit = self.sheep[i].hit_center();
                let id = self.sheep[i].id;
                return Some((id, hit));
            }
        }
        None
    }
}

fn resolve_fence(pos: &mut Vec3, blocks: &BlockGrid) {
    if blocks.blocks_movement(*pos) {
        let gx = (pos.x / GRID).round() as i32;
        let gz = (pos.z / GRID).round() as i32;
        let cell = crate::game::building::BlockPos { x: gx, z: gz };
        let kind = blocks
            .kind_at(cell)
            .unwrap_or(crate::game::building::BlockKind::Fence);
        let center = crate::game::building::BlockGrid::world_position(cell, kind);
        let push = (*pos - center).normalize_or_zero() * FENCE_RADIUS;
        pos.x = center.x + push.x;
        pos.z = center.z + push.z;
    }
}

/// Mantem ovelhas dentro de cercado fechado (retangulo min/max).
fn keep_in_pen(pos: &mut Vec3, blocks: &BlockGrid) {
    let fence_count = blocks.fence_posts().count();
    if fence_count < 8 {
        return;
    }
    let mut min_x = i32::MAX;
    let mut max_x = i32::MIN;
    let mut min_z = i32::MAX;
    let mut max_z = i32::MIN;
    for p in blocks.fence_posts() {
        min_x = min_x.min(p.x);
        max_x = max_x.max(p.x);
        min_z = min_z.min(p.z);
        max_z = max_z.max(p.z);
    }
    let margin = 0.35;
    let min_wx = min_x as f32 * GRID + margin;
    let max_wx = max_x as f32 * GRID - margin;
    let min_wz = min_z as f32 * GRID + margin;
    let max_wz = max_z as f32 * GRID - margin;
    if max_x - min_x >= 2 && max_z - min_z >= 2 {
        pos.x = pos.x.clamp(min_wx, max_wx);
        pos.z = pos.z.clamp(min_wz, max_wz);
    }
}

pub fn sync_sheep_drawables(world: &mut GameWorld, flock: &SheepFlock) {
    world.drawables.retain(|d| d.model_id != "sheep");
    for s in &flock.sheep {
        if !s.alive {
            continue;
        }
        world.add_drawable(Drawable {
            model_id: "sheep".into(),
            position: s.pos,
            rotation: Quat::from_rotation_y(s.yaw),
            scale: Vec3::new(1.0, s.wool_scale(), 1.0),
            material: DrawMaterial::Standard {
                roughness: 0.88,
                metallic: 0.0,
            },
            target_id: None,
        });
    }
}
