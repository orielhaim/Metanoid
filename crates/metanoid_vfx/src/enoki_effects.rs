use bevy::prelude::*;
use bevy_enoki::prelude::*;

#[derive(Resource)]
pub struct EnokiEffects {
    pub powerup_aura: Handle<Particle2dEffect>,
    pub shield_shimmer: Handle<Particle2dEffect>,
}

pub fn setup_enoki_effects(mut commands: Commands, mut effects: ResMut<Assets<Particle2dEffect>>) {
    let powerup_aura = effects.add(Particle2dEffect {
        spawn_rate: 0.1,
        spawn_amount: 1,
        emission_shape: EmissionShape::Circle(10.0),
        lifetime: Rval::new(0.4, 0.2),
        linear_speed: Some(Rval::new(15.0, 0.5)),
        direction: Some(Rval::new(Vec2::Y, 1.0)),
        scale: Some(Rval::new(3.0, 1.0)),
        color: Some(LinearRgba::new(1.0, 1.0, 0.5, 0.8)),
        scale_curve: Some(
            MultiCurve::new()
                .with_point(1.0, 0.0, None)
                .with_point(1.5, 0.5, None)
                .with_point(0.0, 1.0, None),
        ),
        color_curve: Some(
            MultiCurve::new()
                .with_point(LinearRgba::new(1.0, 1.0, 0.5, 0.8), 0.0, None)
                .with_point(LinearRgba::new(1.0, 0.8, 0.0, 0.5), 0.5, None)
                .with_point(LinearRgba::new(1.0, 0.5, 0.0, 0.0), 1.0, None),
        ),
        ..default()
    });

    let shield_shimmer = effects.add(Particle2dEffect {
        spawn_rate: 0.01,
        spawn_amount: 3,
        emission_shape: EmissionShape::Circle(200.0),
        lifetime: Rval::new(0.8, 0.3),
        linear_speed: Some(Rval::new(10.0, 0.5)),
        direction: Some(Rval::new(Vec2::Y, 1.0)),
        scale: Some(Rval::new(2.0, 1.0)),
        color: Some(LinearRgba::new(0.0, 0.8, 1.0, 0.6)),
        scale_curve: Some(
            MultiCurve::new()
                .with_point(0.5, 0.0, None)
                .with_point(1.0, 0.5, None)
                .with_point(0.0, 1.0, None),
        ),
        color_curve: Some(
            MultiCurve::new()
                .with_point(LinearRgba::new(0.0, 0.8, 1.0, 0.6), 0.0, None)
                .with_point(LinearRgba::new(0.0, 0.4, 0.8, 0.0), 1.0, None),
        ),
        ..default()
    });

    commands.insert_resource(EnokiEffects {
        powerup_aura,
        shield_shimmer,
    });
}
