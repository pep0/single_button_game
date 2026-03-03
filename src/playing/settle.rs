use avian2d::prelude::*;
use bevy::prelude::*;

use crate::blueprint::Blueprint;
use crate::constants::*;
use crate::editor::EditorTestPlay;
use crate::state::{FailureReason, GameState};
use bevy::ecs::message::MessageWriter;
use super::audio::BlockLanded;
use super::components::*;
use super::particles;
use super::resources::*;

pub fn check_settle(
    mut commands: Commands,
    time: Res<Time>,
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
            commands.insert_resource(FailureReason {
                message: "Block tilted too far".to_string(),
            });
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
            commands.insert_resource(FailureReason {
                message: "Block fell off target position".to_string(),
            });
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

    // Per-block 20% failure check
    for &(_, score, _, _) in &block_data {
        if score < 0.20 {
            commands.insert_resource(FailureReason {
                message: format!("Block too inaccurate ({:.0}%)", score * 100.0),
            });
            if testplay.is_some() {
                next_state.set(GameState::Editor);
            } else {
                next_state.set(GameState::Failed);
            }
            return;
        }
    }

    // Store scores
    produced.scores = scores;

    build_state.waiting_for_settle = false;
    build_state.showing_popups = true;
    // State transition will be fired by animate_score_popups after popup animation completes
}

pub fn check_per_block_settle(
    time: Res<Time>,
    mut commands: Commands,
    build_state: Res<BuildState>,
    blueprint: Res<Blueprint>,
    produced: Res<ProducedDimensions>,
    mut level_score: ResMut<LevelScoreBar>,
    camera_query: Query<&Transform, With<Camera2d>>,
    mut block_query: Query<(
        &TowerBlock,
        &mut BlockSettleTimer,
        &Transform,
        Option<&Sleeping>,
        &LinearVelocity,
    )>,
) {
    if build_state.showing_popups {
        return;
    }

    let cam_y = camera_query.single().map(|t| t.translation.y).unwrap_or(0.0);
    // Keep popups within the top 80 px of margin so they can float without clipping
    let popup_y_max = cam_y + 384.0 - 80.0;

    let dt = time.delta_secs();
    for (tower_block, mut timer, transform, sleeping, vel) in &mut block_query {
        if timer.popup_shown {
            continue;
        }
        let i = tower_block.0;
        // Guard: dimensions must already be recorded (pushed on Space-release)
        if i >= produced.widths.len() {
            continue;
        }

        if sleeping.is_some() || vel.0.length() < 2.0 {
            timer.rest_secs += dt;
        } else {
            timer.rest_secs = 0.0;
        }

        if timer.rest_secs >= 0.4 {
            let slot = &blueprint.slots[i];
            let pw = produced.widths[i];
            let ph = produced.heights[i];
            let score = (pw / slot.width).min(slot.width / pw)
                * (ph / slot.height).min(slot.height / ph);

            // Accumulate tier points into the level score bar
            let tier: u8 = if score >= 0.80 { 2 } else if score >= 0.60 { 1 } else { 0 };
            level_score.accumulated += tier as i32;

            let (r, g, b, font_size) = score_visuals(score);
            let spawn_y = (transform.translation.y + ph / 2.0 + 10.0).min(popup_y_max);
            commands.spawn((
                PlayingEntity,
                ScorePopup { age: 0.0, base_r: r, base_g: g, base_b: b },
                Text2d::new(format!("{:.0}%", score * 100.0)),
                TextFont { font_size, ..default() },
                TextColor(Color::srgba(r, g, b, 1.0)),
                Transform::from_xyz(transform.translation.x, spawn_y, 2.0),
            ));
            timer.popup_shown = true;
        }
    }
}

fn score_visuals(score: f32) -> (f32, f32, f32, f32) {
    if score >= 0.80 {
        (0.38, 0.88, 0.55, 28.0)  // spring green, large
    } else if score >= 0.60 {
        (0.90, 0.75, 0.30, 22.0)  // warm gold, medium
    } else {
        (0.60, 0.58, 0.62, 16.0)  // medium grey, small
    }
}

pub fn check_failure(
    mut commands: Commands,
    block_query: Query<&Transform, With<TowerBlock>>,
    mut next_state: ResMut<NextState<GameState>>,
    testplay: Option<Res<EditorTestPlay>>,
) {
    for transform in &block_query {
        if transform.translation.y < FAIL_Y_THRESHOLD {
            commands.insert_resource(FailureReason {
                message: "Block fell off the structure".to_string(),
            });
            if testplay.is_some() {
                next_state.set(GameState::Editor);
            } else {
                next_state.set(GameState::Failed);
            }
            return;
        }
    }
}

pub fn detect_landings(
    time: Res<Time>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut shake: ResMut<ScreenShake>,
    produced: Res<ProducedDimensions>,
    mut block_query: Query<(&TowerBlock, &mut BlockSettleTimer, &LinearVelocity, &TowerBlockDims, &Transform)>,
    mut landed: MessageWriter<BlockLanded>,
) {
    // Decay trauma each frame
    shake.trauma = (shake.trauma - time.delta_secs() * 2.5).max(0.0);

    for (block, mut timer, vel, dims, transform) in &mut block_query {
        let speed = vel.0.length();
        let vel_drop = timer.prev_speed - speed;
        timer.prev_speed = speed;

        // Landing: significant deceleration and block still moving (not drifting at rest)
        if vel_drop > 80.0 && speed < 100.0 {
            let i = block.0;
            let width = if i < produced.widths.len() { produced.widths[i] } else { 80.0 };
            let area_ratio = (width * dims.height / (SLOT_MAX_WIDTH * MAX_HEIGHT)).sqrt();
            let trauma_add = (0.4 + area_ratio * 0.5).clamp(0.4, 0.9);
            shake.trauma = (shake.trauma + trauma_add).min(1.0);
            landed.write(BlockLanded { area_ratio, impact_speed: vel_drop });
            particles::spawn_smoke_burst(
                &mut commands,
                &mut meshes,
                &mut materials,
                transform.translation.x,
                transform.translation.y - dims.height / 2.0,
                width,
                block.0 as u32,
            );
        }
    }
}
