//! Sistema climático que afeta a física.

use crate::math::Vec3;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WeatherKind {
    Clear,
    Windy,
    Rain,
    Fog,
    Storm,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeatherSystem {
    pub kind: WeatherKind,
    pub wind: Vec3,
    pub wind_target: Vec3,
    pub temperature_c: f32,
    pub fog_density: f32,
    pub time: f32,
}

impl Default for WeatherSystem {
    fn default() -> Self {
        Self {
            kind: WeatherKind::Windy,
            wind: Vec3::new(2.0, 0.0, -1.5),
            wind_target: Vec3::new(3.0, 0.0, -2.0),
            temperature_c: 22.0,
            fog_density: 0.0,
            time: 0.0,
        }
    }
}

impl WeatherSystem {
    pub fn update(&mut self, dt: f32) {
        self.time += dt;

        // Vento dinâmico — interpola para alvo que muda
        if (self.time % 8.0) < dt {
            self.wind_target = Vec3::new(
                (self.time * 0.7).sin() * 6.0,
                0.0,
                (self.time * 0.5).cos() * 4.0,
            );
        }
        self.wind = self.wind.lerp(self.wind_target, dt * 0.3);

        match self.kind {
            WeatherKind::Storm => {
                self.wind *= 1.0 + (self.time * 3.0).sin() * 0.3;
            }
            WeatherKind::Fog => {
                self.fog_density = 0.4;
            }
            WeatherKind::Rain => {
                self.fog_density = 0.15;
            }
            _ => {
                self.fog_density = 0.0;
            }
        }
    }

    /// Densidade do ar ajustada por temperatura (kg/m³).
    pub fn air_density(&self) -> f32 {
        let base = 1.225;
        let temp_factor = 1.0 - (self.temperature_c - 15.0) * 0.004;
        let rain_factor = if self.kind == WeatherKind::Rain { 1.05 } else { 1.0 };
        base * temp_factor * rain_factor
    }

    /// Multiplicador de arrasto.
    pub fn drag_multiplier(&self) -> f32 {
        match self.kind {
            WeatherKind::Rain => 1.2,
            WeatherKind::Storm => 1.35,
            WeatherKind::Fog => 1.05,
            _ => 1.0,
        }
    }

    /// Multiplicador de gravidade (altitude/temperatura).
    pub fn gravity_scale(&self) -> f32 {
        1.0 - (self.temperature_c - 20.0) * 0.0005
    }

    pub fn wind_strength(&self) -> f32 {
        self.wind.length()
    }

    pub fn wind_direction_label(&self) -> &'static str {
        let angle = self.wind.x.atan2(self.wind.z).to_degrees();
        match angle as i32 {
            a if a >= -22 && a < 22 => "N",
            a if a >= 22 && a < 67 => "NE",
            a if a >= 67 && a < 112 => "E",
            a if a >= 112 && a < 157 => "SE",
            a if a >= 157 || a < -157 => "S",
            a if a >= -157 && a < -112 => "SW",
            a if a >= -112 && a < -67 => "W",
            _ => "NW",
        }
    }
}
