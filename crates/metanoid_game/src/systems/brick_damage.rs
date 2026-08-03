//! Dynamic brick damage: ball impacts carve unique crack patterns into bricks
//! at the exact point of contact. Every hit bakes a fresh texture for that
//! brick, so damage marks are always distinct and anchored to where the ball
//! struck.

use bevy::asset::RenderAssetUsages;
use bevy::image::ImageSampler;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use metanoid_core::components::brick::{Brick, BrickType};
use metanoid_core::events::{BrickHitEvent, BrickRegenEvent};
use metanoid_visuals::brick_damage::{Crack, add_impact, apply_cracks};
use metanoid_visuals::material::BrickMatKind;
use metanoid_visuals::recipe::TextureKind;

/// Per-brick damage state: the accumulated cracks + the base texture to bake on.
#[derive(Component)]
pub struct BrickDamage {
    pub cracks: Vec<Crack>,
    pub base_image: Handle<Image>,
    pub glow: LinearRgba,
    pub molten: bool,
}

fn bake_cracks_material(
    damage: &BrickDamage,
    base: &Image,
    images: &mut Assets<Image>,
    materials: &mut Assets<ColorMaterial>,
) -> Handle<ColorMaterial> {
    let w = base.texture_descriptor.size.width;
    let h = base.texture_descriptor.size.height;
    let data = base.data.clone().unwrap_or_default();
    let pixels = apply_cracks(&data, w, h, &damage.cracks, damage.glow, damage.molten);

    let mut image = Image::new(
        Extent3d {
            width: w,
            height: h,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        pixels,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    );
    image.sampler = ImageSampler::nearest();
    let image = images.add(image);
    materials.add(ColorMaterial {
        texture: Some(image),
        ..default()
    })
}

/// Carve cracks at the impact point and re-bake the brick's material.
pub fn on_brick_hit_damage(
    trigger: On<BrickHitEvent>,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut bricks: Query<(
        Entity,
        &Transform,
        &Brick,
        &mut BrickDamage,
        &mut MeshMaterial2d<ColorMaterial>,
    )>,
) {
    let Ok((brick_entity, tf, brick, mut damage, mut mat)) = bricks.get_mut(trigger.brick) else {
        return;
    };
    // Only mark destructible bricks that survived the hit.
    if brick.brick_type == BrickType::Invincible || brick.health == 0 {
        return;
    }

    // Impact point in brick-local UV (0..1). Image space has y=0 at the top,
    // while world +y is up, so the vertical axis is flipped.
    let local = trigger.position - tf.translation.truncate();
    let uv = Vec2::new(
        (local.x / (2.0 * brick.brick_half_w) + 0.5).clamp(0.0, 1.0),
        (0.5 - local.y / (2.0 * brick.brick_half_h)).clamp(0.0, 1.0),
    );

    // Deterministic per-impact seed: entity + hit count + impact location.
    let seed = brick_entity.to_bits()
        ^ (damage.cracks.len() as u64).wrapping_mul(0xC0FF_EE)
        ^ ((uv.x * 4096.0) as u64).wrapping_mul(0x1_0000_1)
        ^ ((uv.y * 4096.0) as u64).wrapping_mul(0x1000_3);

    add_impact(&mut damage.cracks, uv, seed, trigger.severity);

    let Some(base) = images.get(&damage.base_image).cloned() else {
        return;
    };
    let new_mat = bake_cracks_material(&damage, &base, &mut images, &mut materials);
    mat.0 = new_mat;
}

/// Clear a regenerating brick's damage when it heals back to full.
pub fn on_brick_regen_clear(
    trigger: On<BrickRegenEvent>,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut bricks: Query<(&mut BrickDamage, &mut MeshMaterial2d<ColorMaterial>)>,
) {
    let Ok((mut damage, mut mat)) = bricks.get_mut(trigger.brick) else {
        return;
    };
    if damage.cracks.is_empty() {
        return;
    }
    damage.cracks.clear();
    let Some(base) = images.get(&damage.base_image).cloned() else {
        return;
    };
    let new_mat = bake_cracks_material(&damage, &base, &mut images, &mut materials);
    mat.0 = new_mat;
}

/// Convenience: map a brick type to its material family.
pub fn damage_kind_for(brick_type: BrickType) -> BrickMatKind {
    match brick_type {
        BrickType::MultiHit => BrickMatKind::MultiHit,
        BrickType::Invincible => BrickMatKind::Invincible,
        BrickType::Explosive => BrickMatKind::Explosive,
        _ => BrickMatKind::Normal,
    }
}

/// Whether a brick family should show glowing "molten" cracks.
pub fn is_molten(kind: BrickMatKind, texture: TextureKind) -> bool {
    kind == BrickMatKind::Explosive || texture == TextureKind::Lava
}
