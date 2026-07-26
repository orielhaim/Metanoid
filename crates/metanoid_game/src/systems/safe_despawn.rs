use bevy::prelude::*;

/// Safely despawn an entity — no-op if already despawned.
pub fn safe_despawn(commands: &mut Commands, entity: Entity) {
    commands.entity(entity).despawn();
}
