pub mod sfx;
pub mod music;

use bevy::prelude::*;
use bevy_kira_audio::{AudioApp, AudioPlugin, AudioSource};

pub use music::{MusicAssets, MusicState, MusicTrack, play_music, crossfade_to, track_for_biome};

#[derive(Resource, Default)]
pub struct SfxChannel;

#[derive(Resource, Default)]
pub struct MusicChannel;

pub struct MetanoidAudioPlugin;

impl Plugin for MetanoidAudioPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(AudioPlugin)
            .add_audio_channel::<SfxChannel>()
            .add_audio_channel::<MusicChannel>()
            .init_resource::<SfxAssets>()
            .init_resource::<MusicAssets>()
            .init_resource::<MusicState>()
            .add_observer(sfx::on_ball_paddle_bounce)
            .add_observer(sfx::on_ball_wall_bounce)
            .add_observer(sfx::on_brick_hit)
            .add_observer(sfx::on_brick_destroyed)
            .add_observer(sfx::on_powerup_collected)
            .add_observer(sfx::on_powerup_spawned)
            .add_observer(sfx::on_life_lost)
            .add_observer(sfx::on_level_clear)
            .add_observer(sfx::on_combo_milestone)
            .add_observer(sfx::on_laser_fire)
            .add_observer(sfx::on_shield_activate)
            .add_observer(sfx::on_shield_hit)
            .add_observer(sfx::on_ball_split)
            .add_observer(sfx::on_ball_fast)
            .add_observer(sfx::on_bullet_time_enter)
            .add_observer(sfx::on_shockwave)
            .add_observer(sfx::on_lightning)
            .add_observer(sfx::on_teleport)
            .add_observer(sfx::on_brick_regen)
            .add_observer(sfx::on_game_over);
    }
}

#[derive(Resource)]
pub struct SfxAssets {
    // E1 — Core
    pub bounce_wall: Handle<AudioSource>,
    pub bounce_paddle: Handle<AudioSource>,
    pub brick_hit: Handle<AudioSource>,
    pub brick_hit_metal: Handle<AudioSource>,
    pub brick_destroy_normal: Handle<AudioSource>,
    pub brick_destroy_explosive: Handle<AudioSource>,
    pub powerup_collect: Handle<AudioSource>,
    pub powerup_negative: Handle<AudioSource>,
    pub life_lost: Handle<AudioSource>,
    pub ui_hover: Handle<AudioSource>,
    pub ui_select: Handle<AudioSource>,
    // E2 — Dynamic
    pub brick_destroy_glass: Handle<AudioSource>,
    pub brick_destroy_wood: Handle<AudioSource>,
    pub brick_destroy_stone: Handle<AudioSource>,
    pub brick_regen: Handle<AudioSource>,
    pub laser_fire: Handle<AudioSource>,
    pub chain_electricity: Handle<AudioSource>,
    pub lightning_strike: Handle<AudioSource>,
    pub teleport: Handle<AudioSource>,
    pub shield_activate: Handle<AudioSource>,
    pub shield_hit: Handle<AudioSource>,
    pub ball_fast_whoosh: Handle<AudioSource>,
    pub bullet_time_enter: Handle<AudioSource>,
    pub bullet_time_exit: Handle<AudioSource>,
    pub level_clear: Handle<AudioSource>,
    pub combo_milestone: Handle<AudioSource>,
    pub powerup_drop: Handle<AudioSource>,
    pub shockwave: Handle<AudioSource>,
    pub ball_split: Handle<AudioSource>,
    pub game_over: Handle<AudioSource>,
}

impl FromWorld for SfxAssets {
    fn from_world(world: &mut World) -> Self {
        let a = world.resource::<AssetServer>();
        Self {
            bounce_wall: a.load("audio/sfx/bounce_wall.ogg"),
            bounce_paddle: a.load("audio/sfx/bounce_paddle.ogg"),
            brick_hit: a.load("audio/sfx/brick_hit.ogg"),
            brick_hit_metal: a.load("audio/sfx/brick_hit_metal.ogg"),
            brick_destroy_normal: a.load("audio/sfx/brick_destroy_normal.ogg"),
            brick_destroy_explosive: a.load("audio/sfx/brick_destroy_explosive.ogg"),
            powerup_collect: a.load("audio/sfx/powerup_collect.ogg"),
            powerup_negative: a.load("audio/sfx/powerup_negative.ogg"),
            life_lost: a.load("audio/sfx/life_lost.ogg"),
            ui_hover: a.load("audio/sfx/ui_hover.ogg"),
            ui_select: a.load("audio/sfx/ui_select.ogg"),
            brick_destroy_glass: a.load("audio/sfx/brick_destroy_glass.ogg"),
            brick_destroy_wood: a.load("audio/sfx/brick_destroy_wood.ogg"),
            brick_destroy_stone: a.load("audio/sfx/brick_destroy_stone.ogg"),
            brick_regen: a.load("audio/sfx/brick_regen.ogg"),
            laser_fire: a.load("audio/sfx/laser_fire.ogg"),
            chain_electricity: a.load("audio/sfx/chain_electricity.ogg"),
            lightning_strike: a.load("audio/sfx/lightning_strike.ogg"),
            teleport: a.load("audio/sfx/teleport.ogg"),
            shield_activate: a.load("audio/sfx/shield_activate.ogg"),
            shield_hit: a.load("audio/sfx/shield_hit.ogg"),
            ball_fast_whoosh: a.load("audio/sfx/ball_fast_whoosh.ogg"),
            bullet_time_enter: a.load("audio/sfx/bullet_time_enter.ogg"),
            bullet_time_exit: a.load("audio/sfx/bullet_time_exit.ogg"),
            level_clear: a.load("audio/sfx/level_clear.ogg"),
            combo_milestone: a.load("audio/sfx/combo_milestone.ogg"),
            powerup_drop: a.load("audio/sfx/powerup_drop.ogg"),
            shockwave: a.load("audio/sfx/shockwave.ogg"),
            ball_split: a.load("audio/sfx/ball_split.ogg"),
            game_over: a.load("audio/sfx/game_over.ogg"),
        }
    }
}
