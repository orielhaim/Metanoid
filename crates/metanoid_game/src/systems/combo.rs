use bevy::prelude::*;
use metanoid_core::components::brick::{Brick, BrickType};
use metanoid_core::events::{
    BrickDestroyedEvent, BrickHitEvent, ComboMilestoneEvent, FloatingTextEvent, FloatingTextKind,
    LifeLostEvent, PaddleHitEvent,
};
use metanoid_core::rating::LevelRunStats;
use metanoid_core::resources::combo::ComboCounter;
use metanoid_core::resources::game_state::GameState;

pub fn update_combo(time: Res<Time>, mut combo: ResMut<ComboCounter>) {
    combo.tick(time.delta());
}

fn brick_base_points(brick_type: BrickType) -> u64 {
    match brick_type {
        BrickType::Normal => 10,
        BrickType::MultiHit => 25,
        BrickType::Explosive => 30,
        BrickType::Moving => 35,
        BrickType::Regenerating => 40,
        BrickType::Invincible => 0,
    }
}

/// Vulnerability hit — builds the combo chain without requiring a break.
pub fn on_brick_hit_combo(
    trigger: On<BrickHitEvent>,
    mut commands: Commands,
    mut combo: ResMut<ComboCounter>,
    mut game_state: Option<ResMut<GameState>>,
    bricks: Query<(&Brick, &Transform)>,
) {
    let prev = combo.count;
    combo.register_hit();

    let (chip_base, pos) = if let Ok((brick, tf)) = bricks.get(trigger.brick) {
        let base = if brick.brick_type == BrickType::Invincible {
            2u64
        } else {
            3 + combo.count.min(12) as u64
        };
        (base, tf.translation.truncate())
    } else {
        (3u64, Vec2::Y * 40.0)
    };

    let chip_score = ((chip_base as f32) * combo.multiplier * 0.55)
        .round()
        .max(1.0) as u64;
    if let Some(ref mut state) = game_state {
        state.score += chip_score;
    }

    // Floating vuln text (every 2nd hit / milestones) — break path also shows on destroy
    if combo.count >= 2 && (combo.count % 2 == 0 || combo.count % 5 == 0) {
        let is_milestone = combo.count % 5 == 0 && combo.count != prev;
        let kind = if is_milestone {
            FloatingTextKind::Milestone
        } else {
            FloatingTextKind::Combo
        };
        let color = if is_milestone {
            Color::srgb(1.0, 0.85, 0.25)
        } else {
            Color::srgb(0.55, 0.95, 1.0)
        };
        let label = if is_milestone {
            format!("{}x VULN!", combo.count)
        } else {
            format!("{}x", combo.count)
        };
        commands.trigger(FloatingTextEvent {
            text: label,
            position: pos + Vec2::new(0.0, 16.0),
            color,
            kind,
        });
    }

    if combo.count > 0 && combo.count % 5 == 0 && combo.count != prev {
        commands.trigger(ComboMilestoneEvent { count: combo.count });
    }
}

/// Brick broken — cash out with break multiplier (counts more than a chip).
pub fn on_brick_destroyed_score(
    trigger: On<BrickDestroyedEvent>,
    mut commands: Commands,
    combo: Res<ComboCounter>,
    mut game_state: Option<ResMut<GameState>>,
    mut stats: Option<ResMut<LevelRunStats>>,
) {
    if let Some(ref mut s) = stats {
        s.bricks_destroyed = s.bricks_destroyed.saturating_add(1);
        s.max_combo = s.max_combo.max(combo.count);
    }

    let base = brick_base_points(trigger.brick_type);
    let points = (base as f32 * combo.break_multiplier()).round() as u64;

    if let Some(ref mut state) = game_state {
        state.score += points;
    }

    let pos = trigger.position;

    if combo.count >= 2 {
        let is_milestone = combo.count % 5 == 0;
        let kind = if is_milestone {
            FloatingTextKind::Milestone
        } else {
            FloatingTextKind::Combo
        };
        let color = if is_milestone {
            Color::srgb(1.0, 0.9, 0.3)
        } else if combo.count >= 10 {
            Color::srgb(1.0, 0.45, 0.85)
        } else {
            Color::srgb(0.35, 1.0, 0.55)
        };
        let label = if is_milestone {
            format!("{}x BREAK!", combo.count)
        } else {
            format!("+{}", points)
        };
        commands.trigger(FloatingTextEvent {
            text: label,
            position: pos + Vec2::new(0.0, 10.0),
            color,
            kind,
        });
    }

    if combo.count > 0 && combo.count % 5 == 0 {
        let bonus = 40u64 * (combo.count as u64 / 5);
        if let Some(ref mut state) = game_state {
            state.score += bonus;
        }
    }
}

pub fn on_paddle_hit_reset_combo(_trigger: On<PaddleHitEvent>, mut combo: ResMut<ComboCounter>) {
    combo.on_paddle_return();
}

pub fn on_life_lost_reset_combo(_trigger: On<LifeLostEvent>, mut combo: ResMut<ComboCounter>) {
    combo.on_life_lost();
}
