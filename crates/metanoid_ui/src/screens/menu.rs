//! Career hub main menu — ASCII-safe labels only.

use bevy::prelude::*;
use metanoid_core::save_data::{SaveData, mastery_percent, s_rank_count};
use metanoid_core::states::AppState;

use crate::theme::UiTheme;
use crate::widgets::{
    entrance_scale, primary_button_colors, secondary_button_colors, spawn_label, spawn_title,
    styled_button_node,
};

#[derive(Component)]
pub struct MenuRoot;

#[derive(Component)]
pub struct ContinueButton;

#[derive(Component)]
pub struct GalaxyMapButton;

#[derive(Component)]
pub struct SettingsMenuButton;

/// Pulsing glow behind the game title.
#[derive(Component)]
pub struct TitleGlow;

/// Gentle pulsing halo under the title.
pub fn tick_title_glow(time: Res<Time>, mut q: Query<&mut UiTransform, With<TitleGlow>>) {
    let t = time.elapsed_secs();
    for mut transform in &mut q {
        let s = 1.0 + (t * 1.4).sin() * 0.06;
        transform.scale = Vec2::splat(s);
    }
}

pub fn setup_menu(mut commands: Commands, save: Res<SaveData>, theme: Res<UiTheme>) {
    let mastery = mastery_percent(&save);
    let s_ranks = s_rank_count(&save);
    let high = save.career_high_score;
    let bricks = save.total_bricks_destroyed;
    let cleared = save.total_levels_cleared;
    let frontier = format!(
        "G{} / B{} / L{}",
        save.highest_galaxy + 1,
        save.highest_biome + 1,
        save.highest_level + 1
    );

    let recent_lines: Vec<String> = save
        .recent_clears
        .iter()
        .take(5)
        .map(|r| {
            let pb = if r.is_pb { " PB" } else { "" };
            format!(
                "G{} B{} L{}  -  {} {}{}",
                r.galaxy + 1,
                r.biome + 1,
                r.level + 1,
                r.rating,
                r.grade,
                pb
            )
        })
        .collect();

    commands
        .spawn((
            MenuRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(24.0)),
                row_gap: Val::Px(12.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.0)),
        ))
        .with_children(|root| {
            // Dark scrim so text stays readable over the animated backdrop.
            root.spawn(Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                right: Val::Px(0.0),
                top: Val::Px(0.0),
                bottom: Val::Px(0.0),
                ..default()
            })
            .insert(BackgroundColor(Color::srgba(0.01, 0.012, 0.05, 0.68)));

            // Career strip
            root.spawn((
                Node {
                    width: Val::Percent(100.0),
                    max_width: Val::Px(900.0),
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::SpaceBetween,
                    column_gap: Val::Px(16.0),
                    padding: UiRect::all(Val::Px(12.0)),
                    border: UiRect::all(Val::Px(1.0)),
                    border_radius: BorderRadius::all(Val::Px(8.0)),
                    ..default()
                },
                BackgroundColor(theme.panel),
                BorderColor::all(theme.panel_border),
            ))
            .with_children(|bar| {
                spawn_label(
                    bar,
                    format!("SEED #{}  |  {}", save.master_seed, save.player_name),
                    14.0,
                    theme.text_muted,
                );
                spawn_label(bar, format!("MASTERY {:.0}%", mastery), 16.0, theme.accent);
                spawn_label(bar, format!("CAREER BEST {high}"), 14.0, theme.text_primary);
            });

            root.spawn(Node {
                width: Val::Percent(100.0),
                max_width: Val::Px(900.0),
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::SpaceBetween,
                margin: UiRect::bottom(Val::Px(8.0)),
                ..default()
            })
            .with_children(|stats| {
                spawn_label(
                    stats,
                    format!("Bricks {bricks}  |  Clears {cleared}  |  S-ranks {s_ranks}"),
                    13.0,
                    theme.text_muted,
                );
            });

            // Title
            root.spawn(Node {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: Val::Px(6.0),
                margin: UiRect::vertical(Val::Px(20.0)),
                ..default()
            })
            .insert(entrance_scale(0.05, 0.5))
            .with_children(|title| {
                // Glow halo behind the wordmark.
                title
                    .spawn(Node {
                        position_type: PositionType::Absolute,
                        left: Val::Percent(50.0),
                        top: Val::Percent(50.0),
                        width: Val::Px(520.0),
                        height: Val::Px(110.0),
                        border_radius: BorderRadius::all(Val::Px(60.0)),
                        ..default()
                    })
                    .insert((
                        TitleGlow,
                        UiTransform {
                            translation: Val2::px(-260.0, -60.0),
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.2, 0.5, 1.0, 0.10)),
                    ));
                spawn_title(title, "METANOID", theme.font_title, theme.accent);
                spawn_label(
                    title,
                    "Infinite Breakout  -  Competitive",
                    16.0,
                    theme.text_muted,
                );
            });

            // Buttons
            let (pbg, pbd) = primary_button_colors(&theme);
            let (sbg, sbd) = secondary_button_colors(&theme);

            root.spawn((
                ContinueButton,
                Button,
                styled_button_node(280.0, 56.0),
                pbg,
                pbd,
            ))
            .insert(entrance_scale(0.18, 0.6))
            .with_children(|btn| {
                spawn_label(
                    btn,
                    format!("CONTINUE  {frontier}"),
                    22.0,
                    theme.text_primary,
                );
            });

            root.spawn((
                GalaxyMapButton,
                Button,
                Node {
                    width: Val::Px(280.0),
                    height: Val::Px(48.0),
                    border: UiRect::all(Val::Px(2.0)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    margin: UiRect::top(Val::Px(8.0)),
                    ..default()
                },
                sbg.clone(),
                sbd.clone(),
            ))
            .insert(entrance_scale(0.3, 0.6))
            .with_children(|btn| {
                spawn_label(btn, "GALAXY MAP", 20.0, theme.text_primary);
            });

            root.spawn((
                SettingsMenuButton,
                Button,
                Node {
                    width: Val::Px(280.0),
                    height: Val::Px(44.0),
                    border: UiRect::all(Val::Px(2.0)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    margin: UiRect::top(Val::Px(8.0)),
                    ..default()
                },
                sbg,
                sbd,
            ))
            .insert(entrance_scale(0.42, 0.6))
            .with_children(|btn| {
                spawn_label(btn, "SETTINGS", 18.0, theme.text_muted);
            });

            // Recent clears
            root.spawn(Node {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::FlexStart,
                row_gap: Val::Px(4.0),
                margin: UiRect::top(Val::Px(28.0)),
                width: Val::Percent(100.0),
                max_width: Val::Px(480.0),
                ..default()
            })
            .with_children(|recent| {
                spawn_label(recent, "RECENT", 14.0, theme.accent);
                if recent_lines.is_empty() {
                    spawn_label(
                        recent,
                        "No clears yet - open the galaxy and play.",
                        13.0,
                        theme.text_muted,
                    );
                } else {
                    for line in recent_lines {
                        spawn_label(recent, line, 13.0, theme.text_muted);
                    }
                }
            });

            root.spawn(Node {
                margin: UiRect::top(Val::Px(20.0)),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: Val::Px(2.0),
                ..default()
            })
            .with_children(|hint| {
                spawn_label(
                    hint,
                    "Arrows / A D - Move  |  Space - Launch  |  Esc - Pause",
                    12.0,
                    theme.text_muted,
                );
            });
        });
}

pub fn teardown_menu(mut commands: Commands, q: Query<Entity, With<MenuRoot>>) {
    for e in &q {
        commands.entity(e).despawn();
    }
}

pub fn continue_button_interaction(
    mut q: Query<
        (&Interaction, &mut BackgroundColor, &mut BorderColor),
        (Changed<Interaction>, With<ContinueButton>),
    >,
    mut next: ResMut<NextState<AppState>>,
    theme: Res<UiTheme>,
) {
    for (interaction, mut bg, mut border) in &mut q {
        match *interaction {
            Interaction::Pressed => {
                next.set(AppState::LevelSelect);
            }
            Interaction::Hovered => {
                *bg = BackgroundColor(Color::srgb(0.1, 0.18, 0.3));
                *border = BorderColor::all(Color::srgb(0.5, 0.85, 1.0));
            }
            Interaction::None => {
                let (b, d) = primary_button_colors(&theme);
                *bg = b;
                *border = d;
            }
        }
    }
}

pub fn galaxy_map_button_interaction(
    mut q: Query<
        (&Interaction, &mut BackgroundColor, &mut BorderColor),
        (Changed<Interaction>, With<GalaxyMapButton>),
    >,
    mut next: ResMut<NextState<AppState>>,
    theme: Res<UiTheme>,
) {
    for (interaction, mut bg, mut border) in &mut q {
        match *interaction {
            Interaction::Pressed => next.set(AppState::LevelSelect),
            Interaction::Hovered => {
                *bg = BackgroundColor(Color::srgb(0.1, 0.14, 0.22));
                *border = BorderColor::all(theme.accent);
            }
            Interaction::None => {
                let (b, d) = secondary_button_colors(&theme);
                *bg = b;
                *border = d;
            }
        }
    }
}

pub fn settings_menu_button_interaction(
    mut q: Query<
        (&Interaction, &mut BackgroundColor, &mut BorderColor),
        (Changed<Interaction>, With<SettingsMenuButton>),
    >,
    mut next: ResMut<NextState<AppState>>,
    theme: Res<UiTheme>,
) {
    for (interaction, mut bg, mut border) in &mut q {
        match *interaction {
            Interaction::Pressed => next.set(AppState::Settings),
            Interaction::Hovered => {
                *bg = BackgroundColor(Color::srgb(0.1, 0.12, 0.18));
                *border = BorderColor::all(Color::srgb(0.5, 0.55, 0.65));
            }
            Interaction::None => {
                let (b, d) = secondary_button_colors(&theme);
                *bg = b;
                *border = d;
            }
        }
    }
}
