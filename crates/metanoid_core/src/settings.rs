//! Game settings persisted to metanoid_settings.json.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ParticleQuality {
    Low,
    #[default]
    Medium,
    High,
}

impl ParticleQuality {
    pub fn label(self) -> &'static str {
        match self {
            ParticleQuality::Low => "Low",
            ParticleQuality::Medium => "Medium",
            ParticleQuality::High => "High",
        }
    }

    pub fn next(self) -> Self {
        match self {
            ParticleQuality::Low => ParticleQuality::Medium,
            ParticleQuality::Medium => ParticleQuality::High,
            ParticleQuality::High => ParticleQuality::Low,
        }
    }
}

#[derive(Serialize, Deserialize, Resource, Clone, Debug)]
pub struct GameSettings {
    pub show_fps: bool,
    pub master_volume: f32,
    pub sfx_volume: f32,
    pub music_volume: f32,
    /// Multiplier on bloom intensity (0..1.5).
    pub bloom_intensity: f32,
    /// Multiplier on camera trauma (0..1).
    pub shake_intensity: f32,
    pub particle_quality: ParticleQuality,
    pub reduce_motion: bool,
    pub high_contrast_bricks: bool,
    pub large_hud: bool,
}

impl Default for GameSettings {
    fn default() -> Self {
        Self {
            show_fps: true,
            master_volume: 0.8,
            sfx_volume: 1.0,
            music_volume: 0.5,
            bloom_intensity: 1.0,
            shake_intensity: 1.0,
            particle_quality: ParticleQuality::Medium,
            reduce_motion: false,
            high_contrast_bricks: false,
            large_hud: false,
        }
    }
}

impl GameSettings {
    /// Linear SFX gain 0.0–1.0 after master * sfx.
    pub fn effective_sfx(&self) -> f32 {
        (self.master_volume * self.sfx_volume).clamp(0.0, 1.0)
    }

    /// Linear music gain 0.0–1.0 after master * music.
    pub fn effective_music(&self) -> f32 {
        (self.master_volume * self.music_volume).clamp(0.0, 1.0)
    }

    /// Channel volumes in decibels for bevy_kira_audio `set_volume`.
    pub fn channel_volumes_db(&self) -> (f32, f32) {
        (
            linear_amplitude_to_db(self.effective_sfx()),
            linear_amplitude_to_db(self.effective_music()),
        )
    }

    pub fn trauma_scale(&self) -> f32 {
        if self.reduce_motion {
            0.0
        } else {
            self.shake_intensity.clamp(0.0, 1.0)
        }
    }

    pub fn bloom_scale(&self) -> f32 {
        self.bloom_intensity.clamp(0.0, 1.5)
    }
}

/// Convert linear amplitude 0.0–1.0 to kira/bevy_kira_audio decibels.
/// 1.0 → 0 dB (identity), 0.0 → silence (-60 dB).
pub fn linear_amplitude_to_db(linear: f32) -> f32 {
    let a = linear.clamp(0.0, 1.0);
    if a <= 0.0001 { -60.0 } else { 20.0 * a.log10() }
}

/// How a level was launched.
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LevelLaunchMode {
    #[default]
    Campaign,
    Challenge,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_are_sane() {
        let s = GameSettings::default();
        assert!(s.master_volume > 0.0);
        assert!(s.shake_intensity > 0.0);
        assert!(!s.reduce_motion);
    }

    #[test]
    fn reduce_motion_zeros_trauma() {
        let mut s = GameSettings::default();
        s.reduce_motion = true;
        assert_eq!(s.trauma_scale(), 0.0);
    }

    #[test]
    fn effective_volumes_clamp() {
        let mut s = GameSettings::default();
        s.master_volume = 2.0;
        s.sfx_volume = 2.0;
        assert!((s.effective_sfx() - 1.0).abs() < 1e-5);
    }

    #[test]
    fn linear_to_db_unity_and_silence() {
        assert!((linear_amplitude_to_db(1.0) - 0.0).abs() < 1e-4);
        assert!(linear_amplitude_to_db(0.0) <= -60.0);
        // 0.5 amplitude ≈ -6.02 dB
        let half = linear_amplitude_to_db(0.5);
        assert!((half - (-6.0206)).abs() < 0.05, "half db={half}");
    }

    #[test]
    fn channel_volumes_db_use_effective_gains() {
        let mut s = GameSettings::default();
        s.master_volume = 1.0;
        s.sfx_volume = 0.5;
        s.music_volume = 0.25;
        let (sfx_db, music_db) = s.channel_volumes_db();
        assert!((sfx_db - linear_amplitude_to_db(0.5)).abs() < 1e-4);
        assert!((music_db - linear_amplitude_to_db(0.25)).abs() < 1e-4);
        // Quieter music than sfx
        assert!(music_db < sfx_db);
    }

    #[test]
    fn master_zero_silences_both_channels() {
        let mut s = GameSettings::default();
        s.master_volume = 0.0;
        s.sfx_volume = 1.0;
        s.music_volume = 1.0;
        let (sfx_db, music_db) = s.channel_volumes_db();
        assert!(sfx_db <= -60.0);
        assert!(music_db <= -60.0);
    }
}
