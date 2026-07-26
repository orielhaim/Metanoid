use bevy::prelude::*;
use bevy_kira_audio::AudioControl;
use bevy_kira_audio::prelude::AudioChannel;
use metanoid_core::components::brick::{Brick, BrickType};
use metanoid_core::components::powerup::PowerUpKind;
use metanoid_core::events::*;
use metanoid_core::resources::combo::ComboCounter;

use super::{SfxAssets, SfxChannel};

fn combo_pitch(combo: &ComboCounter) -> f64 {
    1.0 + (combo.count as f64 * 0.02).min(0.5)
}

// ── Bounces ──────────────────────────────────────────────

pub fn on_ball_wall_bounce(
    _trigger: On<WallHitEvent>,
    sfx: Res<SfxAssets>,
    audio: Res<AudioChannel<SfxChannel>>,
) {
    audio.play(sfx.bounce_wall.clone());
}

pub fn on_ball_paddle_bounce(
    _trigger: On<PaddleHitEvent>,
    sfx: Res<SfxAssets>,
    audio: Res<AudioChannel<SfxChannel>>,
) {
    audio.play(sfx.bounce_paddle.clone());
}

// ── Brick hit / destroy ──────────────────────────────────

pub fn on_brick_hit(
    trigger: On<BrickHitEvent>,
    bricks: Query<&Brick>,
    combo: Res<ComboCounter>,
    sfx: Res<SfxAssets>,
    audio: Res<AudioChannel<SfxChannel>>,
) {
    if let Ok(brick) = bricks.get(trigger.brick) {
        let sound = match brick.brick_type {
            BrickType::Invincible => &sfx.brick_hit_metal,
            BrickType::MultiHit if brick.health > 1 => &sfx.brick_hit_metal,
            _ => &sfx.brick_hit,
        };

        // Scale volume by ball speed: quiet rub → loud slam
        // speed ~200 = gentle, ~600 = fast, ~900+ = very fast
        let vol_db = ((trigger.ball_speed / 600.0).clamp(0.2, 1.5) * 20.0 - 15.0).clamp(-20.0, 5.0);

        audio.play(sound.clone())
            .with_playback_rate(combo_pitch(&combo))
            .with_volume(vol_db);
    }
}

pub fn on_brick_destroyed(
    trigger: On<BrickDestroyedEvent>,
    combo: Res<ComboCounter>,
    sfx: Res<SfxAssets>,
    audio: Res<AudioChannel<SfxChannel>>,
) {
    let sound = match trigger.brick_type {
        BrickType::Explosive => &sfx.brick_destroy_explosive,
        BrickType::MultiHit => &sfx.brick_destroy_glass,
        _ => &sfx.brick_destroy_normal,
    };
    audio.play(sound.clone()).with_playback_rate(combo_pitch(&combo));
}

// ── Power-ups ────────────────────────────────────────────

pub fn on_powerup_spawned(
    _trigger: On<PowerUpSpawnedEvent>,
    sfx: Res<SfxAssets>,
    audio: Res<AudioChannel<SfxChannel>>,
) {
    audio.play(sfx.powerup_drop.clone());
}

pub fn on_powerup_collected(
    trigger: On<PowerUpCollectedEvent>,
    sfx: Res<SfxAssets>,
    audio: Res<AudioChannel<SfxChannel>>,
) {
    let is_negative = matches!(
        trigger.kind,
        PowerUpKind::KillPaddle
            | PowerUpKind::ShrinkPaddle
            | PowerUpKind::FallingBricks
            | PowerUpKind::Blackout
    );
    let sound = if is_negative {
        &sfx.powerup_negative
    } else {
        &sfx.powerup_collect
    };
    audio.play(sound.clone());
}

// ── Life / Level / Game Over ─────────────────────────────

pub fn on_life_lost(
    _trigger: On<LifeLostEvent>,
    sfx: Res<SfxAssets>,
    audio: Res<AudioChannel<SfxChannel>>,
) {
    audio.play(sfx.life_lost.clone());
}

pub fn on_level_clear(
    _trigger: On<LevelClearEvent>,
    sfx: Res<SfxAssets>,
    audio: Res<AudioChannel<SfxChannel>>,
) {
    audio.play(sfx.level_clear.clone());
}

pub fn on_game_over(
    _trigger: On<GameOverEvent>,
    sfx: Res<SfxAssets>,
    audio: Res<AudioChannel<SfxChannel>>,
) {
    audio.play(sfx.game_over.clone());
}

// ── Combo ────────────────────────────────────────────────

pub fn on_combo_milestone(
    trigger: On<ComboMilestoneEvent>,
    sfx: Res<SfxAssets>,
    audio: Res<AudioChannel<SfxChannel>>,
) {
    let pitch = 1.0 + (trigger.count as f64 * 0.05).min(0.8);
    audio
        .play(sfx.combo_milestone.clone())
        .with_playback_rate(pitch);
}

// ── Laser / Shield ───────────────────────────────────────

pub fn on_laser_fire(
    _trigger: On<LaserFireEvent>,
    sfx: Res<SfxAssets>,
    audio: Res<AudioChannel<SfxChannel>>,
) {
    audio.play(sfx.laser_fire.clone());
}

pub fn on_shield_activate(
    _trigger: On<ShieldActivateEvent>,
    sfx: Res<SfxAssets>,
    audio: Res<AudioChannel<SfxChannel>>,
) {
    audio.play(sfx.shield_activate.clone());
}

pub fn on_shield_hit(
    _trigger: On<ShieldHitEvent>,
    sfx: Res<SfxAssets>,
    audio: Res<AudioChannel<SfxChannel>>,
) {
    audio.play(sfx.shield_hit.clone());
}

// ── Ball effects ─────────────────────────────────────────

pub fn on_ball_split(
    _trigger: On<BallSplitEvent>,
    sfx: Res<SfxAssets>,
    audio: Res<AudioChannel<SfxChannel>>,
) {
    audio.play(sfx.ball_split.clone());
}

pub fn on_ball_fast(
    _trigger: On<BallSpeedChangeEvent>,
    sfx: Res<SfxAssets>,
    audio: Res<AudioChannel<SfxChannel>>,
) {
    audio.play(sfx.ball_fast_whoosh.clone());
}

// ── Bullet time ──────────────────────────────────────────

pub fn on_bullet_time_enter(
    trigger: On<BulletTimeEvent>,
    sfx: Res<SfxAssets>,
    audio: Res<AudioChannel<SfxChannel>>,
) {
    if trigger.entering {
        audio.play(sfx.bullet_time_enter.clone());
    } else {
        audio.play(sfx.bullet_time_exit.clone());
    }
}

// ── Board effects ────────────────────────────────────────

pub fn on_shockwave(
    _trigger: On<ShockwaveEvent>,
    sfx: Res<SfxAssets>,
    audio: Res<AudioChannel<SfxChannel>>,
) {
    audio.play(sfx.shockwave.clone());
}

pub fn on_lightning(
    _trigger: On<LightningEvent>,
    sfx: Res<SfxAssets>,
    audio: Res<AudioChannel<SfxChannel>>,
) {
    audio.play(sfx.lightning_strike.clone());
}

pub fn on_teleport(
    _trigger: On<TeleportEvent>,
    sfx: Res<SfxAssets>,
    audio: Res<AudioChannel<SfxChannel>>,
) {
    audio.play(sfx.teleport.clone());
}

// ── Brick special ────────────────────────────────────────

pub fn on_brick_regen(
    _trigger: On<BrickRegenEvent>,
    sfx: Res<SfxAssets>,
    audio: Res<AudioChannel<SfxChannel>>,
) {
    audio.play(sfx.brick_regen.clone());
}
