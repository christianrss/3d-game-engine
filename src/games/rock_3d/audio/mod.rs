//! Áudio específico do Rock 3D (eventos de arremesso).

use crate::audio::AudioEngine;

pub struct RockAudio;

impl RockAudio {
    pub fn on_throw(audio: &AudioEngine, speed: f32) {
        let _ = speed;
        audio.play_gunshot();
    }

    pub fn on_hit(audio: &AudioEngine) {
        audio.play_hit();
    }

    pub fn on_bounce(audio: &AudioEngine) {
        audio.play_hit();
    }

    pub fn on_empty(audio: &AudioEngine) {
        audio.play_empty();
    }
}
