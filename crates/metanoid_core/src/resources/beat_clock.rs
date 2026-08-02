//! Shared real-time "beat clock" that drives music-reactive visuals.
//!
//! Populated by the audio crate (which owns the kira playback position) and
//! consumed by the visuals crate to pulse bloom, vignette and backgrounds.

use bevy::prelude::*;

/// A lightweight, allocation-free beat/energy clock.
///
/// `beat_phase` is `0.0..1.0` across the current beat, `energy` is a smoothed
/// loudness estimate in `0.0..1.0`, and `pulse` is a short 1.0->0.0 envelope
/// that spikes on each detected beat.
#[derive(Resource, Debug, Clone, Copy)]
pub struct BeatClock {
    pub beat_phase: f32,
    pub energy: f32,
    pub pulse: f32,
    pub bpm: f32,
}

impl Default for BeatClock {
    fn default() -> Self {
        Self {
            beat_phase: 0.0,
            energy: 0.0,
            pulse: 0.0,
            bpm: 118.0,
        }
    }
}
