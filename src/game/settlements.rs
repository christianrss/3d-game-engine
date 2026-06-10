//! Assentamentos — cabanas, casas, cidades, castelos, NPCs autônomos.

use crate::assets::sample_desert_height;
use crate::game::building::{BlockGrid, BlockKey, BlockKind, PlacedBlock};
use crate::game::ecosystem::{CreatureKind, Ecosystem};
use crate::game::events::{EntityId, EventLog, GameEvent, SimTick};
use crate::game::world::{Drawable, GameWorld};
use crate::game::world_gen::OASIS_POSITIONS;
use crate::graphics::DrawMaterial;
use crate::math::{Quat, Vec3};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StructureKind {
    Cabana,
    Casa,
    Mercado,
    Castelo,
    Torre,
    Caravana,
    Poço,
}

impl StructureKind {
    pub fn model_id(self) -> &'static str {
        match self {
            StructureKind::Cabana => "desert_cabin",
            StructureKind::Casa => "desert_house",
            StructureKind::Mercado => "desert_market",
            StructureKind::Castelo => "desert_castle",
            StructureKind::Torre => "desert_tower",
            StructureKind::Caravana => "desert_caravan",
            StructureKind::Poço => "well",
        }
    }

    pub fn max_health(self) -> f32 {
        match self {
            StructureKind::Cabana => 80.0,
            StructureKind::Casa => 120.0,
            StructureKind::Mercado => 100.0,
            StructureKind::Castelo => 400.0,
            StructureKind::Torre => 150.0,
            StructureKind::Caravana => 60.0,
            StructureKind::Poço => 200.0,
        }
    }

    pub fn scale(self) -> Vec3 {
        match self {
            StructureKind::Cabana => Vec3::splat(1.0),
            StructureKind::Casa => Vec3::splat(1.1),
            StructureKind::Mercado => Vec3::splat(1.2),
            StructureKind::Castelo => Vec3::splat(1.4),
            StructureKind::Torre => Vec3::splat(1.0),
            StructureKind::Caravana => Vec3::splat(1.0),
            StructureKind::Poço => Vec3::ONE,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NpcRole {
    Vendor,
    Soldier,
    CaravanTrader,
    Herder,
    Hunter,
    Builder,
    Citizen,
}

impl NpcRole {
    pub fn model_id(self) -> &'static str {
        match self {
            NpcRole::Vendor => "npc_vendor",
            NpcRole::Soldier => "npc_soldier",
            NpcRole::CaravanTrader => "npc_caravan",
            NpcRole::Herder => "hermit",
            NpcRole::Hunter => "npc_hunter",
            NpcRole::Builder => "npc_builder",
            NpcRole::Citizen => "npc_citizen",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Structure {
    pub id: u32,
    pub kind: StructureKind,
    pub pos: Vec3,
    pub yaw: f32,
    pub health: f32,
    pub alive: bool,
    pub settlement_id: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NpcAgent {
    pub id: u32,
    pub role: NpcRole,
    pub pos: Vec3,
    pub yaw: f32,
    pub vel: Vec3,
    pub settlement_id: u32,
    pub timer: f32,
    pub target: Option<Vec3>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settlement {
    pub id: u32,
    pub name: String,
    pub center: Vec3,
    pub radius: f32,
    pub structures: Vec<u32>,
    pub npcs: Vec<u32>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SettlementSim {
    pub settlements: Vec<Settlement>,
    pub structures: Vec<Structure>,
    pub npcs: Vec<NpcAgent>,
    #[serde(default)]
    next_id: u32,
}

impl SettlementSim {
    pub fn populate_desert(&mut self) {
        if !self.settlements.is_empty() {
            return;
        }

        let cities = [
            ("Al-Badia", -150.0, -80.0, 55.0_f32),
            ("Ksar Duna", 200.0, -50.0, 65.0),
            ("Oued Dourado", -50.0, 200.0, 50.0),
        ];
        for (name, cx, cz, r) in cities {
            self.spawn_city(name, cx, cz, r);
        }

        for &(ox, oz) in OASIS_POSITIONS {
            self.spawn_oasis_village(ox, oz);
        }
    }

    fn alloc_id(&mut self) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    fn spawn_city(&mut self, name: &str, cx: f32, cz: f32, radius: f32) {
        let sid = self.alloc_id();
        let y = sample_desert_height(cx, cz);
        let center = Vec3::new(cx, y, cz);
        let mut structures = Vec::new();
        let mut npcs = Vec::new();

        let castle_id = self.add_structure(sid, StructureKind::Castelo, cx, cz, 0.0);
        structures.push(castle_id);
        for i in 0..3 {
            let a = i as f32 * 2.1;
            npcs.push(self.add_npc(sid, NpcRole::Soldier, cx + a.cos() * 12.0, cz + a.sin() * 12.0));
        }

        for i in 0..6 {
            let a = i as f32 * 1.05;
            let r = 18.0 + (i % 2) as f32 * 6.0;
            let kind = if i == 0 {
                StructureKind::Mercado
            } else if i % 3 == 0 {
                StructureKind::Casa
            } else {
                StructureKind::Cabana
            };
            let id = self.add_structure(sid, kind, cx + a.cos() * r, cz + a.sin() * r, a);
            structures.push(id);
        }

        npcs.push(self.add_npc(sid, NpcRole::Vendor, cx + 8.0, cz + 4.0));
        npcs.push(self.add_npc(sid, NpcRole::Builder, cx - 6.0, cz + 8.0));
        npcs.push(self.add_npc(sid, NpcRole::Hunter, cx + 20.0, cz - 10.0));
        npcs.push(self.add_npc(sid, NpcRole::Herder, cx - 15.0, cz + 12.0));
        npcs.push(self.add_npc(sid, NpcRole::CaravanTrader, cx + 25.0, cz));

        for i in 0..4 {
            npcs.push(self.add_npc(
                sid,
                NpcRole::Citizen,
                cx + (i as f32 * 3.7).cos() * 10.0,
                cz + (i as f32 * 3.7).sin() * 10.0,
            ));
        }

        self.settlements.push(Settlement {
            id: sid,
            name: name.into(),
            center,
            radius,
            structures,
            npcs,
        });
    }

    fn spawn_oasis_village(&mut self, ox: f32, oz: f32) {
        let sid = self.alloc_id();
        let y = sample_desert_height(ox, oz);
        let mut structures = Vec::new();
        let mut npcs = Vec::new();
        for i in 0..3 {
            let a = i as f32 * 2.2;
            let id = self.add_structure(sid, StructureKind::Cabana, ox + a.cos() * 14.0, oz + a.sin() * 14.0, a);
            structures.push(id);
        }
        npcs.push(self.add_npc(sid, NpcRole::Herder, ox + 5.0, oz + 3.0));
        npcs.push(self.add_npc(sid, NpcRole::Builder, ox - 4.0, oz + 6.0));
        self.settlements.push(Settlement {
            id: sid,
            name: format!("Oásis {:.0},{:.0}", ox, oz),
            center: Vec3::new(ox, y, oz),
            radius: 35.0,
            structures,
            npcs,
        });
    }

    fn add_structure(&mut self, sid: u32, kind: StructureKind, x: f32, z: f32, yaw: f32) -> u32 {
        let id = self.alloc_id();
        let y = sample_desert_height(x, z);
        self.structures.push(Structure {
            id,
            kind,
            pos: Vec3::new(x, y, z),
            yaw,
            health: kind.max_health(),
            alive: true,
            settlement_id: sid,
        });
        id
    }

    fn add_npc(&mut self, sid: u32, role: NpcRole, x: f32, z: f32) -> u32 {
        let id = self.alloc_id();
        let y = sample_desert_height(x, z);
        self.npcs.push(NpcAgent {
            id,
            role,
            pos: Vec3::new(x, y, z),
            yaw: 0.0,
            vel: Vec3::ZERO,
            settlement_id: sid,
            timer: role_timer(role),
            target: None,
        });
        id
    }

    pub fn update(
        &mut self,
        dt: f32,
        player_pos: Vec3,
        blocks: &mut BlockGrid,
        eco: &mut Ecosystem,
        events: &mut EventLog,
        tick: SimTick,
    ) {
        for npc in &mut self.npcs {
            npc.timer -= dt;
            let settlement = self
                .settlements
                .iter()
                .find(|s| s.id == npc.settlement_id);
            let Some(home) = settlement else { continue };

            match npc.role {
                NpcRole::Herder => update_herder(npc, dt, player_pos, home.center),
                NpcRole::Hunter => update_hunter(npc, dt, eco, home.center, events, tick),
                NpcRole::Builder => {
                    if npc.timer <= 0.0 {
                        try_npc_build(npc, home, blocks, events, tick);
                        npc.timer = 8.0 + (npc.id % 5) as f32;
                    }
                    wander_near(npc, dt, home.center, home.radius * 0.5);
                }
                NpcRole::Soldier => patrol(npc, dt, home.center, home.radius * 0.7),
                NpcRole::Vendor | NpcRole::Citizen => {
                    wander_near(npc, dt, home.center, home.radius * 0.4);
                }
                NpcRole::CaravanTrader => {
                    if npc.timer <= 0.0 {
                        npc.target = Some(random_offset(home.center, home.radius * 1.5, npc.id));
                        npc.timer = 12.0;
                    }
                    move_toward(npc, dt, 2.5);
                }
            }
            npc.pos.y = sample_desert_height(npc.pos.x, npc.pos.z);
            if npc.vel.length_squared() > 0.01 {
                npc.yaw = npc.vel.x.atan2(npc.vel.z);
            }
        }
    }

    pub fn damage_at(
        &mut self,
        wx: f32,
        wz: f32,
        damage: f32,
        events: &mut EventLog,
        tick: SimTick,
    ) {
        for s in &mut self.structures {
            if !s.alive {
                continue;
            }
            let dx = s.pos.x - wx;
            let dz = s.pos.z - wz;
            if dx * dx + dz * dz < 36.0 {
                s.health -= damage;
                events.push(GameEvent::StructureDamaged {
                    tick,
                    structure_id: s.id,
                    health: s.health,
                });
                if s.health <= 0.0 {
                    s.alive = false;
                    events.push(GameEvent::StructureDestroyed {
                        tick,
                        structure_id: s.id,
                    });
                }
            }
        }
    }

    pub fn territory_seeds(
        &self,
    ) -> Vec<(u32, String, f32, f32, crate::game::territory::Faction, f32)> {
        use crate::game::territory::Faction;
        self.settlements
            .iter()
            .map(|s| {
                let faction = match s.name.as_str() {
                    "Al-Badia" => Faction::Mercadores,
                    "Ksar Duna" => Faction::Sultao,
                    "Oued Dourado" => Faction::Nomades,
                    _ => Faction::Nomades,
                };
                (
                    s.id,
                    s.name.clone(),
                    s.center.x,
                    s.center.z,
                    faction,
                    s.radius,
                )
            })
            .collect()
    }

    pub fn nearest_vendor(&self, pos: Vec3) -> Option<&NpcAgent> {
        self.npcs
            .iter()
            .filter(|n| n.role == NpcRole::Vendor)
            .min_by(|a, b| {
                a.pos
                    .distance(pos)
                    .partial_cmp(&b.pos.distance(pos))
                    .unwrap()
            })
            .filter(|n| n.pos.distance(pos) < 6.0)
    }
}

fn role_timer(role: NpcRole) -> f32 {
    match role {
        NpcRole::Builder => 4.0,
        NpcRole::CaravanTrader => 6.0,
        _ => 2.0,
    }
}

fn wander_near(npc: &mut NpcAgent, dt: f32, center: Vec3, radius: f32) {
    if npc.timer <= 0.0 {
        npc.target = Some(random_offset(center, radius, npc.id));
        npc.timer = 2.0 + (npc.id % 4) as f32;
    }
    move_toward(npc, dt, 1.8);
}

fn patrol(npc: &mut NpcAgent, dt: f32, center: Vec3, radius: f32) {
    if npc.timer <= 0.0 {
        npc.target = Some(random_offset(center, radius, npc.id.wrapping_add(7)));
        npc.timer = 4.0;
    }
    move_toward(npc, dt, 2.2);
}

fn update_herder(npc: &mut NpcAgent, dt: f32, player_pos: Vec3, home: Vec3) {
    let to_player = player_pos - npc.pos;
    if to_player.length() < 25.0 {
        npc.target = Some(player_pos + Vec3::new(3.0, 0.0, 3.0));
    } else {
        wander_near(npc, dt, home, 20.0);
    }
    move_toward(npc, dt, 2.0);
}

fn update_hunter(
    npc: &mut NpcAgent,
    dt: f32,
    eco: &mut Ecosystem,
    home: Vec3,
    events: &mut EventLog,
    tick: SimTick,
) {
    if let Some(bird_id) = eco.npc_hunt_birds(npc.pos, 8.0) {
        events.push(GameEvent::CreatureHunted {
            tick,
            creature_id: bird_id,
            hunter_npc: npc.id,
        });
        npc.timer = 2.0;
    }
    if npc.timer <= 0.0 {
        if let Some((bx, bz)) = nearest_bird_xz(eco, home) {
            npc.target = Some(Vec3::new(bx, sample_desert_height(bx, bz), bz));
        } else {
            npc.target = Some(
                home
                    + Vec3::new(
                        (npc.id as f32 * 1.3).sin() * 30.0,
                        0.0,
                        (npc.id as f32 * 0.9).cos() * 30.0,
                    ),
            );
        }
        npc.timer = 4.0;
    }
    move_toward(npc, dt, 3.2);
}

fn nearest_bird_xz(eco: &Ecosystem, from: Vec3) -> Option<(f32, f32)> {
    eco.creatures
        .iter()
        .filter(|c| c.alive && c.kind == CreatureKind::Bird)
        .min_by(|a, b| {
            a.pos
                .distance(from)
                .partial_cmp(&b.pos.distance(from))
                .unwrap()
        })
        .map(|c| (c.pos.x, c.pos.z))
}

fn move_toward(npc: &mut NpcAgent, dt: f32, speed: f32) {
    if let Some(t) = npc.target {
        let to = t - npc.pos;
        let d = to.length();
        if d < 0.8 {
            npc.target = None;
            npc.vel = Vec3::ZERO;
        } else {
            npc.vel = to.normalize() * speed.min(d * 2.0);
            npc.pos += npc.vel * dt;
        }
    }
}

fn random_offset(center: Vec3, radius: f32, seed: u32) -> Vec3 {
    let a = seed as f32 * 0.71;
    Vec3::new(
        center.x + a.sin() * radius,
        center.y,
        center.z + a.cos() * radius,
    )
}

fn try_npc_build(
    npc: &NpcAgent,
    home: &Settlement,
    blocks: &mut BlockGrid,
    events: &mut EventLog,
    tick: SimTick,
) {
    let gx = (home.center.x / 2.0).round() as i32 + (npc.id % 5) as i32;
    let gz = (home.center.z / 2.0).round() as i32 + ((npc.id / 5) % 5) as i32;
    let key = BlockKey { x: gx, z: gz, level: 0 };
    if blocks.place(
        key,
        PlacedBlock {
            kind: if npc.id % 2 == 0 {
                BlockKind::Dirt
            } else {
                BlockKind::Wall
            },
            yaw: (npc.id % 4) as u8,
        },
    ) {
        events.push(GameEvent::NpcBuilt {
            tick,
            npc_id: npc.id,
            settlement_id: home.id,
        });
    }
}

const SETTLEMENT_MODELS: &[&str] = &[
    "desert_cabin",
    "desert_house",
    "desert_market",
    "desert_castle",
    "desert_tower",
    "desert_caravan",
    "npc_vendor",
    "npc_soldier",
    "npc_caravan",
    "npc_hunter",
    "npc_builder",
    "npc_citizen",
];

pub fn sync_settlement_drawables(
    world: &mut GameWorld,
    sim: &SettlementSim,
    chunks: Option<&crate::game::chunks::ChunkManager>,
) {
    world
        .drawables
        .retain(|d| !SETTLEMENT_MODELS.contains(&d.model_id.as_str()) && d.model_id != "well");

    for s in &sim.structures {
        if !s.alive {
            continue;
        }
        if chunks.is_some_and(|c| !c.is_active(s.pos.x, s.pos.z)) {
            continue;
        }
        world.add_drawable(Drawable {
            model_id: s.kind.model_id().into(),
            position: s.pos,
            rotation: Quat::from_rotation_y(s.yaw),
            scale: s.kind.scale(),
            material: if matches!(s.kind, StructureKind::Castelo | StructureKind::Torre) {
                DrawMaterial::rock()
            } else {
                DrawMaterial::wood()
            },
            target_id: None,
        });
    }

    for n in &sim.npcs {
        if chunks.is_some_and(|c| !c.is_active(n.pos.x, n.pos.z)) {
            continue;
        }
        world.add_drawable(Drawable {
            model_id: n.role.model_id().into(),
            position: n.pos,
            rotation: Quat::from_rotation_y(n.yaw),
            scale: Vec3::ONE,
            material: DrawMaterial::Standard {
                roughness: 0.85,
                metallic: 0.0,
            },
            target_id: None,
        });
    }
}
