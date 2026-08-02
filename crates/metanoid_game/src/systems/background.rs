use bevy::prelude::*;
use bevy_enoki::prelude::*;
use metanoid_core::constants::*;
use metanoid_core::settings::{GameSettings, ParticleQuality};
use metanoid_visuals::SkyMaterial;
use metanoid_visuals::ambient::{AmbientEmitter, spawn_ambient};
use metanoid_visuals::silhouette::{SilhouettePart, spawn_silhouette_layers};
use metanoid_visuals::sky::{SkyQuad, spawn_sky};

use super::level_progression::ActiveLevelVisuals;
use super::level_spawner::LevelEntity;

#[derive(Resource)]
pub struct BackgroundSpawned;

/// Tag any gameplay-scene visuals with `LevelEntity` so they're cleaned up
/// between levels (sky, silhouettes and ambient emitters spawned without it).
pub fn tag_level_scene(
    mut commands: Commands,
    sky: Query<Entity, (With<SkyQuad>, Without<LevelEntity>)>,
    silhouettes: Query<Entity, (With<SilhouettePart>, Without<LevelEntity>)>,
    ambient: Query<Entity, (With<AmbientEmitter>, Without<LevelEntity>)>,
) {
    for e in &sky {
        commands.entity(e).insert(LevelEntity);
    }
    for e in &silhouettes {
        commands.entity(e).insert(LevelEntity);
    }
    for e in &ambient {
        commands.entity(e).insert(LevelEntity);
    }
}

/// Build the full procedural backdrop for the current level's recipe:
/// shader sky, parallax silhouettes, ground strip and ambient particles.
pub fn setup_background(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut sky_materials: ResMut<Assets<SkyMaterial>>,
    mut enoki: ResMut<Assets<Particle2dEffect>>,
    visuals: Option<Res<ActiveLevelVisuals>>,
    settings: Res<GameSettings>,
    spawned: Option<Res<BackgroundSpawned>>,
) {
    if spawned.is_some() {
        return;
    }
    let Some(visuals) = visuals else {
        return;
    };
    commands.insert_resource(BackgroundSpawned);

    let recipe = &visuals.recipe;
    let reduce_motion = settings.reduce_motion;

    spawn_sky(
        &mut commands,
        &mut meshes,
        &mut sky_materials,
        recipe,
        reduce_motion,
    );
    spawn_silhouette_layers(&mut commands, &mut meshes, &mut materials, recipe);

    // Ground strip with glow line.
    let ground_mesh = meshes.add(Rectangle::new(ARENA_WIDTH + 240.0, 46.0));
    commands.spawn((
        LevelEntity,
        Mesh2d(ground_mesh),
        MeshMaterial2d(visuals.materials.ground.clone()),
        Transform::from_xyz(0.0, -ARENA_HEIGHT / 2.0 + 17.0, -3.0),
    ));
    let glow_mesh = meshes.add(Rectangle::new(ARENA_WIDTH + 240.0, 3.0));
    let glow_mat = materials.add(ColorMaterial::from_color(recipe.ground.glow));
    commands.spawn((
        LevelEntity,
        Mesh2d(glow_mesh),
        MeshMaterial2d(glow_mat),
        Transform::from_xyz(0.0, -ARENA_HEIGHT / 2.0 + 41.0, -2.5),
    ));

    let particle_scale = match settings.particle_quality {
        ParticleQuality::Low => 0.4,
        ParticleQuality::Medium => 1.0,
        ParticleQuality::High => 1.6,
    };
    spawn_ambient(
        &mut commands,
        &mut enoki,
        recipe,
        particle_scale,
        reduce_motion,
    );
}
