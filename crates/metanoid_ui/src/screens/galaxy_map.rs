//! Interactive galaxy map — constellation path UI, ASCII-safe labels, biome color theming.

use bevy::prelude::*;
use metanoid_core::rating::grade_from_rating;
use metanoid_core::resources::game_state::GameState;
use metanoid_core::save_data::{SaveData, is_level_unlocked, level_key, mastery_percent};
use metanoid_core::settings::LevelLaunchMode;
use metanoid_core::states::AppState;
use metanoid_procgen::biome::generator::BiomeGenerator;
use metanoid_procgen::biome::palette::ProceduralPalette;
use metanoid_procgen::biome::theme::BiomeTheme;
use metanoid_procgen::seed::hierarchy::MasterSeed;
use metanoid_procgen::universe::galaxy::GalaxyDefinition;
use metanoid_procgen::universe::progression::LEVELS_PER_BIOME;

use crate::scroll::ScrollArea;
use crate::theme::{UiTheme, grade_color};
use crate::widgets::{primary_button_colors, secondary_button_colors, spawn_label, spawn_title};

#[derive(Component)]
pub struct GalaxyMapRoot;

#[derive(Component)]
pub struct MapBackButton;

#[derive(Component)]
pub struct PlayChallengeButton;

#[derive(Component)]
pub struct PlayCampaignButton;

#[derive(Component, Clone, Copy)]
pub struct LevelNodeButton {
    pub galaxy: u64,
    pub biome: u64,
    pub level: u64,
    pub unlocked: bool,
    pub is_boss: bool,
}

/// Pulsing scale animation for the currently selected node.
#[derive(Component)]
pub struct NodePulse;

/// Gentle pulse on the selected node.
pub fn tick_node_pulse(time: Res<Time>, mut q: Query<&mut UiTransform, With<NodePulse>>) {
    let t = time.elapsed_secs();
    for mut transform in &mut q {
        let s = 1.0 + (t * 3.0).sin() * 0.08;
        transform.scale = Vec2::splat(s);
    }
}

#[derive(Component, Clone, Copy)]
pub struct GalaxyTabButton {
    pub galaxy: u64,
    pub unlocked: bool,
}

#[derive(Resource, Debug, Clone)]
pub struct GalaxyMapSelection {
    pub galaxy: u64,
    pub biome: u64,
    pub level: u64,
}

impl Default for GalaxyMapSelection {
    fn default() -> Self {
        Self {
            galaxy: 0,
            biome: 0,
            level: 0,
        }
    }
}

#[derive(Resource, Default)]
pub struct GalaxyMapNeedsRebuild(pub bool);

fn hsl_color(h: f32, s: f32, l: f32) -> Color {
    let (r, g, b) = metanoid_procgen::biome::palette::Hsl::new(h, s, l).to_rgb();
    Color::srgb(r, g, b)
}

fn palette_primary(p: &ProceduralPalette) -> Color {
    hsl_color(p.primary.h, p.primary.s, p.primary.l)
}

fn palette_glow(p: &ProceduralPalette) -> Color {
    hsl_color(p.glow.h, p.glow.s, p.glow.l.min(0.75))
}

fn palette_bg(p: &ProceduralPalette) -> Color {
    hsl_color(
        p.background.h,
        p.background.s.max(0.15),
        p.background.l.max(0.06),
    )
}

fn biome_flavor_name(params: &metanoid_procgen::biome::parameters::BiomeParams) -> &'static str {
    if params.temperature > 0.85 {
        "Volcanic Core"
    } else if params.temperature > 0.65 {
        "Neon Reach"
    } else if params.temperature < 0.15 {
        "Arctic Drift"
    } else if params.temperature < 0.3 {
        "Crystal Depths"
    } else if params.weirdness > 0.7 {
        "Cosmic Void"
    } else if params.density > 0.7 {
        "Dense Nebula"
    } else if params.energy > 0.75 {
        "Pulse Field"
    } else if params.chaos > 0.6 {
        "Chaos Belt"
    } else {
        "Starlane"
    }
}

fn mastery_bar(parent: &mut ChildSpawnerCommands, mastery: f32, theme: &UiTheme) {
    let pct = mastery.clamp(0.0, 100.0);
    parent
        .spawn(Node {
            width: Val::Px(180.0),
            height: Val::Px(10.0),
            border_radius: BorderRadius::all(Val::Px(5.0)),
            overflow: Overflow::clip(),
            ..default()
        })
        .insert(BackgroundColor(Color::srgb(0.08, 0.1, 0.14)))
        .with_children(|bar| {
            bar.spawn((
                Node {
                    width: Val::Percent(pct),
                    height: Val::Percent(100.0),
                    border_radius: BorderRadius::all(Val::Px(5.0)),
                    ..default()
                },
                BackgroundColor(if pct >= 80.0 {
                    theme.success
                } else if pct >= 50.0 {
                    theme.accent
                } else {
                    Color::srgb(0.5, 0.45, 0.2)
                }),
            ));
        });
}

// ChildSpawnerCommands alias for with_children in Bevy 0.19
use bevy::ecs::relationship::RelatedSpawnerCommands;
type ChildSpawnerCommands<'a> = RelatedSpawnerCommands<'a, ChildOf>;

pub fn sync_map_selection_from_save(
    save: Res<SaveData>,
    mut selection: ResMut<GalaxyMapSelection>,
) {
    selection.galaxy = save.highest_galaxy;
    selection.biome = save.highest_biome;
    selection.level = save.highest_level;
}

pub fn setup_galaxy_map(
    mut commands: Commands,
    save: Res<SaveData>,
    theme: Res<UiTheme>,
    selection: Res<GalaxyMapSelection>,
) {
    let master = MasterSeed::new(save.master_seed);
    let display_galaxies = (save.highest_galaxy + 2).max(3).min(8);
    let sel = selection.clone();
    let galaxy_def = GalaxyDefinition::generate(master.galaxy(sel.galaxy));
    let mastery = mastery_percent(&save);

    let key = level_key(sel.galaxy, sel.biome, sel.level);
    let pb = save.level_results.get(&key);
    let unlocked = is_level_unlocked(
        sel.galaxy,
        sel.biome,
        sel.level,
        save.highest_galaxy,
        save.highest_biome,
        save.highest_level,
    );
    let is_boss = sel.level + 1 == LEVELS_PER_BIOME;
    let is_frontier = sel.galaxy == save.highest_galaxy
        && sel.biome == save.highest_biome
        && sel.level == save.highest_level;

    // Selected biome palette for accent theming
    let sel_biome_params = BiomeGenerator::generate(master.galaxy(sel.galaxy).biome(sel.biome));
    let sel_theme = BiomeTheme::generate(&sel_biome_params);
    let accent = palette_glow(&sel_theme.palette);
    let panel_bg = palette_bg(&sel_theme.palette);

    commands
        .spawn((
            GalaxyMapRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(14.0)),
                row_gap: Val::Px(10.0),
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
            .insert(BackgroundColor(Color::srgba(0.01, 0.012, 0.05, 0.62)));

            // Decorative starfield (pure colored squares — no glyphs)
            root.spawn(Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                ..default()
            })
            .with_children(|stars| {
                for i in 0..48 {
                    let x = ((i * 47) % 97) as f32 + 1.5;
                    let y = ((i * 31) % 93) as f32 + 2.0;
                    let size = 1.5 + (i % 3) as f32;
                    let bright = 0.25 + (i % 5) as f32 * 0.08;
                    stars.spawn((
                        Node {
                            position_type: PositionType::Absolute,
                            left: Val::Percent(x),
                            top: Val::Percent(y),
                            width: Val::Px(size),
                            height: Val::Px(size),
                            border_radius: BorderRadius::all(Val::Px(size)),
                            ..default()
                        },
                        BackgroundColor(Color::srgba(bright, bright, bright + 0.1, 0.7)),
                    ));
                }
            });

            // Header
            root.spawn(Node {
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                padding: UiRect::horizontal(Val::Px(8.0)),
                column_gap: Val::Px(16.0),
                ..default()
            })
            .with_children(|h| {
                h.spawn(Node {
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(2.0),
                    ..default()
                })
                .with_children(|t| {
                    spawn_title(t, "GALAXY MAP", 30.0, theme.text_primary);
                    spawn_label(
                        t,
                        format!(
                            "Seed {}  |  Sector G{}",
                            save.master_seed,
                            sel.galaxy + 1
                        ),
                        13.0,
                        theme.text_muted,
                    );
                });

                h.spawn(Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::FlexEnd,
                    row_gap: Val::Px(4.0),
                    ..default()
                })
                .with_children(|m| {
                    spawn_label(m, format!("Mastery {:.0}%", mastery), 16.0, accent);
                    mastery_bar(m, mastery, &theme);
                });
            });

            // Galaxy tabs
            root.spawn(Node {
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(8.0),
                flex_wrap: FlexWrap::Wrap,
                padding: UiRect::horizontal(Val::Px(4.0)),
                ..default()
            })
            .with_children(|tabs| {
                for g in 0..display_galaxies {
                    let g_unlocked = g <= save.highest_galaxy;
                    let selected = g == sel.galaxy;
                    let label = if g_unlocked {
                        format!("GALAXY {}", g + 1)
                    } else {
                        format!("G{} LOCKED", g + 1)
                    };
                    let border = if selected {
                        accent
                    } else if g_unlocked {
                        theme.panel_border
                    } else {
                        Color::srgb(0.18, 0.18, 0.22)
                    };
                    tabs.spawn((
                        GalaxyTabButton {
                            galaxy: g,
                            unlocked: g_unlocked,
                        },
                        Button,
                        Node {
                            min_width: Val::Px(96.0),
                            height: Val::Px(38.0),
                            padding: UiRect::horizontal(Val::Px(12.0)),
                            border: UiRect::all(Val::Px(2.0)),
                            border_radius: BorderRadius::all(Val::Px(8.0)),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BackgroundColor(if selected {
                            Color::srgb(0.1, 0.14, 0.26)
                        } else {
                            Color::srgb(0.05, 0.06, 0.1)
                        }),
                        BorderColor::all(border),
                    ))
                    .with_children(|b| {
                        spawn_label(
                            b,
                            label,
                            13.0,
                            if g_unlocked {
                                theme.text_primary
                            } else {
                                theme.text_muted
                            },
                        );
                    });
                }
            });

            // Difficulty strip for current galaxy
            root.spawn(Node {
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::SpaceBetween,
                padding: UiRect::horizontal(Val::Px(8.0)),
                ..default()
            })
            .with_children(|strip| {
                spawn_label(
                    strip,
                    format!(
                        "Biomes: {}  |  Base difficulty: {:.0}%  |  Ball speed bias: {:.0}",
                        galaxy_def.biome_count,
                        galaxy_def.base_difficulty * 100.0,
                        galaxy_def.base_ball_speed
                    ),
                    12.0,
                    theme.text_muted,
                );
            });

            // Body: scrollable biomes + detail
            root.spawn(Node {
                width: Val::Percent(100.0),
                flex_grow: 1.0,
                min_height: Val::Px(0.0),
                height: Val::Px(0.0),
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(14.0),
                ..default()
            })
            .with_children(|body| {
                // Left scroll: biome constellations
                body.spawn((
                    ScrollArea,
                    Interaction::default(),
                    ScrollPosition::default(),
                    Node {
                        flex_grow: 1.0,
                        min_width: Val::Px(0.0),
                        height: Val::Percent(100.0),
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(14.0),
                        padding: UiRect::all(Val::Px(12.0)),
                        overflow: Overflow::scroll_y(),
                        border: UiRect::all(Val::Px(1.0)),
                        border_radius: BorderRadius::all(Val::Px(12.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.03, 0.04, 0.08, 0.92)),
                    BorderColor::all(Color::srgb(0.2, 0.28, 0.4)),
                ))
                .with_children(|list| {
                    let biome_count = if sel.galaxy <= save.highest_galaxy {
                        galaxy_def.biome_count
                    } else {
                        0
                    };

                    for b in 0..biome_count.max(1) {
                        let biome_unlocked = sel.galaxy < save.highest_galaxy
                            || (sel.galaxy == save.highest_galaxy
                                && (b as u64) <= save.highest_biome);

                        let params =
                            BiomeGenerator::generate(master.galaxy(sel.galaxy).biome(b as u64));
                        let btheme = BiomeTheme::generate(&params);
                        let primary = palette_primary(&btheme.palette);
                        let glow = palette_glow(&btheme.palette);
                        let name = biome_flavor_name(&params);

                        // Biome card
                        list.spawn((
                            Node {
                                flex_direction: FlexDirection::Column,
                                row_gap: Val::Px(10.0),
                                padding: UiRect::all(Val::Px(12.0)),
                                border: UiRect {
                                    left: Val::Px(4.0),
                                    ..UiRect::all(Val::Px(1.0))
                                },
                                border_radius: BorderRadius::all(Val::Px(10.0)),
                                ..default()
                            },
                            BackgroundColor(if biome_unlocked {
                                Color::srgba(0.05, 0.06, 0.1, 0.95)
                            } else {
                                Color::srgb(0.03, 0.03, 0.05)
                            }),
                            BorderColor::all(if biome_unlocked {
                                primary
                            } else {
                                Color::srgb(0.15, 0.15, 0.18)
                            }),
                        ))
                        .with_children(|card| {
                            // Title row
                            card.spawn(Node {
                                flex_direction: FlexDirection::Row,
                                justify_content: JustifyContent::SpaceBetween,
                                align_items: AlignItems::Center,
                                width: Val::Percent(100.0),
                                ..default()
                            })
                            .with_children(|title| {
                                let title_text = if biome_unlocked {
                                    format!("BIOME {}  -  {}", b + 1, name)
                                } else {
                                    format!("BIOME {}  -  LOCKED", b + 1)
                                };
                                spawn_label(
                                    title,
                                    title_text,
                                    15.0,
                                    if biome_unlocked { glow } else { theme.text_muted },
                                );
                                if biome_unlocked {
                                    // Mini progress: cleared count in biome
                                    let cleared = (0..LEVELS_PER_BIOME)
                                        .filter(|l| {
                                            save.level_results
                                                .get(&level_key(sel.galaxy, b as u64, *l))
                                                .map(|p| p.clears > 0)
                                                .unwrap_or(false)
                                        })
                                        .count();
                                    spawn_label(
                                        title,
                                        format!("{cleared}/{} cleared", LEVELS_PER_BIOME),
                                        12.0,
                                        theme.text_muted,
                                    );
                                }
                            });

                            if !biome_unlocked {
                                spawn_label(
                                    card,
                                    "Clear the previous biome boss to open this sector.",
                                    12.0,
                                    theme.text_muted,
                                );
                                return;
                            }

                            // Level path
                            card.spawn(Node {
                                flex_direction: FlexDirection::Row,
                                flex_wrap: FlexWrap::Wrap,
                                column_gap: Val::Px(6.0),
                                row_gap: Val::Px(8.0),
                                align_items: AlignItems::Center,
                                ..default()
                            })
                            .with_children(|nodes| {
                                for l in 0..LEVELS_PER_BIOME {
                                    let lvl_unlocked = is_level_unlocked(
                                        sel.galaxy,
                                        b as u64,
                                        l,
                                        save.highest_galaxy,
                                        save.highest_biome,
                                        save.highest_level,
                                    );
                                    let boss = l + 1 == LEVELS_PER_BIOME;
                                    let lk = level_key(sel.galaxy, b as u64, l);
                                    let best = save.level_results.get(&lk);
                                    let selected_node =
                                        sel.biome == b as u64 && sel.level == l;

                                    let (label, fill, border_c, text_c) = if !lvl_unlocked {
                                        (
                                            if boss { "BOSS".into() } else { format!("L{:02}", l + 1) },
                                            Color::srgb(0.06, 0.06, 0.08),
                                            Color::srgb(0.2, 0.2, 0.24),
                                            theme.text_muted,
                                        )
                                    } else if let Some(pb) = best.filter(|p| p.clears > 0) {
                                        let gc = grade_color(grade_from_rating(pb.best_rating));
                                        (
                                            if boss {
                                                format!("B{}", pb.best_rating)
                                            } else {
                                                format!("{}", pb.best_rating)
                                            },
                                            Color::srgb(0.08, 0.1, 0.14),
                                            gc,
                                            gc,
                                        )
                                    } else {
                                        (
                                            if boss {
                                                "BOSS".into()
                                            } else {
                                                format!("L{:02}", l + 1)
                                            },
                                            Color::srgb(0.07, 0.1, 0.16),
                                            if boss { Color::srgb(1.0, 0.55, 0.2) } else { glow },
                                            if boss {
                                                Color::srgb(1.0, 0.7, 0.35)
                                            } else {
                                                theme.text_primary
                                            },
                                        )
                                    };

                                    let border_w = if selected_node {
                                        3.0
                                    } else if boss {
                                        2.5
                                    } else {
                                        2.0
                                    };
                                    let size = if boss { 52.0 } else { 42.0 };

                                    let mut node = nodes.spawn((
                                        LevelNodeButton {
                                            galaxy: sel.galaxy,
                                            biome: b as u64,
                                            level: l,
                                            unlocked: lvl_unlocked,
                                            is_boss: boss,
                                        },
                                        Button,
                                        Node {
                                            width: Val::Px(size),
                                            height: Val::Px(size),
                                            border: UiRect::all(Val::Px(border_w)),
                                            border_radius: BorderRadius::all(Val::Px(
                                                if boss { 10.0 } else { size / 2.0 },
                                            )),
                                            justify_content: JustifyContent::Center,
                                            align_items: AlignItems::Center,
                                            ..default()
                                        },
                                        BackgroundColor(if selected_node {
                                            Color::srgb(0.12, 0.16, 0.28)
                                        } else {
                                            fill
                                        }),
                                        BorderColor::all(if selected_node {
                                            theme.accent
                                        } else {
                                            border_c
                                        }),
                                    ));
                                    if selected_node {
                                        node.insert(NodePulse);
                                    }
                                    node.with_children(|n| {
                                            spawn_label(n, label, if boss { 11.0 } else { 12.0 }, text_c);
                                        });

                                    // Connector dash between nodes (visual path)
                                    if l + 1 < LEVELS_PER_BIOME {
                                        let (pr, pg, pb) = btheme.palette.primary.to_rgb();
                                        nodes.spawn((
                                            Node {
                                                width: Val::Px(10.0),
                                                height: Val::Px(3.0),
                                                border_radius: BorderRadius::all(Val::Px(2.0)),
                                                ..default()
                                            },
                                            BackgroundColor(if lvl_unlocked {
                                                Color::srgba(pr, pg, pb, 0.45)
                                            } else {
                                                Color::srgb(0.15, 0.15, 0.18)
                                            }),
                                        ));
                                    }
                                }
                            });
                        });
                    }

                    if biome_count == 0 {
                        spawn_label(
                            list,
                            "This galaxy is locked. Push the frontier to explore deeper space.",
                            14.0,
                            theme.text_muted,
                        );
                    }

                    list.spawn(Node {
                        height: Val::Px(24.0),
                        ..default()
                    });
                });

                // Right detail panel
                body.spawn((
                    Node {
                        width: Val::Px(300.0),
                        height: Val::Percent(100.0),
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(10.0),
                        padding: UiRect::all(Val::Px(16.0)),
                        border: UiRect::all(Val::Px(2.0)),
                        border_radius: BorderRadius::all(Val::Px(12.0)),
                        ..default()
                    },
                    BackgroundColor(panel_bg),
                    BorderColor::all(accent),
                ))
                .with_children(|detail| {
                    // Status chip
                    detail.spawn((
                        Node {
                            padding: UiRect::axes(Val::Px(10.0), Val::Px(4.0)),
                            border_radius: BorderRadius::all(Val::Px(4.0)),
                            ..default()
                        },
                        BackgroundColor(if !unlocked {
                            Color::srgb(0.2, 0.12, 0.12)
                        } else if pb.map(|p| p.clears > 0).unwrap_or(false) {
                            Color::srgb(0.1, 0.2, 0.14)
                        } else {
                            Color::srgb(0.1, 0.14, 0.22)
                        }),
                    ))
                    .with_children(|chip| {
                        let status = if !unlocked {
                            "LOCKED"
                        } else if pb.map(|p| p.clears > 0).unwrap_or(false) {
                            "CLEARED"
                        } else if is_frontier {
                            "NEXT UP"
                        } else {
                            "READY"
                        };
                        spawn_label(chip, status, 12.0, theme.text_primary);
                    });

                    let title = if is_boss {
                        format!(
                            "BOSS  G{} / B{} / L{}",
                            sel.galaxy + 1,
                            sel.biome + 1,
                            sel.level + 1
                        )
                    } else {
                        format!(
                            "LEVEL  G{} / B{} / L{}",
                            sel.galaxy + 1,
                            sel.biome + 1,
                            sel.level + 1
                        )
                    };
                    spawn_label(detail, title, 20.0, theme.text_primary);
                    spawn_label(
                        detail,
                        format!("Sector: {}", biome_flavor_name(&sel_biome_params)),
                        13.0,
                        accent,
                    );

                    // Divider
                    detail.spawn((
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Px(1.0),
                            margin: UiRect::vertical(Val::Px(4.0)),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.25, 0.3, 0.4)),
                    ));

                    if let Some(pb) = pb {
                        if pb.clears > 0 {
                            let g = grade_from_rating(pb.best_rating);
                            spawn_label(
                                detail,
                                format!("Best rating: {}  ({})", pb.best_rating, g.as_str()),
                                18.0,
                                grade_color(g),
                            );
                            spawn_label(
                                detail,
                                format!("Best score: {}", pb.best_score),
                                14.0,
                                theme.text_primary,
                            );
                            spawn_label(
                                detail,
                                format!(
                                    "Max combo: {}  |  Best time: {:.0}s",
                                    pb.best_max_combo,
                                    if pb.best_time_secs.is_finite() {
                                        pb.best_time_secs
                                    } else {
                                        0.0
                                    }
                                ),
                                12.0,
                                theme.text_muted,
                            );
                            spawn_label(
                                detail,
                                format!("Attempts: {}  |  Clears: {}", pb.attempts, pb.clears),
                                12.0,
                                theme.text_muted,
                            );
                        } else {
                            spawn_label(detail, "No clear yet - set your first rating!", 14.0, theme.text_muted);
                        }
                    } else {
                        spawn_label(detail, "No clear yet - set your first rating!", 14.0, theme.text_muted);
                    }

                    spawn_label(
                        detail,
                        format!(
                            "Galaxy difficulty base: {:.0}%",
                            galaxy_def.base_difficulty * 100.0
                        ),
                        12.0,
                        theme.text_muted,
                    );

                    if unlocked {
                        let (pbg, pbd) = primary_button_colors(&theme);
                        detail
                            .spawn((
                                PlayChallengeButton,
                                Button,
                                Node {
                                    width: Val::Percent(100.0),
                                    height: Val::Px(46.0),
                                    border: UiRect::all(Val::Px(2.0)),
                                    border_radius: BorderRadius::all(Val::Px(8.0)),
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    margin: UiRect::top(Val::Px(12.0)),
                                    ..default()
                                },
                                pbg,
                                pbd,
                            ))
                            .with_children(|b| {
                                spawn_label(b, "PLAY CHALLENGE", 16.0, theme.text_primary);
                            });

                        if is_frontier {
                            let (sbg, sbd) = secondary_button_colors(&theme);
                            detail
                                .spawn((
                                    PlayCampaignButton,
                                    Button,
                                    Node {
                                        width: Val::Percent(100.0),
                                        height: Val::Px(42.0),
                                        border: UiRect::all(Val::Px(2.0)),
                                        border_radius: BorderRadius::all(Val::Px(8.0)),
                                        justify_content: JustifyContent::Center,
                                        align_items: AlignItems::Center,
                                        margin: UiRect::top(Val::Px(8.0)),
                                        ..default()
                                    },
                                    sbg,
                                    sbd,
                                ))
                                .with_children(|b| {
                                    spawn_label(b, "PLAY CAMPAIGN", 15.0, theme.text_primary);
                                });
                        }

                        spawn_label(
                            detail,
                            "Challenge: fresh 3 lives, hunt a better rating.",
                            11.0,
                            theme.text_muted,
                        );
                    } else {
                        spawn_label(
                            detail,
                            "Clear prior levels to unlock this node.",
                            13.0,
                            theme.danger,
                        );
                    }
                });
            });

            // Footer
            root.spawn(Node {
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                padding: UiRect::horizontal(Val::Px(4.0)),
                ..default()
            })
            .with_children(|foot| {
                let (sbg, sbd) = secondary_button_colors(&theme);
                foot.spawn((
                    MapBackButton,
                    Button,
                    Node {
                        width: Val::Px(140.0),
                        height: Val::Px(40.0),
                        border: UiRect::all(Val::Px(2.0)),
                        border_radius: BorderRadius::all(Val::Px(6.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    sbg,
                    sbd,
                ))
                .with_children(|b| {
                    spawn_label(b, "< MENU", 16.0, theme.text_primary);
                });
                spawn_label(
                    foot,
                    "Nodes show best rating  |  BOSS is the last of each biome  |  Scroll biomes with wheel",
                    11.0,
                    theme.text_muted,
                );
            });
        });
}

pub fn teardown_galaxy_map(mut commands: Commands, q: Query<Entity, With<GalaxyMapRoot>>) {
    for e in &q {
        commands.entity(e).despawn();
    }
}

pub fn galaxy_tab_interaction(
    mut q: Query<(&Interaction, &GalaxyTabButton), Changed<Interaction>>,
    mut selection: ResMut<GalaxyMapSelection>,
) {
    for (interaction, tab) in &mut q {
        if *interaction != Interaction::Pressed || !tab.unlocked {
            continue;
        }
        selection.galaxy = tab.galaxy;
        selection.biome = 0;
        selection.level = 0;
    }
}

pub fn level_node_interaction(
    mut q: Query<(&Interaction, &LevelNodeButton), Changed<Interaction>>,
    mut selection: ResMut<GalaxyMapSelection>,
) {
    for (interaction, node) in &mut q {
        if *interaction != Interaction::Pressed || !node.unlocked {
            continue;
        }
        selection.galaxy = node.galaxy;
        selection.biome = node.biome;
        selection.level = node.level;
    }
}

pub fn request_rebuild_on_selection_change(
    selection: Res<GalaxyMapSelection>,
    mut flag: ResMut<GalaxyMapNeedsRebuild>,
) {
    if selection.is_changed() {
        flag.0 = true;
    }
}

pub fn apply_galaxy_map_rebuild(
    mut commands: Commands,
    mut flag: ResMut<GalaxyMapNeedsRebuild>,
    save: Res<SaveData>,
    theme: Res<UiTheme>,
    selection: Res<GalaxyMapSelection>,
    roots: Query<Entity, With<GalaxyMapRoot>>,
) {
    if !flag.0 {
        return;
    }
    if roots.is_empty() {
        flag.0 = false;
        return;
    }
    flag.0 = false;
    for e in &roots {
        commands.entity(e).despawn();
    }
    setup_galaxy_map(commands, save, theme, selection);
}

pub fn map_back_interaction(
    mut q: Query<&Interaction, (Changed<Interaction>, With<MapBackButton>)>,
    mut next: ResMut<NextState<AppState>>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    if keyboard.just_pressed(KeyCode::Escape) {
        next.set(AppState::Menu);
        return;
    }
    for interaction in &mut q {
        if *interaction == Interaction::Pressed {
            next.set(AppState::Menu);
        }
    }
}

pub fn play_challenge_interaction(
    mut q: Query<&Interaction, (Changed<Interaction>, With<PlayChallengeButton>)>,
    selection: Res<GalaxyMapSelection>,
    save: Res<SaveData>,
    mut commands: Commands,
    mut next: ResMut<NextState<AppState>>,
) {
    for interaction in &mut q {
        if *interaction == Interaction::Pressed {
            let master = MasterSeed::new(save.master_seed);
            let galaxy_def = GalaxyDefinition::generate(master.galaxy(selection.galaxy));
            let mut state = GameState::new(save.master_seed);
            state.galaxy = selection.galaxy;
            state.biome = selection.biome;
            state.level = selection.level;
            state.biome_count = galaxy_def.biome_count;
            state.lives = 3;
            state.score = 0;
            commands.insert_resource(state);
            commands.insert_resource(LevelLaunchMode::Challenge);
            next.set(AppState::Loading);
        }
    }
}

pub fn play_campaign_interaction(
    mut q: Query<&Interaction, (Changed<Interaction>, With<PlayCampaignButton>)>,
    selection: Res<GalaxyMapSelection>,
    save: Res<SaveData>,
    mut game_state: Option<ResMut<GameState>>,
    mut commands: Commands,
    mut next: ResMut<NextState<AppState>>,
) {
    for interaction in &mut q {
        if *interaction == Interaction::Pressed {
            let master = MasterSeed::new(save.master_seed);
            let galaxy_def = GalaxyDefinition::generate(master.galaxy(selection.galaxy));
            if let Some(ref mut s) = game_state {
                s.master_seed = save.master_seed;
                s.galaxy = selection.galaxy;
                s.biome = selection.biome;
                s.level = selection.level;
                s.biome_count = galaxy_def.biome_count;
                if s.lives <= 0 {
                    s.lives = 3;
                }
            } else {
                let mut state = GameState::new(save.master_seed);
                state.galaxy = selection.galaxy;
                state.biome = selection.biome;
                state.level = selection.level;
                state.biome_count = galaxy_def.biome_count;
                state.lives = 3;
                commands.insert_resource(state);
            }
            commands.insert_resource(LevelLaunchMode::Campaign);
            next.set(AppState::Loading);
        }
    }
}
