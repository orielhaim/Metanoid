use bevy::prelude::*;

#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AppState {
    #[default]
    Menu,
    Loading,
    LevelSelect,
    Playing,
    Paused,
    LevelComplete,
    GameOver,
}
