//! Runtime material cache: bakes recipe-driven textures into `ColorMaterial`
//! handles the game spawns bricks / entities with.

use std::collections::HashMap;

use bevy::asset::RenderAssetUsages;
use bevy::image::ImageSampler;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

use crate::recipe::texture::bake_brick;
use crate::recipe::{BiomeRecipe, TextureKind};

pub const BRICK_TILE_W: u32 = 64;
pub const BRICK_TILE_H: u32 = 32;

/// Which brick material family to bake.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BrickMatKind {
    Normal,
    MultiHit,
    Invincible,
    Explosive,
}

/// All procedurally generated materials for one level.
#[derive(Resource, Clone)]
pub struct ProceduralMaterials {
    bricks: HashMap<(BrickMatKind, u8), Handle<ColorMaterial>>,
    pub paddle: Handle<ColorMaterial>,
    pub wall: Handle<ColorMaterial>,
    pub ball: Handle<ColorMaterial>,
    pub ground: Handle<ColorMaterial>,
    pub ground_glow: Handle<ColorMaterial>,
    pub sky: Handle<ColorMaterial>,
}

impl Default for ProceduralMaterials {
    fn default() -> Self {
        Self {
            bricks: HashMap::new(),
            paddle: Handle::default(),
            wall: Handle::default(),
            ball: Handle::default(),
            ground: Handle::default(),
            ground_glow: Handle::default(),
            sky: Handle::default(),
        }
    }
}

impl ProceduralMaterials {
    /// Fetch a brick material for the given damage bucket (0=broken .. 3=pristine).
    pub fn brick(&self, kind: BrickMatKind, health_pct: f32) -> Handle<ColorMaterial> {
        let bucket = (health_pct.clamp(0.0, 1.0) * 3.0).round() as u8;
        self.bricks
            .get(&(kind, bucket))
            .cloned()
            .unwrap_or_default()
    }
}

fn image_from_pixels(data: Vec<u8>, w: u32, h: u32) -> Image {
    let mut image = Image::new(
        Extent3d {
            width: w,
            height: h,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    );
    image.sampler = ImageSampler::nearest();
    image
}

fn material_from_pixels(
    images: &mut Assets<Image>,
    materials: &mut Assets<ColorMaterial>,
    data: Vec<u8>,
    w: u32,
    h: u32,
) -> Handle<ColorMaterial> {
    let image = images.add(image_from_pixels(data, w, h));
    materials.add(ColorMaterial {
        texture: Some(image),
        ..default()
    })
}

fn brick_bucket_images(
    images: &mut Assets<Image>,
    materials: &mut Assets<ColorMaterial>,
    kind: BrickMatKind,
    mat: &crate::recipe::BrickMat,
    var_seed: u64,
) -> HashMap<u8, Handle<ColorMaterial>> {
    let mut out = HashMap::new();
    // damage: 0 (broken) .. 3 (pristine)
    for bucket in 0..=3u8 {
        let damage = match bucket {
            0 => 0.95,
            1 => 0.6,
            2 => 0.25,
            _ => 0.0,
        };
        let pixels = bake_brick(
            mat.texture,
            mat.base,
            mat.glow,
            BRICK_TILE_W,
            BRICK_TILE_H,
            var_seed ^ (kind as u64 * 0x9E37_79B9),
            damage,
        );
        let h = material_from_pixels(images, materials, pixels.data, BRICK_TILE_W, BRICK_TILE_H);
        out.insert(bucket, h);
    }
    out
}

/// Bake every material needed for a level from its recipe.
pub fn bake_all(
    images: &mut Assets<Image>,
    materials: &mut Assets<ColorMaterial>,
    recipe: &BiomeRecipe,
) -> ProceduralMaterials {
    let mut bricks = HashMap::new();
    for (kind, mat) in [
        (BrickMatKind::Normal, &recipe.bricks.normal),
        (BrickMatKind::MultiHit, &recipe.bricks.multihit),
        (BrickMatKind::Invincible, &recipe.bricks.invincible),
        (BrickMatKind::Explosive, &recipe.bricks.explosive),
    ] {
        for (bucket, handle) in brick_bucket_images(images, materials, kind, mat, recipe.var_seed) {
            bricks.insert((kind, bucket), handle);
        }
    }

    let pixels = |t: TextureKind, base: LinearRgba, glow: LinearRgba, w: u32, h: u32| {
        bake_brick(t, base, glow, w, h, recipe.var_seed, 0.0).data
    };

    let paddle = material_from_pixels(
        images,
        materials,
        pixels(
            TextureKind::Metal,
            recipe.entities.paddle,
            recipe.entities.paddle_glow,
            96,
            24,
        ),
        96,
        24,
    );
    let wall = material_from_pixels(
        images,
        materials,
        pixels(
            TextureKind::Stone,
            recipe.entities.wall,
            recipe.entities.ball_glow,
            64,
            32,
        ),
        64,
        32,
    );
    let ball = material_from_pixels(
        images,
        materials,
        pixels(
            TextureKind::Crystal,
            recipe.entities.ball,
            recipe.entities.ball_glow,
            32,
            32,
        ),
        32,
        32,
    );

    // Ground strip: darker texture with a glow band baked along the bottom.
    let mut ground_data = pixels(
        recipe.ground.kind.texture_kind(),
        recipe.ground.color,
        recipe.ground.glow,
        256,
        32,
    );
    {
        let gw = 256u32;
        for x in 0..gw {
            let i = (x * 4) as usize;
            let glow_px = recipe.ground.glow;
            let s: Srgba = Srgba::from(glow_px);
            ground_data[i + 0] = (s.red * 255.0) as u8;
            ground_data[i + 1] = (s.green * 255.0) as u8;
            ground_data[i + 2] = (s.blue * 255.0) as u8;
            ground_data[i + 3] = 255;
        }
    }
    let ground = material_from_pixels(images, materials, ground_data, 256, 32);

    // Sky uses a custom shader material (see sky.rs); this is just the color fallback.
    let sky = materials.add(ColorMaterial::from_color(recipe.sky.top));

    ProceduralMaterials {
        bricks,
        paddle,
        wall,
        ball,
        ground,
        ground_glow: sky.clone(),
        sky,
    }
}

impl crate::recipe::GroundKind {
    pub fn texture_kind(&self) -> TextureKind {
        match self {
            Self::Grass => TextureKind::Wood,
            Self::Stone | Self::Sand => TextureKind::Stone,
            Self::Lava => TextureKind::Lava,
            Self::Ice => TextureKind::Ice,
            Self::CrystalFloor => TextureKind::Crystal,
            Self::NeonGrid => TextureKind::Neon,
            Self::Void => TextureKind::Cosmic,
        }
    }
}
