use bevy::prelude::*;
use bevy_kira_audio::prelude::AudioChannel;
use bevy_kira_audio::{AudioControl, AudioInstance, AudioTween};

use super::MusicChannel;

#[derive(Resource)]
pub struct MusicAssets {
    pub menu: Handle<bevy_kira_audio::AudioSource>,
    pub neon: Handle<bevy_kira_audio::AudioSource>,
    pub ocean: Handle<bevy_kira_audio::AudioSource>,
    pub volcanic: Handle<bevy_kira_audio::AudioSource>,
    pub crystal: Handle<bevy_kira_audio::AudioSource>,
    pub space: Handle<bevy_kira_audio::AudioSource>,
    pub boss: Handle<bevy_kira_audio::AudioSource>,
}

impl FromWorld for MusicAssets {
    fn from_world(world: &mut World) -> Self {
        let a = world.resource::<AssetServer>();
        Self {
            menu: a.load("audio/music/music_menu.ogg"),
            neon: a.load("audio/music/music_neon.ogg"),
            ocean: a.load("audio/music/music_ocean.ogg"),
            volcanic: a.load("audio/music/music_volcanic.ogg"),
            crystal: a.load("audio/music/music_crystal.ogg"),
            space: a.load("audio/music/music_space.ogg"),
            boss: a.load("audio/music/music_boss.ogg"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MusicTrack {
    #[default]
    None,
    Menu,
    Neon,
    Ocean,
    Volcanic,
    Crystal,
    Space,
    Boss,
}

#[derive(Resource)]
pub struct MusicState {
    pub current: MusicTrack,
    pub volume: f32,
    /// Instance handle of the currently playing bed, for beat tracking.
    pub instance: Option<Handle<AudioInstance>>,
}

impl Default for MusicState {
    fn default() -> Self {
        Self {
            current: MusicTrack::None,
            volume: 0.3,
            instance: None,
        }
    }
}

pub fn play_music(
    track: MusicTrack,
    music_assets: &MusicAssets,
    audio: &AudioChannel<MusicChannel>,
    volume: f32,
    state: &mut MusicState,
) {
    let handle = match track {
        MusicTrack::None => {
            state.instance = None;
            return;
        }
        MusicTrack::Menu => music_assets.menu.clone(),
        MusicTrack::Neon => music_assets.neon.clone(),
        MusicTrack::Ocean => music_assets.ocean.clone(),
        MusicTrack::Volcanic => music_assets.volcanic.clone(),
        MusicTrack::Crystal => music_assets.crystal.clone(),
        MusicTrack::Space => music_assets.space.clone(),
        MusicTrack::Boss => music_assets.boss.clone(),
    };

    // Channel gain is set from GameSettings; instance volume 0 dB = identity.
    // Always loop background beds.
    let _ = volume;
    audio.stop();
    let mut command = audio.play(handle);
    state.instance = Some(command.handle());
    command.with_volume(0.0).looped();
}

pub fn crossfade_to(
    new_track: MusicTrack,
    music_assets: &MusicAssets,
    audio: &AudioChannel<MusicChannel>,
    volume: f32,
    state: &mut MusicState,
) {
    let handle = match new_track {
        MusicTrack::None => {
            state.instance = None;
            audio
                .stop()
                .fade_out(AudioTween::linear(std::time::Duration::from_millis(500)));
            return;
        }
        MusicTrack::Menu => music_assets.menu.clone(),
        MusicTrack::Neon => music_assets.neon.clone(),
        MusicTrack::Ocean => music_assets.ocean.clone(),
        MusicTrack::Volcanic => music_assets.volcanic.clone(),
        MusicTrack::Crystal => music_assets.crystal.clone(),
        MusicTrack::Space => music_assets.space.clone(),
        MusicTrack::Boss => music_assets.boss.clone(),
    };

    let _ = volume;
    audio
        .stop()
        .fade_out(AudioTween::linear(std::time::Duration::from_millis(500)));
    let mut command = audio.play(handle);
    state.instance = Some(command.handle());
    command
        .with_volume(0.0)
        .fade_in(AudioTween::linear(std::time::Duration::from_millis(500)))
        .looped();
}

pub fn track_for_biome(temperature: f32, weirdness: f32) -> MusicTrack {
    if temperature > 0.7 {
        MusicTrack::Volcanic
    } else if temperature < 0.2 {
        MusicTrack::Crystal
    } else if weirdness > 0.6 {
        MusicTrack::Space
    } else if temperature > 0.4 {
        MusicTrack::Neon
    } else {
        MusicTrack::Ocean
    }
}
