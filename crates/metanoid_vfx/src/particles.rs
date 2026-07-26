use bevy::prelude::*;
use bevy_hanabi::prelude::*;
use bevy_hanabi::Gradient as HanabiGradient;
use bevy_enoki::EnokiPlugin;
use bevy_trauma_shake::prelude::TraumaPlugin;
use bevy_tweening::TweeningPlugin;

pub struct VfxPlugin;

impl Plugin for VfxPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(HanabiPlugin)
            .add_plugins(EnokiPlugin)
            .add_plugins(TraumaPlugin)
            .add_plugins(TweeningPlugin);
    }
}

#[derive(Resource)]
pub struct ParticleEffects {
    pub brick_break: Handle<EffectAsset>,
    pub ball_trail: Handle<EffectAsset>,
    pub explosion: Handle<EffectAsset>,
}

pub fn setup_particle_effects(
    mut commands: Commands,
    mut effects: ResMut<Assets<EffectAsset>>,
) {
    let brick_break = create_brick_break_effect(&mut effects);
    let ball_trail = create_ball_trail_effect(&mut effects);
    let explosion = create_explosion_effect(&mut effects);

    commands.insert_resource(ParticleEffects {
        brick_break,
        ball_trail,
        explosion,
    });
}

fn create_brick_break_effect(effects: &mut Assets<EffectAsset>) -> Handle<EffectAsset> {
    let mut module = Module::default();

    let init_pos = SetPositionCircleModifier {
        center: module.lit(Vec3::ZERO),
        axis: module.lit(Vec3::Z),
        radius: module.lit(5.0),
        dimension: ShapeDimension::Surface,
    };

    let init_vel = SetVelocityCircleModifier {
        center: module.lit(Vec3::ZERO),
        axis: module.lit(Vec3::Z),
        speed: module.lit(Vec3::new(80.0, 200.0, 0.0)),
    };

    let init_lifetime = SetAttributeModifier::new(Attribute::LIFETIME, module.lit(0.5));
    let drag = LinearDragModifier::new(module.lit(2.0));

    let color = ColorOverLifetimeModifier::new(HanabiGradient::linear(
        Vec4::new(1.0, 1.0, 1.0, 1.0),
        Vec4::new(1.0, 1.0, 1.0, 0.0),
    ));

    let size = SizeOverLifetimeModifier {
        gradient: HanabiGradient::linear(Vec3::splat(4.0), Vec3::splat(0.0)),
        screen_space_size: false,
    };

    effects.add(
        EffectAsset::new(512, SpawnerSettings::once(20.0.into()), module)
            .with_name("brick_break")
            .init(init_pos)
            .init(init_vel)
            .init(init_lifetime)
            .update(drag)
            .render(color)
            .render(size),
    )
}

fn create_ball_trail_effect(effects: &mut Assets<EffectAsset>) -> Handle<EffectAsset> {
    let mut module = Module::default();

    let init_pos = SetPositionCircleModifier {
        center: module.lit(Vec3::ZERO),
        axis: module.lit(Vec3::Z),
        radius: module.lit(3.0),
        dimension: ShapeDimension::Surface,
    };

    let init_vel = SetVelocitySphereModifier {
        center: module.lit(Vec3::ZERO),
        speed: module.lit(Vec3::new(5.0, 15.0, 0.0)),
    };

    let init_lifetime = SetAttributeModifier::new(Attribute::LIFETIME, module.lit(0.3));

    let color = ColorOverLifetimeModifier::new(HanabiGradient::linear(
        Vec4::new(1.0, 1.0, 1.0, 0.8),
        Vec4::new(1.0, 1.0, 1.0, 0.0),
    ));

    let size = SizeOverLifetimeModifier {
        gradient: HanabiGradient::linear(Vec3::splat(3.0), Vec3::splat(0.0)),
        screen_space_size: false,
    };

    effects.add(
        EffectAsset::new(256, SpawnerSettings::rate(30.0.into()), module)
            .with_name("ball_trail")
            .init(init_pos)
            .init(init_vel)
            .init(init_lifetime)
            .render(color)
            .render(size),
    )
}

fn create_explosion_effect(effects: &mut Assets<EffectAsset>) -> Handle<EffectAsset> {
    let mut module = Module::default();

    let init_pos = SetPositionSphereModifier {
        center: module.lit(Vec3::ZERO),
        radius: module.lit(10.0),
        dimension: ShapeDimension::Surface,
    };

    let init_vel = SetVelocitySphereModifier {
        center: module.lit(Vec3::ZERO),
        speed: module.lit(Vec3::new(100.0, 300.0, 0.0)),
    };

    let init_lifetime = SetAttributeModifier::new(Attribute::LIFETIME, module.lit(0.8));
    let drag = LinearDragModifier::new(module.lit(3.0));

    let color = ColorOverLifetimeModifier::new(HanabiGradient::from_keys([
        (0.0, Vec4::new(1.0, 0.5, 0.0, 1.0)),
        (0.3, Vec4::new(1.0, 0.2, 0.0, 0.8)),
        (1.0, Vec4::new(0.5, 0.0, 0.0, 0.0)),
    ]));

    let size = SizeOverLifetimeModifier {
        gradient: HanabiGradient::linear(Vec3::splat(6.0), Vec3::splat(0.0)),
        screen_space_size: false,
    };

    effects.add(
        EffectAsset::new(512, SpawnerSettings::once(50.0.into()), module)
            .with_name("explosion")
            .init(init_pos)
            .init(init_vel)
            .init(init_lifetime)
            .update(drag)
            .render(color)
            .render(size),
    )
}
