//! # SceneBuilder — Construtor de Cenas
//!
//! Padrão Builder para montar mapa + alvos de forma declarativa.

use crate::assets::sample_desert_height;
use crate::game::desert::build_desert;
use crate::game::physics::CollisionWorld;
use crate::game::player::Player;
use crate::game::world::GameWorld;
use crate::math::Vec3;

/// Constrói a cena do jogo.
#[derive(Debug, Clone)]
pub struct SceneBuilder {
    pub use_desert: bool,
    pub ground_size: f32,
    pub targets: Vec<(Vec3, u32, f32)>,
    pub player_spawn: Vec3,
}

impl SceneBuilder {
    pub fn new() -> Self {
        Self {
            use_desert: false,
            ground_size: 200.0,
            targets: Vec::new(),
            player_spawn: Vec3::new(-30.0, 1.7, -35.0),
        }
    }

    pub fn with_desert_map(mut self) -> Self {
        self.use_desert = true;
        self
    }

    pub fn add_target(mut self, position: Vec3) -> Self {
        self.targets.push((position, 100, 1.0));
        self
    }

    pub fn add_target_at(mut self, x: f32, y: f32, z: f32, points: u32) -> Self {
        self.targets.push((Vec3::new(x, y, z), points, 1.0));
        self
    }

    pub fn with_player_spawn(mut self, pos: Vec3) -> Self {
        self.player_spawn = pos;
        self
    }

    /// Constrói o mundo, jogador e colisores estaticos.
    pub fn build(self) -> (GameWorld, Player, CollisionWorld) {
        let mut world = GameWorld::default();
        let mut collision = CollisionWorld::default();

        if self.use_desert {
            collision = build_desert(&mut world);
        }

        for (pos, points, scale) in self.targets {
            let ground = sample_desert_height(pos.x, pos.z);
            world.add_target(Vec3::new(pos.x, ground, pos.z), points, scale);
        }

        let mut player = Player::default();
        let spawn_y = sample_desert_height(self.player_spawn.x, self.player_spawn.z);
        player.position = Vec3::new(
            self.player_spawn.x,
            spawn_y + 1.7,
            self.player_spawn.z,
        );

        (world, player, collision)
    }
}

impl Default for SceneBuilder {
    fn default() -> Self {
        Self::new()
    }
}
