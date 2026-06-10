//! Multiplayer local por turnos.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalPlayer {
    pub name: String,
    pub score: u32,
    pub throws_left: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalMultiplayer {
    pub players: Vec<LocalPlayer>,
    pub current_player: usize,
    pub round: u32,
}

impl LocalMultiplayer {
    pub fn new(names: &[&str], throws_per_round: u32) -> Self {
        Self {
            players: names
                .iter()
                .map(|n| LocalPlayer {
                    name: n.to_string(),
                    score: 0,
                    throws_left: throws_per_round,
                })
                .collect(),
            current_player: 0,
            round: 1,
        }
    }

    pub fn next_turn(&mut self) {
        self.current_player = (self.current_player + 1) % self.players.len();
        if self.current_player == 0 {
            self.round += 1;
            for p in &mut self.players {
                p.throws_left = 8;
            }
        }
    }

    pub fn winner(&self) -> Option<&LocalPlayer> {
        if self.round <= 1 {
            return None;
        }
        self.players.iter().max_by_key(|p| p.score)
    }
}
