//! Level complete with 0–100 rating breakdown.

use bevy::prelude::*;
use metanoid_core::rating::LastRatingResult;
use metanoid_core::resources::game_state::GameState;
use metanoid_core::states::AppState;

use crate::theme::{UiTheme, grade_color};
use crate::widgets::{primary_button_colors, secondary_button_colors, spawn_label};

#[derive(Component)]
pub struct LevelCompleteRoot;

#[derive(Component)]
pub struct ContinueNextButton;

#[derive(Component)]
pub struct RetryButton;

#[derive(Component)]
pub struct MapFromCompleteButton;

pub fn setup_level_complete(
    mut commands: Commands,
    game_state: Option<Res<GameState>>,
    last: Option<Res<LastRatingResult>>,
    theme: Res<UiTheme>,
) {
    let score = game_state.as_ref().map(|s| s.score).unwrap_or(0);
    let last = last.as_ref();

    let (rating, grade, is_pb, is_boss, g, b, l) = last
        .map(|r| {
            (
                r.rating,
                r.grade.as_str(),
                r.is_pb,
                r.is_boss,
                r.galaxy + 1,
                r.biome + 1,
                r.level + 1,
            )
        })
        .unwrap_or((0, "D", false, false, 1, 1, 1));

    let title = if is_boss {
        "BOSS DEFEATED!"
    } else {
        "LEVEL CLEAR!"
    };

    let confetti = [
        Color::srgb(1.0, 0.2, 0.2),
        Color::srgb(0.2, 1.0, 0.2),
        Color::srgb(0.2, 0.2, 1.0),
        Color::srgb(1.0, 1.0, 0.2),
        Color::srgb(1.0, 0.2, 1.0),
        grade_color(metanoid_core::grade_from_rating(rating)),
    ];

    for i in 0..48 {
        let color = confetti[i % confetti.len()];
        let x = (i as f32 - 24.0) * 18.0;
        commands.spawn((
            LevelCompleteRoot,
            Node {
                position_type: PositionType::Absolute,
                left: Val::Percent(50.0 + x / 14.0),
                top: Val::Percent(15.0 + (i as f32 % 40.0)),
                width: Val::Px(8.0),
                height: Val::Px(8.0),
                ..default()
            },
            BackgroundColor(color),
        ));
    }

    let (pbg, pbd) = primary_button_colors(&theme);
    let (sbg, sbd) = secondary_button_colors(&theme);

    commands
        .spawn((
            LevelCompleteRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                row_gap: Val::Px(10.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.08, 0.82)),
        ))
        .with_children(|parent| {
            spawn_label(parent, title, 48.0, theme.success);
            spawn_label(
                parent,
                format!("Galaxy {g}  |  Biome {b}  |  Level {l}"),
                18.0,
                theme.text_muted,
            );

            spawn_label(
                parent,
                format!("RATING  {rating}  {grade}"),
                40.0,
                grade_color(metanoid_core::grade_from_rating(rating)),
            );

            if is_pb {
                spawn_label(parent, "*** NEW PERSONAL BEST ***", 18.0, theme.accent);
            }

            if let Some(r) = last {
                if let Some(ref bd) = r.breakdown {
                    spawn_label(
                        parent,
                        format!(
                            "Clear {:.0} | Lives {:.0} | Combo {:.0} | Speed {:.0} | Clean {:.0} | Bonus {:.0}",
                            bd.clear_part,
                            bd.lives_part,
                            bd.combo_part,
                            bd.speed_part,
                            bd.clean_part,
                            bd.clean_bonus
                        ),
                        12.0,
                        theme.text_muted,
                    );
                }
                spawn_label(
                    parent,
                    format!("Level score: {}  |  Run score: {}", r.level_score, score),
                    16.0,
                    theme.text_primary,
                );
            } else {
                spawn_label(parent, format!("Score: {score}"), 20.0, theme.text_primary);
            }

            parent
                .spawn((
                    ContinueNextButton,
                    Button,
                    Node {
                        width: Val::Px(220.0),
                        height: Val::Px(48.0),
                        border: UiRect::all(Val::Px(2.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        margin: UiRect::top(Val::Px(12.0)),
                        ..default()
                    },
                    pbg,
                    pbd,
                ))
                .with_children(|btn| {
                    spawn_label(btn, "NEXT", 22.0, theme.text_primary);
                });

            parent
                .spawn((
                    RetryButton,
                    Button,
                    Node {
                        width: Val::Px(220.0),
                        height: Val::Px(40.0),
                        border: UiRect::all(Val::Px(2.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        margin: UiRect::top(Val::Px(6.0)),
                        ..default()
                    },
                    sbg.clone(),
                    sbd.clone(),
                ))
                .with_children(|btn| {
                    spawn_label(btn, "RETRY CHALLENGE", 16.0, theme.text_muted);
                });

            parent
                .spawn((
                    MapFromCompleteButton,
                    Button,
                    Node {
                        width: Val::Px(220.0),
                        height: Val::Px(40.0),
                        border: UiRect::all(Val::Px(2.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        margin: UiRect::top(Val::Px(6.0)),
                        ..default()
                    },
                    sbg,
                    sbd,
                ))
                .with_children(|btn| {
                    spawn_label(btn, "GALAXY MAP", 16.0, theme.text_muted);
                });
        });
}

pub fn teardown_level_complete(mut commands: Commands, q: Query<Entity, With<LevelCompleteRoot>>) {
    for e in &q {
        commands.entity(e).despawn();
    }
}

pub fn continue_next_interaction(
    mut q: Query<&Interaction, (Changed<Interaction>, With<ContinueNextButton>)>,
    mut next: ResMut<NextState<AppState>>,
) {
    for interaction in &mut q {
        if *interaction == Interaction::Pressed {
            next.set(AppState::Loading);
        }
    }
}

pub fn retry_interaction(
    mut q: Query<&Interaction, (Changed<Interaction>, With<RetryButton>)>,
    last: Option<Res<LastRatingResult>>,
    mut game_state: Option<ResMut<GameState>>,
    mut commands: Commands,
    mut next: ResMut<NextState<AppState>>,
) {
    for interaction in &mut q {
        if *interaction != Interaction::Pressed {
            continue;
        }
        if let (Some(last), Some(ref mut state)) = (last.as_ref(), game_state.as_mut()) {
            state.galaxy = last.galaxy;
            state.biome = last.biome;
            state.level = last.level;
            state.lives = 3;
            state.score = 0;
            state.level_clearing = false;
            commands.insert_resource(metanoid_core::LevelLaunchMode::Challenge);
        }
        next.set(AppState::Loading);
    }
}

pub fn map_from_complete_interaction(
    mut q: Query<&Interaction, (Changed<Interaction>, With<MapFromCompleteButton>)>,
    mut next: ResMut<NextState<AppState>>,
) {
    for interaction in &mut q {
        if *interaction == Interaction::Pressed {
            next.set(AppState::LevelSelect);
        }
    }
}
