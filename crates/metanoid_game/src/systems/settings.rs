use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Resource, Clone, Debug)]
pub struct GameSettings {
    pub show_fps: bool,
    pub master_volume: f32,
    pub sfx_volume: f32,
    pub music_volume: f32,
}

impl Default for GameSettings {
    fn default() -> Self {
        Self {
            show_fps: true,
            master_volume: 0.8,
            sfx_volume: 1.0,
            music_volume: 0.5,
        }
    }
}

#[derive(Component)]
pub struct SettingsRoot;

#[derive(Component)]
pub struct FpsToggle;

#[derive(Resource, Default)]
pub struct ShowSettings(pub bool);

pub fn load_settings() -> GameSettings {
    let path = std::path::PathBuf::from("metanoid_settings.json");
    match std::fs::read_to_string(&path) {
        Ok(json) => serde_json::from_str(&json).unwrap_or_default(),
        Err(_) => GameSettings::default(),
    }
}

pub fn save_settings(settings: &GameSettings) {
    let path = std::path::PathBuf::from("metanoid_settings.json");
    if let Ok(json) = serde_json::to_string_pretty(settings) {
        let _ = std::fs::write(path, json);
    }
}

pub fn manage_settings_overlay(
    mut commands: Commands,
    show_settings: Res<ShowSettings>,
    settings_query: Query<Entity, With<SettingsRoot>>,
    settings: Res<GameSettings>,
) {
    // Show settings overlay
    if show_settings.is_changed() && show_settings.0 && settings_query.iter().count() == 0 {
        let fps_label = if settings.show_fps {
            "FPS Display: ON"
        } else {
            "FPS Display: OFF"
        };

        commands
            .spawn((
                SettingsRoot,
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    row_gap: Val::Px(20.0),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.0, 0.0, 0.05, 0.92)),
            ))
            .with_children(|parent| {
                parent.spawn((
                    Text::new("SETTINGS"),
                    TextFont {
                        font_size: FontSize::Px(48.0),
                        ..default()
                    },
                    TextColor(Color::srgb(0.9, 0.95, 1.0)),
                ));

                parent
                    .spawn((
                        FpsToggle,
                        Button,
                        Node {
                            width: Val::Px(260.0),
                            height: Val::Px(50.0),
                            border: UiRect::all(Val::Px(2.0)),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            margin: UiRect::top(Val::Px(20.0)),
                            ..default()
                        },
                        BorderColor::all(Color::srgb(0.4, 0.5, 0.6)),
                        BackgroundColor(Color::srgb(0.08, 0.1, 0.15)),
                    ))
                    .with_children(|btn| {
                        btn.spawn((
                            Text::new(fps_label),
                            TextFont {
                                font_size: FontSize::Px(20.0),
                                ..default()
                            },
                            TextColor(Color::srgb(0.8, 0.85, 0.9)),
                        ));
                    });

                parent.spawn((
                    Text::new("Press Escape to go back"),
                    TextFont {
                        font_size: FontSize::Px(16.0),
                        ..default()
                    },
                    TextColor(Color::srgb(0.4, 0.4, 0.5)),
                ));
            });
    }

    // Hide settings overlay
    if show_settings.is_changed() && !show_settings.0 {
        for entity in &settings_query {
            commands.entity(entity).despawn();
        }
    }
}

pub fn teardown_settings(mut commands: Commands, query: Query<Entity, With<SettingsRoot>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

pub fn settings_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut show_settings: ResMut<ShowSettings>,
) {
    if keyboard.just_pressed(KeyCode::Escape) && show_settings.0 {
        show_settings.0 = false;
    }
    if keyboard.just_pressed(KeyCode::KeyS) && !show_settings.0 {
        show_settings.0 = true;
    }
}

pub fn fps_toggle_interaction(
    mut interaction_query: Query<
        (&Interaction, &Children),
        (Changed<Interaction>, With<FpsToggle>),
    >,
    mut settings: ResMut<GameSettings>,
    mut text_query: Query<&mut Text>,
) {
    for (interaction, children) in &mut interaction_query {
        if *interaction == Interaction::Pressed {
            settings.show_fps = !settings.show_fps;
            save_settings(&settings);

            for child in children.iter() {
                if let Ok(mut text) = text_query.get_mut(child) {
                    **text = if settings.show_fps {
                        "FPS Display: ON".to_string()
                    } else {
                        "FPS Display: OFF".to_string()
                    };
                }
            }
        }
    }
}
