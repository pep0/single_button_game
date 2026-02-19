use avian2d::prelude::*;
use bevy::prelude::*;

use crate::blueprint::Blueprint;
use crate::constants::*;
use crate::editor::EditorTestPlay;
use crate::state::GameState;
use super::components::*;
use super::resources::*;

pub fn check_settle(
    time: Res<Time>,
    mut commands: Commands,
    mut build_state: ResMut<BuildState>,
    blueprint: Res<Blueprint>,
    mut produced: ResMut<ProducedDimensions>,
    block_query: Query<(&TowerBlock, &Transform, Option<&Sleeping>, &LinearVelocity)>,
    mut next_state: ResMut<NextState<GameState>>,
    testplay: Option<Res<EditorTestPlay>>,
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
            if testplay.is_some() {
                next_state.set(GameState::Editor);
            } else {
                next_state.set(GameState::Failed);
            }
            return;
        }

        // Criterion 2: block center more than 100px from blueprint target → off position
        let dx = transform.translation.x - target.x;
        let dy = transform.translation.y - target.y;
        if (dx * dx + dy * dy).sqrt() > SLOT_MAX_WIDTH / 2.0 {
            if testplay.is_some() {
                next_state.set(GameState::Editor);
            } else {
                next_state.set(GameState::Failed);
            }
            return;
        }
    }

    // Compute per-block scores
    let n = blueprint.slots.len();
    let mut scores = vec![0.0f32; n];
    let mut block_data: Vec<(usize, f32, Vec3, f32)> = Vec::new(); // (slot_idx, score, pos, height)

    for (tower_block, transform, _, _) in &block_query {
        let i = tower_block.0;
        let slot = &blueprint.slots[i];
        let pw = produced.widths[i];
        let ph = produced.heights[i];
        let w_ratio = (pw / slot.width).min(slot.width / pw);
        let h_ratio = (ph / slot.height).min(slot.height / ph);
        let score = w_ratio * h_ratio;
        scores[i] = score;
        block_data.push((i, score, transform.translation, ph));
    }

    // Per-block 30% failure check
    for &(_, score, _, _) in &block_data {
        if score < 0.30 {
            if testplay.is_some() {
                next_state.set(GameState::Editor);
            } else {
                next_state.set(GameState::Failed);
            }
            return;
        }
    }

    // Store scores and spawn popups
    produced.scores = scores;

    for &(_, score, pos, height) in &block_data {
        let (r, g, b, font_size) = score_visuals(score);
        commands.spawn((
            PlayingEntity,
            ScorePopup { age: 0.0, base_r: r, base_g: g, base_b: b },
            Text2d::new(format!("{:.0}%", score * 100.0)),
            TextFont { font_size, ..default() },
            TextColor(Color::srgba(r, g, b, 1.0)),
            Transform::from_xyz(pos.x, pos.y + height / 2.0 + 10.0, 2.0),
        ));
    }

    build_state.waiting_for_settle = false;
    build_state.showing_popups = true;
    // State transition will be fired by animate_score_popups after popup animation completes
}

fn score_visuals(score: f32) -> (f32, f32, f32, f32) {
    if score >= 0.80 {
        (0.2, 0.95, 0.55, 28.0)   // bright green, large
    } else if score >= 0.60 {
        (0.95, 0.80, 0.2, 22.0)   // warm yellow, medium
    } else {
        (0.65, 0.65, 0.65, 16.0)  // grey, small
    }
}

pub fn check_failure(
    block_query: Query<&Transform, With<TowerBlock>>,
    mut next_state: ResMut<NextState<GameState>>,
    testplay: Option<Res<EditorTestPlay>>,
) {
    for transform in &block_query {
        if transform.translation.y < FAIL_Y_THRESHOLD {
            if testplay.is_some() {
                next_state.set(GameState::Editor);
            } else {
                next_state.set(GameState::Failed);
            }
            return;
        }
    }
}
