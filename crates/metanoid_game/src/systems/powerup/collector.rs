use bevy::prelude::*;
use avian2d::prelude::*;
use metanoid_core::components::paddle::Paddle;
use metanoid_core::components::powerup::PowerUp;
use metanoid_core::events::PowerUpCollectedEvent;
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
    paddles: Query<&Paddle>,
    mut game_state: Option<ResMut<GameState>>,
) {
    for event in collision_reader.read() {
        let (powerup_entity, _paddle_entity) =
            if powerups.get(event.collider1).is_ok() && paddles.get(event.collider2).is_ok() {
                (event.collider1, event.collider2)
            } else if powerups.get(event.collider2).is_ok() && paddles.get(event.collider1).is_ok()
            {
                (event.collider2, event.collider1)
            } else {
                continue;
            };

        let Ok(powerup) = powerups.get(powerup_entity) else {
            continue;
        };

        let kind = powerup.kind;

        match kind {
            metanoid_core::components::powerup::PowerUpKind::ExtraLife => {
                if let Some(ref mut state) = game_state {
                    state.lives += 1;
                    info!("Extra life! Lives: {}", state.lives);
                }
            }
            metanoid_core::components::powerup::PowerUpKind::DoublePoints => {
                if let Some(ref mut state) = game_state {
                    state.score += 100;
                    info!("Double points bonus! Score: {}", state.score);
                }
            }
            metanoid_core::components::powerup::PowerUpKind::KillPaddle => {
                if let Some(ref mut state) = game_state {
                    state.lives = 0;
                    info!("Kill Paddle! Game over.");
                }
            }
            _ => {
                info!("Power-up collected: {:?}", kind);
            }
        }

        commands.trigger(PowerUpCollectedEvent {
            powerup: powerup_entity,
            kind,
        });
        commands.entity(powerup_entity).despawn();
    }
}

pub fn tick_time_slow(
    time: Res<Time>,
    mut time_slow: ResMut<TimeSlowState>,
) {
    if time_slow.active {
        time_slow.timer.tick(time.delta());
        if time_slow.timer.is_finished() {
            time_slow.active = false;
            info!("Bullet time ended");
        }
    }
}
