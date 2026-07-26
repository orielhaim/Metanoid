use bevy::prelude::*;
use metanoid_core::resources::game_state::GameState;
use metanoid_core::states::AppState;

use super::save::SaveData;
use super::settings::ShowSettings;

#[derive(Component)]
pub struct MenuRoot;

#[derive(Component)]
pub struct GameOverRoot;

#[derive(Component)]
pub struct PauseRoot;

#[derive(Component)]
pub struct PlayButton;

#[derive(Component)]
pub struct SettingsButton;

// ── Main Menu ──────────────────────────────────────────

pub fn setup_menu(mut commands: Commands, save: Res<SaveData>) {
    let high_score = save.high_score;

    commands
        .spawn((
            MenuRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            BackgroundColor(Color::srgb(0.01, 0.01, 0.04)),
        ))
        .with_children(|root| {
            // Top bar with score
            root.spawn(Node {
                position_type: PositionType::Absolute,
                top: Val::Px(0.0),
                width: Val::Percent(100.0),
                height: Val::Px(60.0),
                padding: UiRect::horizontal(Val::Px(30.0)),
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                ..default()
            })
            .with_children(|bar| {
                bar.spawn((
                    Text::new(format!("High Score: {}", high_score)),
                    TextFont { font_size: FontSize::Px(20.0), ..default() },
                    TextColor(Color::srgb(0.7, 0.75, 0.8)),
                ));
            });

            // Title area
            root.spawn(Node {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: Val::Px(8.0),
                margin: UiRect::bottom(Val::Px(40.0)),
                ..default()
            })
            .with_children(|title| {
                title.spawn((
                    Text::new("METANOID"),
                    TextFont { font_size: FontSize::Px(80.0), ..default() },
                    TextColor(Color::srgb(0.3, 0.7, 1.0)),
                ));
                title.spawn((
                    Text::new("Infinite Breakout"),
                    TextFont { font_size: FontSize::Px(18.0), ..default() },
                    TextColor(Color::srgb(0.4, 0.45, 0.55)),
                ));
            });

            // Play button
            root.spawn((
                PlayButton,
                Button,
                Node {
                    width: Val::Px(220.0),
                    height: Val::Px(56.0),
                    border: UiRect::all(Val::Px(2.0)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    margin: UiRect::bottom(Val::Px(20.0)),
                    ..default()
                },
                BorderColor::all(Color::srgb(0.3, 0.7, 1.0)),
                BackgroundColor(Color::srgb(0.08, 0.12, 0.2)),
            ))
            .with_children(|btn| {
                btn.spawn((
                    Text::new("PLAY"),
                    TextFont { font_size: FontSize::Px(28.0), ..default() },
                    TextColor(Color::srgb(0.9, 0.95, 1.0)),
                ));
            });

            // Settings button
            root.spawn((
                SettingsButton,
                Button,
                Node {
                    width: Val::Px(220.0),
                    height: Val::Px(48.0),
                    border: UiRect::all(Val::Px(2.0)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    margin: UiRect::bottom(Val::Px(10.0)),
                    ..default()
                },
                BorderColor::all(Color::srgb(0.4, 0.45, 0.55)),
                BackgroundColor(Color::srgb(0.06, 0.08, 0.12)),
            ))
            .with_children(|btn| {
                btn.spawn((
                    Text::new("SETTINGS"),
                    TextFont { font_size: FontSize::Px(22.0), ..default() },
                    TextColor(Color::srgb(0.6, 0.65, 0.75)),
                ));
            });

            // Controls hint
            root.spawn(Node {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: Val::Px(4.0),
                margin: UiRect::top(Val::Px(30.0)),
                ..default()
            })
            .with_children(|hint| {
                hint.spawn((
                    Text::new("Arrow keys / A, D — Move paddle"),
                    TextFont { font_size: FontSize::Px(14.0), ..default() },
                    TextColor(Color::srgb(0.35, 0.35, 0.45)),
                ));
                hint.spawn((
                    Text::new("Space — Launch ball"),
                    TextFont { font_size: FontSize::Px(14.0), ..default() },
                    TextColor(Color::srgb(0.35, 0.35, 0.45)),
                ));
                hint.spawn((
                    Text::new("Esc — Pause"),
                    TextFont { font_size: FontSize::Px(14.0), ..default() },
                    TextColor(Color::srgb(0.35, 0.35, 0.45)),
                ));
            });
        });
}

pub fn menu_button_interaction(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, &mut BorderColor),
        (Changed<Interaction>, With<PlayButton>),
    >,
    mut next_state: ResMut<NextState<AppState>>,
) {
    for (interaction, mut bg, mut border) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *bg = BackgroundColor(Color::srgb(0.15, 0.3, 0.5));
                *border = BorderColor::all(Color::srgb(0.5, 0.9, 1.0));
                next_state.set(AppState::LevelSelect);
            }
            Interaction::Hovered => {
                *bg = BackgroundColor(Color::srgb(0.1, 0.18, 0.3));
                *border = BorderColor::all(Color::srgb(0.5, 0.85, 1.0));
            }
            Interaction::None => {
                *bg = BackgroundColor(Color::srgb(0.08, 0.12, 0.2));
                *border = BorderColor::all(Color::srgb(0.3, 0.7, 1.0));
            }
        }
    }
}

pub fn settings_button_interaction(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, &mut BorderColor),
        (Changed<Interaction>, With<SettingsButton>),
    >,
    mut show_settings: ResMut<ShowSettings>,
) {
    for (interaction, mut bg, mut border) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *bg = BackgroundColor(Color::srgb(0.15, 0.2, 0.3));
                *border = BorderColor::all(Color::srgb(0.6, 0.7, 0.8));
                show_settings.0 = true;
            }
            Interaction::Hovered => {
                *bg = BackgroundColor(Color::srgb(0.1, 0.14, 0.22));
                *border = BorderColor::all(Color::srgb(0.5, 0.55, 0.65));
            }
            Interaction::None => {
                *bg = BackgroundColor(Color::srgb(0.06, 0.08, 0.12));
                *border = BorderColor::all(Color::srgb(0.4, 0.45, 0.55));
            }
        }
    }
}

pub fn menu_any_key(
    keyboard: Res<ButtonInput<KeyCode>>,
    show_settings: Res<ShowSettings>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    // Don't transition if settings are open
    if show_settings.0 {
        return;
    }

    // Skip Space, Escape, and modifier keys to avoid accidental triggers
    if keyboard.just_pressed(KeyCode::Space)
        || keyboard.just_pressed(KeyCode::Escape)
        || keyboard.just_pressed(KeyCode::ControlLeft)
        || keyboard.just_pressed(KeyCode::ShiftLeft)
    {
        return;
    }

    if keyboard.get_pressed().next().is_some() {
        next_state.set(AppState::LevelSelect);
    }
}

pub fn teardown_menu(
    mut commands: Commands,
    query: Query<Entity, With<MenuRoot>>,
) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

// ── Game Over Screen ───────────────────────────────────

#[derive(Component)]
pub struct GameOverButton;

pub fn setup_game_over(mut commands: Commands, game_state: Option<Res<GameState>>) {
    let score = game_state.map(|s| s.score).unwrap_or(0);

    commands
        .spawn((
            GameOverRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                row_gap: Val::Px(24.0),
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
                Text::new(format!("Score: {score}")),
                TextFont {
                    font_size: FontSize::Px(32.0),
                    ..default()
                },
                TextColor(Color::srgb(1.0, 1.0, 1.0)),
            ));

            // Restart button
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
                        margin: UiRect::top(Val::Px(16.0)),
                        ..default()
                    },
                    BorderColor::all(Color::srgb(0.6, 0.6, 0.7)),
                    BackgroundColor(Color::srgb(0.1, 0.1, 0.15)),
                ))
                .with_children(|btn| {
                    btn.spawn((
                        Text::new("RESTART"),
                        TextFont {
                            font_size: FontSize::Px(22.0),
                            ..default()
                        },
                        TextColor(Color::srgb(0.8, 0.85, 0.9)),
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
    arena_entities: Query<Entity, With<super::arena::ArenaEntity>>,
    level_entities: Query<Entity, With<super::level_spawner::LevelEntity>>,
    hud_entities: Query<Entity, With<super::hud::HudRoot>>,
) {
    for (interaction, mut bg, mut border) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                for entity in &arena_entities {
                    commands.entity(entity).despawn();
                }
                for entity in &level_entities {
                    commands.entity(entity).despawn();
                }
                for entity in &hud_entities {
                    commands.entity(entity).despawn();
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

pub fn teardown_game_over(mut commands: Commands, query: Query<Entity, With<GameOverRoot>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

// ── Pause Overlay ──────────────────────────────────────

#[derive(Component)]
pub struct ResumeButton;

#[derive(Component)]
pub struct PauseMenuButton;

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
                row_gap: Val::Px(20.0),
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
                        width: Val::Px(160.0),
                        height: Val::Px(44.0),
                        border: UiRect::all(Val::Px(2.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        margin: UiRect::top(Val::Px(10.0)),
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

            // Back to Menu button
            parent
                .spawn((
                    PauseMenuButton,
                    Button,
                    Node {
                        width: Val::Px(160.0),
                        height: Val::Px(44.0),
                        border: UiRect::all(Val::Px(2.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        margin: UiRect::top(Val::Px(8.0)),
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
    arena_entities: Query<Entity, With<super::arena::ArenaEntity>>,
    level_entities: Query<Entity, With<super::level_spawner::LevelEntity>>,
    hud_entities: Query<Entity, With<super::hud::HudRoot>>,
) {
    for (interaction, mut bg, mut border) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                time.unpause();
                for entity in &arena_entities {
                    commands.entity(entity).despawn();
                }
                for entity in &level_entities {
                    commands.entity(entity).despawn();
                }
                for entity in &hud_entities {
                    commands.entity(entity).despawn();
                }
                commands.remove_resource::<metanoid_core::resources::game_state::GameState>();
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
        commands.entity(entity).despawn();
    }
}

// ── Pause Input (keyboard shortcut) ────────────────────

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
