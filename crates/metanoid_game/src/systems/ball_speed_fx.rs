//! Visual + audio feedback for high ball speed and sharp side-english hits.
use avian2d::prelude::LinearVelocity;
use bevy::prelude::*;
use bevy_kira_audio::AudioControl;
use bevy_kira_audio::prelude::AudioChannel;
use metanoid_audio::{SfxAssets, SfxChannel};
use metanoid_core::components::ball::Ball;
use metanoid_core::constants::BALL_SPEED;
use metanoid_core::events::PaddleHitEvent;
use metanoid_core::settings::GameSettings;

use super::level_progression::ActiveLevelVisuals;
use super::level_spawner::LevelEntity;

#[derive(Resource, Default)]
pub struct SpeedWhooshCooldown(pub f32);

fn speed_factor(ball: &Ball, speed: f32) -> f32 {
    let base = ball.speed.max(BALL_SPEED * 0.85);
    ((speed / base) - 1.0).clamp(0.0, 1.8) / 1.5
}

#[derive(Component, Clone, Copy, PartialEq)]
pub struct AeroPart {
    pub aero: Entity,
    pub role: AeroRole,
}

#[derive(Component, Clone, Copy, PartialEq)]
pub enum AeroRole {
    /// Soft glow hugging the ball.
    Halo,
    /// White-hot cap on the leading edge.
    Shock,
    /// One link of the flexible wake chain.
    Wake,
}

const WAKE_SEGMENTS: usize = 5;

#[derive(Component)]
pub struct BallAero {
    pub ball: Entity,
    pub glow: LinearRgba,
    pub halo: Entity,
    pub shock: Entity,
    pub wake_parts: Vec<Entity>,
    /// Head..tail wake chain positions (damped follow -> bending tail).
    pub wake_pos: Vec<Vec2>,
    pub last_t: f32,
}

fn spawn_aero_part(
    commands: &mut Commands,
    aero: Entity,
    role: AeroRole,
    glow_tex: &Handle<Image>,
) -> Entity {
    commands
        .spawn((
            AeroPart { aero, role },
            Sprite {
                image: glow_tex.clone(),
                color: Color::NONE,
                custom_size: Some(Vec2::ONE),
                ..default()
            },
            Transform::default(),
            LevelEntity,
        ))
        .id()
}

pub fn spawn_ball_aero(
    mut commands: Commands,
    visuals: Res<ActiveLevelVisuals>,
    balls: Query<(Entity, &Ball, &Transform)>,
    aeroes: Query<&BallAero>,
) {
    let glow_tex = visuals.materials.glow.clone();
    let glow = visuals.recipe.entities.ball_glow;
    for (ball_entity, _ball, ball_tf) in &balls {
        if aeroes.iter().any(|a| a.ball == ball_entity) {
            continue;
        }
        let aero_entity = commands.spawn((LevelEntity,)).id();
        let halo = spawn_aero_part(&mut commands, aero_entity, AeroRole::Halo, &glow_tex);
        let shock = spawn_aero_part(&mut commands, aero_entity, AeroRole::Shock, &glow_tex);
        let ball_pos = ball_tf.translation.truncate();
        let mut wake_parts = Vec::with_capacity(WAKE_SEGMENTS);
        let mut wake_pos = Vec::with_capacity(WAKE_SEGMENTS);
        for _ in 0..WAKE_SEGMENTS {
            wake_parts.push(spawn_aero_part(
                &mut commands,
                aero_entity,
                AeroRole::Wake,
                &glow_tex,
            ));
            wake_pos.push(ball_pos);
        }
        commands.entity(aero_entity).insert(BallAero {
            ball: ball_entity,
            glow,
            halo,
            shock,
            wake_parts,
            wake_pos,
            last_t: 0.0,
        });
    }
}

pub fn update_ball_aero(
    time: Res<Time>,
    settings: Res<GameSettings>,
    balls: Query<(&Ball, &LinearVelocity, &Transform)>,
    mut aeroes: Query<(Entity, &mut BallAero)>,
    mut parts: Query<(&AeroPart, &mut Sprite, &mut Transform), Without<Ball>>,
) {
    let dt = time.delta_secs().min(0.05);
    let motion = if settings.reduce_motion { 0.0 } else { 1.0 };

    for (_aero_entity, mut aero) in &mut aeroes {
        let Ok((ball, vel, ball_tf)) = balls.get(aero.ball) else {
            continue;
        };
        let target_t = if ball.stuck {
            0.0
        } else {
            speed_factor(ball, vel.0.length()) * motion
        };
        // Exponential smoothing so the rig eases in/out instead of popping.
        aero.last_t += (target_t - aero.last_t) * (1.0 - (-dt * 10.0).exp());
        let t = aero.last_t;

        let dir = vel.0.normalize_or_zero();
        let pos = ball_tf.translation.truncate();
        let radius = ball.radius.max(1.0);
        let (gr, gg, gb) = (aero.glow.red, aero.glow.green, aero.glow.blue);

        // Halo.
        if let Ok((_, mut sprite, mut tf)) = parts.get_mut(aero.halo) {
            let size = radius * 3.0 * (1.0 + t * 2.0);
            sprite.custom_size = Some(Vec2::splat(size));
            sprite.color = Color::srgba(gr, gg, gb, (0.10 + t * 0.5).min(0.75));
            tf.translation = pos.extend(0.5);
            tf.rotation = Quat::IDENTITY;
            tf.scale = Vec3::ONE;
        }

        // White-hot shock cap ahead of the ball.
        if let Ok((_, mut sprite, mut tf)) = parts.get_mut(aero.shock) {
            let size = radius * 2.4 * (1.0 + t * 1.4);
            sprite.custom_size = Some(Vec2::splat(size));
            sprite.color = Color::srgba(1.0, 0.96, 0.9, (t * 0.65).clamp(0.0, 0.8));
            let ahead = pos + dir * (radius * (0.5 + t * 1.6));
            tf.translation = ahead.extend(0.6);
            tf.rotation = Quat::IDENTITY;
            tf.scale = Vec3::ONE;
        }

        // Flexible wake: the head eases toward a point behind the ball, and each
        // link eases toward the one ahead of it, so a sharp turn bends the tail
        // into a curve instead of snapping it.
        let spacing = radius * (2.2 + t * 1.6);
        let head_target = pos - dir * spacing;
        let head = aero.wake_pos[0];
        aero.wake_pos[0] = head + (head_target - head) * (1.0 - (-dt * 9.0).exp());
        for i in 1..aero.wake_pos.len() {
            let target = aero.wake_pos[i - 1];
            let cur = aero.wake_pos[i];
            aero.wake_pos[i] = cur + (target - cur) * (1.0 - (-dt * 6.5).exp());
        }

        let n = aero.wake_pos.len();
        for (i, part_entity) in aero.wake_parts.iter().enumerate() {
            let Ok((_, mut sprite, mut tf)) = parts.get_mut(*part_entity) else {
                continue;
            };
            let seg_pos = aero.wake_pos[i];
            let prev = if i == 0 { pos } else { aero.wake_pos[i - 1] };
            let next = if i + 1 < n {
                aero.wake_pos[i + 1]
            } else {
                aero.wake_pos[i]
            };
            let tangent = prev - next;
            let tangent_dir = if tangent.length_squared() > 1e-6 {
                tangent.normalize()
            } else {
                dir
            };
            let ang = tangent_dir.y.atan2(tangent_dir.x);

            let fade = 1.0 - i as f32 / n as f32;
            let seg_len = spacing * (2.2 - i as f32 * 0.22);
            let seg_w = radius * (1.5 - i as f32 * 0.16);
            sprite.custom_size = Some(Vec2::new(seg_w, seg_len));
            let alpha = (t * (0.30 * fade + 0.05)).clamp(0.0, 0.5);
            // Wake cools toward a faint blue-white.
            sprite.color = Color::srgba(
                (gr * 0.6 + 0.35).min(1.0),
                (gg * 0.6 + 0.4).min(1.0),
                1.0,
                alpha,
            );
            tf.translation = seg_pos.extend(0.45);
            // Local +y (the long axis) aligned with the local tangent.
            tf.rotation = Quat::from_rotation_z(ang - std::f32::consts::FRAC_PI_2);
            tf.scale = Vec3::ONE;
        }
    }
}

/// Remove atmosphere rigs for balls that no longer exist.
pub fn cleanup_orphaned_aeros(
    mut commands: Commands,
    aeroes: Query<(Entity, &BallAero)>,
    balls: Query<&Ball>,
) {
    for (aero_entity, aero) in &aeroes {
        if balls.get(aero.ball).is_err() {
            for p in &aero.wake_parts {
                commands.entity(*p).try_despawn();
            }
            commands.entity(aero.halo).try_despawn();
            commands.entity(aero.shock).try_despawn();
            commands.entity(aero_entity).try_despawn();
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
