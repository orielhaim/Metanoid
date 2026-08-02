//! Persist career save (schema v2).

use bevy::prelude::*;

pub use metanoid_core::save_data::SaveData;

fn save_path() -> std::path::PathBuf {
    std::path::PathBuf::from("metanoid_save.json")
}

pub fn load_save() -> SaveData {
    let path = save_path();
    match std::fs::read_to_string(&path) {
        Ok(json) => serde_json::from_str(&json).unwrap_or_else(|_| {
            info!("Save parse failed or old format — starting fresh v2 save");
            SaveData::default()
        }),
        Err(_) => SaveData::default(),
    }
}

pub fn save_game(data: &SaveData) {
    let path = save_path();
    if let Ok(json) = serde_json::to_string_pretty(data) {
        let _ = std::fs::write(path, json);
    }
}

pub fn now_unix() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}
