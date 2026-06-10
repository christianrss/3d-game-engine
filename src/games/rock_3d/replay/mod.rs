//! Gravação e reprodução de arremessos.

use crate::math::Vec3;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplayFrame {
    pub time: f32,
    pub position: Vec3,
    pub velocity: Vec3,
    pub event: Option<ReplayEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReplayEvent {
    Throw,
    Bounce { position: Vec3 },
    Hit { target_id: u32, damage: f32 },
    Land,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ReplayRecording {
    pub frames: Vec<ReplayFrame>,
    pub total_score: u32,
    pub duration: f32,
}

impl ReplayRecording {
    pub fn start(&mut self) {
        self.frames.clear();
        self.total_score = 0;
        self.duration = 0.0;
    }

    pub fn record_frame(&mut self, time: f32, position: Vec3, velocity: Vec3, event: Option<ReplayEvent>) {
        self.duration = time;
        self.frames.push(ReplayFrame {
            time,
            position,
            velocity,
            event,
        });
    }

    pub fn frame_at(&self, time: f32) -> Option<&ReplayFrame> {
        self.frames.iter().rev().find(|f| f.time <= time)
    }
}

#[derive(Default)]
pub struct ReplayPlayer {
    pub recording: Option<ReplayRecording>,
    pub playback_time: f32,
    pub playing: bool,
    pub speed: f32,
}

impl ReplayPlayer {
    pub fn load(&mut self, recording: ReplayRecording) {
        self.recording = Some(recording);
        self.playback_time = 0.0;
        self.playing = false;
    }

    pub fn play(&mut self) {
        self.playing = true;
        self.playback_time = 0.0;
    }

    pub fn update(&mut self, dt: f32) -> Option<Vec3> {
        if !self.playing {
            return None;
        }
        let rec = self.recording.as_ref()?;
        self.playback_time += dt * self.speed;
        if self.playback_time >= rec.duration {
            self.playing = false;
        }
        rec.frame_at(self.playback_time).map(|f| f.position)
    }
}
