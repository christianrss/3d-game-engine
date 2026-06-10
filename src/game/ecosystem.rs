//! Ecossistema vivo — ovelhas, camelos, ermitoes, cobras, ETs, UFOs, miragens.

use crate::assets::sample_desert_height;
use crate::game::building::BlockGrid;
use crate::game::culling::{self, ai_sleeping, in_ai_range};
use crate::game::physics::CollisionWorld;
use crate::game::world::{Drawable, GameWorld};
use crate::game::world_gen::{random_spawn_points, OASIS_POSITIONS};
use crate::graphics::DrawMaterial;
use crate::math::{Quat, Vec3};

const WOOL_REGROW: f32 = 90.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CreatureKind {
    Sheep,
    Camel,
    Goat,
    Snake,
    Scorpion,
    Hermit,
    Et,
    Bird,
    Lion,
    Dog,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CreatureAi {
    Idle,
    Graze,
    EatGrass { timer: f32 },
    Wander { timer: f32 },
    Flee { timer: f32 },
    Herded,
    Hunt { target: u32 },
    Meditate { timer: f32 },
    TendHerd,
    ObserveUfo,
    FleeEt,
    Patrol,
    Fly { timer: f32, height: f32 },
    Stalk { target: u32 },
}

#[derive(Debug, Clone)]
pub struct Creature {
    pub id: u32,
    pub kind: CreatureKind,
    pub pos: Vec3,
    pub vel: Vec3,
    pub yaw: f32,
    pub ai: CreatureAi,
    pub health: f32,
    pub alive: bool,
    pub sheared: bool,
    pub wool_regrow: f32,
    pub timer: f32,
    pub graze_phase: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AerialKind {
    Ufo,
    Mirage,
}

#[derive(Debug, Clone)]
pub struct Aerial {
    pub id: u32,
    pub kind: AerialKind,
    pub pos: Vec3,
    pub vel: Vec3,
    pub phase: f32,
    pub alpha: f32,
    pub beam_target: Option<u32>,
}

#[derive(Default)]
pub struct Ecosystem {
    pub creatures: Vec<Creature>,
    pub aerials: Vec<Aerial>,
    next_id: u32,
    pub herded_count: usize,
}

impl CreatureKind {
    pub fn model_id(self) -> &'static str {
        match self {
            CreatureKind::Sheep => "sheep",
            CreatureKind::Camel => "camel",
            CreatureKind::Goat => "goat",
            CreatureKind::Snake => "snake",
            CreatureKind::Scorpion => "scorpion",
            CreatureKind::Hermit => "hermit",
            CreatureKind::Et => "et",
            CreatureKind::Bird => "bird",
            CreatureKind::Lion => "lion",
            CreatureKind::Dog => "dog",
        }
    }

    pub fn radius(self) -> f32 {
        match self {
            CreatureKind::Sheep => 0.55,
            CreatureKind::Camel => 0.9,
            CreatureKind::Goat => 0.4,
            CreatureKind::Snake => 0.25,
            CreatureKind::Scorpion => 0.2,
            CreatureKind::Hermit => 0.45,
            CreatureKind::Et => 0.35,
            CreatureKind::Bird => 0.12,
            CreatureKind::Lion => 0.75,
            CreatureKind::Dog => 0.35,
        }
    }

    pub fn speed(self) -> f32 {
        match self {
            CreatureKind::Sheep => 2.2,
            CreatureKind::Camel => 1.8,
            CreatureKind::Goat => 2.8,
            CreatureKind::Snake => 1.5,
            CreatureKind::Scorpion => 1.2,
            CreatureKind::Hermit => 2.0,
            CreatureKind::Et => 3.5,
            CreatureKind::Bird => 4.5,
            CreatureKind::Lion => 3.2,
            CreatureKind::Dog => 3.8,
        }
    }

    pub fn max_health(self) -> f32 {
        match self {
            CreatureKind::Bird => 25.0,
            CreatureKind::Dog => 60.0,
            CreatureKind::Goat => 80.0,
            CreatureKind::Lion => 160.0,
            CreatureKind::Camel => 120.0,
            _ => 100.0,
        }
    }

    fn is_herdable(self) -> bool {
        matches!(self, CreatureKind::Sheep | CreatureKind::Camel | CreatureKind::Goat)
    }

    fn is_tameable(self) -> bool {
        matches!(
            self,
            CreatureKind::Sheep | CreatureKind::Camel | CreatureKind::Goat | CreatureKind::Dog
        )
    }

    fn is_predator(self) -> bool {
        matches!(
            self,
            CreatureKind::Snake | CreatureKind::Scorpion | CreatureKind::Lion
        )
    }

    fn is_huntable(self) -> bool {
        matches!(self, CreatureKind::Bird | CreatureKind::Sheep | CreatureKind::Goat)
    }
}

impl Ecosystem {
    /// Rebanho denso perto do jogador / oasis — facil de encontrar.
    pub fn spawn_starter_herd(&mut self, cx: f32, cz: f32) {
        for i in 0..10 {
            let a = i as f32 * 0.63;
            let r = 8.0 + (i % 3) as f32 * 4.0;
            self.spawn(CreatureKind::Sheep, cx + a.cos() * r, cz + a.sin() * r);
        }
        for i in 0..4 {
            self.spawn(
                CreatureKind::Goat,
                cx + 14.0 + i as f32 * 3.0,
                cz + 6.0 + (i as f32 * 1.7).sin() * 4.0,
            );
        }
        self.spawn(CreatureKind::Camel, cx - 12.0, cz + 10.0);
        self.spawn(CreatureKind::Camel, cx - 8.0, cz + 16.0);
        self.spawn(CreatureKind::Hermit, cx + 6.0, cz - 4.0);
    }

    pub fn populate_desert(&mut self) {
        for &(ox, oz) in OASIS_POSITIONS {
            self.spawn_starter_herd(ox, oz);
            for i in 0..6 {
                let a = i as f32 * 1.1;
                self.spawn(CreatureKind::Bird, ox + a.cos() * 14.0, oz + a.sin() * 14.0);
            }
            self.spawn(CreatureKind::Dog, ox + 8.0, oz + 3.0);
            self.spawn(CreatureKind::Dog, ox - 6.0, oz - 5.0);
        }

        for (x, z) in random_spawn_points(4, 80.0) {
            self.spawn(CreatureKind::Lion, x, z);
        }
        for (x, z) in random_spawn_points(20, 15.0) {
            self.spawn(CreatureKind::Bird, x, z);
        }

        let sheep_pts = random_spawn_points(18, 18.0);
        for (x, z) in sheep_pts {
            self.spawn(CreatureKind::Sheep, x, z);
        }
        let camel_pts = random_spawn_points(8, 30.0);
        for (x, z) in camel_pts {
            self.spawn(CreatureKind::Camel, x, z);
        }
        for (x, z) in random_spawn_points(8, 22.0) {
            self.spawn(CreatureKind::Snake, x, z);
        }
        for (x, z) in random_spawn_points(10, 20.0) {
            self.spawn(CreatureKind::Scorpion, x, z);
        }
        for (x, z) in [(250.0, -300.0), (-420.0, 360.0), (500.0, 400.0)] {
            self.spawn(CreatureKind::Et, x, z);
        }
        for i in 0..4 {
            let angle = i as f32 * 1.57;
            let id = self.alloc_id();
            self.aerials.push(Aerial {
                id,
                kind: AerialKind::Ufo,
                pos: Vec3::new(angle.cos() * 200.0, 45.0 + i as f32 * 8.0, angle.sin() * 200.0),
                vel: Vec3::new(12.0, 0.0, 8.0),
                phase: i as f32,
                alpha: 1.0,
                beam_target: None,
            });
        }
        for _ in 0..12 {
            let (x, z) = random_spawn_points(1, 0.0).pop().unwrap_or((0.0, 0.0));
            let id = self.alloc_id();
            self.aerials.push(Aerial {
                id,
                kind: AerialKind::Mirage,
                pos: Vec3::new(x, sample_desert_height(x, z) + 1.0, z),
                vel: Vec3::ZERO,
                phase: 0.0,
                alpha: 0.0,
                beam_target: None,
            });
        }
    }

    fn alloc_id(&mut self) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    pub fn spawn(&mut self, kind: CreatureKind, x: f32, z: f32) -> u32 {
        let id = self.alloc_id();
        let y = sample_desert_height(x, z);
        let ai = match kind {
            CreatureKind::Hermit => CreatureAi::Meditate { timer: 5.0 },
            CreatureKind::Snake | CreatureKind::Scorpion => CreatureAi::Patrol,
            CreatureKind::Et => CreatureAi::ObserveUfo,
            CreatureKind::Bird => CreatureAi::Fly {
                timer: 3.0,
                height: 0.8 + (id % 5) as f32 * 0.15,
            },
            CreatureKind::Lion => CreatureAi::Patrol,
            CreatureKind::Dog => CreatureAi::Wander { timer: 4.0 },
            _ => CreatureAi::Graze,
        };
        let fly_y = if kind == CreatureKind::Bird {
            y + 0.6
        } else {
            y
        };
        self.creatures.push(Creature {
            id,
            kind,
            pos: Vec3::new(x, fly_y, z),
            vel: Vec3::ZERO,
            yaw: (x * 0.1 + z * 0.07).sin(),
            ai,
            health: kind.max_health(),
            alive: true,
            sheared: false,
            wool_regrow: 0.0,
            timer: 0.0,
            graze_phase: (id as f32 * 1.7).sin(),
        });
        id
    }

    pub fn alive_count(&self) -> usize {
        self.creatures.iter().filter(|c| c.alive).count()
    }

    pub fn sheep_alive(&self) -> usize {
        self.creatures
            .iter()
            .filter(|c| c.alive && c.kind == CreatureKind::Sheep)
            .count()
    }

    pub fn update(
        &mut self,
        dt: f32,
        player_pos: Vec3,
        player_sprinting: bool,
        gunshot_pos: Option<Vec3>,
        is_night: bool,
        blocks: &BlockGrid,
        physics: &CollisionWorld,
    ) {
        self.herded_count = 0;

        let ufo_pos = self
            .aerials
            .iter()
            .find(|a| a.kind == AerialKind::Ufo)
            .map(|a| a.pos);

        for aerial in &mut self.aerials {
            aerial.phase += dt;
            match aerial.kind {
                AerialKind::Ufo => {
                    aerial.pos += aerial.vel * dt;
                    aerial.pos.y = 40.0 + aerial.phase.sin() * 6.0;
                    if aerial.pos.x.abs() > 900.0 {
                        aerial.vel.x *= -1.0;
                    }
                    if aerial.pos.z.abs() > 900.0 {
                        aerial.vel.z *= -1.0;
                    }
                    aerial.beam_target = None;
                    if is_night {
                        if let Some(prey) = self
                            .creatures
                            .iter()
                            .find(|c| c.alive && c.kind == CreatureKind::Sheep && c.pos.distance(aerial.pos) < 80.0)
                        {
                            aerial.beam_target = Some(prey.id);
                        }
                    }
                }
                AerialKind::Mirage => {
                    let heat = (aerial.phase * 0.3).sin().max(0.0);
                    aerial.alpha = heat * 0.55;
                    aerial.pos.y = sample_desert_height(aerial.pos.x, aerial.pos.z)
                        + 1.0
                        + heat * 2.0;
                }
            }
        }

        let mut snapshots: Vec<(u32, CreatureKind, Vec3, CreatureAi)> = self
            .creatures
            .iter()
            .filter(|c| c.alive)
            .map(|c| (c.id, c.kind, c.pos, c.ai))
            .collect();

        for c in &mut self.creatures {
            if !c.alive {
                continue;
            }
            if ai_sleeping(c.pos, player_pos) {
                continue;
            }
            if c.kind == CreatureKind::Sheep && c.sheared {
                c.wool_regrow += dt;
                if c.wool_regrow >= WOOL_REGROW {
                    c.sheared = false;
                    c.wool_regrow = 0.0;
                }
            }
            if matches!(c.ai, CreatureAi::Herded) {
                self.herded_count += 1;
            }

            if let Some(shot) = gunshot_pos {
                if c.pos.distance(shot) < 22.0 {
                    c.ai = CreatureAi::Flee { timer: 4.0 };
                }
            }
            if player_sprinting && c.pos.distance(player_pos) < 9.0 && c.kind != CreatureKind::Et {
                c.ai = CreatureAi::Flee { timer: 2.5 };
            }

            for &(oid, okind, opos, _) in &snapshots {
                if oid == c.id {
                    continue;
                }
                let dist = c.pos.distance(opos);
                if okind.is_predator() && c.kind.is_herdable() && dist < 6.0 {
                    c.ai = CreatureAi::Flee { timer: 3.0 };
                }
                if c.kind == CreatureKind::Snake && okind == CreatureKind::Hermit && dist < 8.0 {
                    c.ai = CreatureAi::Flee { timer: 5.0 };
                }
                if c.kind == CreatureKind::Hermit && okind.is_herdable() && dist < 25.0 && dist > 4.0 {
                    c.ai = CreatureAi::TendHerd;
                }
                if c.kind == CreatureKind::Snake
                    && matches!(okind, CreatureKind::Goat | CreatureKind::Sheep)
                    && dist < 4.0
                {
                    c.ai = CreatureAi::Hunt { target: oid };
                }
                if c.kind == CreatureKind::Lion
                    && matches!(okind, CreatureKind::Goat | CreatureKind::Sheep | CreatureKind::Dog)
                    && dist < 35.0
                {
                    c.ai = CreatureAi::Stalk { target: oid };
                }
                if c.kind == CreatureKind::Dog
                    && okind == CreatureKind::Lion
                    && dist < 20.0
                {
                    c.ai = CreatureAi::Flee { timer: 4.0 };
                }
            }

            if let Some(ufo) = ufo_pos {
                if c.kind == CreatureKind::Hermit && c.pos.distance(ufo) < 60.0 {
                    c.ai = CreatureAi::ObserveUfo;
                }
                if is_night
                    && c.kind == CreatureKind::Sheep
                    && c.pos.distance(ufo) < 35.0
                    && self
                        .aerials
                        .iter()
                        .any(|a| a.beam_target == Some(c.id))
                {
                    c.pos.y += dt * 4.0;
                    c.ai = CreatureAi::Flee { timer: 6.0 };
                }
            }

            if c.kind == CreatureKind::Et && player_pos.distance(c.pos) < 12.0 {
                c.ai = CreatureAi::FleeEt;
            }
        }

        for c in &mut self.creatures {
            if c.alive {
                c.graze_phase += dt * (0.8 + (c.id % 3) as f32 * 0.15);
            }
        }

        self.tick_ai_timers(dt);

        let wishes: Vec<Vec3> = self
            .creatures
            .iter()
            .map(|c| {
                if c.alive && in_ai_range(c.pos, player_pos) {
                    wish_velocity(c, &self.creatures, player_pos)
                } else {
                    Vec3::ZERO
                }
            })
            .collect();

        for (c, wish) in self.creatures.iter_mut().zip(wishes.iter()) {
            if !c.alive || ai_sleeping(c.pos, player_pos) {
                continue;
            }
            if !in_ai_range(c.pos, player_pos) {
                let ground = sample_desert_height(c.pos.x, c.pos.z);
                c.pos.y = if c.kind == CreatureKind::Bird {
                    ground + 0.6
                } else {
                    ground
                };
                continue;
            }
            c.vel = *wish;
            physics.move_creature(&mut c.pos, *wish, dt, c.kind.radius(), blocks);
            if c.kind == CreatureKind::Bird {
                let ground = sample_desert_height(c.pos.x, c.pos.z);
                let hover = match c.ai {
                    CreatureAi::Fly { height, .. } => height,
                    _ => 0.5,
                };
                c.pos.y = ground + hover + (c.graze_phase * 3.2).sin() * 0.12;
            }
            if wish.length_squared() > 0.01 {
                c.yaw = wish.x.atan2(wish.z);
            }
        }

        let mut nearby: Vec<usize> = self
            .creatures
            .iter()
            .enumerate()
            .filter(|(_, c)| c.alive && in_ai_range(c.pos, player_pos))
            .map(|(i, _)| i)
            .collect();
        if nearby.len() > 1 {
            let mut positions: Vec<Vec3> = nearby.iter().map(|&i| self.creatures[i].pos).collect();
            let radii: Vec<f32> = nearby
                .iter()
                .map(|&i| self.creatures[i].kind.radius())
                .collect();
            let alive = vec![true; nearby.len()];
            CollisionWorld::separate_entities(&mut positions, &radii, &alive);
            for (slot, &i) in nearby.iter().enumerate() {
                self.creatures[i].pos = positions[slot];
                self.creatures[i].pos.y =
                    sample_desert_height(self.creatures[i].pos.x, self.creatures[i].pos.z);
            }
        }
    }
}

fn wish_velocity(c: &Creature, others: &[Creature], player_pos: Vec3) -> Vec3 {
        let speed = c.kind.speed();
        match c.ai {
            CreatureAi::Graze | CreatureAi::Idle => {
                let chew = (c.graze_phase * 2.1).sin();
                if chew > 0.6 {
                    Vec3::ZERO
                } else {
                    Vec3::new(c.yaw.cos(), 0.0, c.yaw.sin()) * speed * 0.35
                }
            }
            CreatureAi::EatGrass { .. } => Vec3::ZERO,
            CreatureAi::Wander { .. } => {
                Vec3::new(c.yaw.cos(), 0.0, c.yaw.sin()) * speed * 0.5
            }
            CreatureAi::Fly { .. } => {
                Vec3::new(c.yaw.cos(), 0.0, c.yaw.sin()) * speed * 0.55
            }
            CreatureAi::Stalk { target } => {
                if let Some(prey) = others.iter().find(|x| x.id == target && x.alive) {
                    let to = prey.pos - c.pos;
                    if to.length() < 1.5 {
                        Vec3::ZERO
                    } else {
                        to.normalize() * speed * 1.1
                    }
                } else {
                    Vec3::new(c.yaw.cos(), 0.0, c.yaw.sin()) * speed * 0.3
                }
            }
            CreatureAi::Flee { .. } => {
                let away = (c.pos - player_pos).normalize_or_zero();
                away * speed * 2.2
            }
            CreatureAi::Herded => {
                let off = Vec3::new(
                    (c.id as f32 * 2.1).sin() * 2.5,
                    0.0,
                    (c.id as f32 * 1.3).cos() * 2.5,
                );
                let to = player_pos + off - c.pos;
                let d = to.length();
                if d > 0.5 {
                    to.normalize() * (speed * 1.6).min(d * 2.0)
                } else {
                    Vec3::ZERO
                }
            }
            CreatureAi::Hunt { target } => {
                if let Some(prey) = others.iter().find(|x| x.id == target && x.alive) {
                    let to = prey.pos - c.pos;
                    if to.length() < 1.2 {
                        Vec3::ZERO
                    } else {
                        to.normalize() * speed * 1.4
                    }
                } else {
                    Vec3::ZERO
                }
            }
            CreatureAi::Meditate { .. } => Vec3::ZERO,
            CreatureAi::TendHerd => {
                let herd = others
                    .iter()
                    .filter(|x| x.alive && x.kind.is_herdable() && x.pos.distance(c.pos) < 30.0)
                    .min_by(|a, b| {
                        a.pos
                            .distance(c.pos)
                            .partial_cmp(&b.pos.distance(c.pos))
                            .unwrap()
                    });
                if let Some(animal) = herd {
                    let to = animal.pos - c.pos;
                    if to.length() > 3.0 {
                        to.normalize() * speed
                    } else {
                        Vec3::ZERO
                    }
                } else {
                    Vec3::ZERO
                }
            }
            CreatureAi::ObserveUfo => Vec3::new(c.yaw.cos(), 0.0, c.yaw.sin()) * speed * 0.2,
            CreatureAi::FleeEt => {
                let away = (c.pos - player_pos).normalize_or_zero();
                away * speed * 2.5
            }
            CreatureAi::Patrol => Vec3::new(c.yaw.cos(), 0.0, c.yaw.sin()) * speed * 0.35,
        }
}

impl Ecosystem {
    pub fn tick_ai_timers(&mut self, dt: f32) {
        for c in &mut self.creatures {
            if !c.alive {
                continue;
            }
            match c.ai {
                CreatureAi::Wander { ref mut timer } => {
                    *timer -= dt;
                    if *timer <= 0.0 {
                        c.yaw += ((c.id as f32 * 1.7).sin()) * 1.2;
                        c.ai = CreatureAi::Graze;
                    }
                }
                CreatureAi::Fly { ref mut timer, .. } => {
                    *timer -= dt;
                    if *timer <= 0.0 {
                        c.yaw += ((c.id as f32 * 2.3).cos()) * 1.5;
                        *timer = 2.0 + (c.id % 3) as f32;
                    }
                }
                CreatureAi::EatGrass { ref mut timer } => {
                    *timer -= dt;
                    if *timer <= 0.0 {
                        c.ai = CreatureAi::Wander {
                            timer: 1.5 + (c.id % 3) as f32,
                        };
                    }
                }
                CreatureAi::Flee { ref mut timer } => {
                    *timer -= dt;
                    if *timer <= 0.0 {
                        c.ai = CreatureAi::Graze;
                    }
                }
                CreatureAi::Meditate { ref mut timer } => {
                    *timer -= dt;
                    if *timer <= 0.0 {
                        c.ai = CreatureAi::Wander { timer: 4.0 };
                    }
                }
                CreatureAi::Patrol => {
                    c.timer += dt;
                    if c.timer > 2.5 {
                        c.yaw += 0.9;
                        c.timer = 0.0;
                    }
                }
                CreatureAi::Stalk { .. } => {
                    c.timer += dt;
                    if c.timer > 4.0 {
                        c.ai = CreatureAi::Patrol;
                        c.timer = 0.0;
                    }
                }
                _ => {}
            }
            if c.ai == CreatureAi::Graze && c.kind.is_herdable() {
                let chew = (c.graze_phase * 1.8).sin();
                if chew > 0.92 {
                    c.ai = CreatureAi::EatGrass {
                        timer: 1.2 + (c.id % 4) as f32 * 0.4,
                    };
                } else if chew < -0.85 {
                    c.ai = CreatureAi::Wander {
                        timer: 2.0 + (c.id % 4) as f32,
                    };
                    c.yaw += ((c.id as f32 * 1.3).cos()) * 0.9;
                }
            }
        }
    }

    /// Domar cão, cabra, camelo ou ovelha próxima.
    pub fn try_tame_near(&mut self, player_pos: Vec3, range: f32) -> Option<(u32, CreatureKind)> {
        let mut best: Option<(usize, f32)> = None;
        for (i, c) in self.creatures.iter().enumerate() {
            if !c.alive || !c.kind.is_tameable() {
                continue;
            }
            let d = c.pos.distance(player_pos);
            if d < range {
                match best {
                    None => best = Some((i, d)),
                    Some((_, bd)) if d < bd => best = Some((i, d)),
                    _ => {}
                }
            }
        }
        if let Some((i, _)) = best {
            self.creatures[i].ai = CreatureAi::Herded;
            let id = self.creatures[i].id;
            let kind = self.creatures[i].kind;
            return Some((id, kind));
        }
        None
    }

    /// Caçador NPC mata pássaro próximo.
    pub fn npc_hunt_birds(&mut self, npc_pos: Vec3, range: f32) -> Option<u32> {
        let mut best: Option<(usize, f32)> = None;
        for (i, c) in self.creatures.iter().enumerate() {
            if !c.alive || c.kind != CreatureKind::Bird {
                continue;
            }
            let d = c.pos.distance(npc_pos);
            if d < range {
                match best {
                    None => best = Some((i, d)),
                    Some((_, bd)) if d < bd => best = Some((i, d)),
                    _ => {}
                }
            }
        }
        if let Some((i, _)) = best {
            self.creatures[i].alive = false;
            return Some(self.creatures[i].id);
        }
        None
    }

    pub fn toggle_herd_near(&mut self, player_pos: Vec3, range: f32) -> bool {
        let mut best: Option<usize> = None;
        let mut dist = range;
        for (i, c) in self.creatures.iter().enumerate() {
            if !c.alive || !c.kind.is_herdable() {
                continue;
            }
            let d = c.pos.distance(player_pos);
            if d < dist {
                dist = d;
                best = Some(i);
            }
        }
        if let Some(i) = best {
            match self.creatures[i].ai {
                CreatureAi::Herded => self.creatures[i].ai = CreatureAi::Graze,
                _ => self.creatures[i].ai = CreatureAi::Herded,
            }
            return true;
        }
        false
    }

    pub fn release_all_herd(&mut self) {
        for c in &mut self.creatures {
            if c.alive && matches!(c.ai, CreatureAi::Herded) {
                c.ai = CreatureAi::Graze;
            }
        }
    }

    pub fn try_shear_near(&mut self, player_pos: Vec3) -> Option<(Vec3, u32)> {
        let mut best: Option<usize> = None;
        let mut dist = 3.5f32;
        for (i, c) in self.creatures.iter().enumerate() {
            if !c.alive || c.kind != CreatureKind::Sheep || c.sheared {
                continue;
            }
            let d = c.pos.distance(player_pos);
            if d < dist {
                dist = d;
                best = Some(i);
            }
        }
        if let Some(i) = best {
            self.creatures[i].sheared = true;
            self.creatures[i].wool_regrow = 0.0;
            let wool = 1 + (self.creatures[i].id % 3);
            let pos = self.creatures[i].pos + Vec3::new(0.0, 0.55, 0.0);
            return Some((pos, wool));
        }
        None
    }

    pub fn damage_at(
        &mut self,
        bullet_pos: Vec3,
        bullet_radius: f32,
        damage: f32,
    ) -> Option<(u32, Vec3, CreatureKind)> {
        let mut best: Option<(usize, f32)> = None;
        for (i, c) in self.creatures.iter().enumerate() {
            if !c.alive {
                continue;
            }
            let center = c.pos + Vec3::new(0.0, c.kind.radius(), 0.0);
            let d = bullet_pos.distance(center);
            if d <= c.kind.radius() + bullet_radius {
                match best {
                    None => best = Some((i, d)),
                    Some((_, bd)) if d < bd => best = Some((i, d)),
                    _ => {}
                }
            }
        }
        if let Some((i, _)) = best {
            self.creatures[i].health -= damage;
            if self.creatures[i].health <= 0.0 {
                self.creatures[i].alive = false;
                let hit = self.creatures[i].pos + Vec3::Y * 0.5;
                let id = self.creatures[i].id;
                let kind = self.creatures[i].kind;
                return Some((id, hit, kind));
            }
        }
        None
    }

    pub fn melee_damage_at(
        &mut self,
        origin: Vec3,
        forward: Vec3,
        range: f32,
        damage: f32,
    ) -> Option<(u32, Vec3, CreatureKind)> {
        let fwd = forward.normalize();
        let mut best: Option<(usize, f32)> = None;
        for (i, c) in self.creatures.iter().enumerate() {
            if !c.alive {
                continue;
            }
            let to = c.pos - origin;
            let along = to.dot(fwd);
            if along < 0.2 || along > range {
                continue;
            }
            let lateral = (to - fwd * along).length();
            if lateral > c.kind.radius() + 0.6 {
                continue;
            }
            match best {
                None => best = Some((i, along)),
                Some((_, ba)) if along < ba => best = Some((i, along)),
                _ => {}
            }
        }
        if let Some((i, _)) = best {
            self.creatures[i].health -= damage;
            if self.creatures[i].health <= 0.0 {
                self.creatures[i].alive = false;
                let hit = self.creatures[i].pos + Vec3::Y * 0.5;
                let id = self.creatures[i].id;
                let kind = self.creatures[i].kind;
                return Some((id, hit, kind));
            }
        }
        None
    }

    pub fn build_radar(&self, player_pos: Vec3, player_yaw: f32, range: f32) -> Vec<(f32, f32, u8)> {
        let mut blips = Vec::with_capacity(24);
        for c in &self.creatures {
            if !c.alive {
                continue;
            }
            let dx = c.pos.x - player_pos.x;
            let dz = c.pos.z - player_pos.z;
            let dist = (dx * dx + dz * dz).sqrt();
            if dist > range || dist < 0.5 {
                continue;
            }
            let world_a = dx.atan2(dz);
            let mut rel = world_a - player_yaw;
            while rel > std::f32::consts::PI {
                rel -= std::f32::consts::TAU;
            }
            while rel < -std::f32::consts::PI {
                rel += std::f32::consts::TAU;
            }
            let kind = match c.kind {
                CreatureKind::Sheep => 0,
                CreatureKind::Hermit => 1,
                CreatureKind::Camel | CreatureKind::Goat => 2,
                CreatureKind::Snake | CreatureKind::Scorpion | CreatureKind::Lion => 3,
                CreatureKind::Et => 4,
                CreatureKind::Bird => 5,
                CreatureKind::Dog => 6,
            };
            blips.push((rel / std::f32::consts::PI, dist / range, kind));
        }
        blips.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
        blips.truncate(20);
        blips
    }

    pub fn nearest_interact_dist(&self, player_pos: Vec3) -> f32 {
        self.creatures
            .iter()
            .filter(|c| {
                c.alive
                    && (c.kind.is_herdable() || c.kind == CreatureKind::Hermit)
            })
            .map(|c| c.pos.distance(player_pos))
            .min_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(f32::INFINITY)
    }

    pub fn beacon_positions(&self, player_pos: Vec3, range: f32, max: usize) -> Vec<Vec3> {
        let mut list: Vec<(f32, Vec3)> = self
            .creatures
            .iter()
            .filter(|c| {
                c.alive
                    && (c.kind.is_herdable() || c.kind == CreatureKind::Hermit)
                    && culling::dist_sq_xz(c.pos, player_pos) < range * range
            })
            .map(|c| (culling::dist_sq_xz(c.pos, player_pos), c.pos))
            .collect();
        list.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
        list.into_iter().take(max).map(|(_, p)| p).collect()
    }
}

pub fn sync_ecosystem_drawables(world: &mut GameWorld, eco: &Ecosystem, viewer: Vec3) {
    const MODELS: &[&str] = &[
        "sheep", "camel", "goat", "snake", "scorpion", "hermit", "et", "bird", "lion", "dog",
        "ufo", "mirage",
    ];
    world.drawables.retain(|d| !MODELS.contains(&d.model_id.as_str()));

    for c in &eco.creatures {
        if !c.alive {
            continue;
        }
        if culling::dist_sq_xz(c.pos, viewer) > 200.0 * 200.0 {
            continue;
        }
        let scale = if c.kind == CreatureKind::Sheep && c.sheared {
            Vec3::new(1.0, 0.82, 1.0)
        } else if matches!(c.ai, CreatureAi::EatGrass { .. }) && c.kind.is_herdable() {
            Vec3::new(1.0, 0.88, 1.0)
        } else {
            Vec3::ONE
        };
        world.add_drawable(Drawable {
            model_id: c.kind.model_id().into(),
            position: c.pos,
            rotation: Quat::from_rotation_y(c.yaw),
            scale,
            material: DrawMaterial::Standard {
                roughness: 0.88,
                metallic: 0.0,
            },
            target_id: None,
        });
    }

    for a in &eco.aerials {
        if culling::dist_sq_xz(a.pos, viewer) > 350.0 * 350.0 {
            continue;
        }
        if a.kind == AerialKind::Mirage && a.alpha < 0.05 {
            continue;
        }
        let model = match a.kind {
            AerialKind::Ufo => "ufo",
            AerialKind::Mirage => "mirage",
        };
        world.add_drawable(Drawable {
            model_id: model.into(),
            position: a.pos,
            rotation: Quat::from_rotation_y(a.phase),
            scale: Vec3::splat(if a.kind == AerialKind::Mirage { 1.0 + a.alpha } else { 1.0 }),
            material: DrawMaterial::Standard {
                roughness: 0.5,
                metallic: if a.kind == AerialKind::Ufo { 0.8 } else { 0.0 },
            },
            target_id: None,
        });
    }
}
