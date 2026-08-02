//! Competitive in-run HUD: score, combo, lives, rating estimate, PB chip.

use bevy::prelude::*;
use metanoid_core::rating::{LevelRunStats, estimate_rating, grade_from_rating};
use metanoid_core::resources::combo::ComboCounter;
use metanoid_core::resources::game_state::GameState;
use metanoid_core::save_data::{SaveData, level_key};
use metanoid_core::settings::GameSettings;

use crate::theme::{UiTheme, grade_color};

#[derive(Component)]
pub struct HudRoot;

#[derive(Component)]
pub struct ScoreText;

#[derive(Component)]
pub struct LivesText;

#[derive(Component)]
pub struct ComboText;

#[derive(Component)]
pub struct EstimateText;

#[derive(Component)]
pub struct PbText;

#[derive(Component)]
pub struct LevelChipText;

pub fn setup_hud(
    mut commands: Commands,
    theme: Res<UiTheme>,
    settings: Res<GameSettings>,
    game_state: Option<Res<GameState>>,
    save: Res<SaveData>,
    stats: Option<Res<LevelRunStats>>,
) {
    let font = if settings.large_hud { 26.0 } else { 20.0 };
    let small = if settings.large_hud { 16.0 } else { 13.0 };

    let (g, b, l) = game_state
        .as_ref()
        .map(|s| (s.galaxy, s.biome, s.level))
        .unwrap_or((0, 0, 0));
    let key = level_key(g, b, l);
    let pb_label = save
        .level_results
        .get(&key)
        .filter(|p| p.clears > 0)
        .map(|p| {
            format!(
                "PB {} {}",
                p.best_rating,
                grade_from_rating(p.best_rating).as_str()
            )
        })
        .unwrap_or_else(|| "PB --".into());

    let est = stats.as_ref().map(|s| estimate_rating(s)).unwrap_or(35);

    commands
        .spawn((
            HudRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Px(if settings.large_hud { 56.0 } else { 48.0 }),
                padding: UiRect::horizontal(Val::Px(16.0)),
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                column_gap: Val::Px(12.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.55)),
        ))
        .with_children(|parent| {
            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(16.0),
                    align_items: AlignItems::Center,
                    ..default()
                })
                .with_children(|left| {
                    left.spawn((
                        ScoreText,
                        Text::new("Score: 0"),
                        TextFont {
                            font_size: FontSize::Px(font),
                            ..default()
                        },
                        TextColor(theme.text_primary),
                    ));
                    left.spawn((
                        ComboText,
                        Text::new("COMBO x0  1.0x"),
                        TextFont {
                            font_size: FontSize::Px(font),
                            ..default()
                        },
                        TextColor(theme.accent),
                    ));
                });

            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(14.0),
                    align_items: AlignItems::Center,
                    ..default()
                })
                .with_children(|mid| {
                    mid.spawn((
                        LevelChipText,
                        Text::new(format!("G{}-B{}-L{}", g + 1, b + 1, l + 1)),
                        TextFont {
                            font_size: FontSize::Px(small),
                            ..default()
                        },
                        TextColor(theme.text_muted),
                    ));
                    mid.spawn((
                        PbText,
                        Text::new(pb_label),
                        TextFont {
                            font_size: FontSize::Px(small),
                            ..default()
                        },
                        TextColor(theme.text_muted),
                    ));
                    mid.spawn((
                        EstimateText,
                        Text::new(format!("Est ~{est}")),
                        TextFont {
                            font_size: FontSize::Px(small),
                            ..default()
                        },
                        TextColor(grade_color(grade_from_rating(est))),
                    ));
                });

            parent.spawn((
                LivesText,
                Text::new("Lives: "),
                TextFont {
                    font_size: FontSize::Px(font),
                    ..default()
                },
                TextColor(theme.danger),
            ));
        });
}

pub fn teardown_hud(mut commands: Commands, q: Query<Entity, With<HudRoot>>) {
    for e in &q {
        commands.entity(e).despawn();
    }
}

pub fn update_hud(
    game_state: Option<Res<GameState>>,
    combo: Res<ComboCounter>,
    stats: Option<Res<LevelRunStats>>,
    mut score_q: Query<
        &mut Text,
        (
            With<ScoreText>,
            Without<LivesText>,
            Without<ComboText>,
            Without<EstimateText>,
            Without<PbText>,
            Without<LevelChipText>,
        ),
    >,
    mut lives_q: Query<
        &mut Text,
        (
            With<LivesText>,
            Without<ScoreText>,
            Without<ComboText>,
            Without<EstimateText>,
            Without<PbText>,
            Without<LevelChipText>,
        ),
    >,
    mut combo_q: Query<
        &mut Text,
        (
            With<ComboText>,
            Without<ScoreText>,
            Without<LivesText>,
            Without<EstimateText>,
            Without<PbText>,
            Without<LevelChipText>,
        ),
    >,
    mut est_q: Query<
        &mut Text,
        (
            With<EstimateText>,
            Without<ScoreText>,
            Without<LivesText>,
            Without<ComboText>,
            Without<PbText>,
            Without<LevelChipText>,
        ),
    >,
    mut est_color: Query<&mut TextColor, (With<EstimateText>, Without<ScoreText>)>,
) {
    let Some(state) = game_state else {
        return;
    };

    for mut text in &mut score_q {
        **text = format!("Score: {}", state.score);
    }

    // ASCII lives markers (default font has no heart glyph)
    let hearts: String = (0..state.lives.max(0)).map(|_| "* ").collect();
    for mut text in &mut lives_q {
        **text = format!("Lives: {}", hearts);
    }

    for mut text in &mut combo_q {
        **text = if combo.count > 0 {
            format!("VULN x{}  {:.1}x", combo.count, combo.multiplier)
        } else {
            "VULN x0".into()
        };
    }

    if let Some(stats) = stats {
        // Keep lives/max_combo synced for estimate
        let mut snap = stats.clone();
        snap.lives_remaining = state.lives;
        snap.max_combo = snap.max_combo.max(combo.count);
        let est = estimate_rating(&snap);
        for mut text in &mut est_q {
            **text = format!("Est ~{} {}", est, grade_from_rating(est).as_str());
        }
        for mut c in &mut est_color {
            c.0 = grade_color(grade_from_rating(est));
        }
    }
}
