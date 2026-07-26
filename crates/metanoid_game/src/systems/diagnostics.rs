use bevy::prelude::*;
use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};

use super::settings::GameSettings;

#[derive(Component)]
pub struct FpsText;

pub struct DiagnosticsPlugin;

impl Plugin for DiagnosticsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(FrameTimeDiagnosticsPlugin::default())
            .add_systems(Update, update_fps_display);
    }
}

pub fn setup_fps_display(mut commands: Commands) {
    commands.spawn((
        FpsText,
        Text::new("FPS: --"),
        TextFont {
            font_size: FontSize::Px(16.0),
            ..default()
        },
        TextColor(Color::srgba(0.5, 1.0, 0.5, 0.6)),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(8.0),
            right: Val::Px(8.0),
            ..default()
        },
    ));
}

fn update_fps_display(
    diagnostics: Res<DiagnosticsStore>,
    settings: Res<GameSettings>,
    mut query: Query<&mut Text, With<FpsText>>,
    mut visibility: Query<&mut Visibility, With<FpsText>>,
) {
    for mut vis in &mut visibility {
        *vis = if settings.show_fps {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }

    if !settings.show_fps {
        return;
    }

    for mut text in &mut query {
        if let Some(fps) = diagnostics.get(&FrameTimeDiagnosticsPlugin::FPS) {
            if let Some(value) = fps.smoothed() {
                **text = format!("FPS: {:.0}", value);
            }
        }
    }
}
