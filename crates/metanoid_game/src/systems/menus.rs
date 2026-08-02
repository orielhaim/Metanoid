//! Pause and game-over overlays (main menu lives in metanoid_ui).

use bevy::prelude::*;
use metanoid_core::resources::game_state::GameState;
use metanoid_core::save_data::SaveData;
use metanoid_core::states::AppState;

use super::arena::ArenaEntity;
use super::level_spawner::LevelEntity;
use super::save::save_game;

#[derive(Component)]
pub struct GameOverRoot;

#[derive(Component)]
pub struct PauseRoot;

#[derive(Component)]
pub struct GameOverButton;

#[derive(Component)]
pub struct ResumeButton;

#[derive(Component)]
pub struct PauseMenuButton;

#[derive(Component)]
pub struct GameOverMapButton;

pub fn setup_game_over(
    mut commands: Commands,
    game_state: Option<Res<GameState>>,
    mut save: ResMut<SaveData>,
) {
    let score = game_state.map(|s| s.score).unwrap_or(0);
    if score > save.career_high_score {
        save.career_high_score = score;
        save_game(&save);
    }

    commands
        .spawn((
            GameOverRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                row_gap: Val::Px(20.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.85)),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("GAME OVER"),
                TextFont {
                    font_size: FontSize::Px(64.0),
                    ..default()
                },
                TextColor(Color::srgb(1.0, 0.3, 0.3)),
            ));
            parent.spawn((
                Text::new(format!("Run Score: {score}")),
                TextFont {
                    font_size: FontSize::Px(28.0),
                    ..default()
                },
                TextColor(Color::srgb(1.0, 1.0, 1.0)),
            ));
            parent.spawn((
                Text::new(format!("Career Best: {}", save.career_high_score)),
                TextFont {
                    font_size: FontSize::Px(18.0),
                    ..default()
                },
                TextColor(Color::srgb(0.6, 0.7, 0.8)),
            ));

            parent
                .spawn((
                    GameOverButton,
                    Button,
                    Node {
                        width: Val::Px(200.0),
                        height: Val::Px(48.0),
                        border: UiRect::all(Val::Px(2.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        margin: UiRect::top(Val::Px(12.0)),
                        ..default()
                    },
                    BorderColor::all(Color::srgb(0.6, 0.6, 0.7)),
                    BackgroundColor(Color::srgb(0.1, 0.1, 0.15)),
                ))
                .with_children(|btn| {
                    btn.spawn((
                        Text::new("MENU"),
                        TextFont {
                            font_size: FontSize::Px(22.0),
                            ..default()
                        },
                        TextColor(Color::srgb(0.8, 0.85, 0.9)),
                    ));
                });

            parent
                .spawn((
                    GameOverMapButton,
                    Button,
                    Node {
                        width: Val::Px(200.0),
                        height: Val::Px(44.0),
                        border: UiRect::all(Val::Px(2.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        margin: UiRect::top(Val::Px(8.0)),
                        ..default()
                    },
                    BorderColor::all(Color::srgb(0.3, 0.7, 1.0)),
                    BackgroundColor(Color::srgb(0.08, 0.12, 0.2)),
                ))
                .with_children(|btn| {
                    btn.spawn((
                        Text::new("GALAXY MAP"),
                        TextFont {
                            font_size: FontSize::Px(18.0),
                            ..default()
                        },
                        TextColor(Color::srgb(0.8, 0.9, 1.0)),
                    ));
                });
        });
}

pub fn game_over_button_interaction(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, &mut BorderColor),
        (Changed<Interaction>, With<GameOverButton>),
    >,
    mut next_state: ResMut<NextState<AppState>>,
    mut commands: Commands,
    arena_entities: Query<Entity, With<ArenaEntity>>,
    level_entities: Query<Entity, With<LevelEntity>>,
) {
    for (interaction, mut bg, mut border) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                for entity in &arena_entities {
                    commands.entity(entity).try_despawn();
                }
                for entity in &level_entities {
                    commands.entity(entity).try_despawn();
                }
                commands.remove_resource::<GameState>();
                commands.remove_resource::<super::arena::ArenaSpawned>();
                next_state.set(AppState::Menu);
            }
            Interaction::Hovered => {
                *bg = BackgroundColor(Color::srgb(0.15, 0.15, 0.2));
                *border = BorderColor::all(Color::srgb(0.8, 0.85, 0.9));
            }
            Interaction::None => {
                *bg = BackgroundColor(Color::srgb(0.1, 0.1, 0.15));
                *border = BorderColor::all(Color::srgb(0.6, 0.6, 0.7));
            }
        }
    }
}

pub fn game_over_map_interaction(
    mut q: Query<&Interaction, (Changed<Interaction>, With<GameOverMapButton>)>,
    mut next_state: ResMut<NextState<AppState>>,
    mut commands: Commands,
    arena_entities: Query<Entity, With<ArenaEntity>>,
    level_entities: Query<Entity, With<LevelEntity>>,
) {
    for interaction in &mut q {
        if *interaction == Interaction::Pressed {
            for entity in &arena_entities {
                commands.entity(entity).try_despawn();
            }
            for entity in &level_entities {
                commands.entity(entity).try_despawn();
            }
            commands.remove_resource::<super::arena::ArenaSpawned>();
            next_state.set(AppState::LevelSelect);
        }
    }
}

pub fn teardown_game_over(mut commands: Commands, query: Query<Entity, With<GameOverRoot>>) {
    for entity in &query {
        commands.entity(entity).try_despawn();
    }
}

pub fn setup_pause(mut commands: Commands) {
    commands
        .spawn((
            PauseRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                row_gap: Val::Px(16.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.6)),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("PAUSED"),
                TextFont {
                    font_size: FontSize::Px(48.0),
                    ..default()
                },
                TextColor(Color::srgb(1.0, 1.0, 1.0)),
            ));

            parent
                .spawn((
                    ResumeButton,
                    Button,
                    Node {
                        width: Val::Px(180.0),
                        height: Val::Px(44.0),
                        border: UiRect::all(Val::Px(2.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BorderColor::all(Color::srgb(0.5, 0.8, 0.5)),
                    BackgroundColor(Color::srgb(0.08, 0.15, 0.08)),
                ))
                .with_children(|btn| {
                    btn.spawn((
                        Text::new("RESUME"),
                        TextFont {
                            font_size: FontSize::Px(20.0),
                            ..default()
                        },
                        TextColor(Color::srgb(0.7, 1.0, 0.7)),
                    ));
                });

            parent
                .spawn((
                    PauseMenuButton,
                    Button,
                    Node {
                        width: Val::Px(180.0),
                        height: Val::Px(44.0),
                        border: UiRect::all(Val::Px(2.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BorderColor::all(Color::srgb(0.7, 0.4, 0.4)),
                    BackgroundColor(Color::srgb(0.15, 0.08, 0.08)),
                ))
                .with_children(|btn| {
                    btn.spawn((
                        Text::new("MENU"),
                        TextFont {
                            font_size: FontSize::Px(20.0),
                            ..default()
                        },
                        TextColor(Color::srgb(1.0, 0.6, 0.6)),
                    ));
                });
        });
}

pub fn pause_button_interaction(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, &mut BorderColor),
        (Changed<Interaction>, With<ResumeButton>),
    >,
    mut next_state: ResMut<NextState<AppState>>,
    mut time: ResMut<Time<Virtual>>,
) {
    for (interaction, mut bg, mut border) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                time.unpause();
                next_state.set(AppState::Playing);
            }
            Interaction::Hovered => {
                *bg = BackgroundColor(Color::srgb(0.12, 0.25, 0.12));
                *border = BorderColor::all(Color::srgb(0.6, 1.0, 0.6));
            }
            Interaction::None => {
                *bg = BackgroundColor(Color::srgb(0.08, 0.15, 0.08));
                *border = BorderColor::all(Color::srgb(0.5, 0.8, 0.5));
            }
        }
    }
}

pub fn pause_menu_button_interaction(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, &mut BorderColor),
        (Changed<Interaction>, With<PauseMenuButton>),
    >,
    mut next_state: ResMut<NextState<AppState>>,
    mut commands: Commands,
    mut time: ResMut<Time<Virtual>>,
    arena_entities: Query<Entity, With<ArenaEntity>>,
    level_entities: Query<Entity, With<LevelEntity>>,
) {
    for (interaction, mut bg, mut border) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                time.unpause();
                for entity in &arena_entities {
                    commands.entity(entity).try_despawn();
                }
                for entity in &level_entities {
                    commands.entity(entity).try_despawn();
                }
                commands.remove_resource::<GameState>();
                commands.remove_resource::<super::arena::ArenaSpawned>();
                next_state.set(AppState::Menu);
            }
            Interaction::Hovered => {
                *bg = BackgroundColor(Color::srgb(0.25, 0.12, 0.12));
                *border = BorderColor::all(Color::srgb(1.0, 0.7, 0.7));
            }
            Interaction::None => {
                *bg = BackgroundColor(Color::srgb(0.15, 0.08, 0.08));
                *border = BorderColor::all(Color::srgb(0.7, 0.4, 0.4));
            }
        }
    }
}

pub fn teardown_pause(mut commands: Commands, query: Query<Entity, With<PauseRoot>>) {
    for entity in &query {
        commands.entity(entity).try_despawn();
    }
}

pub fn toggle_pause(
    keyboard: Res<ButtonInput<KeyCode>>,
    current_state: Res<State<AppState>>,
    mut next_state: ResMut<NextState<AppState>>,
    mut time: ResMut<Time<Virtual>>,
) {
    if keyboard.just_pressed(KeyCode::Escape) {
        match current_state.get() {
            AppState::Playing => {
                time.pause();
                next_state.set(AppState::Paused);
            }
            AppState::Paused => {
                time.unpause();
                next_state.set(AppState::Playing);
            }
            _ => {}
        }
    }
}

/// Despawn arena/level when leaving play for complete/game over.
pub fn cleanup_play_entities(
    mut commands: Commands,
    arena_entities: Query<Entity, With<ArenaEntity>>,
    level_entities: Query<Entity, With<LevelEntity>>,
) {
    for entity in &arena_entities {
        commands.entity(entity).try_despawn();
    }
    for entity in &level_entities {
        commands.entity(entity).try_despawn();
    }
    commands.remove_resource::<super::arena::ArenaSpawned>();
}
