//! Tipos de pedra com propriedades físicas.

use serde::{Deserialize, Serialize};

pub const STONE_COUNT: usize = 7;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum StoneKind {
    Small,
    Medium,
    Large,
    Smooth,
    Irregular,
    Metallic,
    Explosive,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct StoneStats {
    pub kind: StoneKind,
    pub mass_kg: f32,
    pub radius_m: f32,
    pub drag_coeff: f32,
    pub damage_mult: f32,
    pub dispersion: f32,
    pub unlock_level: u32,
    pub consumable: bool,
}

impl StoneKind {
    pub fn stats(self) -> StoneStats {
        match self {
            StoneKind::Small => StoneStats {
                kind: self,
                mass_kg: 0.15,
                radius_m: 0.04,
                drag_coeff: 0.42,
                damage_mult: 1.0,
                dispersion: 0.02,
                unlock_level: 0,
                consumable: false,
            },
            StoneKind::Medium => StoneStats {
                kind: self,
                mass_kg: 0.45,
                radius_m: 0.07,
                drag_coeff: 0.40,
                damage_mult: 2.0,
                dispersion: 0.015,
                unlock_level: 0,
                consumable: false,
            },
            StoneKind::Large => StoneStats {
                kind: self,
                mass_kg: 1.2,
                radius_m: 0.12,
                drag_coeff: 0.38,
                damage_mult: 4.0,
                dispersion: 0.025,
                unlock_level: 0,
                consumable: false,
            },
            StoneKind::Smooth => StoneStats {
                kind: self,
                mass_kg: 0.35,
                radius_m: 0.06,
                drag_coeff: 0.28,
                damage_mult: 1.5,
                dispersion: 0.01,
                unlock_level: 3,
                consumable: false,
            },
            StoneKind::Irregular => StoneStats {
                kind: self,
                mass_kg: 0.55,
                radius_m: 0.08,
                drag_coeff: 0.55,
                damage_mult: 2.5,
                dispersion: 0.04,
                unlock_level: 10,
                consumable: false,
            },
            StoneKind::Metallic => StoneStats {
                kind: self,
                mass_kg: 2.0,
                radius_m: 0.10,
                drag_coeff: 0.25,
                damage_mult: 5.0,
                dispersion: 0.012,
                unlock_level: 15,
                consumable: false,
            },
            StoneKind::Explosive => StoneStats {
                kind: self,
                mass_kg: 0.40,
                radius_m: 0.07,
                drag_coeff: 0.40,
                damage_mult: 3.0,
                dispersion: 0.02,
                unlock_level: 25,
                consumable: true,
            },
        }
    }

    pub fn all() -> [StoneKind; STONE_COUNT] {
        [
            StoneKind::Small,
            StoneKind::Medium,
            StoneKind::Large,
            StoneKind::Smooth,
            StoneKind::Irregular,
            StoneKind::Metallic,
            StoneKind::Explosive,
        ]
    }

    pub fn display_name(self) -> &'static str {
        match self {
            StoneKind::Small => "Pequena",
            StoneKind::Medium => "Média",
            StoneKind::Large => "Grande",
            StoneKind::Smooth => "Lisa",
            StoneKind::Irregular => "Irregular",
            StoneKind::Metallic => "Metálica",
            StoneKind::Explosive => "Explosiva",
        }
    }
}
