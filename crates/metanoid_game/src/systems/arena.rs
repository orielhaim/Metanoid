use avian2d::prelude::*;
use bevy::camera::{Hdr, ScalingMode};
use bevy::post_process::bloom::Bloom;
use bevy::post_process::effect_stack::{ChromaticAberration, LensDistortion, Vignette};
use bevy::prelude::*;
use bevy::render::view::{ColorGrading, ColorGradingGlobal, ColorGradingSection};
use bevy_trauma_shake::prelude::*;
use metanoid_core::components::paddle::Paddle;
use metanoid_core::constants::*;

use super::level_progression::ActiveLevelVisuals;
use super::physics_layers::{layers_paddle, layers_wall};

#[derive(Component)]
pub struct GameCamera;

#[derive(Component)]
pub struct ArenaEntity;

/// Marker for arena walls (for bounce SFX / wall hit events).
#[derive(Component)]
pub struct Wall;

#[derive(Resource)]
pub struct ArenaSpawned;

/// Spawn a single persistent camera on Startup. Never despawned.
pub fn setup_persistent_camera(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        Hdr,
        GameCamera,
        Projection::from(OrthographicProjection {
            scaling_mode: ScalingMode::FixedVertical {
                viewport_height: ARENA_HEIGHT,
            },
            ..OrthographicProjection::default_2d()
        }),
    ));
}

/// Add post-processing components to the existing camera when entering gameplay.
pub fn setup_camera_effects(
    mut commands: Commands,
    cameras: Query<Entity, With<GameCamera>>,
    active: Option<Res<CameraFxActive>>,
) {
    if active.is_some() {
        return;
    }
    commands.insert_resource(CameraFxActive);
    for entity in &cameras {
        commands.entity(entity).insert((
            Shake::default(),
            Bloom {
                intensity: 0.15,
                ..default()
            },
            ChromaticAberration {
                intensity: 0.0,
                ..default()
            },
            Vignette {
                intensity: 0.3,
                radius: 0.8,
                smoothness: 5.0,
                roundness: 1.0,
                ..default()
            },
            LensDistortion {
                intensity: 0.0,
                ..default()
            },
            // Slightly punchier, crisper image.
            ColorGrading {
                global: ColorGradingGlobal {
                    post_saturation: 1.04,
                    ..default()
                },
                shadows: ColorGradingSection {
                    contrast: 1.10,
                    saturation: 0.98,
                    ..default()
                },
                midtones: ColorGradingSection {
                    contrast: 1.12,
                    ..default()
                },
                highlights: ColorGradingSection {
                    contrast: 1.06,
                    ..default()
                },
            },
        ));
    }
}

/// Remove post-processing when leaving gameplay.
pub fn teardown_camera_effects(mut commands: Commands, cameras: Query<Entity, With<GameCamera>>) {
    commands.remove_resource::<CameraFxActive>();
    for entity in &cameras {
        commands
            .entity(entity)
            .remove::<Shake>()
            .remove::<Bloom>()
            .remove::<ChromaticAberration>()
            .remove::<Vignette>()
            .remove::<LensDistortion>();
    }
}

/// Guards `setup_camera_effects` against re-insertion every frame during Loading.
#[derive(Resource)]
pub struct CameraFxActive;

pub fn setup_arena(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    arena_spawned: Option<Res<ArenaSpawned>>,
    visuals: Option<Res<ActiveLevelVisuals>>,
) {
    if arena_spawned.is_some() {
        return;
    }
    commands.insert_resource(ArenaSpawned);
    commands.insert_resource(Gravity(Vec2::ZERO));

    let wall_mesh = meshes.add(Rectangle::new(ARENA_WIDTH, WALL_THICKNESS));
    let wall_mat = visuals
        .as_ref()
        .map(|v| v.materials.wall.clone())
        .unwrap_or_else(|| materials.add(Color::srgb(0.3, 0.3, 0.3)));
    let half_w = (ARENA_WIDTH - WALL_THICKNESS) / 2.0;
    let half_h = (ARENA_HEIGHT - WALL_THICKNESS) / 2.0;

    for (x, y, w, h) in [
        (0.0, half_h, ARENA_WIDTH, WALL_THICKNESS),
        (-half_w, 0.0, WALL_THICKNESS, ARENA_HEIGHT),
        (half_w, 0.0, WALL_THICKNESS, ARENA_HEIGHT),
    ] {
        commands.spawn((
            ArenaEntity,
            Wall,
            RigidBody::Static,
            Collider::rectangle(w, h),
            Transform::from_xyz(x, y, 0.0),
            layers_wall(),
            CollisionEventsEnabled,
            Mesh2d(wall_mesh.clone()),
            MeshMaterial2d(wall_mat.clone()),
        ));
    }

    // Accent glow frame along the inner edges of the walls.
    if let Some(v) = visuals.as_ref() {
        let accent: Srgba = Srgba::from(v.recipe.palette.accent);
        let glow_color = Color::srgb(accent.red, accent.green, accent.blue);
        let glow_mat = materials.add(ColorMaterial::from_color(glow_color));
        let inner_x = half_w - WALL_THICKNESS / 2.0;
        let inner_y = half_h - WALL_THICKNESS / 2.0;
        let border_mesh = meshes.add(Rectangle::new(2.0, ARENA_HEIGHT));
        commands.spawn((
            ArenaEntity,
            Mesh2d(border_mesh.clone()),
            MeshMaterial2d(glow_mat.clone()),
            Transform::from_xyz(-inner_x, 0.0, 1.0),
        ));
        commands.spawn((
            ArenaEntity,
            Mesh2d(border_mesh),
            MeshMaterial2d(glow_mat.clone()),
            Transform::from_xyz(inner_x, 0.0, 1.0),
        ));
        let top_mesh = meshes.add(Rectangle::new(ARENA_WIDTH, 2.0));
        commands.spawn((
            ArenaEntity,
            Mesh2d(top_mesh),
            MeshMaterial2d(glow_mat),
            Transform::from_xyz(0.0, inner_y, 1.0),
        ));
    }

    let paddle_mesh = meshes.add(Rectangle::new(PADDLE_WIDTH, PADDLE_HEIGHT));
    let paddle_mat = visuals
        .as_ref()
        .map(|v| v.materials.paddle.clone())
        .unwrap_or_else(|| materials.add(Color::srgb(0.9, 0.9, 0.95)));
    commands.spawn((
        ArenaEntity,
        Paddle::default(),
        RigidBody::Kinematic,
        Collider::rectangle(PADDLE_WIDTH, PADDLE_HEIGHT),
        Transform::from_xyz(0.0, PADDLE_Y, 0.0),
        LinearVelocity::default(),
        layers_paddle(),
        CollisionEventsEnabled,
        Mesh2d(paddle_mesh),
        MeshMaterial2d(paddle_mat),
    ));
}
