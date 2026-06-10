//! Partículas — fumaça do cano, poeira de impacto.

use crate::math::Vec3;

#[derive(Clone, Copy)]
pub struct Particle {
    pub pos: Vec3,
    pub vel: Vec3,
    pub life: f32,
    pub max_life: f32,
    pub size: f32,
    pub kind: u8, // 0 = smoke, 1 = dust, 2 = sand
}

#[derive(Default)]
pub struct ParticleSystem {
    pub particles: Vec<Particle>,
}

impl ParticleSystem {
    pub fn emit_muzzle_smoke(&mut self, origin: Vec3, forward: Vec3) {
        for i in 0..14 {
            let spread = Vec3::new(
                (i as f32 * 0.37).sin() * 0.04,
                (i as f32 * 0.21).cos() * 0.03 + 0.02,
                (i as f32 * 0.53).sin() * 0.04,
            );
            self.particles.push(Particle {
                pos: origin + spread,
                vel: forward * 1.2 + spread * 2.0 + Vec3::new(0.0, 0.4, 0.0),
                life: 1.0,
                max_life: 0.35 + (i as f32) * 0.02,
                size: 0.04 + (i as f32) * 0.003,
                kind: 0,
            });
        }
    }

    pub fn emit_hit_dust(&mut self, pos: Vec3) {
        for i in 0..8 {
            let a = i as f32 * 0.9;
            self.particles.push(Particle {
                pos,
                vel: Vec3::new(a.cos() * 0.8, 0.5, a.sin() * 0.8),
                life: 1.0,
                max_life: 0.5,
                size: 0.06,
                kind: 1,
            });
        }
    }

    pub fn update(&mut self, dt: f32) {
        for p in &mut self.particles {
            p.life -= dt / p.max_life;
            p.pos += p.vel * dt;
            p.vel *= 1.0 - dt * 1.5;
            if p.kind == 0 {
                p.vel.y += dt * 0.6;
                p.size += dt * 0.08;
            }
        }
        self.particles.retain(|p| p.life > 0.0);
    }
}
