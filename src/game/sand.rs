//! Simulação de areia — emissores CPU + física GPU (Transform Feedback).

use crate::assets::sample_desert_height;
use crate::math::Vec3;

#[derive(Default)]
pub struct SandSimulator {
    footstep_timer: f32,
    wind_timer: f32,
    pending: Vec<(Vec3, Vec3, usize)>,
}

impl SandSimulator {
    pub fn update(&mut self, dt: f32, player_pos: Vec3, player_vel: Vec3) {
        self.footstep_timer -= dt;
        self.wind_timer -= dt;

        let speed = player_vel.length();
        if speed > 0.5 && self.footstep_timer <= 0.0 {
            let interval = if speed > 7.0 { 0.12 } else { 0.22 };
            self.footstep_timer = interval;
            self.queue_footstep(player_pos, player_vel);
        }

        if self.wind_timer <= 0.0 {
            self.wind_timer = 0.06;
            self.queue_wind(player_pos);
        }
    }

    pub fn emit_impact(&mut self, pos: Vec3) {
        let ground = sample_desert_height(pos.x, pos.z);
        let base = Vec3::new(pos.x, ground + 0.08, pos.z);
        self.pending.push((base, Vec3::new(0.0, 1.2, 0.0), 22));
    }

    pub fn drain_emits(&mut self) -> Vec<(Vec3, Vec3, usize)> {
        std::mem::take(&mut self.pending)
    }

    fn queue_footstep(&mut self, pos: Vec3, vel: Vec3) {
        let ground = sample_desert_height(pos.x, pos.z);
        let base = Vec3::new(pos.x, ground + 0.05, pos.z);
        let dir = if vel.length_squared() > 0.01 {
            vel.normalize()
        } else {
            Vec3::NEG_Z
        };
        self.pending.push((base, dir * 0.8 + Vec3::new(0.0, 0.6, 0.0), 12));
    }

    fn queue_wind(&mut self, near: Vec3) {
        for i in 0..4 {
            let ox = near.x + (i as f32 * 2.1).sin() * 6.0;
            let oz = near.z + (i as f32 * 1.7).cos() * 6.0;
            let ground = sample_desert_height(ox, oz);
            self.pending.push((
                Vec3::new(ox, ground + 0.12, oz),
                Vec3::new(2.8, 0.2, -1.0),
                1,
            ));
        }
    }
}
