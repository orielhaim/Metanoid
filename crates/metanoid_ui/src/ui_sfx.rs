//! Menu / settings UI click & hover SFX.

use bevy::prelude::*;
use bevy_kira_audio::AudioControl;
use bevy_kira_audio::prelude::AudioChannel;
use metanoid_audio::{SfxAssets, SfxChannel};

/// Play select/hover for any button that just changed interaction.
pub fn button_ui_sfx(
    interaction: Query<&Interaction, (Changed<Interaction>, With<Button>)>,
    sfx: Option<Res<SfxAssets>>,
    audio: Option<Res<AudioChannel<SfxChannel>>>,
) {
    let (Some(sfx), Some(audio)) = (sfx, audio) else {
        return;
    };
    for interaction in &interaction {
        match *interaction {
            Interaction::Pressed => {
                audio.play(sfx.ui_select.clone()).with_volume(-4.0);
            }
            Interaction::Hovered => {
                audio.play(sfx.ui_hover.clone()).with_volume(-10.0);
            }
            Interaction::None => {}
        }
    }
}
