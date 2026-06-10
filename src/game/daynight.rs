//! Ciclo dia/noite — sol, lua, neblina e iluminacao dinamica.

use crate::graphics::Color;

const CYCLE_HOURS: f32 = 24.0;

#[derive(Debug, Clone)]
pub struct DayNightCycle {
    pub hour: f32,
    /// Horas de jogo por segundo real (24h = ~2 min reais com 0.2).
    pub speed: f32,
}

impl Default for DayNightCycle {
    fn default() -> Self {
        Self {
            hour: 8.5,
            speed: 0.2,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct DayNightLighting {
    pub sun_dir: [f32; 3],
    pub horizon: [f32; 3],
    pub zenith: [f32; 3],
    pub fog_color: [f32; 3],
    pub clear: Color,
    pub hour: f32,
    pub is_night: bool,
}

impl DayNightCycle {
    pub fn update(&mut self, dt: f32) {
        self.hour = (self.hour + self.speed * dt) % CYCLE_HOURS;
    }

    pub fn lighting(&self) -> DayNightLighting {
        let sun_elev = ((self.hour - 6.0) / 12.0 * std::f32::consts::PI).sin().max(0.0);
        let moon_elev = ((self.hour - 18.0) / 12.0 * std::f32::consts::PI).sin().max(0.0);
        let is_night = (sun_elev < 0.05 && self.hour > 18.0) || self.hour < 5.5;

        let sun_az = self.hour / CYCLE_HOURS * std::f32::consts::TAU;
        let sun_dir = if sun_elev > 0.01 {
            [
                sun_az.cos() * 0.65,
                sun_elev,
                -sun_az.sin() * 0.55,
            ]
        } else {
            let moon_az = sun_az + std::f32::consts::PI;
            [
                moon_az.cos() * 0.4,
                moon_elev.max(0.12),
                -moon_az.sin() * 0.35,
            ]
        };
        let len = (sun_dir[0] * sun_dir[0] + sun_dir[1] * sun_dir[1] + sun_dir[2] * sun_dir[2]).sqrt();
        let sun_dir = [sun_dir[0] / len, sun_dir[1] / len, sun_dir[2] / len];

        let day_mix = sun_elev.clamp(0.0, 1.0);
        let night_mix = 1.0 - day_mix;

        let day_horizon = [0.95, 0.68, 0.38];
        let day_zenith = [0.22, 0.48, 0.82];
        let night_horizon = [0.08, 0.1, 0.22];
        let night_zenith = [0.02, 0.04, 0.12];

        let horizon = lerp3(day_horizon, night_horizon, night_mix * 0.92);
        let zenith = lerp3(day_zenith, night_zenith, night_mix * 0.95);
        let fog_color = lerp3(day_horizon, night_horizon, night_mix * 0.85);
        let clear = Color::rgb(
            horizon[0] * 0.55 + zenith[0] * 0.45,
            horizon[1] * 0.55 + zenith[1] * 0.45,
            horizon[2] * 0.55 + zenith[2] * 0.45,
        );

        DayNightLighting {
            sun_dir,
            horizon,
            zenith,
            fog_color,
            clear,
            hour: self.hour,
            is_night: is_night || sun_elev < 0.08,
        }
    }
}

fn lerp3(a: [f32; 3], b: [f32; 3], t: f32) -> [f32; 3] {
    [
        a[0] + (b[0] - a[0]) * t,
        a[1] + (b[1] - a[1]) * t,
        a[2] + (b[2] - a[2]) * t,
    ]
}
