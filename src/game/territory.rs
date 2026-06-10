//! Territórios — zonas de influência e disputas entre facções do deserto.

use crate::game::events::{EventLog, GameEvent, SimTick};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Faction {
    Nomades,
    Mercadores,
    Sultao,
    Bandidos,
    Jogador,
}

impl Faction {
    pub fn name(self) -> &'static str {
        match self {
            Faction::Nomades => "Nômades",
            Faction::Mercadores => "Mercadores",
            Faction::Sultao => "Sultão",
            Faction::Bandidos => "Bandidos",
            Faction::Jogador => "Jogador",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerritoryZone {
    pub id: u32,
    pub name: String,
    pub cx: f32,
    pub cz: f32,
    pub radius: f32,
    pub owner: Faction,
    pub control: f32,
    pub settlement_id: u32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TerritorySim {
    pub zones: Vec<TerritoryZone>,
    #[serde(default)]
    dispute_cooldown: f32,
}

impl TerritorySim {
    pub fn populate_with_settlements(
        &mut self,
        settlements: &[(u32, String, f32, f32, Faction, f32)],
    ) {
        self.zones.clear();
        for &(id, ref name, cx, cz, owner, radius) in settlements {
            self.zones.push(TerritoryZone {
                id,
                name: name.clone(),
                cx,
                cz,
                radius,
                owner,
                control: 1.0,
                settlement_id: id,
            });
        }
    }

    pub fn update(&mut self, dt: f32, player_x: f32, player_z: f32, events: &mut EventLog, tick: SimTick) {
        self.dispute_cooldown = (self.dispute_cooldown - dt).max(0.0);

        for zone in &mut self.zones {
            let dx = player_x - zone.cx;
            let dz = player_z - zone.cz;
            let dist = (dx * dx + dz * dz).sqrt();
            if dist < zone.radius && zone.owner != Faction::Jogador {
                zone.control = (zone.control - dt * 0.02).max(0.0);
            }
        }

        if self.dispute_cooldown <= 0.0 {
            let n = self.zones.len();
            for i in 0..n {
                for j in (i + 1)..n {
                    let a = &self.zones[i];
                    let b = &self.zones[j];
                    if a.owner == b.owner {
                        continue;
                    }
                    let dx = a.cx - b.cx;
                    let dz = a.cz - b.cz;
                    let overlap = a.radius + b.radius - (dx * dx + dz * dz).sqrt();
                    if overlap > 20.0 {
                        events.push(GameEvent::TerritoryDispute {
                            tick,
                            zone_a: a.id,
                            zone_b: b.id,
                        });
                        self.dispute_cooldown = 25.0;
                        return;
                    }
                }
            }
        }
    }

    pub fn zone_at(&self, x: f32, wz: f32) -> Option<&TerritoryZone> {
        self.zones
            .iter()
            .filter(|zone| {
                let dx = x - zone.cx;
                let dz = wz - zone.cz;
                dx * dx + dz * dz <= zone.radius * zone.radius
            })
            .min_by(|a, b| {
                let da = (x - a.cx).powi(2) + (wz - a.cz).powi(2);
                let db = (x - b.cx).powi(2) + (wz - b.cz).powi(2);
                da.partial_cmp(&db).unwrap()
            })
    }
}
