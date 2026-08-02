//! Mouse-wheel scrolling for Bevy UI panels with Overflow::scroll_y.

use bevy::input::mouse::{MouseScrollUnit, MouseWheel};
use bevy::prelude::*;

/// Mark a Node that has `overflow: scroll_y` and should receive wheel input.
#[derive(Component)]
pub struct ScrollArea;

pub fn mouse_wheel_scroll(
    mut wheel: MessageReader<MouseWheel>,
    mut areas: Query<(&Interaction, &mut ScrollPosition), With<ScrollArea>>,
) {
    let mut dy = 0.0_f32;
    for ev in wheel.read() {
        dy += match ev.unit {
            MouseScrollUnit::Line => ev.y * 32.0,
            MouseScrollUnit::Pixel => ev.y,
        };
    }
    if dy.abs() < f32::EPSILON {
        return;
    }

    // Prefer hovered scroll areas; otherwise scroll all active ones.
    let mut any_hovered = false;
    for (interaction, mut pos) in &mut areas {
        if *interaction == Interaction::Hovered || *interaction == Interaction::Pressed {
            any_hovered = true;
            // Wheel up (positive y) should move content down: decrease scroll offset
            pos.0.y = (pos.0.y - dy).max(0.0);
        }
    }
    if !any_hovered {
        for (_interaction, mut pos) in &mut areas {
            pos.0.y = (pos.0.y - dy).max(0.0);
        }
    }
}
