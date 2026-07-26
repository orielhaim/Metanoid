use bevy::prelude::*;
use metanoid_core::resources::game_state::GameState;

#[derive(Component)]
pub struct HudRoot;

#[derive(Component)]
pub struct ScoreText;

#[derive(Component)]
pub struct LivesText;

pub fn setup_hud(mut commands: Commands) {
    commands
        .spawn((
            HudRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Px(40.0),
                padding: UiRect::horizontal(Val::Px(20.0)),
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.4)),
        ))
        .with_children(|parent| {
            parent.spawn((
                ScoreText,
                Text::new("Score: 0"),
                TextFont {
                    font_size: FontSize::Px(24.0),
                    ..default()
                },
                TextColor(Color::srgb(1.0, 1.0, 1.0)),
            ));

            parent.spawn((
                LivesText,
                Text::new("Lives: 3"),
                TextFont {
                    font_size: FontSize::Px(24.0),
                    ..default()
                },
                TextColor(Color::srgb(1.0, 0.4, 0.4)),
            ));
        });
}

pub fn update_hud(
    game_state: Option<Res<GameState>>,
    mut score_text: Query<&mut Text, (With<ScoreText>, Without<LivesText>)>,
    mut lives_text: Query<&mut Text, (With<LivesText>, Without<ScoreText>)>,
) {
    let Some(state) = game_state else { return };

    for mut text in &mut score_text {
        **text = format!("Score: {}", state.score);
    }

    let hearts: String = (0..state.lives.max(0)).map(|_| "\u{2764} ").collect();
    for mut text in &mut lives_text {
        **text = format!("Lives: {}", hearts);
    }
}
