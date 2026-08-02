//! Settings load/save and apply hooks.
//!
//! Volume **application** to audio channels lives in `metanoid_audio::volume`
//! (`apply_settings_volumes` / `apply_settings_volumes_startup`), which calls
//! `GameSettings::channel_volumes_db` → `AudioChannel::set_volume`.

use bevy::prelude::*;

pub use metanoid_core::settings::GameSettings;

fn settings_path() -> std::path::PathBuf {
    std::path::PathBuf::from("metanoid_settings.json")
}

pub fn load_settings() -> GameSettings {
    let path = settings_path();
    match std::fs::read_to_string(&path) {
        Ok(json) => serde_json::from_str(&json).unwrap_or_default(),
        Err(_) => GameSettings::default(),
    }
}

pub fn save_settings(settings: &GameSettings) {
    let path = settings_path();
    if let Ok(json) = serde_json::to_string_pretty(settings) {
        let _ = std::fs::write(path, json);
    }
}

/// Persist settings whenever they change.
/// Audio channel volumes are applied separately by `metanoid_audio::apply_settings_volumes`.
pub fn persist_settings_on_change(settings: Res<GameSettings>) {
    if settings.is_changed() {
        save_settings(&settings);
    }
}
