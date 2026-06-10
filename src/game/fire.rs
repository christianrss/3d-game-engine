//! Fogo — propagação, dano a estruturas de madeira, base para incendiar cidades.

use crate::game::events::{EntityId, EventLog, GameEvent, SimTick};
use crate::game::settlements::{SettlementSim, StructureKind};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

const CELL: f32 = 2.0;
const SPREAD_INTERVAL: f32 = 0.85;
const BURN_DAMAGE: f32 = 18.0;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct FireCell {
    pub x: i32,
    pub z: i32,
    pub intensity: f32,
    pub age: f32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FireSim {
    pub cells: HashMap<(i32, i32), FireCell>,
    #[serde(default)]
    spread_timer: f32,
}

impl FireSim {
    pub fn ignite(&mut self, wx: f32, wz: f32, events: &mut EventLog, tick: SimTick, actor: EntityId) {
        let key = world_to_cell(wx, wz);
        self.cells.insert(
            key,
            FireCell {
                x: key.0,
                z: key.1,
                intensity: 1.0,
                age: 0.0,
            },
        );
        events.push(GameEvent::FireStarted {
            tick,
            x: wx,
            z: wz,
            actor,
        });
    }

    pub fn update(
        &mut self,
        dt: f32,
        settlements: &mut SettlementSim,
        events: &mut EventLog,
        tick: SimTick,
    ) -> Vec<(f32, f32, f32)> {
        // (wx, wz, intensity)
        let mut visuals = Vec::new();
        self.spread_timer += dt;

        let mut to_add = Vec::new();
        if self.spread_timer >= SPREAD_INTERVAL {
            self.spread_timer = 0.0;
            for (&(x, z), cell) in &self.cells {
                if cell.intensity < 0.35 {
                    continue;
                }
                for (dx, dz) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                    let nx = x + dx;
                    let nz = z + dz;
                    if !self.cells.contains_key(&(nx, nz)) {
                        to_add.push((nx, nz, cell.intensity * 0.82));
                    }
                }
            }
            for (x, z, int) in to_add {
                self.cells.insert(
                    (x, z),
                    FireCell {
                        x,
                        z,
                        intensity: int,
                        age: 0.0,
                    },
                );
            }
        }

        let mut dead = Vec::new();
        for (&key, cell) in self.cells.iter_mut() {
            cell.age += dt;
            cell.intensity -= dt * 0.08;
            let wx = key.0 as f32 * CELL;
            let wz = key.1 as f32 * CELL;
            visuals.push((wx, wz, cell.intensity));
            settlements.damage_at(wx, wz, BURN_DAMAGE * dt, events, tick);
            if cell.intensity <= 0.05 {
                dead.push(key);
            }
        }
        for k in dead {
            self.cells.remove(&k);
        }
        visuals
    }

    pub fn is_burning_at(&self, wx: f32, wz: f32) -> bool {
        self.cells.contains_key(&world_to_cell(wx, wz))
    }
}

fn world_to_cell(wx: f32, wz: f32) -> (i32, i32) {
    ((wx / CELL).round() as i32, (wz / CELL).round() as i32)
}

pub fn structure_flammable(kind: StructureKind) -> bool {
    matches!(
        kind,
        StructureKind::Cabana | StructureKind::Casa | StructureKind::Mercado | StructureKind::Caravana
    )
}
