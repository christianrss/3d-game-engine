//! Modo Distância — alvos extremamente distantes.

use crate::games::rock_3d::targets::{TargetKind, TargetRegistry};
use crate::math::Vec3;

pub struct DistanceMode {
    pub current_distance: f32,
    pub best_distance: f32,
}

impl Default for DistanceMode {
    fn default() -> Self {
        Self {
            current_distance: 50.0,
            best_distance: 0.0,
        }
    }
}

impl DistanceMode {
    pub fn setup(targets: &mut TargetRegistry, distance: f32) {
        targets.targets.clear();
        targets.spawn(TargetKind::Plate, Vec3::new(0.0, 2.0, -distance));
        targets.spawn(TargetKind::Bell, Vec3::new(0.0, 3.0, -(distance + 30.0)));
    }
}
