//! Sistema de pontuação, combos e estrelas.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ScoreSystem {
    pub total: u32,
    pub session: u32,
    pub combo: u32,
    pub max_combo: u32,
    pub hits: u32,
    pub misses: u32,
    pub ricochet_bonus: u32,
    pub time_bonus: u32,
}

impl ScoreSystem {
    pub fn register_hit(
        &mut self,
        base_points: u32,
        distance: f32,
        impact_speed: f32,
        ricochets: u32,
    ) -> u32 {
        self.hits += 1;
        self.combo += 1;
        self.max_combo = self.max_combo.max(self.combo);

        let dist_mult = 1.0 + distance / 50.0;
        let speed_mult = 1.0 + impact_speed / 20.0;
        let combo_mult = 1.0 + (self.combo.saturating_sub(1) as f32 * 0.15).min(0.75);
        let bounce_bonus = ricochets * 50;

        let points = ((base_points as f32 * dist_mult * speed_mult * combo_mult) as u32) + bounce_bonus;
        self.ricochet_bonus += bounce_bonus;
        self.session += points;
        self.total += points;
        points
    }

    pub fn register_miss(&mut self) {
        self.misses += 1;
        self.combo = 0;
    }

    pub fn apply_time_bonus(&mut self, time_remaining: f32) {
        let bonus = (time_remaining * 2.0) as u32;
        self.time_bonus += bonus;
        self.session += bonus;
        self.total += bonus;
    }

    pub fn accuracy(&self) -> f32 {
        let total = self.hits + self.misses;
        if total == 0 {
            return 0.0;
        }
        self.hits as f32 / total as f32
    }
}
