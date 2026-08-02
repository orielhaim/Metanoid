use bevy::prelude::*;

/// Despawn without warning if the entity is already gone (common with multi-hit / AOE).
pub fn safe_despawn(commands: &mut Commands, entity: Entity) {
    commands.entity(entity).try_despawn();
}
