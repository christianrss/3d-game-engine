//! Pontuação do jogador.

#[derive(Debug, Default, Clone)]
pub struct Score {
    pub total: u32,
    pub hits: u32,
    pub misses: u32,
}

impl Score {
    pub fn register_hit(&mut self, points: u32) {
        self.total += points;
        self.hits += 1;
    }

    pub fn register_miss(&mut self) {
        self.misses += 1;
    }

    pub fn accuracy(&self) -> f32 {
        let total = self.hits + self.misses;
        if total == 0 {
            0.0
        } else {
            (self.hits as f32 / total as f32) * 100.0
        }
    }
}
