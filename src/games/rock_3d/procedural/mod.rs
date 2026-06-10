//! Geração procedural de layouts e desafios.

use crate::games::rock_3d::targets::TargetKind;
use crate::math::Vec3;

/// PRNG simples determinístico (xorshift64).
#[derive(Debug, Clone)]
pub struct Rng {
    state: u64,
}

impl Rng {
    pub fn new(seed: u64) -> Self {
        Self {
            state: if seed == 0 { 0xDEAD_BEEF_CAFE_BABE } else { seed },
        }
    }

    pub fn next_u32(&mut self) -> u32 {
        self.state ^= self.state << 13;
        self.state ^= self.state >> 7;
        self.state ^= self.state << 17;
        (self.state & 0xFFFF_FFFF) as u32
    }

    pub fn next_f32(&mut self) -> f32 {
        (self.next_u32() as f32) / (u32::MAX as f32)
    }

    pub fn range_f32(&mut self, min: f32, max: f32) -> f32 {
        min + self.next_f32() * (max - min)
    }
}

pub struct DailySeed {
    seed: u64,
}

impl DailySeed {
    pub fn new(seed: u64) -> Self {
        Self { seed }
    }

    pub fn today_seed() -> u64 {
        // Seed baseada no dia (simplificado)
        20260610u64
    }

    pub fn generate_layout(&self) -> ProceduralLayout {
        let mut rng = Rng::new(self.seed);
        let target_count = 6 + (rng.next_u32() % 6) as usize;
        let mut targets = Vec::with_capacity(target_count);

        for i in 0..target_count {
            let kind = match rng.next_u32() % 5 {
                0 => TargetKind::Plate,
                1 => TargetKind::Can,
                2 => TargetKind::Bottle,
                3 => TargetKind::Bell,
                _ => TargetKind::Drone,
            };
            let x = rng.range_f32(-12.0, 12.0);
            let y = rng.range_f32(1.0, 3.5);
            let z = rng.range_f32(-40.0, -15.0);
            targets.push(ProceduralTarget {
                kind,
                position: Vec3::new(x, y, z),
            });
        }

        ProceduralLayout { targets }
    }
}

#[derive(Debug, Clone)]
pub struct ProceduralTarget {
    pub kind: TargetKind,
    pub position: Vec3,
}

#[derive(Debug, Clone)]
pub struct ProceduralLayout {
    pub targets: Vec<ProceduralTarget>,
}
