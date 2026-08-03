use avian2d::prelude::*;
use bevy::prelude::*;
use metanoid_core::components::powerup::{PowerUp, PowerUpKind};
use metanoid_core::constants::*;
use metanoid_core::events::BrickDestroyedEvent;
use rand::prelude::*;

use crate::systems::level_progression::ActiveLevelVisuals;
use crate::systems::level_spawner::LevelEntity;

#[derive(Resource)]
pub struct PowerUpState {
    pub bricks_since_drop: usize,
}

impl Default for PowerUpState {
    fn default() -> Self {
        Self {
            bricks_since_drop: 0,
        }
    }
}

const PITY_THRESHOLD: usize = 10;
const BASE_DROP_CHANCE: f32 = 0.12;

/// Noble glow aura on a powerup: a soft halo + a thin ring that gently breathes.
#[derive(Component)]
pub struct PowerUpGlow {
    pub phase: f32,
    pub base: Srgba,
    pub base_size: Vec2,
    /// Rotate the sprite (used for the ring).
    pub rotate: bool,
}

/// Gentle breathing for powerup halos + rings.
pub fn tick_powerup_glow(
    time: Res<Time>,
    mut q: Query<(&PowerUpGlow, &mut Sprite, &mut Transform)>,
) {
    let t = time.elapsed_secs();
    for (glow, mut sprite, mut transform) in &mut q {
        let breathe = 0.5 + 0.5 * (t * 2.6 + glow.phase).sin();
        sprite.color = Color::srgba(
            glow.base.red,
            glow.base.green,
            glow.base.blue,
            (glow.base.alpha * (0.55 + 0.45 * breathe)).clamp(0.0, 1.0),
        );
        sprite.custom_size = Some(glow.base_size * (1.0 + 0.14 * breathe));
        if glow.rotate {
            transform.rotate_z(t * 1.2 + glow.phase);
        }
    }
}

pub fn spawn_powerup_on_destroy(
    trigger: On<BrickDestroyedEvent>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut powerup_state: ResMut<PowerUpState>,
    visuals: Option<Res<ActiveLevelVisuals>>,
) {
    powerup_state.bricks_since_drop += 1;

    let pos = trigger.position;

    let mut rng = rand::rng();
    let mut chance = BASE_DROP_CHANCE;

    if powerup_state.bricks_since_drop >= PITY_THRESHOLD {
        chance = 1.0;
    }

    if rng.random::<f32>() >= chance {
        return;
    }

    powerup_state.bricks_since_drop = 0;

    let kind = random_powerup_kind(&mut rng);

    let mut entity = commands.spawn((
        PowerUp {
            kind,
            fall_speed: POWERUP_FALL_SPEED,
        },
        LevelEntity,
        RigidBody::Kinematic,
        Collider::circle(POWERUP_RADIUS),
        Sensor,
        super::super::physics_layers::layers_powerup(),
        CollisionEventsEnabled,
        Transform::from_xyz(pos.x, pos.y, 0.0),
    ));

    if let Some(v) = visuals.as_ref() {
        // Distinctive procedural texture per kind + a noble glow halo + ring.
        let accent = powerup_color(kind);
        let accent_srgba = Srgba::from(accent);
        let texture = v.materials.powerup_texture(kind);
        let glow_tex = v.materials.glow.clone();
        let ring_tex = v.materials.ring.clone();

        entity.insert((Sprite {
            image: texture,
            color: Color::WHITE,
            custom_size: Some(Vec2::splat(POWERUP_RADIUS * 2.0)),
            ..default()
        },));

        entity.with_children(|parent| {
            parent.spawn((
                PowerUpGlow {
                    phase: rng.random::<f32>() * 6.2832,
                    base: Srgba::new(accent_srgba.red, accent_srgba.green, accent_srgba.blue, 0.5),
                    base_size: Vec2::splat(POWERUP_RADIUS * 3.4),
                    rotate: false,
                },
                Sprite {
                    image: glow_tex,
                    color: Color::NONE,
                    custom_size: Some(Vec2::splat(POWERUP_RADIUS * 3.4)),
                    ..default()
                },
                Transform::from_xyz(0.0, 0.0, -0.1),
            ));
            parent.spawn((
                PowerUpGlow {
                    phase: rng.random::<f32>() * 6.2832,
                    base: Srgba::new(accent_srgba.red, accent_srgba.green, accent_srgba.blue, 0.7),
                    base_size: Vec2::splat(POWERUP_RADIUS * 3.0),
                    rotate: true,
                },
                Sprite {
                    image: ring_tex,
                    color: Color::NONE,
                    custom_size: Some(Vec2::splat(POWERUP_RADIUS * 3.0)),
                    ..default()
                },
                Transform::from_xyz(0.0, 0.0, 0.1),
            ));
        });
    } else {
        // Fallback: plain colored disc.
        let color = powerup_color(kind);
        let mesh = meshes.add(Circle::new(POWERUP_RADIUS));
        let material = materials.add(color);
        entity.insert((Mesh2d(mesh), MeshMaterial2d(material)));
    }
}

pub fn despawn_offscreen_powerups(
    mut commands: Commands,
    query: Query<(Entity, &Transform), With<PowerUp>>,
) {
    for (entity, transform) in &query {
        if transform.translation.y < -ARENA_HEIGHT / 2.0 - 50.0 {
            commands.entity(entity).try_despawn();
        }
    }
}

pub fn fall_powerups(time: Res<Time>, mut query: Query<(&PowerUp, &mut Transform)>) {
    for (powerup, mut transform) in &mut query {
        transform.translation.y -= powerup.fall_speed * time.delta_secs();
    }
}

fn random_powerup_kind(rng: &mut impl Rng) -> PowerUpKind {
    match rng.random_range(0..23) {
        0 => PowerUpKind::Fireball,
        1 => PowerUpKind::MegaBall,
        2 => PowerUpKind::SplitBall,
        3 => PowerUpKind::FastBall,
        4 => PowerUpKind::SlowBall,
        5 => PowerUpKind::LaserPaddle,
        6 => PowerUpKind::GrabPaddle,
        7 => PowerUpKind::ExpandPaddle,
        8 => PowerUpKind::ShrinkPaddle,
        9 => PowerUpKind::Shield,
        10 => PowerUpKind::ExtraLife,
        11 => PowerUpKind::DoublePoints,
        12 => PowerUpKind::LevelWarp,
        13 => PowerUpKind::KillPaddle,
        14 => PowerUpKind::TimeSlow,
        15 => PowerUpKind::FallingBricks,
        16 => PowerUpKind::Zap,
        17 => PowerUpKind::Explode,
        18 => PowerUpKind::ExpandExploding,
        19 => PowerUpKind::Lightning,
        20 => PowerUpKind::Shockwave,
        21 => PowerUpKind::ShuffleBricks,
        _ => PowerUpKind::Blackout,
    }
}

fn powerup_color(kind: PowerUpKind) -> Color {
    match kind {
        PowerUpKind::Fireball => Color::srgb(1.0, 0.3, 0.0),
        PowerUpKind::MegaBall => Color::srgb(1.0, 1.0, 0.0),
        PowerUpKind::SplitBall => Color::srgb(0.0, 1.0, 0.5),
        PowerUpKind::FastBall => Color::srgb(1.0, 0.5, 0.0),
        PowerUpKind::SlowBall => Color::srgb(0.0, 0.5, 1.0),
        PowerUpKind::LaserPaddle => Color::srgb(1.0, 0.0, 0.0),
        PowerUpKind::GrabPaddle => Color::srgb(0.5, 0.0, 1.0),
        PowerUpKind::ExpandPaddle => Color::srgb(0.0, 1.0, 1.0),
        PowerUpKind::ShrinkPaddle => Color::srgb(0.8, 0.0, 0.5),
        PowerUpKind::Shield => Color::srgb(0.0, 0.8, 1.0),
        PowerUpKind::ExtraLife => Color::srgb(1.0, 0.0, 0.5),
        PowerUpKind::DoublePoints => Color::srgb(1.0, 1.0, 0.5),
        PowerUpKind::LevelWarp => Color::srgb(0.5, 1.0, 0.0),
        PowerUpKind::KillPaddle => Color::srgb(0.2, 0.0, 0.0),
        PowerUpKind::TimeSlow => Color::srgb(0.3, 0.3, 1.0),
        PowerUpKind::FallingBricks => Color::srgb(0.6, 0.4, 0.2),
        PowerUpKind::Zap => Color::srgb(1.0, 1.0, 0.8),
        PowerUpKind::Explode => Color::srgb(1.0, 0.1, 0.0),
        PowerUpKind::ExpandExploding => Color::srgb(1.0, 0.4, 0.0),
        PowerUpKind::Lightning => Color::srgb(1.0, 1.0, 0.0),
        PowerUpKind::Shockwave => Color::srgb(0.5, 0.5, 1.0),
        PowerUpKind::ShuffleBricks => Color::srgb(0.8, 0.5, 1.0),
        PowerUpKind::Blackout => Color::srgb(0.1, 0.0, 0.2),
    }
}
