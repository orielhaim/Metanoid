//! The resolved, concrete visual definition for a biome (or blended biomes).
//!
//! A `BiomeRecipe` is the *output* of the procedural assets engine: everything
//! the game needs to render a stage — sky, parallax silhouettes, ground,
//! per-brick textures, entity materials, ambient particles, lighting and the
//! loading curtain. Recipes are fully interpolable so adjacent biomes blend
//! smoothly and hybrids ("forest on fire") emerge naturally.

use bevy::color::Mix;
use bevy::prelude::*;

use metanoid_procgen::biome::composition::BiomeComposition;

pub mod flavor;
pub mod texture;

pub use flavor::Flavor;

/// The kind of parallax silhouette layer decorating the horizon.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SilhouetteKind {
    Trees,
    Crags,
    Crystals,
    Dunes,
    Skyline,
    FrozenSpires,
    Coral,
    Monoliths,
}

/// Procedural texture recipe for bricks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextureKind {
    Wood,
    Stone,
    Lava,
    Ice,
    Crystal,
    Neon,
    Metal,
    Cosmic,
    Charred,
}

/// Ground styling.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GroundKind {
    Grass,
    Stone,
    Lava,
    Ice,
    CrystalFloor,
    NeonGrid,
    Sand,
    Void,
}

/// Ambient particle motif drifting through the arena.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AmbientKind {
    Pollen,
    Embers,
    Bubbles,
    Snow,
    Dust,
    Sparks,
    Glitch,
    None,
}

/// Loading-curtain motif (what covers the screen before the stage reveals).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CurtainKind {
    Trees,
    Columns,
    Shards,
    Blinds,
    Void,
    IceWalls,
}

/// A single brick's material recipe.
#[derive(Debug, Clone, Copy)]
pub struct BrickMat {
    pub base: LinearRgba,
    pub texture: TextureKind,
    pub glow: LinearRgba,
    pub glow_strength: f32,
}

/// Full recipe for all brick kinds.
#[derive(Debug, Clone, Copy)]
pub struct BrickMaterialSpec {
    pub normal: BrickMat,
    pub multihit: BrickMat,
    pub invincible: BrickMat,
    pub explosive: BrickMat,
}

/// Entity materials (paddle / walls / ball).
#[derive(Debug, Clone, Copy)]
pub struct EntityMaterialSpec {
    pub paddle: LinearRgba,
    pub paddle_glow: LinearRgba,
    pub wall: LinearRgba,
    pub ball: LinearRgba,
    pub ball_glow: LinearRgba,
}

/// Sky shader parameters.
#[derive(Debug, Clone, Copy)]
pub struct SkySpec {
    pub top: LinearRgba,
    pub bottom: LinearRgba,
    pub nebula: LinearRgba,
    pub nebula_strength: f32,
    pub star_density: f32,
    pub aurora: f32,
}

/// Parallax silhouette parameters.
#[derive(Debug, Clone, Copy)]
pub struct SilhouetteSpec {
    pub kind: SilhouetteKind,
    pub far: LinearRgba,
    pub near: LinearRgba,
    pub density: f32,
    pub sway: f32,
}

/// Ground strip parameters.
#[derive(Debug, Clone, Copy)]
pub struct GroundSpec {
    pub kind: GroundKind,
    pub color: LinearRgba,
    pub glow: LinearRgba,
}

/// Ambient particle parameters.
#[derive(Debug, Clone, Copy)]
pub struct AmbientSpec {
    pub kind: AmbientKind,
    pub color: LinearRgba,
    pub rate: f32,
    pub speed: f32,
}

/// Lighting / post-processing parameters.
#[derive(Debug, Clone, Copy)]
pub struct LightSpec {
    pub bloom: f32,
    pub chromatic: f32,
    pub vignette: f32,
    pub lens: f32,
    pub tint: LinearRgba,
}

/// Loading-curtain parameters.
#[derive(Debug, Clone, Copy)]
pub struct CurtainSpec {
    pub kind: CurtainKind,
    pub primary: LinearRgba,
    pub secondary: LinearRgba,
}

/// The complete, resolved visual recipe for a level's biome.
#[derive(Debug, Clone)]
pub struct BiomeRecipe {
    pub name: String,
    pub palette: BiomeRecipePalette,
    pub sky: SkySpec,
    pub silhouettes: SilhouetteSpec,
    pub ground: GroundSpec,
    pub bricks: BrickMaterialSpec,
    pub entities: EntityMaterialSpec,
    pub ambient: AmbientSpec,
    pub light: LightSpec,
    pub curtain: CurtainSpec,
    /// Deterministic per-level texture variation seed.
    pub var_seed: u64,
}

/// A few named colors the UI / HUD can reach for (biome accent theming).
#[derive(Debug, Clone, Copy)]
pub struct BiomeRecipePalette {
    pub primary: LinearRgba,
    pub accent: LinearRgba,
    pub glow: LinearRgba,
    pub background: LinearRgba,
}

impl BiomeRecipe {
    /// Convenience: the recipe most representative of a single composition.
    pub fn for_composition(comp: &BiomeComposition, var_seed: u64) -> Self {
        flavor::recipe_for(comp, var_seed)
    }
}

/// Context for building a *per-level* recipe that blends neighboring biomes.
#[derive(Debug, Clone)]
pub struct LevelVisualContext {
    pub current: BiomeComposition,
    pub prev: BiomeComposition,
    pub next: BiomeComposition,
    /// 0.0 at the start of a biome, 1.0 right before the boss.
    pub progress: f32,
    pub is_boss: bool,
    pub var_seed: u64,
}

/// Build the recipe for a level, blending previous/current/next biomes so
/// entering a lava biome after a forest shows a burning-grove transition.
pub fn recipe_for_context(ctx: &LevelVisualContext) -> BiomeRecipe {
    let r_prev = flavor::recipe_for(&ctx.prev, ctx.var_seed ^ 0x51_7c_c1);
    let r_cur = flavor::recipe_for(&ctx.current, ctx.var_seed);
    let r_next = flavor::recipe_for(&ctx.next, ctx.var_seed ^ 0x9e_37_79);

    let w_prev = 0.24 * (1.0 - ctx.progress);
    let w_next = 0.24 * ctx.progress;
    let w_cur = 1.0 - w_prev - w_next;

    let name = if ctx.is_boss {
        format!("{}  (Boss)", r_cur.name)
    } else {
        r_cur.name.clone()
    };

    let mut mixed = mix_recipes(&[(r_prev, w_prev), (r_cur, w_cur), (r_next, w_next)]);
    mixed.name = name;
    mixed.var_seed = ctx.var_seed;
    mixed
}

/// Weighted blend of any number of recipes. Continuous fields are interpolated;
/// categorical fields (texture/silhouette/ground/ambient/curtain) come from the
/// recipe with the highest weight.
pub fn mix_recipes(recipes: &[(BiomeRecipe, f32)]) -> BiomeRecipe {
    let total: f32 = recipes.iter().map(|(_, w)| w).sum();
    if recipes.is_empty() {
        return flavor::default_recipe(0);
    }
    if total <= 0.0 {
        return recipes[0].0.clone();
    }

    // Continuous mixing helpers.
    let mix_f32 = |f: &dyn Fn(&BiomeRecipe) -> f32| -> f32 {
        recipes.iter().map(|(r, w)| f(r) * w / total).sum()
    };
    let mix_color = |f: &dyn Fn(&BiomeRecipe) -> LinearRgba| -> LinearRgba {
        let mut acc = LinearRgba::new(0.0, 0.0, 0.0, 0.0);
        for (r, w) in recipes {
            acc = acc.mix(&f(r), w / total);
        }
        acc
    };
    let dominant = |f: &dyn Fn(&BiomeRecipe) -> f32| -> usize {
        recipes
            .iter()
            .enumerate()
            .max_by(|(_, (r1, w1)), (_, (r2, w2))| {
                let v1 = f(r1) * w1;
                let v2 = f(r2) * w2;
                v1.partial_cmp(&v2).unwrap()
            })
            .map(|(i, _)| i)
            .unwrap_or(0)
    };

    let refs: Vec<&BiomeRecipe> = recipes.iter().map(|(r, _)| r).collect();

    let palette = BiomeRecipePalette {
        primary: mix_color(&|r| r.palette.primary),
        accent: mix_color(&|r| r.palette.accent),
        glow: mix_color(&|r| r.palette.glow),
        background: mix_color(&|r| r.palette.background),
    };
    let sky = SkySpec {
        top: mix_color(&|r| r.sky.top),
        bottom: mix_color(&|r| r.sky.bottom),
        nebula: mix_color(&|r| r.sky.nebula),
        nebula_strength: mix_f32(&|r| r.sky.nebula_strength),
        star_density: mix_f32(&|r| r.sky.star_density),
        aurora: mix_f32(&|r| r.sky.aurora),
    };
    let silhouettes = SilhouetteSpec {
        kind: refs[dominant(&|r| kind_rank(r.silhouettes.kind) as f32)]
            .silhouettes
            .kind,
        far: mix_color(&|r| r.silhouettes.far),
        near: mix_color(&|r| r.silhouettes.near),
        density: mix_f32(&|r| r.silhouettes.density),
        sway: mix_f32(&|r| r.silhouettes.sway),
    };
    let ground = GroundSpec {
        kind: refs[dominant(&|r| kind_rank_g(r.ground.kind) as f32)]
            .ground
            .kind,
        color: mix_color(&|r| r.ground.color),
        glow: mix_color(&|r| r.ground.glow),
    };
    let bricks = BrickMaterialSpec {
        normal: mix_brickmat(recipes, total, |r| r.bricks.normal),
        multihit: mix_brickmat(recipes, total, |r| r.bricks.multihit),
        invincible: mix_brickmat(recipes, total, |r| r.bricks.invincible),
        explosive: mix_brickmat(recipes, total, |r| r.bricks.explosive),
    };
    let entities = EntityMaterialSpec {
        paddle: mix_color(&|r| r.entities.paddle),
        paddle_glow: mix_color(&|r| r.entities.paddle_glow),
        wall: mix_color(&|r| r.entities.wall),
        ball: mix_color(&|r| r.entities.ball),
        ball_glow: mix_color(&|r| r.entities.ball_glow),
    };
    let ambient = AmbientSpec {
        kind: refs[dominant(&|r| kind_rank_a(r.ambient.kind) as f32)]
            .ambient
            .kind,
        color: mix_color(&|r| r.ambient.color),
        rate: mix_f32(&|r| r.ambient.rate),
        speed: mix_f32(&|r| r.ambient.speed),
    };
    let light = LightSpec {
        bloom: mix_f32(&|r| r.light.bloom),
        chromatic: mix_f32(&|r| r.light.chromatic),
        vignette: mix_f32(&|r| r.light.vignette),
        lens: mix_f32(&|r| r.light.lens),
        tint: mix_color(&|r| r.light.tint),
    };
    let curtain = CurtainSpec {
        kind: refs[dominant(&|r| kind_rank_c(r.curtain.kind) as f32)]
            .curtain
            .kind,
        primary: mix_color(&|r| r.curtain.primary),
        secondary: mix_color(&|r| r.curtain.secondary),
    };

    let d = dominant(&|r| kind_rank_s(r.silhouettes.kind) as f32);
    let name = refs[d].name.clone();

    BiomeRecipe {
        name,
        palette,
        sky,
        silhouettes,
        ground,
        bricks,
        entities,
        ambient,
        light,
        curtain,
        var_seed: recipes
            .iter()
            .map(|(r, w)| (r.var_seed as f64 * *w as f64) as u64)
            .sum(),
    }
}

fn mix_brickmat(
    recipes: &[(BiomeRecipe, f32)],
    total: f32,
    pick: impl Fn(&BiomeRecipe) -> BrickMat,
) -> BrickMat {
    let mut base = LinearRgba::new(0.0, 0.0, 0.0, 0.0);
    let mut glow = LinearRgba::new(0.0, 0.0, 0.0, 0.0);
    let mut glow_strength = 0.0f32;
    let mut texture = TextureKind::Stone;
    let mut best_w = -1.0f32;
    for (r, w) in recipes {
        let m = pick(r);
        base = base.mix(&m.base, w / total);
        glow = glow.mix(&m.glow, w / total);
        glow_strength += m.glow_strength * w / total;
        if *w > best_w {
            best_w = *w;
            texture = m.texture;
        }
    }
    BrickMat {
        base,
        texture,
        glow,
        glow_strength,
    }
}

fn kind_rank(_k: SilhouetteKind) -> u32 {
    0
}
fn kind_rank_g(_k: GroundKind) -> u32 {
    0
}
fn kind_rank_a(_k: AmbientKind) -> u32 {
    0
}
fn kind_rank_c(_k: CurtainKind) -> u32 {
    0
}
fn kind_rank_s(k: SilhouetteKind) -> u32 {
    kind_rank(k)
}

#[cfg(test)]
mod tests {
    use super::*;
    use metanoid_procgen::biome::composition::generate_composition;
    use metanoid_procgen::seed::hierarchy::MasterSeed;

    fn sample_ctx() -> LevelVisualContext {
        let master = MasterSeed(42);
        let g = master.galaxy(0);
        LevelVisualContext {
            current: generate_composition(g.biome(1)),
            prev: generate_composition(g.biome(0)),
            next: generate_composition(g.biome(2)),
            progress: 0.5,
            is_boss: false,
            var_seed: 12345,
        }
    }

    #[test]
    fn recipe_deterministic() {
        let a = recipe_for_context(&sample_ctx());
        let b = recipe_for_context(&sample_ctx());
        assert_eq!(format!("{:?}", a.sky), format!("{:?}", b.sky));
        assert_eq!(format!("{:?}", a.bricks), format!("{:?}", b.bricks));
        assert_eq!(a.var_seed, b.var_seed);
    }

    #[test]
    fn mix_endpoints_match_pure() {
        let master = MasterSeed(7);
        let c = generate_composition(master.galaxy(0).biome(0));
        let pure = flavor::recipe_for(&c, 99);
        let mixed = mix_recipes(&[(pure.clone(), 1.0)]);
        let pa = pure.sky.top;
        let ma = mixed.sky.top;
        assert!((pa.red - ma.red).abs() < 1e-4);
    }

    #[test]
    fn colors_in_valid_range() {
        let r = recipe_for_context(&sample_ctx());
        let colors = [
            r.sky.top,
            r.sky.bottom,
            r.bricks.normal.base,
            r.entities.paddle,
            r.ground.color,
        ];
        for c in colors {
            assert!(c.red >= 0.0 && c.red <= 1.0);
            assert!(c.green >= 0.0 && c.green <= 1.0);
            assert!(c.blue >= 0.0 && c.blue <= 1.0);
        }
    }
}
