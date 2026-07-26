use bevy::prelude::*;
use bevy_hanabi::prelude::*;
use bevy_enoki::prelude::*;
use metanoid_core::components::ball::Ball;
use metanoid_core::components::brick::BrickType;
use metanoid_core::components::powerup::PowerUp;
use metanoid_core::events::BrickDestroyedEvent;
use metanoid_vfx::particles::ParticleEffects;
use metanoid_vfx::enoki_effects::EnokiEffects;

use crate::systems::level_spawner::LevelEntity;

#[derive(Component)]
pub struct BallTrail {
    pub ball: Entity,
}

pub fn spawn_ball_trail_for_new_balls(
    mut commands: Commands,
    particle_effects: Option<Res<ParticleEffects>>,
    balls: Query<(Entity, &Transform), With<Ball>>,
    trails: Query<&BallTrail>,
) {
    let Some(effects) = particle_effects else { return };

    for (ball_entity, transform) in &balls {
        let already_has_trail = trails.iter().any(|t| t.ball == ball_entity);
        if already_has_trail {
            continue;
        }

        commands.spawn((
            ParticleEffect::new(effects.ball_trail.clone()),
            Transform::from_translation(transform.translation),
            BallTrail { ball: ball_entity },
            LevelEntity,
        ));
    }
}

pub fn update_ball_trail_positions(
    balls: Query<&Transform, With<Ball>>,
    mut trails: Query<(&BallTrail, &mut Transform), Without<Ball>>,
) {
    for (trail, mut trail_transform) in &mut trails {
        if let Ok(ball_transform) = balls.get(trail.ball) {
            trail_transform.translation = ball_transform.translation;
        }
    }
}

pub fn cleanup_orphaned_trails(
    mut commands: Commands,
    trails: Query<(Entity, &BallTrail)>,
    balls: Query<&Ball>,
) {
    for (entity, trail) in &trails {
        if balls.get(trail.ball).is_err() {
            commands.entity(entity).despawn();
        }
    }
}

pub fn on_brick_destroyed_particles(
    trigger: On<BrickDestroyedEvent>,
    mut commands: Commands,
    particle_effects: Option<Res<ParticleEffects>>,
) {
    let Some(effects) = particle_effects else { return };

    let is_explosive = trigger.brick_type == BrickType::Explosive;
    let effect_handle = if is_explosive {
        effects.explosion.clone()
    } else {
        effects.brick_break.clone()
    };

    let pos = trigger.position;

    commands.spawn((
        ParticleEffect::new(effect_handle),
        Transform::from_xyz(pos.x, pos.y, 1.0),
        LevelEntity,
    ));
}

pub fn spawn_powerup_auras(
    mut commands: Commands,
    enoki_effects: Option<Res<EnokiEffects>>,
    powerups: Query<(Entity, &Transform), (With<PowerUp>, Without<PowerUpAura>)>,
) {
    let Some(effects) = enoki_effects else { return };

    for (entity, transform) in &powerups {
        commands.spawn((
            ParticleSpawner::<ColorParticle2dMaterial>::default(),
            ParticleEffectHandle(effects.powerup_aura.clone()),
            Transform::from_translation(transform.translation),
            PowerUpAura { powerup: entity },
            LevelEntity,
        ));
    }
}

#[derive(Component)]
pub struct PowerUpAura {
    pub powerup: Entity,
}

pub fn update_powerup_aura_positions(
    powerups: Query<&Transform, With<PowerUp>>,
    mut auras: Query<(&PowerUpAura, &mut Transform), Without<PowerUp>>,
) {
    for (aura, mut aura_transform) in &mut auras {
        if let Ok(powerup_transform) = powerups.get(aura.powerup) {
            aura_transform.translation = powerup_transform.translation;
        }
    }
}

pub fn cleanup_powerup_auras(
    mut commands: Commands,
    auras: Query<(Entity, &PowerUpAura)>,
    powerups: Query<&PowerUp>,
) {
    for (entity, aura) in &auras {
        if powerups.get(aura.powerup).is_err() {
            commands.entity(entity).despawn();
        }
    }
}
