//! Flavor profiles for the biome attractors, plus the recipe assembly logic.
//!
//! Each attractor (Forest, Volcanic, ...) contributes a `FlavorKit` — a full
//! base visual profile. `recipe_for` blends the kits by their composition
//! weights, then modulates by the raw biome parameters (temperature shifts the
//! hue, energy boosts glow, weirdness adds nebula/glitch, ...).

use bevy::color::Mix;
use bevy::prelude::*;

use metanoid_procgen::biome::composition::BiomeComposition;

use super::{
    AmbientKind, AmbientSpec, BiomeRecipe, BiomeRecipePalette, BrickMat, BrickMaterialSpec,
    CurtainKind, CurtainSpec, EntityMaterialSpec, GroundKind, GroundSpec, LightSpec,
    SilhouetteKind, SilhouetteSpec, SkySpec, TextureKind,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Flavor {
    Forest,
    Volcanic,
    Ocean,
    Crystal,
    Neon,
    Void,
    Desert,
    Arctic,
}

impl Flavor {
    pub fn from_name(name: &str) -> Option<Self> {
        Some(match name {
            "Forest" => Self::Forest,
            "Volcanic" => Self::Volcanic,
            "Deep Ocean" => Self::Ocean,
            "Crystal Cavern" => Self::Crystal,
            "Neon City" => Self::Neon,
            "Cosmic Void" => Self::Void,
            "Desert" => Self::Desert,
            "Arctic" => Self::Arctic,
            _ => return None,
        })
    }

    pub fn base_name(self) -> &'static str {
        match self {
            Self::Forest => "Forest",
            Self::Volcanic => "Volcanic",
            Self::Ocean => "Ocean",
            Self::Crystal => "Crystal",
            Self::Neon => "Neon",
            Self::Void => "Void",
            Self::Desert => "Desert",
            Self::Arctic => "Arctic",
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct FlavorKit {
    pub sky_top: LinearRgba,
    pub sky_bottom: LinearRgba,
    pub nebula: LinearRgba,
    pub nebula_strength: f32,
    pub star_density: f32,
    pub aurora: f32,
    pub primary: LinearRgba,
    pub multihit: LinearRgba,
    pub invincible: LinearRgba,
    pub explosive: LinearRgba,
    pub paddle: LinearRgba,
    pub paddle_glow: LinearRgba,
    pub wall: LinearRgba,
    pub ball_glow: LinearRgba,
    pub ground: LinearRgba,
    pub ground_glow: LinearRgba,
    pub silhouette_far: LinearRgba,
    pub silhouette_near: LinearRgba,
    pub silhouette: SilhouetteKind,
    pub texture: TextureKind,
    pub ground_kind: GroundKind,
    pub ambient: AmbientKind,
    pub ambient_color: LinearRgba,
    pub curtain: CurtainKind,
    pub curtain_primary: LinearRgba,
    pub curtain_secondary: LinearRgba,
    pub bloom: f32,
    pub chromatic: f32,
    pub vignette: f32,
}

fn srgb(r: f32, g: f32, b: f32) -> LinearRgba {
    LinearRgba::from(Srgba::new(r, g, b, 1.0))
}

impl Flavor {
    pub(crate) fn kit(self) -> FlavorKit {
        match self {
            Self::Forest => FlavorKit {
                sky_top: srgb(0.03, 0.09, 0.06),
                sky_bottom: srgb(0.12, 0.22, 0.15),
                nebula: srgb(0.28, 0.38, 0.22),
                nebula_strength: 0.3,
                star_density: 0.15,
                aurora: 0.15,
                primary: srgb(0.34, 0.43, 0.24),
                multihit: srgb(0.18, 0.31, 0.18),
                invincible: srgb(0.42, 0.43, 0.38),
                explosive: srgb(0.44, 0.22, 0.1),
                paddle: srgb(0.68, 0.74, 0.6),
                paddle_glow: srgb(0.9, 0.95, 0.72),
                wall: srgb(0.34, 0.4, 0.35),
                ball_glow: srgb(0.9, 1.0, 0.8),
                ground: srgb(0.05, 0.1, 0.07),
                ground_glow: srgb(0.14, 0.33, 0.18),
                silhouette_far: srgb(0.05, 0.11, 0.08),
                silhouette_near: srgb(0.03, 0.07, 0.05),
                silhouette: SilhouetteKind::Trees,
                texture: TextureKind::Wood,
                ground_kind: GroundKind::Grass,
                ambient: AmbientKind::Pollen,
                ambient_color: srgb(0.96, 0.86, 0.55),
                curtain: CurtainKind::Trees,
                curtain_primary: srgb(0.09, 0.06, 0.03),
                curtain_secondary: srgb(0.07, 0.14, 0.09),
                bloom: 0.5,
                chromatic: 0.002,
                vignette: 0.3,
            },
            Self::Volcanic => FlavorKit {
                sky_top: srgb(0.05, 0.012, 0.03),
                sky_bottom: srgb(0.28, 0.08, 0.03),
                nebula: srgb(0.55, 0.17, 0.07),
                nebula_strength: 0.22,
                star_density: 0.08,
                aurora: 0.0,
                primary: srgb(0.32, 0.24, 0.22),
                multihit: srgb(0.2, 0.15, 0.17),
                invincible: srgb(0.25, 0.24, 0.24),
                explosive: srgb(1.0, 0.35, 0.12),
                paddle: srgb(0.72, 0.55, 0.44),
                paddle_glow: srgb(1.0, 0.66, 0.4),
                wall: srgb(0.38, 0.27, 0.24),
                ball_glow: srgb(1.0, 0.8, 0.55),
                ground: srgb(0.1, 0.05, 0.03),
                ground_glow: srgb(1.0, 0.35, 0.12),
                silhouette_far: srgb(0.1, 0.05, 0.03),
                silhouette_near: srgb(0.06, 0.02, 0.01),
                silhouette: SilhouetteKind::Crags,
                texture: TextureKind::Lava,
                ground_kind: GroundKind::Lava,
                ambient: AmbientKind::Embers,
                ambient_color: srgb(1.0, 0.55, 0.2),
                curtain: CurtainKind::Columns,
                curtain_primary: srgb(0.1, 0.05, 0.04),
                curtain_secondary: srgb(0.45, 0.12, 0.04),
                bloom: 0.9,
                chromatic: 0.012,
                vignette: 0.4,
            },
            Self::Ocean => FlavorKit {
                sky_top: srgb(0.01, 0.05, 0.1),
                sky_bottom: srgb(0.03, 0.2, 0.28),
                nebula: srgb(0.04, 0.32, 0.38),
                nebula_strength: 0.45,
                star_density: 0.3,
                aurora: 0.1,
                primary: srgb(0.2, 0.52, 0.52),
                multihit: srgb(0.1, 0.32, 0.35),
                invincible: srgb(0.5, 0.6, 0.6),
                explosive: srgb(1.0, 0.5, 0.3),
                paddle: srgb(0.6, 0.82, 0.84),
                paddle_glow: srgb(0.72, 0.96, 1.0),
                wall: srgb(0.28, 0.45, 0.47),
                ball_glow: srgb(0.68, 0.96, 1.0),
                ground: srgb(0.015, 0.08, 0.1),
                ground_glow: srgb(0.2, 0.68, 0.75),
                silhouette_far: srgb(0.02, 0.1, 0.13),
                silhouette_near: srgb(0.01, 0.05, 0.07),
                silhouette: SilhouetteKind::Coral,
                texture: TextureKind::Stone,
                ground_kind: GroundKind::Sand,
                ambient: AmbientKind::Bubbles,
                ambient_color: srgb(0.7, 0.95, 1.0),
                curtain: CurtainKind::Shards,
                curtain_primary: srgb(0.02, 0.1, 0.14),
                curtain_secondary: srgb(0.1, 0.4, 0.45),
                bloom: 0.45,
                chromatic: 0.002,
                vignette: 0.3,
            },
            Self::Crystal => FlavorKit {
                sky_top: srgb(0.02, 0.05, 0.12),
                sky_bottom: srgb(0.06, 0.18, 0.32),
                nebula: srgb(0.18, 0.4, 0.62),
                nebula_strength: 0.55,
                star_density: 0.5,
                aurora: 0.25,
                primary: srgb(0.28, 0.47, 0.72),
                multihit: srgb(0.16, 0.32, 0.54),
                invincible: srgb(0.58, 0.7, 0.84),
                explosive: srgb(0.7, 0.94, 1.0),
                paddle: srgb(0.7, 0.84, 1.0),
                paddle_glow: srgb(0.84, 0.94, 1.0),
                wall: srgb(0.34, 0.47, 0.64),
                ball_glow: srgb(0.84, 0.96, 1.0),
                ground: srgb(0.03, 0.07, 0.12),
                ground_glow: srgb(0.44, 0.7, 0.9),
                silhouette_far: srgb(0.03, 0.07, 0.12),
                silhouette_near: srgb(0.02, 0.04, 0.08),
                silhouette: SilhouetteKind::Crystals,
                texture: TextureKind::Crystal,
                ground_kind: GroundKind::CrystalFloor,
                ambient: AmbientKind::Sparks,
                ambient_color: srgb(0.75, 0.9, 1.0),
                curtain: CurtainKind::Shards,
                curtain_primary: srgb(0.04, 0.08, 0.15),
                curtain_secondary: srgb(0.3, 0.55, 0.85),
                bloom: 0.7,
                chromatic: 0.004,
                vignette: 0.32,
            },
            Self::Neon => FlavorKit {
                sky_top: srgb(0.03, 0.01, 0.09),
                sky_bottom: srgb(0.1, 0.03, 0.24),
                nebula: srgb(0.42, 0.14, 0.94),
                nebula_strength: 0.7,
                star_density: 0.2,
                aurora: 0.2,
                primary: srgb(0.72, 0.29, 0.88),
                multihit: srgb(0.4, 0.16, 0.6),
                invincible: srgb(0.74, 0.74, 0.84),
                explosive: srgb(1.0, 0.28, 0.6),
                paddle: srgb(0.82, 0.74, 1.0),
                paddle_glow: srgb(1.0, 0.48, 0.82),
                wall: srgb(0.45, 0.36, 0.62),
                ball_glow: srgb(1.0, 0.6, 0.9),
                ground: srgb(0.05, 0.02, 0.12),
                ground_glow: srgb(0.62, 0.28, 1.0),
                silhouette_far: srgb(0.05, 0.03, 0.12),
                silhouette_near: srgb(0.03, 0.01, 0.07),
                silhouette: SilhouetteKind::Skyline,
                texture: TextureKind::Neon,
                ground_kind: GroundKind::NeonGrid,
                ambient: AmbientKind::Sparks,
                ambient_color: srgb(1.0, 0.5, 0.85),
                curtain: CurtainKind::Blinds,
                curtain_primary: srgb(0.05, 0.02, 0.12),
                curtain_secondary: srgb(0.62, 0.28, 1.0),
                bloom: 0.95,
                chromatic: 0.02,
                vignette: 0.42,
            },
            Self::Void => FlavorKit {
                sky_top: srgb(0.004, 0.006, 0.03),
                sky_bottom: srgb(0.03, 0.03, 0.1),
                nebula: srgb(0.22, 0.16, 0.42),
                nebula_strength: 0.9,
                star_density: 0.9,
                aurora: 0.0,
                primary: srgb(0.34, 0.28, 0.52),
                multihit: srgb(0.22, 0.16, 0.36),
                invincible: srgb(0.44, 0.44, 0.54),
                explosive: srgb(0.6, 0.48, 1.0),
                paddle: srgb(0.66, 0.66, 0.82),
                paddle_glow: srgb(0.68, 0.6, 1.0),
                wall: srgb(0.34, 0.34, 0.46),
                ball_glow: srgb(0.78, 0.7, 1.0),
                ground: srgb(0.015, 0.015, 0.05),
                ground_glow: srgb(0.4, 0.28, 0.82),
                silhouette_far: srgb(0.02, 0.02, 0.07),
                silhouette_near: srgb(0.01, 0.01, 0.04),
                silhouette: SilhouetteKind::Monoliths,
                texture: TextureKind::Cosmic,
                ground_kind: GroundKind::Void,
                ambient: AmbientKind::Dust,
                ambient_color: srgb(0.65, 0.6, 0.9),
                curtain: CurtainKind::Void,
                curtain_primary: srgb(0.02, 0.02, 0.07),
                curtain_secondary: srgb(0.25, 0.2, 0.5),
                bloom: 0.6,
                chromatic: 0.008,
                vignette: 0.35,
            },
            Self::Desert => FlavorKit {
                sky_top: srgb(0.09, 0.05, 0.02),
                sky_bottom: srgb(0.38, 0.24, 0.08),
                nebula: srgb(0.56, 0.4, 0.15),
                nebula_strength: 0.16,
                star_density: 0.25,
                aurora: 0.0,
                primary: srgb(0.56, 0.42, 0.23),
                multihit: srgb(0.4, 0.28, 0.12),
                invincible: srgb(0.72, 0.65, 0.53),
                explosive: srgb(1.0, 0.7, 0.32),
                paddle: srgb(0.82, 0.76, 0.6),
                paddle_glow: srgb(1.0, 0.88, 0.6),
                wall: srgb(0.56, 0.5, 0.42),
                ball_glow: srgb(1.0, 0.9, 0.7),
                ground: srgb(0.1, 0.07, 0.03),
                ground_glow: srgb(0.72, 0.5, 0.2),
                silhouette_far: srgb(0.12, 0.09, 0.04),
                silhouette_near: srgb(0.07, 0.05, 0.02),
                silhouette: SilhouetteKind::Dunes,
                texture: TextureKind::Stone,
                ground_kind: GroundKind::Sand,
                ambient: AmbientKind::Dust,
                ambient_color: srgb(0.95, 0.85, 0.6),
                curtain: CurtainKind::Columns,
                curtain_primary: srgb(0.1, 0.07, 0.03),
                curtain_secondary: srgb(0.62, 0.45, 0.2),
                bloom: 0.6,
                chromatic: 0.006,
                vignette: 0.35,
            },
            Self::Arctic => FlavorKit {
                sky_top: srgb(0.01, 0.05, 0.09),
                sky_bottom: srgb(0.05, 0.22, 0.34),
                nebula: srgb(0.26, 0.58, 0.75),
                nebula_strength: 0.32,
                star_density: 0.4,
                aurora: 0.6,
                primary: srgb(0.5, 0.7, 0.84),
                multihit: srgb(0.32, 0.5, 0.65),
                invincible: srgb(0.84, 0.9, 0.94),
                explosive: srgb(0.9, 0.98, 1.0),
                paddle: srgb(0.82, 0.93, 1.0),
                paddle_glow: srgb(0.95, 0.98, 1.0),
                wall: srgb(0.5, 0.62, 0.72),
                ball_glow: srgb(0.95, 0.99, 1.0),
                ground: srgb(0.02, 0.06, 0.1),
                ground_glow: srgb(0.6, 0.85, 1.0),
                silhouette_far: srgb(0.03, 0.08, 0.13),
                silhouette_near: srgb(0.01, 0.04, 0.07),
                silhouette: SilhouetteKind::FrozenSpires,
                texture: TextureKind::Ice,
                ground_kind: GroundKind::Ice,
                ambient: AmbientKind::Snow,
                ambient_color: srgb(0.9, 0.96, 1.0),
                curtain: CurtainKind::IceWalls,
                curtain_primary: srgb(0.03, 0.08, 0.13),
                curtain_secondary: srgb(0.55, 0.8, 0.95),
                bloom: 0.5,
                chromatic: 0.001,
                vignette: 0.3,
            },
        }
    }
}

fn blend_kits(kits: &[(FlavorKit, f32)]) -> FlavorKit {
    if kits.is_empty() {
        return Flavor::Void.kit();
    }
    let total: f32 = kits.iter().map(|(_, w)| w).sum();
    let total = if total <= 0.0 { 1.0 } else { total };

    let mix = |f: &dyn Fn(&FlavorKit) -> LinearRgba| -> LinearRgba {
        let mut acc = LinearRgba::new(0.0, 0.0, 0.0, 0.0);
        for (k, w) in kits {
            acc = acc.mix(&f(k), *w / total);
        }
        acc
    };
    let mixf =
        |f: &dyn Fn(&FlavorKit) -> f32| -> f32 { kits.iter().map(|(k, w)| f(k) * w / total).sum() };
    let dominant = |f: &dyn Fn(&FlavorKit) -> u32| -> FlavorKit {
        kits.iter()
            .max_by(|(k1, w1), (k2, w2)| {
                (f(k1) as f32 * *w1)
                    .partial_cmp(&(f(k2) as f32 * *w2))
                    .unwrap()
            })
            .map(|(k, _)| *k)
            .unwrap_or(kits[0].0)
    };

    FlavorKit {
        sky_top: mix(&|k| k.sky_top),
        sky_bottom: mix(&|k| k.sky_bottom),
        nebula: mix(&|k| k.nebula),
        nebula_strength: mixf(&|k| k.nebula_strength),
        star_density: mixf(&|k| k.star_density),
        aurora: mixf(&|k| k.aurora),
        primary: mix(&|k| k.primary),
        multihit: mix(&|k| k.multihit),
        invincible: mix(&|k| k.invincible),
        explosive: mix(&|k| k.explosive),
        paddle: mix(&|k| k.paddle),
        paddle_glow: mix(&|k| k.paddle_glow),
        wall: mix(&|k| k.wall),
        ball_glow: mix(&|k| k.ball_glow),
        ground: mix(&|k| k.ground),
        ground_glow: mix(&|k| k.ground_glow),
        silhouette_far: mix(&|k| k.silhouette_far),
        silhouette_near: mix(&|k| k.silhouette_near),
        silhouette: dominant(&|k| k.silhouette as u32).silhouette,
        texture: dominant(&|k| k.texture as u32).texture,
        ground_kind: dominant(&|k| k.ground_kind as u32).ground_kind,
        ambient: dominant(&|k| k.ambient as u32).ambient,
        ambient_color: mix(&|k| k.ambient_color),
        curtain: dominant(&|k| k.curtain as u32).curtain,
        curtain_primary: mix(&|k| k.curtain_primary),
        curtain_secondary: mix(&|k| k.curtain_secondary),
        bloom: mixf(&|k| k.bloom),
        chromatic: mixf(&|k| k.chromatic),
        vignette: mixf(&|k| k.vignette),
    }
}

fn shift_hue(c: LinearRgba, deg: f32) -> LinearRgba {
    if deg.abs() < 0.001 {
        return c;
    }
    let h: Hsla = Hsla::from(c);
    let shifted = Hsla::new(
        (h.hue + deg).rem_euclid(360.0),
        h.saturation,
        h.lightness,
        h.alpha,
    );
    LinearRgba::from(shifted)
}

fn modulate(mut kit: FlavorKit, p: &metanoid_procgen::biome::parameters::BiomeParams) -> FlavorKit {
    // Temperature shifts hue toward warm (high) or cool (low).
    let hue_shift = (p.temperature - 0.5) * 60.0;
    kit.primary = shift_hue(kit.primary, hue_shift);
    kit.multihit = shift_hue(kit.multihit, hue_shift * 0.8);
    kit.explosive = shift_hue(kit.explosive, hue_shift * 0.5);
    kit.nebula = shift_hue(kit.nebula, -hue_shift * 0.4);

    // Energy boosts glow + bloom.
    kit.bloom = (kit.bloom + p.energy * 0.3).min(1.2);
    let glow_boost = 0.15 + p.energy * 0.2;
    kit.paddle_glow = kit.paddle_glow.mix(&LinearRgba::WHITE, glow_boost * 0.4);
    kit.ball_glow = kit.ball_glow.mix(&LinearRgba::WHITE, glow_boost * 0.4);

    // Weirdness -> nebula + chromatic aberration, occasionally glitch particles.
    kit.nebula_strength = (kit.nebula_strength + p.weirdness * 0.4).min(1.0);
    kit.chromatic = (kit.chromatic + p.weirdness * 0.02).min(0.06);
    kit.star_density = (kit.star_density + p.weirdness * 0.2).min(1.0);
    if p.weirdness > 0.75 {
        kit.ambient = AmbientKind::Glitch;
        kit.ambient_color = srgb(0.8, 0.6, 1.0);
    }

    // Chaos -> stronger vignette, hotter ground.
    kit.vignette = (kit.vignette + p.chaos * 0.12).min(0.6);

    // Density -> busier silhouettes (handled at spawn time via density field).
    kit
}

pub fn recipe_for(comp: &BiomeComposition, var_seed: u64) -> BiomeRecipe {
    let kits: Vec<(FlavorKit, f32)> = comp
        .parts
        .iter()
        .filter_map(|p| Flavor::from_name(p.name).map(|f| (f.kit(), p.weight)))
        .collect();
    let kit = blend_kits(&kits);
    let kit = modulate(kit, &comp.params);
    let name = flavor_name(comp);
    assemble(kit, name, var_seed, &comp.params)
}

fn assemble(
    kit: FlavorKit,
    name: String,
    var_seed: u64,
    params: &metanoid_procgen::biome::parameters::BiomeParams,
) -> BiomeRecipe {
    let saturation_boost = 0.12 + params.energy * 0.15;
    let glow = kit.paddle_glow;
    let accent = kit.explosive;

    let base = kit.primary;
    let brick_glow_strength = 0.25 + params.energy * 0.35;

    BiomeRecipe {
        name,
        palette: BiomeRecipePalette {
            primary: base,
            accent,
            glow,
            background: kit.sky_top,
        },
        sky: SkySpec {
            top: kit.sky_top,
            bottom: kit.sky_bottom,
            nebula: kit.nebula,
            nebula_strength: kit.nebula_strength,
            star_density: kit.star_density,
            aurora: kit.aurora,
        },
        silhouettes: SilhouetteSpec {
            kind: kit.silhouette,
            far: kit.silhouette_far,
            near: kit.silhouette_near,
            density: 0.6 + params.density * 0.4,
            sway: 0.3 + params.chaos * 0.4,
        },
        ground: GroundSpec {
            kind: kit.ground_kind,
            color: kit.ground,
            glow: kit.ground_glow,
        },
        bricks: BrickMaterialSpec {
            normal: BrickMat {
                base,
                texture: kit.texture,
                glow,
                glow_strength: brick_glow_strength,
            },
            multihit: BrickMat {
                base: kit.multihit,
                texture: kit.texture,
                glow: kit.explosive,
                glow_strength: brick_glow_strength * 1.2,
            },
            invincible: BrickMat {
                base: kit.invincible,
                texture: if kit.texture == TextureKind::Lava {
                    TextureKind::Charred
                } else {
                    TextureKind::Metal
                },
                glow: kit.ball_glow,
                glow_strength: 0.15,
            },
            explosive: BrickMat {
                base: kit.explosive,
                texture: kit.texture,
                glow: kit.explosive,
                glow_strength: 1.0,
            },
        },
        entities: EntityMaterialSpec {
            paddle: kit.paddle,
            paddle_glow: kit.paddle_glow,
            wall: kit.wall,
            ball: srgb(0.95, 0.97, 1.0),
            ball_glow: kit.ball_glow,
        },
        ambient: AmbientSpec {
            kind: kit.ambient,
            color: kit.ambient_color,
            rate: 0.05 + params.density * 0.04 + params.energy * 0.03,
            speed: 1.0 + params.energy,
        },
        light: LightSpec {
            bloom: kit.bloom + saturation_boost * 0.3,
            chromatic: kit.chromatic,
            vignette: kit.vignette,
            lens: 0.0,
            tint: kit.sky_bottom,
        },
        curtain: CurtainSpec {
            kind: kit.curtain,
            primary: kit.curtain_primary,
            secondary: kit.curtain_secondary,
        },
        var_seed,
    }
}

pub fn default_recipe(var_seed: u64) -> BiomeRecipe {
    let comp = metanoid_procgen::biome::composition::BiomeComposition {
        params: metanoid_procgen::biome::parameters::BiomeParams {
            temperature: 0.5,
            density: 0.5,
            chaos: 0.3,
            energy: 0.5,
            weirdness: 0.3,
        },
        parts: vec![metanoid_procgen::biome::composition::BiomePart {
            name: "Forest",
            weight: 1.0,
        }],
    };
    recipe_for(&comp, var_seed)
}

pub fn texture_for_silhouette(kind: SilhouetteKind) -> TextureKind {
    match kind {
        SilhouetteKind::Trees => TextureKind::Wood,
        SilhouetteKind::Crags => TextureKind::Lava,
        SilhouetteKind::Crystals => TextureKind::Crystal,
        SilhouetteKind::Dunes => TextureKind::Stone,
        SilhouetteKind::Skyline => TextureKind::Neon,
        SilhouetteKind::FrozenSpires => TextureKind::Ice,
        SilhouetteKind::Coral => TextureKind::Stone,
        SilhouetteKind::Monoliths => TextureKind::Cosmic,
    }
}

fn flavor_name(comp: &BiomeComposition) -> String {
    let mut parts = comp.parts.clone();
    parts.sort_by(|a, b| b.weight.partial_cmp(&a.weight).unwrap());
    let primary = Flavor::from_name(parts[0].name)
        .map(|f| f.base_name())
        .unwrap_or("Unknown");
    if parts.len() == 1 {
        return format!("{primary}");
    }
    let secondary = parts[1];
    if secondary.weight >= 0.3 {
        let (a, b) = (
            Flavor::from_name(parts[0].name).unwrap_or(Flavor::Void),
            Flavor::from_name(secondary.name).unwrap_or(Flavor::Void),
        );
        return hybrid_name(a, b);
    }
    primary.to_string()
}

fn hybrid_name(a: Flavor, b: Flavor) -> String {
    match (a, b) {
        (Flavor::Forest, Flavor::Volcanic) => "Ember Grove".to_string(),
        (Flavor::Volcanic, Flavor::Forest) => "Ember Grove".to_string(),
        (Flavor::Forest, Flavor::Neon) => "Neon Grove".to_string(),
        (Flavor::Neon, Flavor::Forest) => "Neon Grove".to_string(),
        (Flavor::Forest, Flavor::Void) => "Wraithwood".to_string(),
        (Flavor::Void, Flavor::Forest) => "Wraithwood".to_string(),
        (Flavor::Forest, Flavor::Arctic) => "Frostpine".to_string(),
        (Flavor::Arctic, Flavor::Forest) => "Frostpine".to_string(),
        (Flavor::Volcanic, Flavor::Void) => "Ash Abyss".to_string(),
        (Flavor::Void, Flavor::Volcanic) => "Ash Abyss".to_string(),
        (Flavor::Volcanic, Flavor::Desert) => "Charred Dunes".to_string(),
        (Flavor::Desert, Flavor::Volcanic) => "Charred Dunes".to_string(),
        (Flavor::Volcanic, Flavor::Arctic) => "Steam Crags".to_string(),
        (Flavor::Arctic, Flavor::Volcanic) => "Steam Crags".to_string(),
        (Flavor::Ocean, Flavor::Arctic) => "Glacier Reef".to_string(),
        (Flavor::Arctic, Flavor::Ocean) => "Glacier Reef".to_string(),
        (Flavor::Crystal, Flavor::Arctic) => "Frozen Cathedral".to_string(),
        (Flavor::Arctic, Flavor::Crystal) => "Frozen Cathedral".to_string(),
        (Flavor::Crystal, Flavor::Neon) => "Prism City".to_string(),
        (Flavor::Neon, Flavor::Crystal) => "Prism City".to_string(),
        (Flavor::Void, Flavor::Crystal) => "Starfall Cavern".to_string(),
        (Flavor::Crystal, Flavor::Void) => "Starfall Cavern".to_string(),
        (Flavor::Void, Flavor::Neon) => "Glitch Nebula".to_string(),
        (Flavor::Neon, Flavor::Void) => "Glitch Nebula".to_string(),
        (Flavor::Ocean, Flavor::Void) => "Abyssal Void".to_string(),
        (Flavor::Void, Flavor::Ocean) => "Abyssal Void".to_string(),
        (Flavor::Ocean, Flavor::Crystal) => "Sunken Cathedral".to_string(),
        (Flavor::Crystal, Flavor::Ocean) => "Sunken Cathedral".to_string(),
        (Flavor::Desert, Flavor::Void) => "Waste of Stars".to_string(),
        (Flavor::Void, Flavor::Desert) => "Waste of Stars".to_string(),
        (Flavor::Desert, Flavor::Arctic) => "Frozen Wastes".to_string(),
        (Flavor::Arctic, Flavor::Desert) => "Frozen Wastes".to_string(),
        _ => format!("{} {}", a.base_name(), b.base_name()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_flavors_have_kits() {
        for f in [
            Flavor::Forest,
            Flavor::Volcanic,
            Flavor::Ocean,
            Flavor::Crystal,
            Flavor::Neon,
            Flavor::Void,
            Flavor::Desert,
            Flavor::Arctic,
        ] {
            let k = f.kit();
            assert!(k.bloom > 0.0);
            assert!(k.primary.red >= 0.0);
        }
    }

    #[test]
    fn name_roundtrip() {
        assert_eq!(Flavor::from_name("Forest"), Some(Flavor::Forest));
        assert_eq!(Flavor::from_name("Deep Ocean"), Some(Flavor::Ocean));
        assert_eq!(Flavor::from_name("nope"), None);
    }
}
