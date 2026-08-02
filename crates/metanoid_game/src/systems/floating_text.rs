//! Bouncy floating combat text (combos, power-up names).

use bevy::prelude::*;
use metanoid_core::events::{FloatingTextEvent, FloatingTextKind};

#[derive(Component)]
pub struct FloatingPopup {
    pub age: f32,
    pub lifetime: f32,
    pub velocity: Vec2,
    pub angular_vel: f32,
    pub base_scale: f32,
}

fn style_for(kind: FloatingTextKind) -> (f32, f32, f32, f32, f32) {
    // lifetime, rise_speed, angular, scale, font
    match kind {
        FloatingTextKind::Combo => (1.35, 90.0, 1.8, 1.15, 28.0),
        FloatingTextKind::Milestone => (1.6, 110.0, 2.4, 1.35, 34.0),
        FloatingTextKind::PowerUp => (1.5, 70.0, 2.8, 1.2, 26.0),
        FloatingTextKind::Score => (1.0, 80.0, 1.2, 1.0, 22.0),
    }
}

/// Observer used by combat systems (`commands.trigger(FloatingTextEvent {..})`).
pub fn on_floating_text_event(trigger: On<FloatingTextEvent>, mut commands: Commands) {
    let ev = trigger.event().clone();
    let (lifetime, speed, angular, scale, font) = style_for(ev.kind);
    let jitter = (ev.position.x * 0.13).sin() * 0.4;

    commands.spawn((
        FloatingPopup {
            age: 0.0,
            lifetime,
            velocity: Vec2::new(jitter * 40.0, speed),
            angular_vel: angular * if jitter >= 0.0 { 1.0 } else { -1.0 },
            base_scale: scale,
        },
        Text2d::new(ev.text),
        TextFont {
            font_size: FontSize::Px(font),
            ..default()
        },
        TextColor(ev.color),
        TextLayout::justify(Justify::Center),
        Transform::from_translation(ev.position.extend(20.0)).with_scale(Vec3::splat(0.2)),
    ));
}

pub fn update_floating_text(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut FloatingPopup, &mut Transform, &mut TextColor)>,
) {
    let dt = time.delta_secs();
    for (entity, mut popup, mut transform, mut color) in &mut query {
        popup.age += dt;
        let t = (popup.age / popup.lifetime).clamp(0.0, 1.0);

        // Pop-in ease then settle
        let pop = if t < 0.15 {
            let u = t / 0.15;
            0.2 + popup.base_scale * (u * u * (3.0 - 2.0 * u)) * 1.25
        } else if t < 0.35 {
            let u = (t - 0.15) / 0.2;
            popup.base_scale * (1.15 - 0.15 * u)
        } else {
            popup.base_scale * (1.0 - (t - 0.35) * 0.15)
        };

        transform.translation.x += popup.velocity.x * dt;
        transform.translation.y += popup.velocity.y * dt;
        popup.velocity.y -= 40.0 * dt;
        popup.velocity.x *= 1.0 - 1.5 * dt;

        transform.rotate_z(popup.angular_vel * dt);
        popup.angular_vel *= 1.0 - 1.8 * dt;

        transform.scale = Vec3::splat(pop.max(0.05));

        let alpha = if t > 0.6 {
            (1.0 - (t - 0.6) / 0.4).clamp(0.0, 1.0)
        } else {
            1.0
        };
        let c = color.0.to_srgba();
        color.0 = Color::srgba(c.red, c.green, c.blue, alpha);

        if popup.age >= popup.lifetime {
            commands.entity(entity).try_despawn();
        }
    }
}
