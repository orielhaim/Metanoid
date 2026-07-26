use bevy::prelude::*;
use metanoid_core::resources::game_state::GameState;
use metanoid_core::states::AppState;

use super::arena::ArenaEntity;
use super::level_spawner::LevelEntity;
use super::hud::HudRoot;

#[derive(Component)]
pub struct LevelCompleteRoot;

#[derive(Component)]
pub struct ContinueButton;

pub fn setup_level_complete(
    mut commands: Commands,
    game_state: Option<Res<GameState>>,
    level_entities: Query<Entity, With<LevelEntity>>,
    arena_entities: Query<Entity, With<ArenaEntity>>,
    hud_entities: Query<Entity, With<HudRoot>>,
) {
    let state_ref = game_state.as_ref();
    let score = state_ref.map(|s| s.score).unwrap_or(0);
    let galaxy = state_ref.map(|s| s.galaxy + 1).unwrap_or(1);
    let biome = state_ref.map(|s| s.biome + 1).unwrap_or(1);
    let level = state_ref.map(|s| s.level).unwrap_or(1);

    // Clean up game entities
    for entity in &level_entities {
        commands.entity(entity).despawn();
    }
    for entity in &arena_entities {
        commands.entity(entity).despawn();
    }
    for entity in &hud_entities {
        commands.entity(entity).despawn();
    }

    // Spawn confetti particles
    let confetti_colors = [
        Color::srgb(1.0, 0.2, 0.2),
        Color::srgb(0.2, 1.0, 0.2),
        Color::srgb(0.2, 0.2, 1.0),
        Color::srgb(1.0, 1.0, 0.2),
        Color::srgb(1.0, 0.2, 1.0),
        Color::srgb(0.2, 1.0, 1.0),
    ];

    for i in 0..60 {
        let color = confetti_colors[i % confetti_colors.len()];
        let x = (i as f32 - 30.0) * 20.0 + (i as f32 * 7.0).sin() * 30.0;
        let _start_y = 300.0 + (i as f32 * 13.0).cos() * 50.0;

        commands.spawn((
            LevelCompleteRoot,
            Node {
                position_type: PositionType::Absolute,
                left: Val::Percent(50.0 + x / 12.8),
                top: Val::Percent(20.0 + (i as f32 % 60.0) / 60.0 * 50.0),
                width: Val::Px(8.0),
                height: Val::Px(8.0),
                ..default()
            },
            BackgroundColor(color),
        ));
    }

    // UI overlay
    commands
        .spawn((
            LevelCompleteRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                row_gap: Val::Px(16.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.1, 0.75)),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("LEVEL CLEAR!"),
                TextFont {
                    font_size: FontSize::Px(56.0),
                    ..default()
                },
                TextColor(Color::srgb(0.3, 1.0, 0.5)),
            ));

            parent.spawn((
                Text::new(format!("Galaxy {} — Biome {} — Level {}", galaxy, biome, level)),
                TextFont {
                    font_size: FontSize::Px(22.0),
                    ..default()
                },
                TextColor(Color::srgb(0.7, 0.75, 0.85)),
            ));

            parent.spawn((
                Text::new(format!("Score: {}", score)),
                TextFont {
                    font_size: FontSize::Px(28.0),
                    ..default()
                },
                TextColor(Color::srgb(1.0, 1.0, 1.0)),
            ));

            // Continue button
            parent
                .spawn((
                    ContinueButton,
                    Button,
                    Node {
                        width: Val::Px(200.0),
                        height: Val::Px(50.0),
                        border: UiRect::all(Val::Px(2.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        margin: UiRect::top(Val::Px(16.0)),
                        ..default()
                    },
                    BorderColor::all(Color::srgb(0.3, 1.0, 0.5)),
                    BackgroundColor(Color::srgb(0.05, 0.15, 0.08)),
                ))
                .with_children(|btn| {
                    btn.spawn((
                        Text::new("CONTINUE"),
                        TextFont {
                            font_size: FontSize::Px(24.0),
                            ..default()
                        },
                        TextColor(Color::srgb(0.8, 1.0, 0.85)),
                    ));
                });
        });
}

pub fn continue_button_interaction(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, &mut BorderColor),
        (Changed<Interaction>, With<ContinueButton>),
    >,
    mut next_state: ResMut<NextState<AppState>>,
    mut commands: Commands,
    arena_entities: Query<Entity, With<ArenaEntity>>,
    level_entities: Query<Entity, With<LevelEntity>>,
    hud_entities: Query<Entity, With<HudRoot>>,
) {
    for (interaction, mut bg, mut border) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                // Clean up remaining game entities
                for entity in &arena_entities {
                    commands.entity(entity).despawn();
                }
                for entity in &level_entities {
                    commands.entity(entity).despawn();
                }
                for entity in &hud_entities {
                    commands.entity(entity).despawn();
                }
                commands.remove_resource::<super::arena::ArenaSpawned>();
                next_state.set(AppState::Loading);
            }
            Interaction::Hovered => {
                *bg = BackgroundColor(Color::srgb(0.1, 0.25, 0.15));
                *border = BorderColor::all(Color::srgb(0.5, 1.0, 0.7));
            }
            Interaction::None => {
                *bg = BackgroundColor(Color::srgb(0.05, 0.15, 0.08));
                *border = BorderColor::all(Color::srgb(0.3, 1.0, 0.5));
            }
        }
    }
}

pub fn teardown_level_complete(
    mut commands: Commands,
    query: Query<Entity, With<LevelCompleteRoot>>,
) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}
