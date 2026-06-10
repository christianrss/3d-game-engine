//! Sistema de XP e níveis.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerProfile {
    pub xp: u64,
    pub level: u32,
    pub skill_points: u32,
    pub total_throws: u64,
    pub total_hits: u64,
    pub best_combo: u32,
}

impl Default for PlayerProfile {
    fn default() -> Self {
        Self {
            xp: 0,
            level: 1,
            skill_points: 0,
            total_throws: 0,
            total_hits: 0,
            best_combo: 0,
        }
    }
}

pub struct XpSystem;

impl XpSystem {
    pub fn xp_for_level(level: u32) -> u64 {
        (level as u64).pow(2) * 100 + (level as u64) * 50
    }

    pub fn award(profile: &mut PlayerProfile, amount: u64) -> bool {
        profile.xp += amount;
        let needed = Self::xp_for_level(profile.level + 1);
        if profile.xp >= needed {
            profile.xp -= needed;
            profile.level += 1;
            profile.skill_points += 1;
            return true;
        }
        false
    }

    pub fn hit_xp(target_points: u32, combo: u32) -> u64 {
        let base = (target_points / 5).max(10) as u64;
        let mult = 1.0 + (combo as f32 * 0.15).min(0.75);
        (base as f32 * mult) as u64
    }
}
