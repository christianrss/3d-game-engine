mod arcade;
mod daily;
mod distance;
mod local_mp;
mod precision;
mod survival;

pub use arcade::ArcadeMode;
pub use daily::DailyChallenge;
pub use distance::DistanceMode;
pub use local_mp::LocalMultiplayer;
pub use precision::PrecisionMode;
pub use survival::SurvivalMode;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GameMode {
    Arcade,
    Precision,
    Distance,
    Survival,
    Daily,
    LocalMultiplayer,
}

impl GameMode {
    pub fn display_name(self) -> &'static str {
        match self {
            GameMode::Arcade => "Arcade",
            GameMode::Precision => "Precisão",
            GameMode::Distance => "Distância",
            GameMode::Survival => "Sobrevivência",
            GameMode::Daily => "Desafio Diário",
            GameMode::LocalMultiplayer => "Multiplayer Local",
        }
    }

    pub fn max_throws(self) -> u32 {
        match self {
            GameMode::Arcade => 10,
            GameMode::Precision => 15,
            GameMode::Distance => 20,
            GameMode::Survival => u32::MAX,
            GameMode::Daily => 12,
            GameMode::LocalMultiplayer => 8,
        }
    }

    pub fn time_limit_secs(self) -> Option<f32> {
        match self {
            GameMode::Arcade => Some(120.0),
            GameMode::Precision => Some(180.0),
            GameMode::Distance => None,
            GameMode::Survival => None,
            GameMode::Daily => Some(150.0),
            GameMode::LocalMultiplayer => Some(300.0),
        }
    }
}
