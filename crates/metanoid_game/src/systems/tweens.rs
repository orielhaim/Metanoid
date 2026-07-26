use bevy::prelude::*;
use bevy_tweening::*;
use bevy_tweening::lens::TransformScaleLens;
use std::time::Duration;

use metanoid_core::components::brick::Brick;
use metanoid_core::events::BrickHitEvent;

pub fn on_brick_hit_flash(
    trigger: On<BrickHitEvent>,
    mut commands: Commands,
    bricks: Query<Entity, With<Brick>>,
) {
    if bricks.get(trigger.brick).is_err() {
        return;
    }

    let flash_tween = Tween::new(
        EaseFunction::QuadraticInOut,
        Duration::from_millis(60),
        TransformScaleLens {
            start: Vec3::splat(1.0),
            end: Vec3::splat(1.15),
        },
    )
    .then(Tween::new(
        EaseFunction::QuadraticOut,
        Duration::from_millis(100),
        TransformScaleLens {
            start: Vec3::splat(1.15),
            end: Vec3::splat(1.0),
        },
    ));

    commands
        .entity(trigger.brick)
        .insert(TweenAnim::new(flash_tween));
}
