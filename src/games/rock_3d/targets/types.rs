//! Tipos de alvo e registro.

use crate::math::Vec3;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TargetKind {
    Plate,
    Can,
    Bottle,
    Bell,
    Drone,
    Cart,
    Platform,
    Npc,
}

impl TargetKind {
    pub fn base_points(self) -> u32 {
        match self {
            TargetKind::Plate => 50,
            TargetKind::Can => 75,
            TargetKind::Bottle => 100,
            TargetKind::Bell => 150,
            TargetKind::Drone => 200,
            TargetKind::Cart => 175,
            TargetKind::Platform => 125,
            TargetKind::Npc => 300,
        }
    }

    pub fn radius(self) -> f32 {
        match self {
            TargetKind::Plate => 0.35,
            TargetKind::Can => 0.15,
            TargetKind::Bottle => 0.12,
            TargetKind::Bell => 0.25,
            TargetKind::Drone => 0.30,
            TargetKind::Cart => 0.50,
            TargetKind::Platform => 0.40,
            TargetKind::Npc => 0.35,
        }
    }

    pub fn is_mobile(self) -> bool {
        matches!(
            self,
            TargetKind::Drone | TargetKind::Cart | TargetKind::Platform | TargetKind::Npc
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TargetInstance {
    pub id: u32,
    pub kind: TargetKind,
    pub position: Vec3,
    pub velocity: Vec3,
    pub hp: f32,
    pub max_hp: f32,
    pub alive: bool,
    pub points: u32,
}

impl TargetInstance {
    pub fn new(id: u32, kind: TargetKind, position: Vec3) -> Self {
        let hp = match kind {
            TargetKind::Plate => 30.0,
            TargetKind::Can => 20.0,
            TargetKind::Bottle => 15.0,
            TargetKind::Bell => 50.0,
            TargetKind::Drone => 40.0,
            TargetKind::Cart => 60.0,
            TargetKind::Platform => 45.0,
            TargetKind::Npc => 80.0,
        };
        Self {
            id,
            kind,
            position,
            velocity: Vec3::ZERO,
            hp,
            max_hp: hp,
            alive: true,
            points: kind.base_points(),
        }
    }

    pub fn take_damage(&mut self, damage: f32) -> bool {
        if !self.alive {
            return false;
        }
        self.hp -= damage;
        if self.hp <= 0.0 {
            self.alive = false;
            true
        } else {
            false
        }
    }
}

#[derive(Default)]
pub struct TargetRegistry {
    pub targets: Vec<TargetInstance>,
    next_id: u32,
}

impl TargetRegistry {
    pub fn spawn(&mut self, kind: TargetKind, position: Vec3) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        self.targets.push(TargetInstance::new(id, kind, position));
        id
    }

    pub fn alive_count(&self) -> usize {
        self.targets.iter().filter(|t| t.alive).count()
    }
}
