use bevy::prelude::*;
use metanoid_core::resources::game_state::GameState;
use metanoid_core::states::AppState;
use metanoid_procgen::seed::hierarchy::MasterSeed;
use metanoid_procgen::universe::galaxy::GalaxyDefinition;
use metanoid_procgen::universe::progression::LEVELS_PER_BIOME;

use super::save::SaveData;

#[derive(Component)]
pub struct GalaxyMapRoot;

#[derive(Component)]
pub struct MapInfoText;

pub fn setup_galaxy_map(
    mut commands: Commands,
    save: Res<SaveData>,
    existing_map: Query<Entity, With<GalaxyMapRoot>>,
) {
    for e in &existing_map {
        commands.entity(e).despawn();
    }

    let master = MasterSeed::new(save.master_seed);

    commands
        .spawn((
            GalaxyMapRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(30.0)),
                row_gap: Val::Px(20.0),
                overflow: Overflow::scroll_y(),
                ..default()
            },
            BackgroundColor(Color::srgb(0.02, 0.02, 0.06)),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("GALAXY MAP"),
                TextFont {
                    font_size: FontSize::Px(36.0),
                    ..default()
                },
                TextColor(Color::srgb(0.9, 0.95, 1.0)),
            ));

            parent.spawn((
                MapInfoText,
                Text::new(format!(
                    "High Score: {} | Progress: G{} B{} L{}",
                    save.high_score, save.highest_galaxy, save.highest_biome, save.highest_level
                )),
                TextFont {
                    font_size: FontSize::Px(16.0),
                    ..default()
                },
                TextColor(Color::srgb(0.6, 0.6, 0.7)),
            ));

            let display_galaxies = (save.highest_galaxy + 2).max(3).min(10);
            for g in 0..display_galaxies {
                let galaxy_seed = master.galaxy(g);
                let galaxy_def = GalaxyDefinition::generate(galaxy_seed);
                let unlocked = g <= save.highest_galaxy;

                parent
                    .spawn(Node {
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        margin: UiRect::vertical(Val::Px(10.0)),
                        padding: UiRect::all(Val::Px(15.0)),
                        border_radius: BorderRadius::all(Val::Px(8.0)),
                        ..default()
                    })
                    .with_children(|parent| {
                        let color = if unlocked {
                            Color::srgb(0.9, 0.95, 1.0)
                        } else {
                            Color::srgb(0.3, 0.3, 0.4)
                        };
                        parent.spawn((
                            Text::new(format!("Galaxy {}", g + 1)),
                            TextFont {
                                font_size: FontSize::Px(24.0),
                                ..default()
                            },
                            TextColor(color),
                        ));

                        let biome_limit = if unlocked {
                            galaxy_def.biome_count
                        } else {
                            0
                        };
                        for b in 0..biome_limit {
                            let biome_unlocked = g < save.highest_galaxy
                                || (g == save.highest_galaxy && b as u64 <= save.highest_biome);

                            let label = if biome_unlocked {
                                let level_limit = if g == save.highest_galaxy && b as u64 == save.highest_biome {
                                    (save.highest_level + 1).min(LEVELS_PER_BIOME)
                                } else {
                                    LEVELS_PER_BIOME
                                };
                                format!("  Biome {} ({} levels)", b + 1, level_limit)
                            } else {
                                "  ??? (locked)".to_string()
                            };

                            let biome_color = if biome_unlocked {
                                Color::srgb(0.7, 0.8, 0.9)
                            } else {
                                Color::srgb(0.25, 0.25, 0.3)
                            };

                            parent.spawn((
                                Text::new(label),
                                TextFont {
                                    font_size: FontSize::Px(16.0),
                                    ..default()
                                },
                                TextColor(biome_color),
                            ));
                        }
                    });
            }

            parent.spawn((
                Text::new("\nPress Enter to play from your highest unlocked level"),
                TextFont {
                    font_size: FontSize::Px(18.0),
                    ..default()
                },
                TextColor(Color::srgb(0.5, 0.8, 0.5)),
            ));

            parent.spawn((
                Text::new("Press Escape to return to menu"),
                TextFont {
                    font_size: FontSize::Px(14.0),
                    ..default()
                },
                TextColor(Color::srgb(0.4, 0.4, 0.5)),
            ));
        });
}

pub fn teardown_galaxy_map(
    mut commands: Commands,
    query: Query<Entity, With<GalaxyMapRoot>>,
) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

pub fn galaxy_map_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    save: Res<SaveData>,
    mut game_state: ResMut<GameState>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if keyboard.just_pressed(KeyCode::Enter) {
        game_state.galaxy = save.highest_galaxy;
        game_state.biome = save.highest_biome;
        game_state.level = save.highest_level;
        let master = MasterSeed::new(game_state.master_seed);
        let galaxy_def = GalaxyDefinition::generate(master.galaxy(game_state.galaxy));
        game_state.biome_count = galaxy_def.biome_count;
        game_state.lives = 3;
        game_state.score = 0;
        next_state.set(AppState::Loading);
    }

    if keyboard.just_pressed(KeyCode::Escape) {
        next_state.set(AppState::Menu);
    }
}
