use bevy::prelude::*;

#[derive(Resource)]
pub struct GameState {
    pub master_seed: u64,
    pub galaxy: u64,
    pub biome: u64,
    pub level: u64,
    pub biome_count: usize,
    pub lives: i32,
    pub score: u64,
    pub level_clearing: bool,
}

impl GameState {
    pub fn new(master_seed: u64) -> Self {
        Self {
            master_seed,
            galaxy: 0,
            biome: 0,
            level: 0,
            biome_count: 4,
            lives: 3,
            score: 0,
            level_clearing: false,
        }
    }

    pub fn is_boss(&self, levels_per_biome: u64) -> bool {
        self.level == levels_per_biome - 1
    }
}
