use bevy::prelude::*;

use crate::constants::EDITOR_FALL_SPEED;
use super::components::*;

/// Moves falling blocks downward each frame; snaps them when they reach their target Y
/// and converts them to landed `EditorBlock` components.
pub fn editor_fall_system(
    time: Res<Time>,
    mut commands: Commands,
    mut falling_query: Query<(Entity, &mut Transform, &FallingBlock)>,
) {
    for (entity, mut transform, falling) in &mut falling_query {
        transform.translation.y -= EDITOR_FALL_SPEED * time.delta_secs();

        if transform.translation.y <= falling.target_y {
            transform.translation.y = falling.target_y;
            commands
                .entity(entity)
                .remove::<FallingBlock>()
                .insert(EditorBlock {
                    width: falling.width,
                    height: falling.height,
                });
        }
    }
}
