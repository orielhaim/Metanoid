use bevy::prelude::*;
use bevy::post_process::bloom::Bloom;
use bevy::post_process::effect_stack::{ChromaticAberration, Vignette, LensDistortion};
use metanoid_core::components::brick::Brick;
use metanoid_core::resources::combo::ComboCounter;

use super::arena::GameCamera;

pub fn update_post_processing(
    combo: Res<ComboCounter>,
    bricks: Query<&Brick>,
    mut cameras: Query<(&mut Bloom, &mut ChromaticAberration, &mut Vignette), With<GameCamera>>,
) {
    let Ok((mut bloom, mut chromatic, mut vignette)) = cameras.single_mut() else {
        return;
    };

    bloom.intensity = 0.15 + combo.multiplier * 0.05;

    if combo.count >= 10 {
        chromatic.intensity = 0.005 + (combo.count as f32 - 10.0) * 0.001;
    } else {
        chromatic.intensity *= 0.9;
    }

    let total = bricks.iter().count();
    if total > 0 {
        let remaining_ratio = bricks.iter().filter(|b| {
            b.brick_type != metanoid_core::components::brick::BrickType::Invincible
        }).count() as f32 / total.max(1) as f32;

        if remaining_ratio < 0.3 {
            vignette.intensity = 0.3 + (1.0 - remaining_ratio / 0.3) * 0.4;
        } else {
            vignette.intensity = 0.3;
        }
    }
}

pub fn pulse_lens_distortion(
    _time: Res<Time>,
    mut cameras: Query<&mut LensDistortion, With<GameCamera>>,
) {
    let Ok(mut lens) = cameras.single_mut() else {
        return;
    };

    if lens.intensity.abs() > 0.001 {
        lens.intensity *= 0.95;
    } else {
        lens.intensity = 0.0;
    }
}
