//! Visual + audio feedback for high ball speed and sharp side-english hits.

use avian2d::prelude::LinearVelocity;
use bevy::prelude::*;
use bevy_kira_audio::AudioControl;
use bevy_kira_audio::prelude::AudioChannel;
use metanoid_audio::{SfxAssets, SfxChannel};
use metanoid_core::components::ball::Ball;
use metanoid_core::constants::BALL_SPEED;
use metanoid_core::events::PaddleHitEvent;

use crate::systems::level_spawner::LevelEntity;

/// Small red heat chevron ahead of a fast ball (spaceship reentry feel).
#[derive(Component)]
pub struct SpeedHeatMark {
    pub ball: Entity,
    pub age: f32,
}

#[derive(Resource, Default)]
pub struct SpeedWhooshCooldown(pub f32);

fn speed_heat_color(t: f32) -> Color {
    if t < 0.3 {
        let u = t / 0.3;
        Color::srgb(1.0, 1.0 - 0.2 * u, 1.0 - 0.1 * u)
    } else if t < 0.65 {
        let u = (t - 0.3) / 0.35;
        Color::srgb(1.0, 0.8 - 0.4 * u, 0.5 - 0.35 * u)
    } else {
        let u = ((t - 0.65) / 0.35).clamp(0.0, 1.0);
        Color::srgb(1.0, 0.35 - 0.15 * u, 0.12)
    }
}

fn speed_factor(ball: &Ball, speed: f32) -> f32 {
    let base = ball.speed.max(BALL_SPEED * 0.85);
    ((speed / base) - 1.0).clamp(0.0, 1.8) / 1.5
}

/// Recolor the ball when overspeeding; spawn heat marks for new fast balls.
pub fn update_ball_speed_visuals(
    mut commands: Commands,
    balls: Query<(
        Entity,
        &Ball,
        &LinearVelocity,
        &Transform,
        &MeshMaterial2d<ColorMaterial>,
    )>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    marks: Query<(Entity, &SpeedHeatMark)>,
) {
    for (entity, mark) in &marks {
        if balls.get(mark.ball).is_err() {
            commands.entity(entity).try_despawn();
        }
    }

    for (entity, ball, vel, transform, mat_handle) in &balls {
        if ball.stuck {
            if let Some(mut mat) = materials.get_mut(&mat_handle.0) {
                mat.color = Color::srgb(1.0, 1.0, 1.0);
            }
            continue;
        }

        let speed = vel.0.length();
        let t = speed_factor(ball, speed);

        if let Some(mut mat) = materials.get_mut(&mat_handle.0) {
            mat.color = speed_heat_color(t);
        }

        if t < 0.12 {
            for (me, mark) in &marks {
                if mark.ball == entity {
                    commands.entity(me).try_despawn();
                }
            }
            continue;
        }

        let already = marks.iter().any(|(_, m)| m.ball == entity);
        if already {
            continue;
        }

        let dir = vel.0.normalize_or_zero();
        let angle = vel.0.y.atan2(vel.0.x);
        let ahead = transform.translation.truncate() + dir * (14.0 + t * 32.0);
        let size = 7.0 + t * 18.0;
        let heat = Color::srgba(1.0, 0.12 + 0.2 * (1.0 - t), 0.04, 0.22 + t * 0.55);

        let mesh = meshes.add(Triangle2d::new(
            Vec2::new(0.0, size * 0.55),
            Vec2::new(-size * 0.28, -size * 0.45),
            Vec2::new(size * 0.28, -size * 0.45),
        ));
        commands.spawn((
            SpeedHeatMark {
                ball: entity,
                age: 0.0,
            },
            LevelEntity,
            Mesh2d(mesh),
            MeshMaterial2d(materials.add(heat)),
            Transform::from_translation(ahead.extend(6.0))
                .with_rotation(Quat::from_rotation_z(angle - std::f32::consts::FRAC_PI_2)),
        ));
    }
}

pub fn update_speed_heat_marks(
    time: Res<Time>,
    mut commands: Commands,
    balls: Query<(&Ball, &LinearVelocity, &Transform)>,
    mut marks: Query<
        (
            Entity,
            &mut SpeedHeatMark,
            &mut Transform,
            &MeshMaterial2d<ColorMaterial>,
        ),
        Without<Ball>,
    >,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let dt = time.delta_secs();
    for (entity, mut mark, mut tf, mat_h) in &mut marks {
        let Ok((ball, vel, ball_tf)) = balls.get(mark.ball) else {
            commands.entity(entity).try_despawn();
            continue;
        };
        if ball.stuck {
            commands.entity(entity).try_despawn();
            continue;
        }
        let speed = vel.0.length();
        let t = speed_factor(ball, speed);
        if t < 0.1 {
            commands.entity(entity).try_despawn();
            continue;
        }
        mark.age += dt;
        let dir = vel.0.normalize_or_zero();
        let angle = vel.0.y.atan2(vel.0.x);
        let ahead = ball_tf.translation.truncate() + dir * (14.0 + t * 34.0);
        let pulse = 1.0 + 0.1 * (mark.age * 16.0).sin();
        tf.translation = ahead.extend(6.0);
        tf.rotation = Quat::from_rotation_z(angle - std::f32::consts::FRAC_PI_2);
        tf.scale = Vec3::new((0.5 + t * 0.55) * pulse, (0.9 + t * 1.2) * pulse, 1.0);

        if let Some(mut mat) = materials.get_mut(&mat_h.0) {
            let a = (0.2 + t * 0.65).clamp(0.15, 0.9);
            mat.color = Color::srgba(1.0, 0.1 + 0.22 * (1.0 - t), 0.03, a);
        }
    }
}

/// Sharp side-english paddle contact -> whoosh (ball_fast_whoosh.ogg).
pub fn on_paddle_side_hit_fx(
    _trigger: On<PaddleHitEvent>,
    balls: Query<&Ball>,
    sfx: Option<Res<SfxAssets>>,
    audio: Option<Res<AudioChannel<SfxChannel>>>,
) {
    let (Some(sfx), Some(audio)) = (sfx, audio) else {
        return;
    };
    for ball in &balls {
        if ball.spin.abs() > 0.42 {
            let spin = ball.spin.abs();
            audio
                .play(sfx.ball_fast_whoosh.clone())
                .with_playback_rate(0.95 + spin as f64 * 0.4)
                .with_volume(-10.0 + spin * 7.0);
            break;
        }
    }
}

/// Occasional whoosh while sustaining overdrive speed.
pub fn speed_overdrive_whoosh(
    time: Res<Time>,
    mut cd: ResMut<SpeedWhooshCooldown>,
    balls: Query<(&Ball, &LinearVelocity)>,
    sfx: Option<Res<SfxAssets>>,
    audio: Option<Res<AudioChannel<SfxChannel>>>,
) {
    cd.0 = (cd.0 - time.delta_secs()).max(0.0);
    if cd.0 > 0.0 {
        return;
    }
    let (Some(sfx), Some(audio)) = (sfx, audio) else {
        return;
    };
    for (ball, vel) in &balls {
        if ball.stuck {
            continue;
        }
        let speed = vel.0.length();
        let base = ball.speed.max(BALL_SPEED);
        if speed > base * 1.4 {
            let t = ((speed / base) - 1.0).clamp(0.0, 1.5);
            audio
                .play(sfx.ball_fast_whoosh.clone())
                .with_playback_rate(0.88 + t as f64 * 0.55)
                .with_volume(-16.0 + t * 10.0);
            cd.0 = (0.5 - t * 0.18).clamp(0.25, 0.5);
            break;
        }
    }
}
