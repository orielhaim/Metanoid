//! Metanoid's procedural assets engine.
//!
//! Turns a biome's `BiomeComposition` (pure logic, from `metanoid_procgen`)
//! into concrete, spectacular visuals: per-biome recipes, procedural textures,
//! a shader-driven sky, parallax silhouettes, ambient particles and the
//! curtain-reveal loading transition.

pub mod ambient;
pub mod material;
pub mod recipe;
pub mod silhouette;
pub mod sky;
pub mod transition;

use bevy::prelude::*;
use bevy::sprite_render::Material2dPlugin;

pub use material::{BrickMatKind, ProceduralMaterials};
pub use recipe::{BiomeRecipe, LevelVisualContext, recipe_for_context};
pub use silhouette::{ParallaxLayer, SilhouettePart, Sway};
pub use sky::{SkyMaterial, SkyQuad};
pub use transition::{Curtain, CurtainPanel, CurtainRoot};

pub struct VisualsPlugin;

impl Plugin for VisualsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(Material2dPlugin::<SkyMaterial>::default())
            .init_resource::<ProceduralMaterials>()
            .add_systems(
                Update,
                (silhouette::tick_silhouette_sway, transition::tick_curtain),
            );
    }
}
