//! Persistência do perfil Rock 3D.

use crate::games::rock_3d::modes::DailyChallenge;
use crate::games::rock_3d::progression::{PlayerProfile, SkillTree, UnlockRegistry};
use crate::games::rock_3d::scoring::ScoreSystem;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

pub const PROFILE_PATH: &str = "saves/rock_3d/profile.json";
pub const DAILY_PATH: &str = "saves/rock_3d/daily.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rock3DSave {
    pub version: u32,
    pub profile: PlayerProfile,
    pub skills: SkillTree,
    pub unlocks: UnlockRegistry,
    pub best_scores: ScoreSystem,
    pub daily: DailyChallenge,
}

impl Default for Rock3DSave {
    fn default() -> Self {
        Self {
            version: 1,
            profile: PlayerProfile::default(),
            skills: SkillTree::default(),
            unlocks: UnlockRegistry::default(),
            best_scores: ScoreSystem::default(),
            daily: DailyChallenge::default(),
        }
    }
}

pub fn load_profile() -> Rock3DSave {
    let path = PathBuf::from(PROFILE_PATH);
    if path.exists() {
        if let Ok(data) = fs::read_to_string(&path) {
            if let Ok(save) = serde_json::from_str(&data) {
                return save;
            }
        }
    }
    Rock3DSave::default()
}

pub fn save_profile(save: &Rock3DSave) -> std::io::Result<()> {
    if let Some(parent) = PathBuf::from(PROFILE_PATH).parent() {
        fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_string_pretty(save)?;
    fs::write(PROFILE_PATH, json)
}
