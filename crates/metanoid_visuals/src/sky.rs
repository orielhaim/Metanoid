//! Procedural sky: a custom `Material2d` driven by a WGSL fragment shader.
//! All animation reads the engine-provided `globals.time`, so the material is
//! set once at spawn and never mutated per frame (mutating a material asset
//! every frame forces constant GPU re-preparation and can cause flicker).

use bevy::prelude::*;
use bevy::reflect::TypePath;
use bevy::render::render_resource::AsBindGroup;
use bevy::shader::ShaderRef;
use bevy::sprite_render::Material2d;

use crate::recipe::BiomeRecipe;

pub const SKY_SHADER_PATH: &str = "shaders/sky_material.wgsl";

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct SkyMaterial {
    #[uniform(0)]
    pub top: LinearRgba,
    #[uniform(1)]
    pub bottom: LinearRgba,
    #[uniform(2)]
    pub nebula: LinearRgba,
    /// x = nebula strength, y = star density, z = aurora, w = unused.
    #[uniform(3)]
    pub params: Vec4,
    /// x = motion scale (0 with reduced-motion), y = drift phase seed.
    #[uniform(4)]
    pub seed: Vec4,
}

impl Material2d for SkyMaterial {
    fn fragment_shader() -> ShaderRef {
        ShaderRef::Path(SKY_SHADER_PATH.into())
    }
}

/// Marker on the sky quad entity so the game can find/clean it up.
#[derive(Component)]
pub struct SkyQuad;

/// Spawn the fullscreen sky quad for a recipe.
pub fn spawn_sky(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<SkyMaterial>,
    recipe: &BiomeRecipe,
    reduce_motion: bool,
) {
    let quad = meshes.add(Rectangle::new(2560.0, 1440.0));
    let params = Vec4::new(
        recipe.sky.nebula_strength,
        recipe.sky.star_density,
        recipe.sky.aurora,
        0.0,
    );
    let phase = (recipe.var_seed % 997) as f32 / 997.0;
    let seed = Vec4::new(if reduce_motion { 0.0 } else { 1.0 }, phase, 0.0, 0.0);
    let material = materials.add(SkyMaterial {
        top: recipe.sky.top,
        bottom: recipe.sky.bottom,
        nebula: recipe.sky.nebula,
        params,
        seed,
    });
    commands.spawn((
        SkyQuad,
        Mesh2d(quad),
        MeshMaterial2d(material),
        Transform::from_xyz(0.0, 0.0, -20.0),
    ));
}
