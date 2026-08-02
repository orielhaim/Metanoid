//! Shared UI theme tokens.

use bevy::prelude::*;
use metanoid_core::rating::Grade;

#[derive(Resource, Clone, Debug)]
pub struct UiTheme {
    pub bg_deep: Color,
    pub panel: Color,
    pub panel_border: Color,
    pub text_primary: Color,
    pub text_muted: Color,
    pub accent: Color,
    pub danger: Color,
    pub success: Color,
    pub font_title: f32,
    pub font_body: f32,
    pub font_small: f32,
}

impl Default for UiTheme {
    fn default() -> Self {
        Self {
            bg_deep: Color::srgb(0.01, 0.01, 0.04),
            panel: Color::srgb(0.06, 0.08, 0.12),
            panel_border: Color::srgb(0.25, 0.35, 0.5),
            text_primary: Color::srgb(0.9, 0.95, 1.0),
            text_muted: Color::srgb(0.5, 0.55, 0.65),
            accent: Color::srgb(0.3, 0.7, 1.0),
            danger: Color::srgb(1.0, 0.35, 0.35),
            success: Color::srgb(0.3, 1.0, 0.5),
            font_title: 72.0,
            font_body: 20.0,
            font_small: 14.0,
        }
    }
}

pub fn grade_color(grade: Grade) -> Color {
    match grade {
        Grade::D => Color::srgb(0.45, 0.45, 0.5),
        Grade::C => Color::srgb(0.55, 0.6, 0.7),
        Grade::B => Color::srgb(0.3, 0.75, 0.75),
        Grade::A => Color::srgb(0.95, 0.8, 0.25),
        Grade::S => Color::srgb(1.0, 0.75, 0.45),
        Grade::Ss => Color::srgb(1.0, 0.95, 0.9),
    }
}

pub fn rating_color(rating: u8) -> Color {
    grade_color(metanoid_core::grade_from_rating(rating))
}
