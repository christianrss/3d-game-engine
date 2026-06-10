//! Armas — pistola, arco, funda, machado, espada, pedra, martelo, metralhadora.

use crate::math::Vec3;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum WeaponKind {
    #[default]
    Gun,
    Bow,
    Sling,
    MachineGun,
    Axe,
    Sword,
    Rock,
    Hammer,
    Torch,
}

impl WeaponKind {
    pub const ALL: [WeaponKind; 9] = [
        WeaponKind::Gun,
        WeaponKind::Bow,
        WeaponKind::Sling,
        WeaponKind::MachineGun,
        WeaponKind::Axe,
        WeaponKind::Sword,
        WeaponKind::Rock,
        WeaponKind::Hammer,
        WeaponKind::Torch,
    ];

    pub fn next(self) -> Self {
        let i = Self::ALL.iter().position(|&w| w == self).unwrap_or(0);
        Self::ALL[(i + 1) % Self::ALL.len()]
    }

    pub fn label(self) -> &'static str {
        match self {
            WeaponKind::Gun => "Pistola",
            WeaponKind::Bow => "Arco",
            WeaponKind::Sling => "Funda",
            WeaponKind::MachineGun => "Metralhadora",
            WeaponKind::Axe => "Machado",
            WeaponKind::Sword => "Espada",
            WeaponKind::Rock => "Pedra",
            WeaponKind::Hammer => "Martelo",
            WeaponKind::Torch => "Tocha",
        }
    }

    pub fn cooldown(self) -> f32 {
        match self {
            WeaponKind::Gun => 0.25,
            WeaponKind::Bow => 0.9,
            WeaponKind::Sling => 0.55,
            WeaponKind::MachineGun => 0.08,
            WeaponKind::Axe => 0.65,
            WeaponKind::Sword => 0.45,
            WeaponKind::Rock => 0.7,
            WeaponKind::Hammer => 0.5,
            WeaponKind::Torch => 0.35,
        }
    }

    pub fn damage(self) -> f32 {
        match self {
            WeaponKind::Gun => 100.0,
            WeaponKind::Bow => 85.0,
            WeaponKind::Sling => 45.0,
            WeaponKind::MachineGun => 35.0,
            WeaponKind::Axe => 120.0,
            WeaponKind::Sword => 90.0,
            WeaponKind::Rock => 30.0,
            WeaponKind::Hammer => 15.0,
            WeaponKind::Torch => 20.0,
        }
    }

    pub fn projectile_speed(self) -> Option<f32> {
        match self {
            WeaponKind::Gun => Some(180.0),
            WeaponKind::Bow => Some(95.0),
            WeaponKind::Sling => Some(55.0),
            WeaponKind::MachineGun => Some(140.0),
            WeaponKind::Rock => Some(32.0),
            _ => None,
        }
    }

    pub fn projectile_radius(self) -> f32 {
        match self {
            WeaponKind::MachineGun => 0.1,
            WeaponKind::Rock => 0.22,
            WeaponKind::Sling => 0.12,
            _ => 0.14,
        }
    }

    pub fn is_melee(self) -> bool {
        matches!(
            self,
            WeaponKind::Axe | WeaponKind::Sword | WeaponKind::Hammer | WeaponKind::Torch
        )
    }

    pub fn melee_range(self) -> f32 {
        match self {
            WeaponKind::Axe => 2.8,
            WeaponKind::Sword => 2.4,
            WeaponKind::Hammer => 2.2,
            WeaponKind::Torch => 2.6,
            _ => 0.0,
        }
    }

    pub fn ignites(self) -> bool {
        self == WeaponKind::Torch
    }

    pub fn uses_projectile(self) -> bool {
        self.projectile_speed().is_some()
    }

    pub fn build_tool(self) -> bool {
        self == WeaponKind::Hammer
    }
}

pub struct WeaponState {
    pub active: WeaponKind,
    pub cooldown: f32,
}

impl Default for WeaponState {
    fn default() -> Self {
        Self {
            active: WeaponKind::Gun,
            cooldown: 0.0,
        }
    }
}

impl WeaponState {
    pub fn cycle(&mut self) {
        self.active = self.active.next();
    }
}

/// Hitscan curto para machado/espada.
pub fn melee_hit(origin: Vec3, forward: Vec3, range: f32) -> Vec3 {
    origin + forward.normalize() * range * 0.5
}
