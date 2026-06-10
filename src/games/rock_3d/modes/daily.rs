//! Desafio Diário — seed procedural.

use crate::games::rock_3d::procedural::DailySeed;
use crate::games::rock_3d::targets::TargetRegistry;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyChallenge {
    pub seed: u64,
    pub best_score: u32,
    pub attempts_today: u32,
    pub completed: bool,
}

impl Default for DailyChallenge {
    fn default() -> Self {
        Self {
            seed: DailySeed::today_seed(),
            best_score: 0,
            attempts_today: 0,
            completed: false,
        }
    }
}

impl DailyChallenge {
    pub fn setup(&self, targets: &mut TargetRegistry) {
        let layout = DailySeed::new(self.seed).generate_layout();
        targets.targets.clear();
        for t in layout.targets {
            targets.spawn(t.kind, t.position);
        }
    }
}
