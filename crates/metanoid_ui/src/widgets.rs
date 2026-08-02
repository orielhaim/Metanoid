//! Small Bevy UI helpers.

use std::sync::OnceLock;
use std::time::Duration;

use bevy::ecs::relationship::RelatedSpawnerCommands;
use bevy::math::curve::EaseFunction;
use bevy::prelude::*;
use bevy_tweening::lens::{UiTransformScaleLens, UiTransformTranslationPxLens};
use bevy_tweening::{Delay, Tween, TweenAnim};

use crate::theme::UiTheme;

/// Loaded fonts, set once during plugin build.
static BODY_FONT: OnceLock<Handle<Font>> = OnceLock::new();
static TITLE_FONT: OnceLock<Handle<Font>> = OnceLock::new();

/// Register the UI fonts (called from the UI plugin build).
pub fn set_fonts(title: Handle<Font>, body: Handle<Font>) {
    let _ = TITLE_FONT.set(title);
    let _ = BODY_FONT.set(body);
}

fn font_source() -> FontSource {
    BODY_FONT
        .get()
        .cloned()
        .map(FontSource::Handle)
        .unwrap_or_default()
}

fn title_font_source() -> FontSource {
    TITLE_FONT
        .get()
        .cloned()
        .map(FontSource::Handle)
        .unwrap_or_else(font_source)
}

/// Public accessors so raw `Text` spawns (HUD, overlays) use the loaded fonts.
pub fn body_font() -> FontSource {
    font_source()
}

pub fn title_font() -> FontSource {
    title_font_source()
}

/// Entrance animation: scale a UI node up from `from` after `delay`.
pub fn entrance_scale(delay: f32, from: f32) -> TweenAnim {
    TweenAnim::new(Delay::new(Duration::from_secs_f32(delay)).then(Tween::new(
        EaseFunction::BackOut,
        Duration::from_millis(400),
        UiTransformScaleLens {
            start: Vec2::splat(from),
            end: Vec2::splat(1.0),
        },
    )))
}

/// Entrance animation that also slides a UI node up into place.
pub fn entrance_rise(delay: f32, from_px: f32) -> TweenAnim {
    TweenAnim::new(Delay::new(Duration::from_secs_f32(delay)).then(Tween::new(
        EaseFunction::BackOut,
        Duration::from_millis(450),
        UiTransformTranslationPxLens {
            start: Vec2::new(0.0, from_px),
            end: Vec2::ZERO,
        },
    )))
}

pub fn spawn_label(
    parent: &mut RelatedSpawnerCommands<ChildOf>,
    text: impl Into<String>,
    size: f32,
    color: Color,
) {
    parent.spawn((
        Text::new(text.into()),
        TextFont {
            font: font_source(),
            font_size: FontSize::Px(size),
            ..default()
        },
        TextColor(color),
    ));
}

/// Label using the display/title font (for headings and the game title).
pub fn spawn_title(
    parent: &mut RelatedSpawnerCommands<ChildOf>,
    text: impl Into<String>,
    size: f32,
    color: Color,
) {
    parent.spawn((
        Text::new(text.into()),
        TextFont {
            font: title_font_source(),
            font_size: FontSize::Px(size),
            ..default()
        },
        TextColor(color),
    ));
}

pub fn styled_button_node(width: f32, height: f32) -> Node {
    Node {
        width: Val::Px(width),
        height: Val::Px(height),
        border: UiRect::all(Val::Px(2.0)),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
    }
}

pub fn primary_button_colors(theme: &UiTheme) -> (BackgroundColor, BorderColor) {
    (
        BackgroundColor(Color::srgb(0.08, 0.12, 0.2)),
        BorderColor::all(theme.accent),
    )
}

pub fn secondary_button_colors(theme: &UiTheme) -> (BackgroundColor, BorderColor) {
    (
        BackgroundColor(theme.panel),
        BorderColor::all(theme.panel_border),
    )
}
