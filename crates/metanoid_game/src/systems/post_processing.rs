use bevy::post_process::bloom::Bloom;
use bevy::post_process::effect_stack::{ChromaticAberration, LensDistortion, Vignette};
use bevy::prelude::*;
use metanoid_core::components::brick::Brick;
use metanoid_core::resources::combo::ComboCounter;
use metanoid_core::settings::GameSettings;

use super::arena::GameCamera;
use super::level_progression::ActiveLevelVisuals;

/// Last values written to the camera, so we only mutate post-processing
/// components when they actually change. Mutating them every frame forces the
/// effect stack to rebuild GPU resources every frame, which shows up as flicker.
#[derive(Resource, Default, Clone, Copy)]
pub struct AppliedPostFx {
    pub bloom: f32,
    pub chromatic: f32,
    pub vignette: f32,
}

fn write_if_changed(current: &mut f32, target: f32, stored: &mut f32, epsilon: f32) -> bool {
    if (target - *stored).abs() <= epsilon {
        return false;
    }
    *current = target;
    *stored = target;
    true
}

/// Keep post-processing gentle and stable. Values are damped toward targets and
/// only written to the camera when the target actually moves, so the effect
/// stack stays put and the image stays sharp.
pub fn update_post_processing(
    combo: Res<ComboCounter>,
    bricks: Query<&Brick>,
    settings: Res<GameSettings>,
    visuals: Option<Res<ActiveLevelVisuals>>,
    mut applied: ResMut<AppliedPostFx>,
    mut cameras: Query<(&mut Bloom, &mut ChromaticAberration, &mut Vignette), With<GameCamera>>,
) {
    let Ok((mut bloom, mut chromatic, mut vignette)) = cameras.single_mut() else {
        return;
    };

    // Sharper look: bloom is heavily reined in; chromatic aberration is disabled.
    let recipe = visuals.as_ref().map(|v| &v.recipe.light);
    let base_bloom = recipe.map(|l| l.bloom).unwrap_or(0.3) * settings.bloom_scale() * 0.35;
    let target_bloom = (base_bloom + combo.multiplier * 0.02).min(0.35);
    write_if_changed(&mut bloom.intensity, target_bloom, &mut applied.bloom, 0.01);

    let target_ca = if settings.reduce_motion {
        0.0
    } else {
        (recipe.map(|l| l.chromatic).unwrap_or(0.0) * 0.15).min(0.004)
    };
    write_if_changed(
        &mut chromatic.intensity,
        target_ca,
        &mut applied.chromatic,
        0.001,
    );

    let base_vignette = recipe.map(|l| l.vignette).unwrap_or(0.3) * 0.6;
    let total = bricks.iter().count();
    let mut target_vig = base_vignette;
    if total > 0 {
        let remaining_ratio = bricks
            .iter()
            .filter(|b| b.brick_type != metanoid_core::components::brick::BrickType::Invincible)
            .count() as f32
            / total.max(1) as f32;
        if remaining_ratio < 0.3 {
            target_vig += (1.0 - remaining_ratio / 0.3) * 0.3;
        }
    }
    target_vig = target_vig.min(0.55);
    write_if_changed(
        &mut vignette.intensity,
        target_vig,
        &mut applied.vignette,
        0.01,
    );
}

/// Lens distortion only ever decays toward zero and never writes when already
/// settled (no per-frame camera mutation => no flicker).
pub fn pulse_lens_distortion(mut cameras: Query<&mut LensDistortion, With<GameCamera>>) {
    let Ok(mut lens) = cameras.single_mut() else {
        return;
    };
    if lens.intensity.abs() > 0.002 {
        lens.intensity *= 0.95;
        if lens.intensity.abs() < 0.002 {
            lens.intensity = 0.0;
        }
    }
}
