use avian2d::prelude::*;
use bevy::prelude::*;

use crate::blueprint::Blueprint;
use crate::constants::*;
use crate::state::GameState;
use super::components::*;
use super::resources::*;

pub fn check_settle(
    time: Res<Time>,
    mut build_state: ResMut<BuildState>,
    blueprint: Res<Blueprint>,
    block_query: Query<(&TowerBlock, &Transform, Option<&Sleeping>, &LinearVelocity)>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if !build_state.waiting_for_settle {
        return;
    }

    // Minimum wait before checking (let physics initialize and gravity take effect)
    build_state.settle_timer += time.delta_secs();
    if build_state.settle_timer < 1.5 {
        return;
    }

    // Advance or reset the sustained-rest window
    let all_at_rest = block_query
        .iter()
        .all(|(_, _, sleeping, vel)| sleeping.is_some() || vel.0.length() < 2.0);

    if all_at_rest {
        build_state.stability_window += time.delta_secs();
    } else {
        build_state.stability_window = 0.0;
        return;
    }

    // Require all blocks to be continuously at rest for 0.5s before declaring stable
    if build_state.stability_window < 0.5 {
        return;
    }

    // Evaluate pass/fail criteria for every placed block
    for (tower_block, transform, _, _) in &block_query {
        let target = &blueprint.slots[tower_block.0];

        // Criterion 1: block tilted more than 15° → toppled
        let (_, _, angle_z) = transform.rotation.to_euler(EulerRot::XYZ);
        if angle_z.abs() > 15_f32.to_radians() {
            next_state.set(GameState::Failed);
            return;
        }

        // Criterion 2: block center more than 100px from blueprint target → off position
        let dx = transform.translation.x - target.x;
        let dy = transform.translation.y - target.y;
        if (dx * dx + dy * dy).sqrt() > SLOT_MAX_WIDTH / 2.0 {
            next_state.set(GameState::Failed);
            return;
        }
    }

    // All blocks passed → scoring
    next_state.set(GameState::Scoring);
}

pub fn check_failure(
    block_query: Query<&Transform, With<TowerBlock>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    for transform in &block_query {
        if transform.translation.y < FAIL_Y_THRESHOLD {
            next_state.set(GameState::Failed);
            return;
        }
    }
}
