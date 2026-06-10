//! Chefes com múltiplos pontos fracos.

use crate::math::Vec3;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BossKind {
    GiantRobot,
    ArmoredTower,
    ColossalDrone,
}

impl BossKind {
    pub fn max_hp(self) -> f32 {
        match self {
            BossKind::GiantRobot => 500.0,
            BossKind::ArmoredTower => 800.0,
            BossKind::ColossalDrone => 600.0,
        }
    }

    pub fn reward_points(self) -> u32 {
        match self {
            BossKind::GiantRobot => 2000,
            BossKind::ArmoredTower => 3000,
            BossKind::ColossalDrone => 2500,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeakSpot {
    pub name: String,
    pub offset: Vec3,
    pub radius: f32,
    pub damage_mult: f32,
    pub destroyed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BossInstance {
    pub kind: BossKind,
    pub position: Vec3,
    pub hp: f32,
    pub alive: bool,
    pub weak_spots: Vec<WeakSpot>,
}

impl BossInstance {
    pub fn giant_robot(position: Vec3) -> Self {
        Self {
            kind: BossKind::GiantRobot,
            position,
            hp: BossKind::GiantRobot.max_hp(),
            alive: true,
            weak_spots: vec![
                WeakSpot {
                    name: "Joelho Esq.".into(),
                    offset: Vec3::new(-1.2, 1.0, 0.0),
                    radius: 0.4,
                    damage_mult: 2.0,
                    destroyed: false,
                },
                WeakSpot {
                    name: "Antena".into(),
                    offset: Vec3::new(0.0, 4.5, 0.0),
                    radius: 0.25,
                    damage_mult: 3.0,
                    destroyed: false,
                },
            ],
        }
    }
}
