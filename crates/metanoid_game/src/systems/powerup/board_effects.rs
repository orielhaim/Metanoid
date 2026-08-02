use bevy::prelude::*;
use metanoid_core::components::brick::{Brick, BrickType};
use metanoid_core::components::powerup::PowerUpKind;
use metanoid_core::constants::*;
use metanoid_core::events::PowerUpCollectedEvent;
use rand::prelude::*;

use super::super::level_clear::destroy_brick;

pub fn apply_board_effect(
    trigger: On<PowerUpCollectedEvent>,
    mut commands: Commands,
    mut bricks: Query<(Entity, &mut Brick, &mut Transform)>,
) {
    match trigger.kind {
        PowerUpKind::FallingBricks => apply_falling_bricks(&mut bricks),
        PowerUpKind::Zap => apply_zap(&mut bricks),
        PowerUpKind::Explode => apply_explode(&mut commands, &mut bricks),
        PowerUpKind::ExpandExploding => apply_expand_exploding(&mut bricks),
        PowerUpKind::Lightning => {
            commands.trigger(metanoid_core::events::LightningEvent);
            apply_lightning(&mut commands, &mut bricks);
        }
        PowerUpKind::Shockwave => {
            commands.trigger(metanoid_core::events::ShockwaveEvent);
            apply_shockwave(&mut commands, &mut bricks);
        }
        PowerUpKind::ShuffleBricks => apply_shuffle(&mut bricks),
        _ => {}
    }
}

fn apply_falling_bricks(bricks: &mut Query<(Entity, &mut Brick, &mut Transform)>) {
    let drop = BRICK_HEIGHT + BRICK_GAP;
    for (_, _, mut transform) in bricks.iter_mut() {
        transform.translation.y -= drop;
    }
}

fn apply_zap(bricks: &mut Query<(Entity, &mut Brick, &mut Transform)>) {
    for (_, mut brick, _) in bricks.iter_mut() {
        if brick.brick_type == BrickType::Invincible {
            brick.brick_type = BrickType::Normal;
            brick.health = 1;
            brick.max_health = 1;
        }
    }
}

fn apply_explode(
    commands: &mut Commands,
    bricks: &mut Query<(Entity, &mut Brick, &mut Transform)>,
) {
    let explosive_positions: Vec<Vec2> = bricks
        .iter()
        .filter(|(_, b, _)| b.brick_type == BrickType::Explosive)
        .map(|(_, _, t)| t.translation.truncate())
        .collect();

    let blast_radius = (BRICK_WIDTH + BRICK_GAP) * 1.5;
    let mut to_destroy: Vec<(Entity, Vec2)> = Vec::new();

    for (entity, brick, transform) in bricks.iter() {
        if brick.health == 0 {
            continue;
        }
        let pos = transform.translation.truncate();
        let near_explosive = explosive_positions
            .iter()
            .any(|ep| (*ep - pos).length() < blast_radius);

        if near_explosive && brick.brick_type != BrickType::Invincible {
            to_destroy.push((entity, pos));
        }
    }

    for (entity, pos) in to_destroy {
        if let Ok((_, mut brick, _)) = bricks.get_mut(entity) {
            destroy_brick(commands, entity, &mut brick, pos);
        }
    }
}

fn apply_expand_exploding(bricks: &mut Query<(Entity, &mut Brick, &mut Transform)>) {
    let mut rng = rand::rng();
    let candidates: Vec<Entity> = bricks
        .iter()
        .filter(|(_, b, _)| b.brick_type == BrickType::Normal)
        .map(|(e, _, _)| e)
        .collect();

    let count = (candidates.len() as f32 * 0.2).ceil() as usize;
    let selected: Vec<Entity> = candidates.sample(&mut rng, count).copied().collect();

    for entity in selected {
        if let Ok((_, mut brick, _)) = bricks.get_mut(entity) {
            brick.brick_type = BrickType::Explosive;
        }
    }
}

fn apply_lightning(
    commands: &mut Commands,
    bricks: &mut Query<(Entity, &mut Brick, &mut Transform)>,
) {
    let mut rng = rand::rng();
    let targets: Vec<(Entity, Vec2)> = bricks
        .iter()
        .filter(|(_, b, _)| b.brick_type != BrickType::Invincible && b.health > 0)
        .map(|(e, _, t)| (e, t.translation.truncate()))
        .collect();

    let strike_count = (targets.len() as f32 * 0.15).ceil() as usize;
    let strikes: Vec<(Entity, Vec2)> = targets.sample(&mut rng, strike_count).copied().collect();

    for (entity, pos) in strikes {
        if let Ok((_, mut brick, _)) = bricks.get_mut(entity) {
            destroy_brick(commands, entity, &mut brick, pos);
        }
    }
}

fn apply_shockwave(
    commands: &mut Commands,
    bricks: &mut Query<(Entity, &mut Brick, &mut Transform)>,
) {
    let center = Vec2::new(0.0, PADDLE_Y);
    let radius = ARENA_WIDTH * 0.4;
    let mut to_destroy: Vec<(Entity, Vec2)> = Vec::new();

    for (entity, brick, transform) in bricks.iter() {
        if brick.health == 0 {
            continue;
        }
        let pos = transform.translation.truncate();
        let dist = (pos - center).length();

        if dist < radius && brick.brick_type != BrickType::Invincible {
            to_destroy.push((entity, pos));
        }
    }

    for (entity, pos) in to_destroy {
        if let Ok((_, mut brick, _)) = bricks.get_mut(entity) {
            destroy_brick(commands, entity, &mut brick, pos);
        }
    }
}

fn apply_shuffle(bricks: &mut Query<(Entity, &mut Brick, &mut Transform)>) {
    let mut rng = rand::rng();
    let mut positions: Vec<Vec2> = bricks
        .iter()
        .map(|(_, _, t)| t.translation.truncate())
        .collect();

    positions.shuffle(&mut rng);

    let entities: Vec<Entity> = bricks.iter().map(|(e, _, _)| e).collect();
    for (entity, pos) in entities.iter().zip(positions.iter()) {
        if let Ok((_, _, mut transform)) = bricks.get_mut(*entity) {
            transform.translation.x = pos.x;
            transform.translation.y = pos.y;
        }
    }
}
