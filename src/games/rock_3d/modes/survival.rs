//! Modo Sobrevivência — ondas infinitas.

use crate::games::rock_3d::targets::{TargetKind, TargetRegistry};
use crate::math::Vec3;

pub struct SurvivalMode {
    pub wave: u32,
    pub targets_per_wave: u32,
    pub wave_timer: f32,
}

impl Default for SurvivalMode {
    fn default() -> Self {
        Self {
            wave: 1,
            targets_per_wave: 5,
            wave_timer: 0.0,
        }
    }
}

impl SurvivalMode {
    pub fn spawn_wave(&mut self, targets: &mut TargetRegistry) {
        targets.targets.clear();
        let count = self.targets_per_wave + self.wave;
        for i in 0..count {
            let angle = (i as f32 / count as f32) * std::f32::consts::TAU;
            let dist = 15.0 + self.wave as f32 * 3.0;
            let kind = if i % 4 == 0 {
                TargetKind::Drone
            } else if i % 3 == 0 {
                TargetKind::Bottle
            } else {
                TargetKind::Can
            };
            targets.spawn(
                kind,
                Vec3::new(angle.cos() * 8.0, 1.5, -dist - angle.sin().abs() * 5.0),
            );
        }
        self.wave += 1;
    }
}
