use bevy::prelude::*;
use metanoid_core::resources::game_state::GameState;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Resource, Clone, Debug)]
pub struct SaveData {
    pub master_seed: u64,
    pub highest_galaxy: u64,
    pub highest_biome: u64,
    pub highest_level: u64,
    pub high_score: u64,
    pub total_bricks_destroyed: u64,
}

impl Default for SaveData {
    fn default() -> Self {
        Self {
            master_seed: 42,
            highest_galaxy: 0,
            highest_biome: 0,
            highest_level: 0,
            high_score: 0,
            total_bricks_destroyed: 0,
        }
    }
}

fn save_path() -> std::path::PathBuf {
    std::path::PathBuf::from("metanoid_save.json")
}

pub fn load_save() -> SaveData {
    let path = save_path();
    match std::fs::read_to_string(&path) {
        Ok(json) => serde_json::from_str(&json).unwrap_or_default(),
        Err(_) => SaveData::default(),
    }
}

pub fn save_game(data: &SaveData) {
    let path = save_path();
    if let Ok(json) = serde_json::to_string_pretty(data) {
        let _ = std::fs::write(path, json);
    }
}

pub fn update_save_on_level_clear(
    game_state: Res<GameState>,
    mut save: ResMut<SaveData>,
) {
    let gal = game_state.galaxy;
    let bio = game_state.biome;
    let lvl = game_state.level;

    let advanced = gal > save.highest_galaxy
        || (gal == save.highest_galaxy && bio > save.highest_biome)
        || (gal == save.highest_galaxy && bio == save.highest_biome && lvl > save.highest_level);

    if advanced {
        save.highest_galaxy = gal;
        save.highest_biome = bio;
        save.highest_level = lvl;
        save_game(&save);
    }

    if game_state.score > save.high_score {
        save.high_score = game_state.score;
        save_game(&save);
    }
}
