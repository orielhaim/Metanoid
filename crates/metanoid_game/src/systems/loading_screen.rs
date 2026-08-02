//! Loading screen: a biome-specific curtain that covers the screen and parts to
//! reveal the assembled stage (forest trunks, crystal shards, lava columns, ...).

use bevy::prelude::*;
use metanoid_core::states::AppState;
use metanoid_visuals::transition::{Curtain, spawn_curtain};

use super::arena::GameCamera;
use super::level_progression::ActiveLevelVisuals;

#[derive(Component)]
pub struct LoadingScreen;

#[derive(Component)]
pub struct LevelBanner;

/// Build the curtain + level banner from the prepared recipe.
pub fn setup_loading_screen(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    visuals: Option<Res<ActiveLevelVisuals>>,
    game_state: Option<Res<metanoid_core::resources::game_state::GameState>>,
) {
    let Some(visuals) = visuals else {
        // No recipe yet; fall back to a plain overlay so the game still works.
        commands.spawn((
            LoadingScreen,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                ..default()
            },
            BackgroundColor(Color::srgb(0.02, 0.02, 0.05)),
        ));
        return;
    };

    let root = spawn_curtain(&mut commands, &mut meshes, &mut materials, &visuals.recipe);
    commands.entity(root).insert(LoadingScreen);

    // Level intro banner (UI).
    let accent: Srgba = Srgba::from(visuals.recipe.palette.accent);
    let text_color = Color::srgb(accent.red, accent.green, accent.blue);
    commands
        .spawn((
            LevelBanner,
            LoadingScreen,
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                right: Val::Px(0.0),
                top: Val::Percent(22.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: Val::Px(8.0),
                ..default()
            },
            ZIndex(20),
        ))
        .with_children(|banner| {
            let biome_no = game_state.as_ref().map(|s| s.biome + 1).unwrap_or(1);
            banner.spawn((
                Text::new(format!("BIOME {biome_no}")),
                TextFont {
                    font_size: FontSize::Px(18.0),
                    ..default()
                },
                TextColor(text_color),
            ));
            banner.spawn((
                Text::new(visuals.recipe.name.clone()),
                TextFont {
                    font_size: FontSize::Px(44.0),
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
        });
}

/// Once the stage is assembled (no pending level), start the reveal.
pub fn start_curtain(
    mut curtains: Query<&mut Curtain>,
    pending: Option<Res<super::level_spawner::PendingLevel>>,
) {
    if pending.is_some() {
        return;
    }
    if let Ok(mut curtain) = curtains.single_mut() {
        if !curtain.started {
            curtain.started = true;
        }
    }
}

/// Cinematic zoom-out while the curtain parts: camera starts slightly zoomed in
/// and settles to 1.0 as the stage is revealed.
pub fn tick_reveal_zoom(
    mut cameras: Query<&mut Projection, With<GameCamera>>,
    curtains: Query<&Curtain>,
) {
    let Ok(curtain) = curtains.single() else {
        return;
    };
    if !curtain.started {
        return;
    }
    let p = curtain.progress();
    let ease = 1.0 - (1.0 - p).powi(3);
    let scale = 1.0 + 0.08 * (1.0 - ease);
    let Ok(mut projection) = cameras.single_mut() else {
        return;
    };
    if let Projection::Orthographic(ortho) = &mut *projection {
        ortho.scale = scale;
    }
}

/// When the reveal finishes, tear down the curtain + banner and play.
pub fn finish_loading(
    mut commands: Commands,
    curtains: Query<(Entity, &Curtain)>,
    banners: Query<Entity, With<LevelBanner>>,
    pending: Option<Res<super::level_spawner::PendingLevel>>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if pending.is_some() {
        return;
    }
    let Ok((entity, curtain)) = curtains.single() else {
        // No curtain (e.g. reduced-motion path already cleaned up): play.
        next_state.set(AppState::Playing);
        return;
    };
    if curtain.finished() {
        for b in &banners {
            commands.entity(b).try_despawn();
        }
        commands.entity(entity).despawn();
        next_state.set(AppState::Playing);
    }
}
