//! Small Bevy UI helpers.

use bevy::ecs::relationship::RelatedSpawnerCommands;
use bevy::prelude::*;

use crate::theme::UiTheme;

pub fn spawn_label(
    parent: &mut RelatedSpawnerCommands<ChildOf>,
    text: impl Into<String>,
    size: f32,
    color: Color,
) {
    parent.spawn((
        Text::new(text.into()),
        TextFont {
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
