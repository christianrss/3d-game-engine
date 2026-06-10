//! Modo Precisão — alvos pequenos.

use crate::games::rock_3d::targets::{TargetKind, TargetRegistry};
use crate::math::Vec3;

pub struct PrecisionMode {
    pub hits_required: u32,
    pub hits: u32,
}

impl Default for PrecisionMode {
    fn default() -> Self {
        Self {
            hits_required: 10,
            hits: 0,
        }
    }
}

impl PrecisionMode {
    pub fn setup(targets: &mut TargetRegistry) {
        targets.targets.clear();
        for i in 0..10 {
            let angle = i as f32 * 0.6;
            let dist = 20.0 + i as f32 * 2.0;
            targets.spawn(
                TargetKind::Can,
                Vec3::new(angle.sin() * 3.0, 1.2 + (i % 3) as f32 * 0.5, -dist),
            );
        }
    }
}
