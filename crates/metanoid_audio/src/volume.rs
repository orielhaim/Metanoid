//! Apply GameSettings volumes to kira audio channels.

use bevy::prelude::*;
use bevy_kira_audio::AudioControl;
use bevy_kira_audio::prelude::AudioChannel;
use metanoid_core::settings::GameSettings;

use crate::music::MusicState;
use crate::{MusicChannel, SfxChannel};

/// Last channel volumes (dB) applied via `set_volume` — for diagnostics/tests.
#[derive(Resource, Debug, Clone, Copy, Default, PartialEq)]
pub struct AppliedChannelVolumes {
    pub sfx_db: f32,
    pub music_db: f32,
    pub apply_count: u32,
}

/// Pure mapping from settings → channel dB (shipped path used by the apply system).
pub fn channel_db_from_settings(settings: &GameSettings) -> AppliedChannelVolumes {
    let (sfx_db, music_db) = settings.channel_volumes_db();
    AppliedChannelVolumes {
        sfx_db,
        music_db,
        apply_count: 0,
    }
}

/// Push effective volumes onto SFX and Music channels and sync MusicState.
pub fn apply_settings_volumes(
    settings: Res<GameSettings>,
    sfx: Res<AudioChannel<SfxChannel>>,
    music: Res<AudioChannel<MusicChannel>>,
    mut music_state: ResMut<MusicState>,
    mut applied: ResMut<AppliedChannelVolumes>,
) {
    if !settings.is_changed() && applied.apply_count > 0 {
        return;
    }

    let (sfx_db, music_db) = settings.channel_volumes_db();
    sfx.set_volume(sfx_db);
    music.set_volume(music_db);

    // Keep MusicState.volume as linear gain for play_music/crossfade helpers.
    music_state.volume = settings.effective_music();

    applied.sfx_db = sfx_db;
    applied.music_db = music_db;
    applied.apply_count = applied.apply_count.saturating_add(1);

    info!(
        "Audio volumes applied: sfx={sfx_db:.1} dB music={music_db:.1} dB (master={:.0}% sfx={:.0}% music={:.0}%)",
        settings.master_volume * 100.0,
        settings.sfx_volume * 100.0,
        settings.music_volume * 100.0,
    );
}

/// Force apply once at startup even if settings was not marked changed this frame.
pub fn apply_settings_volumes_startup(
    settings: Res<GameSettings>,
    sfx: Res<AudioChannel<SfxChannel>>,
    music: Res<AudioChannel<MusicChannel>>,
    mut music_state: ResMut<MusicState>,
    mut applied: ResMut<AppliedChannelVolumes>,
) {
    let (sfx_db, music_db) = settings.channel_volumes_db();
    sfx.set_volume(sfx_db);
    music.set_volume(music_db);
    music_state.volume = settings.effective_music();
    applied.sfx_db = sfx_db;
    applied.music_db = music_db;
    applied.apply_count = applied.apply_count.saturating_add(1);
    info!("Audio volumes at startup: sfx={sfx_db:.1} dB music={music_db:.1} dB");
}

#[cfg(test)]
mod tests {
    use super::*;
    use metanoid_core::settings::linear_amplitude_to_db;

    #[test]
    fn channel_db_from_settings_matches_core_helpers() {
        let mut s = GameSettings::default();
        s.master_volume = 0.8;
        s.sfx_volume = 1.0;
        s.music_volume = 0.5;
        let applied = channel_db_from_settings(&s);
        let expect_sfx = linear_amplitude_to_db(s.effective_sfx());
        let expect_music = linear_amplitude_to_db(s.effective_music());
        assert!((applied.sfx_db - expect_sfx).abs() < 1e-4);
        assert!((applied.music_db - expect_music).abs() < 1e-4);
        // effective music 0.4 < effective sfx 0.8 → quieter music channel
        assert!(applied.music_db < applied.sfx_db);
    }

    #[test]
    fn mute_master_is_silence_db() {
        let mut s = GameSettings::default();
        s.master_volume = 0.0;
        let applied = channel_db_from_settings(&s);
        assert!(applied.sfx_db <= -60.0);
        assert!(applied.music_db <= -60.0);
    }
}
