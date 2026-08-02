//! Animated procedural backdrop shared by the menu and galaxy map screens:
//! a drifting shader sky + silhouette monoliths + ambient sparks.

use bevy::prelude::*;
use bevy_enoki::prelude::*;
use metanoid_procgen::biome::composition::{BiomeComposition, BiomePart};
use metanoid_procgen::biome::parameters::BiomeParams;
use metanoid_visuals::SkyMaterial;
use metanoid_visuals::ambient::spawn_ambient;
use metanoid_visuals::recipe::flavor::recipe_for;
use metanoid_visuals::silhouette::spawn_silhouette_layers;
use metanoid_visuals::sky::spawn_sky;

#[derive(Component)]
pub struct UiBackdrop;

pub fn setup_ui_backdrop(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut sky_materials: ResMut<Assets<SkyMaterial>>,
    mut enoki: ResMut<Assets<Particle2dEffect>>,
) {
    let comp = BiomeComposition {
        params: BiomeParams {
            temperature: 0.5,
            density: 0.25,
            chaos: 0.4,
            energy: 0.75,
            weirdness: 0.85,
        },
        parts: vec![
            BiomePart {
                name: "Cosmic Void",
                weight: 0.7,
            },
            BiomePart {
                name: "Neon City",
                weight: 0.3,
            },
        ],
    };
    let recipe = recipe_for(&comp, 0x5EED);

    spawn_sky(
        &mut commands,
        &mut meshes,
        &mut sky_materials,
        &recipe,
        false,
    );
    spawn_ambient(&mut commands, &mut enoki, &recipe, 1.0, false);
    spawn_silhouette_layers(&mut commands, &mut meshes, &mut materials, &recipe);
}

pub fn tag_ui_backdrop(
    mut commands: Commands,
    sky: Query<
        Entity,
        (
            With<metanoid_visuals::SkyQuad>,
            Without<super::level_spawner::LevelEntity>,
        ),
    >,
    silhouette: Query<
        Entity,
        (
            With<metanoid_visuals::SilhouettePart>,
            Without<super::level_spawner::LevelEntity>,
        ),
    >,
    ambient: Query<
        Entity,
        (
            With<metanoid_visuals::ambient::AmbientEmitter>,
            Without<super::level_spawner::LevelEntity>,
        ),
    >,
) {
    for e in &sky {
        commands.entity(e).insert(UiBackdrop);
    }
    for e in &silhouette {
        commands.entity(e).insert(UiBackdrop);
    }
    for e in &ambient {
        commands.entity(e).insert(UiBackdrop);
    }
}

pub fn teardown_ui_backdrop(mut commands: Commands, q: Query<Entity, With<UiBackdrop>>) {
    for e in &q {
        commands.entity(e).despawn();
    }
}
