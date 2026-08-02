//! Ambient particle fields (pollen, embers, snow, bubbles, dust, sparks).
//! Built with bevy_enoki and driven by the recipe's ambient spec.

use bevy::prelude::*;
use bevy_enoki::prelude::*;

use crate::recipe::{AmbientKind, AmbientSpec, BiomeRecipe};

/// Marker for ambient emitters so the game can clean them up.
#[derive(Component)]
pub struct AmbientEmitter;

/// Spawn a field of ambient emitters across the arena.
pub fn spawn_ambient(
    commands: &mut Commands,
    effects: &mut Assets<Particle2dEffect>,
    recipe: &BiomeRecipe,
    particle_scale: f32,
    reduce_motion: bool,
) {
    if reduce_motion {
        return;
    }
    let spec = recipe.ambient;
    if spec.kind == AmbientKind::None {
        return;
    }

    let rows = emitters_for(&spec);
    let emitter_count = ((rows as f32) * particle_scale.clamp(0.5, 2.0)).max(1.0) as u32;

    for i in 0..emitter_count {
        let x = (i as f32 / emitter_count as f32 - 0.5) * 2.0 * 620.0;
        let y = match spec.kind {
            AmbientKind::Snow => 320.0,
            AmbientKind::Embers | AmbientKind::Bubbles | AmbientKind::Sparks => -320.0,
            _ => 0.0,
        };
        let effect = effects.add(build_effect(&spec, recipe.var_seed, i));
        commands.spawn((
            AmbientEmitter,
            ParticleSpawner::<ColorParticle2dMaterial>::default(),
            ParticleEffectHandle(effect),
            Transform::from_xyz(x, y, 1.0),
        ));
    }
}

fn emitters_for(spec: &AmbientSpec) -> usize {
    let base = match spec.kind {
        AmbientKind::None => 0,
        AmbientKind::Snow | AmbientKind::Embers | AmbientKind::Bubbles | AmbientKind::Sparks => 6,
        _ => 4,
    };
    ((base as f32) * (0.5 + spec.rate * 3.0)).round() as usize
}

fn build_effect(spec: &AmbientSpec, _seed: u64, _idx: u32) -> Particle2dEffect {
    let v = |base: f32, spread: f32| Rval::new(base, spread);
    let c = spec.color;

    match spec.kind {
        AmbientKind::Pollen => Particle2dEffect {
            spawn_rate: 0.06 / spec.rate.clamp(0.3, 3.0),
            spawn_amount: 1,
            emission_shape: EmissionShape::Circle(90.0),
            lifetime: v(5.0, 2.5),
            linear_speed: Some(v(12.0, 8.0)),
            linear_acceleration: Some(v(1.0, 0.5)),
            direction: Some(Rval::new(Vec2::new(0.4, 1.0), 0.6)),
            gravity_direction: Some(Rval::new(Vec2::new(0.0, 1.0), 0.1)),
            gravity_speed: Some(v(3.0, 1.0)),
            linear_damp: Some(v(0.05, 0.0)),
            scale: Some(v(2.5, 1.5)),
            color: Some(c),
            scale_curve: Some(
                MultiCurve::new()
                    .with_point(0.0, 0.0, None)
                    .with_point(1.0, 0.5, None)
                    .with_point(0.3, 1.0, None),
            ),
            color_curve: Some(MultiCurve::new().with_point(c, 0.0, None).with_point(
                c.with_alpha(0.0),
                1.0,
                None,
            )),
            ..default()
        },
        AmbientKind::Embers => Particle2dEffect {
            spawn_rate: 0.05 / spec.rate.clamp(0.3, 3.0),
            spawn_amount: 1,
            emission_shape: EmissionShape::Circle(120.0),
            lifetime: v(2.6, 1.4),
            linear_speed: Some(v(18.0, 12.0)),
            direction: Some(Rval::new(Vec2::Y, 0.5)),
            gravity_direction: Some(Rval::new(Vec2::Y, 0.1)),
            gravity_speed: Some(v(28.0, 8.0)),
            linear_damp: Some(v(0.4, 0.1)),
            scale: Some(v(2.2, 1.4)),
            color: Some(c),
            scale_curve: Some(
                MultiCurve::new()
                    .with_point(0.6, 0.0, None)
                    .with_point(1.0, 0.35, None)
                    .with_point(0.0, 1.0, None),
            ),
            color_curve: Some(
                MultiCurve::new()
                    .with_point(c, 0.0, None)
                    .with_point(c.mix(&LinearRgba::new(1.0, 1.0, 1.0, 1.0), 0.5), 0.4, None)
                    .with_point(c.with_alpha(0.0), 1.0, None),
            ),
            ..default()
        },
        AmbientKind::Bubbles => Particle2dEffect {
            spawn_rate: 0.1 / spec.rate.clamp(0.3, 3.0),
            spawn_amount: 1,
            emission_shape: EmissionShape::Circle(140.0),
            lifetime: v(3.0, 1.5),
            linear_speed: Some(v(20.0, 10.0)),
            direction: Some(Rval::new(Vec2::Y, 0.3)),
            gravity_direction: Some(Rval::new(Vec2::Y, 0.1)),
            gravity_speed: Some(v(30.0, 10.0)),
            scale: Some(v(1.8, 1.2)),
            color: Some(c),
            scale_curve: Some(
                MultiCurve::new()
                    .with_point(0.4, 0.0, None)
                    .with_point(1.0, 0.4, None)
                    .with_point(0.2, 1.0, None),
            ),
            color_curve: Some(MultiCurve::new().with_point(c, 0.0, None).with_point(
                c.with_alpha(0.0),
                1.0,
                None,
            )),
            ..default()
        },
        AmbientKind::Snow => Particle2dEffect {
            spawn_rate: 0.04 / spec.rate.clamp(0.3, 3.0),
            spawn_amount: 1,
            emission_shape: EmissionShape::Circle(160.0),
            lifetime: v(6.0, 2.0),
            linear_speed: Some(v(8.0, 6.0)),
            direction: Some(Rval::new(Vec2::new(0.3, -1.0), 0.4)),
            gravity_direction: Some(Rval::new(Vec2::new(0.0, -1.0), 0.1)),
            gravity_speed: Some(v(14.0, 4.0)),
            linear_damp: Some(v(0.03, 0.0)),
            scale: Some(v(2.4, 1.4)),
            color: Some(c),
            scale_curve: Some(
                MultiCurve::new()
                    .with_point(0.0, 0.0, None)
                    .with_point(1.0, 0.5, None)
                    .with_point(0.4, 1.0, None),
            ),
            color_curve: Some(MultiCurve::new().with_point(c, 0.0, None).with_point(
                c.with_alpha(0.0),
                1.0,
                None,
            )),
            ..default()
        },
        AmbientKind::Dust => Particle2dEffect {
            spawn_rate: 0.12 / spec.rate.clamp(0.3, 3.0),
            spawn_amount: 1,
            emission_shape: EmissionShape::Circle(160.0),
            lifetime: v(8.0, 3.0),
            linear_speed: Some(v(4.0, 3.0)),
            direction: Some(Rval::new(Vec2::new(0.2, 0.1), 0.9)),
            linear_damp: Some(v(0.02, 0.0)),
            scale: Some(v(1.6, 1.0)),
            color: Some(c),
            color_curve: Some(MultiCurve::new().with_point(c, 0.0, None).with_point(
                c.with_alpha(0.0),
                1.0,
                None,
            )),
            ..default()
        },
        AmbientKind::Sparks => Particle2dEffect {
            spawn_rate: 0.07 / spec.rate.clamp(0.3, 3.0),
            spawn_amount: 1,
            emission_shape: EmissionShape::Circle(120.0),
            lifetime: v(1.8, 0.8),
            linear_speed: Some(v(30.0, 20.0)),
            direction: Some(Rval::new(Vec2::Y, 0.8)),
            gravity_direction: Some(Rval::new(Vec2::Y, 0.1)),
            gravity_speed: Some(v(40.0, 15.0)),
            scale: Some(v(1.6, 1.0)),
            color: Some(c),
            scale_curve: Some(
                MultiCurve::new()
                    .with_point(0.0, 0.0, None)
                    .with_point(1.0, 0.2, None)
                    .with_point(0.2, 1.0, None),
            ),
            color_curve: Some(MultiCurve::new().with_point(c, 0.0, None).with_point(
                c.with_alpha(0.0),
                1.0,
                None,
            )),
            ..default()
        },
        AmbientKind::Glitch => Particle2dEffect {
            spawn_rate: 0.04,
            spawn_amount: 1,
            emission_shape: EmissionShape::Circle(140.0),
            lifetime: v(0.5, 0.3),
            linear_speed: Some(v(60.0, 40.0)),
            direction: Some(Rval::new(Vec2::new(0.0, 1.0), 1.0)),
            scale: Some(v(3.0, 2.0)),
            color: Some(c),
            color_curve: Some(MultiCurve::new().with_point(c, 0.0, None).with_point(
                c.with_alpha(0.0),
                1.0,
                None,
            )),
            ..default()
        },
        AmbientKind::None => Particle2dEffect {
            spawn_rate: 0.0,
            ..default()
        },
    }
}
