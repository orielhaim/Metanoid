use bevy::prelude::*;
use bevy_enoki::prelude::*;
use bevy_hanabi::prelude::*;
use metanoid_core::components::ball::Ball;
use metanoid_core::components::brick::BrickType;
use metanoid_core::components::powerup::PowerUp;
use metanoid_core::events::BrickDestroyedEvent;
use metanoid_vfx::enoki_effects::EnokiEffects;
use metanoid_vfx::particles::ParticleEffects;

use super::level_progression::ActiveLevelVisuals;
use super::level_spawner::LevelEntity;
use metanoid_visuals::material::BrickMatKind;

/// Physical shard debris thrown out when a brick is destroyed.
#[derive(Component)]
pub struct Debris {
    pub vel: Vec2,
    pub spin: f32,
    pub life: f32,
    pub max_life: f32,
}

pub fn on_brick_destroyed_debris(
    trigger: On<BrickDestroyedEvent>,
    mut commands: Commands,
    visuals: Res<ActiveLevelVisuals>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let kind = match trigger.brick_type {
        BrickType::MultiHit => BrickMatKind::MultiHit,
        BrickType::Invincible => BrickMatKind::Invincible,
        BrickType::Explosive => BrickMatKind::Explosive,
        _ => BrickMatKind::Normal,
    };
    let mat = visuals.materials.brick(kind, 1.0);
    let pos = trigger.position;

    for i in 0..6 {
        let angle = (i as f32 / 6.0) * std::f32::consts::TAU + (i as f32 * 0.7);
        let speed = 60.0 + (i as f32 * 37.0) % 120.0;
        let vel = Vec2::new(angle.cos(), angle.sin()) * speed + Vec2::new(0.0, 60.0);
        let size = 4.0 + (i % 3) as f32 * 2.0;
        commands.spawn((
            Debris {
                vel,
                spin: (i as f32 - 3.0) * 6.0,
                life: 0.7,
                max_life: 0.7,
            },
            LevelEntity,
            Mesh2d(meshes.add(Rectangle::new(size, size * 0.6))),
            MeshMaterial2d(mat.clone()),
            Transform::from_xyz(pos.x, pos.y, 2.0),
        ));
    }
}

pub fn tick_debris(
    time: Res<Time>,
    mut commands: Commands,
    mut q: Query<(Entity, &mut Debris, &mut Transform)>,
) {
    let dt = time.delta_secs();
    for (entity, mut debris, mut transform) in &mut q {
        debris.life -= dt;
        if debris.life <= 0.0 {
            commands.entity(entity).try_despawn();
            continue;
        }
        debris.vel.y -= 320.0 * dt;
        debris.vel *= 1.0 - 1.6 * dt;
        transform.translation.x += debris.vel.x * dt;
        transform.translation.y += debris.vel.y * dt;
        transform.rotate_z(debris.spin * dt);
        let t = debris.life / debris.max_life;
        transform.scale = Vec3::splat(t.max(0.0));
    }
}

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
    let Some(effects) = particle_effects else {
        return;
    };

    for (ball_entity, transform) in &balls {
        let already_has_trail = trails.iter().any(|t| t.ball == ball_entity);
        if already_has_trail {
            continue;
        }

        commands.spawn((
            ParticleEffect::new(effects.ball_trail.clone()),
            Transform::from_translation(transform.translation).with_translation(Vec3::new(
                transform.translation.x,
                transform.translation.y,
                0.5,
            )),
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
            // Keep the glow slightly in front of the ball to avoid z-fighting.
            trail_transform.translation.z = 0.5;
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
            commands.entity(entity).try_despawn();
        }
    }
}

pub fn on_brick_destroyed_particles(
    trigger: On<BrickDestroyedEvent>,
    mut commands: Commands,
    particle_effects: Option<Res<ParticleEffects>>,
) {
    let Some(effects) = particle_effects else {
        return;
    };

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
            Transform::from_translation(transform.translation).with_translation(Vec3::new(
                transform.translation.x,
                transform.translation.y,
                0.5,
            )),
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
            aura_transform.translation.z = 0.5;
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
            commands.entity(entity).try_despawn();
        }
    }
}
