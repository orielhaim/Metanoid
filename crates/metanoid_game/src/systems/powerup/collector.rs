use avian2d::prelude::*;
use bevy::prelude::*;
use metanoid_core::components::paddle::Paddle;
use metanoid_core::components::powerup::{PowerUp, PowerUpKind};
use metanoid_core::events::{
    BulletTimeEvent, FloatingTextEvent, FloatingTextKind, PowerUpCollectedEvent,
};
use metanoid_core::resources::game_state::GameState;

#[derive(Resource)]
pub struct TimeSlowState {
    pub active: bool,
    pub timer: Timer,
}

impl Default for TimeSlowState {
    fn default() -> Self {
        Self {
            active: false,
            timer: Timer::from_seconds(8.0, TimerMode::Once),
        }
    }
}

pub fn collect_powerup(
    mut commands: Commands,
    mut collision_reader: MessageReader<CollisionStart>,
    powerups: Query<&PowerUp>,
    paddles: Query<(&Paddle, &Transform)>,
    mut game_state: Option<ResMut<GameState>>,
    mut time_slow: ResMut<TimeSlowState>,
) {
    for event in collision_reader.read() {
        let (powerup_entity, paddle_entity) = if powerups.get(event.collider1).is_ok()
            && paddles.get(event.collider2).is_ok()
        {
            (event.collider1, event.collider2)
        } else if powerups.get(event.collider2).is_ok() && paddles.get(event.collider1).is_ok() {
            (event.collider2, event.collider1)
        } else {
            continue;
        };

        let Ok(powerup) = powerups.get(powerup_entity) else {
            continue;
        };

        let kind = powerup.kind;
        let paddle_pos = paddles
            .get(paddle_entity)
            .map(|(_, t)| t.translation.truncate())
            .unwrap_or(Vec2::ZERO);

        match kind {
            PowerUpKind::ExtraLife => {
                if let Some(ref mut state) = game_state {
                    state.lives += 1;
                    info!("Extra life! Lives: {}", state.lives);
                }
            }
            PowerUpKind::DoublePoints => {
                if let Some(ref mut state) = game_state {
                    state.score += 100;
                    info!("Double points bonus! Score: {}", state.score);
                }
            }
            PowerUpKind::KillPaddle => {
                if let Some(ref mut state) = game_state {
                    state.lives = 0;
                    info!("Kill Paddle! Game over.");
                }
            }
            PowerUpKind::TimeSlow => {
                time_slow.active = true;
                time_slow.timer = Timer::from_seconds(8.0, TimerMode::Once);
                commands.trigger(BulletTimeEvent { entering: true });
                info!("Bullet time!");
            }
            _ => {
                info!("Power-up collected: {:?}", kind);
            }
        }

        let color = if kind.is_negative() {
            Color::srgb(1.0, 0.35, 0.35)
        } else {
            Color::srgb(0.45, 1.0, 0.65)
        };
        commands.trigger(FloatingTextEvent {
            text: kind.display_name().to_string(),
            position: paddle_pos + Vec2::new(0.0, 36.0),
            color,
            kind: FloatingTextKind::PowerUp,
        });

        commands.trigger(PowerUpCollectedEvent {
            powerup: powerup_entity,
            kind,
            position: paddle_pos,
        });
        commands.entity(powerup_entity).try_despawn();
    }
}

pub fn tick_time_slow(
    time: Res<Time>,
    mut commands: Commands,
    mut time_slow: ResMut<TimeSlowState>,
) {
    if time_slow.active {
        time_slow.timer.tick(time.delta());
        if time_slow.timer.is_finished() {
            time_slow.active = false;
            commands.trigger(BulletTimeEvent { entering: false });
            info!("Bullet time ended");
        }
    }
}
