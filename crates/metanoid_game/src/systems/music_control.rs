//! Background music selection: menu track + biome/boss tracks in-game.

use bevy::prelude::*;
use bevy_kira_audio::prelude::AudioChannel;
use metanoid_audio::{
    MusicAssets, MusicChannel, MusicState, MusicTrack, crossfade_to, play_music, track_for_biome,
};
use metanoid_core::resources::game_state::GameState;
use metanoid_core::settings::GameSettings;
use metanoid_procgen::biome::generator::BiomeGenerator;
use metanoid_procgen::seed::hierarchy::MasterSeed;
use metanoid_procgen::universe::progression::LEVELS_PER_BIOME;

fn music_volume(settings: &GameSettings) -> f32 {
    // Channel volume already applies master*music as dB.
    // Per-instance volume is relative dB; 0.0 = identity.
    let _ = settings;
    0.0
}

pub fn start_menu_music(
    music_assets: Res<MusicAssets>,
    audio: Res<AudioChannel<MusicChannel>>,
    mut music_state: ResMut<MusicState>,
    settings: Res<GameSettings>,
) {
    if music_state.current == MusicTrack::Menu {
        return;
    }
    let vol = music_volume(&settings);
    play_music(
        MusicTrack::Menu,
        &music_assets,
        &audio,
        vol,
        &mut music_state,
    );
    music_state.current = MusicTrack::Menu;
    info!("Music: menu");
}

pub fn start_level_music(
    music_assets: Res<MusicAssets>,
    audio: Res<AudioChannel<MusicChannel>>,
    mut music_state: ResMut<MusicState>,
    settings: Res<GameSettings>,
    game_state: Option<Res<GameState>>,
) {
    let Some(state) = game_state else {
        return;
    };

    let track = if state.is_boss(LEVELS_PER_BIOME) {
        MusicTrack::Boss
    } else {
        let master = MasterSeed::new(state.master_seed);
        let params = BiomeGenerator::generate(master.galaxy(state.galaxy).biome(state.biome));
        track_for_biome(params.temperature, params.weirdness)
    };

    if music_state.current == track {
        return;
    }

    let vol = music_volume(&settings);
    crossfade_to(track, &music_assets, &audio, vol, &mut music_state);
    music_state.current = track;
    info!(
        "Music: {:?} (G{} B{} L{})",
        track, state.galaxy, state.biome, state.level
    );
}

pub fn stop_music_on_game_over(
    music_assets: Res<MusicAssets>,
    audio: Res<AudioChannel<MusicChannel>>,
    mut music_state: ResMut<MusicState>,
    settings: Res<GameSettings>,
) {
    // Soft return toward menu bed after a short beat; keep playing menu.
    let vol = music_volume(&settings);
    crossfade_to(
        MusicTrack::Menu,
        &music_assets,
        &audio,
        vol,
        &mut music_state,
    );
    music_state.current = MusicTrack::Menu;
}
