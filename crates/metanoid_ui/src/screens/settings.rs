//! Full settings page (AppState::Settings) with working scroll + ASCII controls.

use bevy::ecs::relationship::RelatedSpawnerCommands;
use bevy::prelude::*;
use metanoid_core::settings::{GameSettings, ParticleQuality};
use metanoid_core::states::AppState;

use crate::scroll::ScrollArea;
use crate::theme::UiTheme;
use crate::widgets::{secondary_button_colors, spawn_label};

#[derive(Component)]
pub struct SettingsRoot;

#[derive(Component)]
pub struct SettingsBackButton;

#[derive(Component, Clone, Copy)]
pub enum SettingsAction {
    MasterDown,
    MasterUp,
    SfxDown,
    SfxUp,
    MusicDown,
    MusicUp,
    BloomDown,
    BloomUp,
    ShakeDown,
    ShakeUp,
    ToggleFps,
    ToggleReduceMotion,
    CycleParticles,
    ToggleLargeHud,
}

pub fn setup_settings(mut commands: Commands, settings: Res<GameSettings>, theme: Res<UiTheme>) {
    let (sbg, sbd) = secondary_button_colors(&theme);

    commands
        .spawn((
            SettingsRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(20.0)),
                row_gap: Val::Px(12.0),
                ..default()
            },
            BackgroundColor(Color::srgb(0.02, 0.02, 0.06)),
        ))
        .with_children(|root| {
            // Fixed header
            spawn_label(root, "SETTINGS", 40.0, theme.text_primary);
            spawn_label(
                root,
                "Audio / Video / Accessibility  |  scroll with mouse wheel",
                13.0,
                theme.text_muted,
            );

            // Scrollable body — constrained height so overflow can scroll
            root.spawn((
                ScrollArea,
                // Make the node interactable for hover-targeted scroll
                Interaction::default(),
                ScrollPosition::default(),
                Node {
                    width: Val::Percent(100.0),
                    max_width: Val::Px(560.0),
                    // Take remaining vertical space under header/footer
                    flex_grow: 1.0,
                    min_height: Val::Px(0.0),
                    height: Val::Px(0.0), // flex child constraint trick
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    row_gap: Val::Px(6.0),
                    padding: UiRect::all(Val::Px(12.0)),
                    overflow: Overflow::scroll_y(),
                    border: UiRect::all(Val::Px(1.0)),
                    border_radius: BorderRadius::all(Val::Px(10.0)),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.04, 0.05, 0.09)),
                BorderColor::all(theme.panel_border),
            ))
            .with_children(|scroll| {
                section(scroll, "AUDIO", &theme);
                stepper_row(
                    scroll,
                    "Master Volume",
                    format!("{:.0}%", settings.master_volume * 100.0),
                    SettingsAction::MasterDown,
                    SettingsAction::MasterUp,
                    &theme,
                );
                stepper_row(
                    scroll,
                    "SFX Volume",
                    format!("{:.0}%", settings.sfx_volume * 100.0),
                    SettingsAction::SfxDown,
                    SettingsAction::SfxUp,
                    &theme,
                );
                stepper_row(
                    scroll,
                    "Music Volume",
                    format!("{:.0}%", settings.music_volume * 100.0),
                    SettingsAction::MusicDown,
                    SettingsAction::MusicUp,
                    &theme,
                );

                section(scroll, "VIDEO", &theme);
                stepper_row(
                    scroll,
                    "Bloom",
                    format!("{:.0}%", settings.bloom_intensity * 100.0),
                    SettingsAction::BloomDown,
                    SettingsAction::BloomUp,
                    &theme,
                );
                stepper_row(
                    scroll,
                    "Screen Shake",
                    format!("{:.0}%", settings.shake_intensity * 100.0),
                    SettingsAction::ShakeDown,
                    SettingsAction::ShakeUp,
                    &theme,
                );
                toggle_row(
                    scroll,
                    "FPS Display",
                    if settings.show_fps { "ON" } else { "OFF" },
                    SettingsAction::ToggleFps,
                    &theme,
                );
                toggle_row(
                    scroll,
                    "Particle Quality",
                    settings.particle_quality.label(),
                    SettingsAction::CycleParticles,
                    &theme,
                );

                section(scroll, "ACCESSIBILITY", &theme);
                toggle_row(
                    scroll,
                    "Reduce Motion",
                    if settings.reduce_motion { "ON" } else { "OFF" },
                    SettingsAction::ToggleReduceMotion,
                    &theme,
                );
                toggle_row(
                    scroll,
                    "Large HUD",
                    if settings.large_hud { "ON" } else { "OFF" },
                    SettingsAction::ToggleLargeHud,
                    &theme,
                );

                section(scroll, "CONTROLS", &theme);
                spawn_label(
                    scroll,
                    "Move: Left/Right arrows or A/D",
                    14.0,
                    theme.text_muted,
                );
                spawn_label(scroll, "Launch ball: Space", 14.0, theme.text_muted);
                spawn_label(scroll, "Pause: Escape", 14.0, theme.text_muted);

                // Extra spacer so last rows can scroll fully into view
                scroll.spawn(Node {
                    height: Val::Px(40.0),
                    width: Val::Percent(100.0),
                    ..default()
                });
            });

            // Fixed footer
            root.spawn((
                SettingsBackButton,
                Button,
                Node {
                    width: Val::Px(200.0),
                    height: Val::Px(48.0),
                    border: UiRect::all(Val::Px(2.0)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                sbg,
                sbd,
            ))
            .with_children(|btn| {
                spawn_label(btn, "BACK", 20.0, theme.text_primary);
            });
        });
}

fn section(parent: &mut RelatedSpawnerCommands<ChildOf>, title: &str, theme: &UiTheme) {
    parent.spawn(Node {
        margin: UiRect::top(Val::Px(14.0)),
        width: Val::Percent(100.0),
        ..default()
    });
    spawn_label(parent, title, 18.0, theme.accent);
}

fn stepper_row(
    parent: &mut RelatedSpawnerCommands<ChildOf>,
    label: &str,
    value: String,
    down: SettingsAction,
    up: SettingsAction,
    theme: &UiTheme,
) {
    parent
        .spawn(Node {
            width: Val::Percent(100.0),
            max_width: Val::Px(480.0),
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::SpaceBetween,
            align_items: AlignItems::Center,
            column_gap: Val::Px(12.0),
            padding: UiRect::vertical(Val::Px(6.0)),
            ..default()
        })
        .with_children(|row| {
            spawn_label(row, label, 15.0, theme.text_primary);
            row.spawn(Node {
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(8.0),
                align_items: AlignItems::Center,
                ..default()
            })
            .with_children(|ctrl| {
                // ASCII only — default font has no fancy minus glyph
                mini_btn(ctrl, "-", down, theme);
                spawn_label(ctrl, value, 15.0, theme.accent);
                mini_btn(ctrl, "+", up, theme);
            });
        });
}

fn toggle_row(
    parent: &mut RelatedSpawnerCommands<ChildOf>,
    label: &str,
    value: &str,
    action: SettingsAction,
    theme: &UiTheme,
) {
    parent
        .spawn(Node {
            width: Val::Percent(100.0),
            max_width: Val::Px(480.0),
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::SpaceBetween,
            align_items: AlignItems::Center,
            padding: UiRect::vertical(Val::Px(6.0)),
            ..default()
        })
        .with_children(|row| {
            spawn_label(row, label, 15.0, theme.text_primary);
            mini_btn(row, value, action, theme);
        });
}

fn mini_btn(
    parent: &mut RelatedSpawnerCommands<ChildOf>,
    text: &str,
    action: SettingsAction,
    theme: &UiTheme,
) {
    parent
        .spawn((
            action,
            Button,
            Node {
                min_width: Val::Px(48.0),
                height: Val::Px(34.0),
                padding: UiRect::horizontal(Val::Px(10.0)),
                border: UiRect::all(Val::Px(1.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border_radius: BorderRadius::all(Val::Px(4.0)),
                ..default()
            },
            BackgroundColor(theme.panel),
            BorderColor::all(theme.panel_border),
        ))
        .with_children(|b| {
            spawn_label(b, text, 14.0, theme.text_primary);
        });
}

pub fn teardown_settings(mut commands: Commands, q: Query<Entity, With<SettingsRoot>>) {
    for e in &q {
        commands.entity(e).despawn();
    }
}

pub fn settings_back_interaction(
    mut q: Query<&Interaction, (Changed<Interaction>, With<SettingsBackButton>)>,
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

fn step(v: f32, delta: f32) -> f32 {
    (v + delta).clamp(0.0, 1.0)
}

fn step_bloom(v: f32, delta: f32) -> f32 {
    (v + delta).clamp(0.0, 1.5)
}

pub fn settings_action_interaction(
    mut q: Query<(&Interaction, &SettingsAction), Changed<Interaction>>,
    mut settings: ResMut<GameSettings>,
) {
    for (interaction, action) in &mut q {
        if *interaction != Interaction::Pressed {
            continue;
        }
        match action {
            SettingsAction::MasterDown => {
                settings.master_volume = step(settings.master_volume, -0.1)
            }
            SettingsAction::MasterUp => settings.master_volume = step(settings.master_volume, 0.1),
            SettingsAction::SfxDown => settings.sfx_volume = step(settings.sfx_volume, -0.1),
            SettingsAction::SfxUp => settings.sfx_volume = step(settings.sfx_volume, 0.1),
            SettingsAction::MusicDown => settings.music_volume = step(settings.music_volume, -0.1),
            SettingsAction::MusicUp => settings.music_volume = step(settings.music_volume, 0.1),
            SettingsAction::BloomDown => {
                settings.bloom_intensity = step_bloom(settings.bloom_intensity, -0.1)
            }
            SettingsAction::BloomUp => {
                settings.bloom_intensity = step_bloom(settings.bloom_intensity, 0.1)
            }
            SettingsAction::ShakeDown => {
                settings.shake_intensity = step(settings.shake_intensity, -0.1)
            }
            SettingsAction::ShakeUp => {
                settings.shake_intensity = step(settings.shake_intensity, 0.1)
            }
            SettingsAction::ToggleFps => settings.show_fps = !settings.show_fps,
            SettingsAction::ToggleReduceMotion => settings.reduce_motion = !settings.reduce_motion,
            SettingsAction::CycleParticles => {
                settings.particle_quality = settings.particle_quality.next()
            }
            SettingsAction::ToggleLargeHud => settings.large_hud = !settings.large_hud,
        }
    }
}

pub fn rebuild_settings_on_change(
    mut commands: Commands,
    settings: Res<GameSettings>,
    theme: Res<UiTheme>,
    roots: Query<Entity, With<SettingsRoot>>,
) {
    if !settings.is_changed() {
        return;
    }
    if roots.is_empty() {
        return;
    }
    for e in &roots {
        commands.entity(e).despawn();
    }
    setup_settings(commands, settings, theme);
}

#[allow(dead_code)]
fn _pq() -> ParticleQuality {
    ParticleQuality::Medium
}
