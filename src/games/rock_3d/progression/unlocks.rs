//! Desbloqueios por nível.

use crate::games::rock_3d::stones::StoneKind;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UnlockRegistry {
    pub stones: Vec<StoneKind>,
    pub maps: Vec<String>,
    pub gloves: Vec<String>,
    pub skins: Vec<String>,
}

impl UnlockRegistry {
    pub fn check_level(&mut self, level: u32) {
        let unlocks: &[(u32, fn(&mut UnlockRegistry))] = &[
            (3, |r| r.unlock_stone(StoneKind::Smooth)),
            (5, |r| r.unlock_map("floresta")),
            (8, |r| r.unlock_glove("precisao")),
            (10, |r| r.unlock_stone(StoneKind::Irregular)),
            (12, |r| r.unlock_map("deserto")),
            (15, |r| r.unlock_stone(StoneKind::Metallic)),
            (18, |r| r.unlock_map("montanha")),
            (20, |r| r.unlock_skin("dourada")),
            (22, |r| r.unlock_map("cidade")),
            (25, |r| r.unlock_stone(StoneKind::Explosive)),
            (30, |r| r.unlock_map("futurista")),
        ];
        for (req, func) in unlocks {
            if level >= *req {
                func(self);
            }
        }
    }

    fn unlock_stone(&mut self, kind: StoneKind) {
        if !self.stones.contains(&kind) {
            self.stones.push(kind);
        }
    }

    fn unlock_map(&mut self, name: &str) {
        let s = name.to_string();
        if !self.maps.contains(&s) {
            self.maps.push(s);
        }
    }

    fn unlock_glove(&mut self, name: &str) {
        let s = name.to_string();
        if !self.gloves.contains(&s) {
            self.gloves.push(s);
        }
    }

    fn unlock_skin(&mut self, name: &str) {
        let s = name.to_string();
        if !self.skins.contains(&s) {
            self.skins.push(s);
        }
    }

    pub fn is_stone_available(&self, kind: StoneKind, debug_all: bool) -> bool {
        if debug_all {
            return true;
        }
        kind.stats().unlock_level == 0 || self.stones.contains(&kind)
    }
}
