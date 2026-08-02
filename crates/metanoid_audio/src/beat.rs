//! Music-reactive beat clock: derives a beat phase from the actual playback
//! position of the music bed, so visuals can pulse on the real beat.

use bevy::prelude::*;
use bevy_kira_audio::prelude::AudioChannel;
use bevy_kira_audio::{AudioControl, PlaybackState};
use metanoid_core::resources::beat_clock::BeatClock;

use super::MusicChannel;
use super::music::MusicState;

pub fn tick_beat_clock(
    channel: Res<AudioChannel<MusicChannel>>,
    music_state: Res<MusicState>,
    mut clock: ResMut<BeatClock>,
) {
    clock.pulse = (clock.pulse * 0.93).max(0.0);
    clock.energy = (clock.energy * 0.99).max(0.0);

    let Some(instance) = music_state.instance.as_ref() else {
        clock.beat_phase = 0.0;
        clock.energy = 0.0;
        return;
    };

    match channel.state(instance) {
        PlaybackState::Playing { position } => {
            let beat_len = 60.0 / clock.bpm.max(1.0);
            let phase = (position / beat_len as f64).fract() as f32;
            let last = clock.beat_phase;
            clock.beat_phase = phase;
            if phase < last && phase < 0.6 {
                clock.pulse = 1.0;
            }
            clock.energy = (clock.energy * 0.85 + 0.4).min(1.0);
        }
        _ => {
            clock.beat_phase = 0.0;
            clock.energy = 0.0;
        }
    }
}
