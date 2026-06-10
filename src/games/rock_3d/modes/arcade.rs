//! Modo Arcade — fases rápidas com estrelas.

use crate::games::rock_3d::targets::{TargetKind, TargetRegistry};
use crate::math::Vec3;

pub struct ArcadeMode {
    pub stage: u32,
    pub stars: u8,
    pub throws_used: u32,
    pub time_elapsed: f32,
}

impl Default for ArcadeMode {
    fn default() -> Self {
        Self {
            stage: 1,
            stars: 0,
            throws_used: 0,
            time_elapsed: 0.0,
        }
    }
}

impl ArcadeMode {
    pub fn setup_stage(stage: u32, targets: &mut TargetRegistry) {
        targets.targets.clear();
        let base_z = -15.0 - stage as f32 * 5.0;
        targets.spawn(TargetKind::Plate, Vec3::new(0.0, 1.5, base_z));
        targets.spawn(TargetKind::Can, Vec3::new(-3.0, 1.0, base_z - 3.0));
        targets.spawn(TargetKind::Can, Vec3::new(3.0, 1.0, base_z - 3.0));
        targets.spawn(TargetKind::Bottle, Vec3::new(-1.5, 2.0, base_z - 6.0));
        targets.spawn(TargetKind::Bell, Vec3::new(0.0, 2.5, base_z - 10.0));
    }

    pub fn evaluate_stars(&mut self, accuracy: f32, time: f32, throws: u32) -> u8 {
        let mut stars = 0u8;
        if accuracy >= 0.7 {
            stars += 1;
        }
        if time < 60.0 {
            stars += 1;
        }
        if throws <= 5 {
            stars += 1;
        }
        self.stars = stars;
        stars
    }
}
