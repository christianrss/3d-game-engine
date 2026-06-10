//! Definições de mapas/ambientes.

use crate::games::rock_3d::weather::{WeatherKind, WeatherSystem};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MapKind {
    Quarry,
    Forest,
    Desert,
    Mountain,
    AbandonedCity,
    Futuristic,
}

impl MapKind {
    pub fn display_name(self) -> &'static str {
        match self {
            MapKind::Quarry => "Pedreira",
            MapKind::Forest => "Floresta",
            MapKind::Desert => "Deserto",
            MapKind::Mountain => "Montanha",
            MapKind::AbandonedCity => "Cidade Abandonada",
            MapKind::Futuristic => "Instalação Futurista",
        }
    }

    pub fn default_weather(self) -> WeatherKind {
        match self {
            MapKind::Quarry => WeatherKind::Windy,
            MapKind::Forest => WeatherKind::Fog,
            MapKind::Desert => WeatherKind::Storm,
            MapKind::Mountain => WeatherKind::Windy,
            MapKind::AbandonedCity => WeatherKind::Rain,
            MapKind::Futuristic => WeatherKind::Storm,
        }
    }

    pub fn ground_friction(self) -> f32 {
        match self {
            MapKind::Quarry => 0.6,
            MapKind::Forest => 0.5,
            MapKind::Desert => 0.35,
            MapKind::Mountain => 0.45,
            MapKind::AbandonedCity => 0.55,
            MapKind::Futuristic => 0.7,
        }
    }

    pub fn gravity_scale(self) -> f32 {
        match self {
            MapKind::Mountain => 0.98,
            _ => 1.0,
        }
    }

    pub fn apply_weather(&self, weather: &mut WeatherSystem) {
        weather.kind = self.default_weather();
        weather.temperature_c = match self {
            MapKind::Desert => 38.0,
            MapKind::Mountain => 5.0,
            MapKind::Futuristic => 18.0,
            _ => 22.0,
        };
    }
}

#[derive(Debug, Clone)]
pub struct MapConfig {
    pub kind: MapKind,
    pub spawn: crate::math::Vec3,
    pub world_half: f32,
}

impl MapConfig {
    pub fn quarry() -> Self {
        Self {
            kind: MapKind::Quarry,
            spawn: crate::math::Vec3::new(0.0, 1.7, 10.0),
            world_half: 256.0,
        }
    }
}
