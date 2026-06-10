//! Sistema de áudio — tiros, impactos, vento do deserto.

use rodio::{OutputStream, OutputStreamHandle, Sink, Source};
use std::f32::consts::PI;

pub struct AudioEngine {
    _stream: OutputStream,
    handle: OutputStreamHandle,
}

impl AudioEngine {
    pub fn new() -> Result<Self, String> {
        let (stream, handle) = OutputStream::try_default().map_err(|e| e.to_string())?;
        Ok(Self {
            _stream: stream,
            handle,
        })
    }

    pub fn play_gunshot(&self) {
        self.play_buffer(synth_gunshot(), 44100);
    }

    pub fn play_hit(&self) {
        self.play_buffer(synth_hit(), 44100);
    }

    pub fn play_empty(&self) {
        self.play_buffer(synth_click(), 44100);
    }

    pub fn play_wind_ambient(&self) {
        self.play_buffer(synth_wind(2.5), 44100);
    }

    fn play_buffer(&self, samples: Vec<f32>, sample_rate: u32) {
        let src = SamplesSource { samples, sample_rate, pos: 0 };
        if let Ok(sink) = Sink::try_new(&self.handle) {
            sink.append(src);
            sink.detach();
        }
    }
}

struct SamplesSource {
    samples: Vec<f32>,
    sample_rate: u32,
    pos: usize,
}

impl Iterator for SamplesSource {
    type Item = f32;
    fn next(&mut self) -> Option<f32> {
        let s = *self.samples.get(self.pos)?;
        self.pos += 1;
        Some(s)
    }
}

impl Source for SamplesSource {
    fn current_frame_len(&self) -> Option<usize> {
        None
    }
    fn channels(&self) -> u16 {
        1
    }
    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }
    fn total_duration(&self) -> Option<std::time::Duration> {
        Some(std::time::Duration::from_secs_f32(
            self.samples.len() as f32 / self.sample_rate as f32,
        ))
    }
}

fn synth_gunshot() -> Vec<f32> {
    let sr = 44100usize;
    let len = (sr as f32 * 0.18) as usize;
    let mut out = Vec::with_capacity(len);
    let mut rng = 12345u32;
    for i in 0..len {
        rng = rng.wrapping_mul(1664525).wrapping_add(1013904223);
        let noise = (rng as f32 / u32::MAX as f32) * 2.0 - 1.0;
        let t = i as f32 / len as f32;
        let env = (1.0 - t).powf(2.5);
        let crack = (t * 40.0 * PI * 2.0).sin() * (-t * 35.0).exp();
        out.push((noise * 0.55 + crack * 0.45) * env * 0.85);
    }
    out
}

fn synth_hit() -> Vec<f32> {
    let sr = 44100usize;
    let len = (sr as f32 * 0.12) as usize;
    (0..len)
        .map(|i| {
            let t = i as f32 / len as f32;
            let ping = (t * 1200.0 * PI * 2.0 / sr as f32).sin();
            ping * (1.0 - t).powf(1.2) * 0.5
        })
        .collect()
}

fn synth_click() -> Vec<f32> {
    (0..800)
        .map(|i| {
            let t = i as f32 / 800.0;
            (t * 200.0 * PI * 2.0 / 44100.0).sin() * (1.0 - t) * 0.2
        })
        .collect()
}

fn synth_wind(seconds: f32) -> Vec<f32> {
    let len = (44100.0 * seconds) as usize;
    let mut rng = 99u32;
    (0..len)
        .map(|i| {
            rng = rng.wrapping_mul(1103515245).wrapping_add(12345);
            let n = (rng as f32 / u32::MAX as f32) * 2.0 - 1.0;
            let t = i as f32 / len as f32;
            n * 0.04 * (0.5 + 0.5 * (t * 3.0 * PI).sin())
        })
        .collect()
}
