//! HUD específico do Rock 3D.

use crate::games::rock_3d::modes::GameMode;
use crate::games::rock_3d::stones::StoneKind;
use crate::games::rock_3d::throw::ThrowPhase;

#[derive(Debug, Clone, Default)]
pub struct RockHud {
    pub stone_name: String,
    pub force_pct: f32,
    pub yaw_deg: f32,
    pub pitch_deg: f32,
    pub spin_lateral: f32,
    pub spin_top: f32,
    pub wind_speed: f32,
    pub wind_dir: String,
    pub distance_m: f32,
    pub score: u32,
    pub combo: u32,
    pub xp: u64,
    pub level: u32,
    pub throws_left: u32,
    pub time_remaining: f32,
    pub phase: String,
    pub mode: String,
    pub show_trajectory: bool,
}

impl RockHud {
    pub fn update(
        &mut self,
        stone: StoneKind,
        force_pct: f32,
        yaw: f32,
        pitch: f32,
        spin_l: f32,
        spin_t: f32,
        wind_speed: f32,
        wind_dir: &str,
        distance: f32,
        score: u32,
        combo: u32,
        xp: u64,
        level: u32,
        throws_left: u32,
        time_remaining: f32,
        phase: ThrowPhase,
        mode: GameMode,
        show_trajectory: bool,
    ) {
        self.stone_name = stone.display_name().to_string();
        self.force_pct = force_pct;
        self.yaw_deg = yaw;
        self.pitch_deg = pitch;
        self.spin_lateral = spin_l;
        self.spin_top = spin_t;
        self.wind_speed = wind_speed;
        self.wind_dir = wind_dir.to_string();
        self.distance_m = distance;
        self.score = score;
        self.combo = combo;
        self.xp = xp;
        self.level = level;
        self.throws_left = throws_left;
        self.time_remaining = time_remaining;
        self.phase = format!("{phase:?}");
        self.mode = mode.display_name().to_string();
        self.show_trajectory = show_trajectory;
    }

    /// Linhas para overlay de debug/log.
    pub fn status_lines(&self) -> Vec<String> {
        vec![
            format!("[{}] {} | Lv.{} XP:{}", self.mode, self.stone_name, self.level, self.xp),
            format!("Score: {} | Combo: x{}", self.score, self.combo.max(1)),
            format!(
                "Força: {:.0}% | Mira V:{:.0}° H:{:.0}° | Spin L:{:.1} T:{:.1}",
                self.force_pct, self.pitch_deg, self.yaw_deg, self.spin_lateral, self.spin_top
            ),
            format!(
                "Vento: {:.1} m/s {} | Dist: {:.0}m | Arremessos: {}",
                self.wind_speed, self.wind_dir, self.distance_m, self.throws_left
            ),
            format!("Fase: {} | Tempo: {:.0}s", self.phase, self.time_remaining),
        ]
    }
}
