//! Metanoid UI plugin — menus, settings, galaxy map, HUD, level complete.

pub mod screens;
pub mod scroll;
pub mod theme;
pub mod ui_sfx;
pub mod widgets;

use bevy::prelude::*;
use metanoid_core::states::AppState;
use scroll::mouse_wheel_scroll;
use ui_sfx::button_ui_sfx;

use screens::galaxy_map::{
    GalaxyMapNeedsRebuild, GalaxyMapSelection, apply_galaxy_map_rebuild, galaxy_tab_interaction,
    level_node_interaction, map_back_interaction, play_campaign_interaction,
    play_challenge_interaction, request_rebuild_on_selection_change, setup_galaxy_map,
    sync_map_selection_from_save, teardown_galaxy_map,
};
use screens::hud::{setup_hud, teardown_hud, update_hud};
use screens::level_complete::{
    continue_next_interaction, map_from_complete_interaction, retry_interaction,
    setup_level_complete, teardown_level_complete,
};
use screens::menu::{
    continue_button_interaction, galaxy_map_button_interaction, settings_menu_button_interaction,
    setup_menu, teardown_menu,
};
use screens::settings::{
    rebuild_settings_on_change, settings_action_interaction, settings_back_interaction,
    setup_settings, teardown_settings,
};
use theme::UiTheme;

pub use screens::hud::HudRoot;
pub use screens::level_complete::LevelCompleteRoot;
pub use screens::menu::MenuRoot;
pub use theme::{UiTheme as UiThemeExport, grade_color, rating_color};

// Re-export selection type for game crate if needed
pub use screens::galaxy_map::GalaxyMapSelection as MapSelection;

pub struct MetanoidUiPlugin;

impl Plugin for MetanoidUiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<UiTheme>()
            .init_resource::<GalaxyMapSelection>()
            .init_resource::<GalaxyMapNeedsRebuild>()
            // Global mouse-wheel scroll for ScrollArea panels + UI SFX
            .add_systems(Update, (mouse_wheel_scroll, button_ui_sfx))
            // Menu
            .add_systems(OnEnter(AppState::Menu), setup_menu)
            .add_systems(OnExit(AppState::Menu), teardown_menu)
            .add_systems(
                Update,
                (
                    continue_button_interaction,
                    galaxy_map_button_interaction,
                    settings_menu_button_interaction,
                )
                    .run_if(in_state(AppState::Menu)),
            )
            // Settings
            .add_systems(OnEnter(AppState::Settings), setup_settings)
            .add_systems(OnExit(AppState::Settings), teardown_settings)
            .add_systems(
                Update,
                (
                    settings_back_interaction,
                    settings_action_interaction,
                    rebuild_settings_on_change,
                )
                    .run_if(in_state(AppState::Settings)),
            )
            // Galaxy map
            .add_systems(
                OnEnter(AppState::LevelSelect),
                (sync_map_selection_from_save, setup_galaxy_map).chain(),
            )
            .add_systems(OnExit(AppState::LevelSelect), teardown_galaxy_map)
            .add_systems(
                Update,
                (
                    galaxy_tab_interaction,
                    level_node_interaction,
                    map_back_interaction,
                    play_challenge_interaction,
                    play_campaign_interaction,
                    request_rebuild_on_selection_change,
                    apply_galaxy_map_rebuild,
                )
                    .run_if(in_state(AppState::LevelSelect)),
            )
            // HUD while playing
            .add_systems(OnEnter(AppState::Playing), setup_hud)
            .add_systems(OnExit(AppState::Playing), teardown_hud)
            .add_systems(Update, update_hud.run_if(in_state(AppState::Playing)))
            // Level complete
            .add_systems(OnEnter(AppState::LevelComplete), setup_level_complete)
            .add_systems(OnExit(AppState::LevelComplete), teardown_level_complete)
            .add_systems(
                Update,
                (
                    continue_next_interaction,
                    retry_interaction,
                    map_from_complete_interaction,
                )
                    .run_if(in_state(AppState::LevelComplete)),
            );
    }
}
