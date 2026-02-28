use bevy::prelude::*;

use crate::constants::*;
use super::components::*;
use super::resources::*;

/// Moves the camera up as the structure grows taller than the initial viewport.
/// Uses the same logic as the playing-mode camera follow.
pub fn editor_camera_follow(
    block_query: Query<
        &Transform,
        (
            Or<(With<EditorBlock>, With<FallingBlock>)>,
            Without<Camera2d>,
        ),
    >,
    mut camera_query: Query<
        &mut Transform,
        (With<Camera2d>, Without<EditorBlock>, Without<FallingBlock>),
    >,
) {
    let max_block_y = block_query
        .iter()
        .map(|t| t.translation.y)
        .fold(f32::NEG_INFINITY, f32::max);

    // No blocks yet — keep camera centered.
    if max_block_y == f32::NEG_INFINITY {
        return;
    }

    let target_top = max_block_y + SPAWN_HEIGHT_ABOVE + 50.0;
    let view_center = (GROUND_Y + target_top) / 2.0;

    let target_y = if target_top - GROUND_Y > 500.0 {
        view_center
    } else {
        0.0
    };

    if let Ok(mut cam_t) = camera_query.single_mut() {
        cam_t.translation.y += (target_y - cam_t.translation.y) * 0.05;
    }
}

/// Refreshes the HUD text with current block count and status message.
pub fn editor_update_hud(
    mut build_state: ResMut<EditorBuildState>,
    time: Res<Time>,
    mut hud_query: Query<&mut Text2d, With<EditorHudText>>,
) {
    // Tick down the status message timer.
    if build_state.status_timer > 0.0 {
        build_state.status_timer -= time.delta_secs();
        if build_state.status_timer <= 0.0 {
            build_state.status_msg.clear();
        }
    }

    if let Ok(mut text) = hud_query.single_mut() {
        // Filename input mode overrides everything else.
        if let Some(ref buf) = build_state.filename_input {
            text.0 = format!("Save as: {buf}|");
            return;
        }

        let status = if build_state.status_msg.is_empty() {
            "Arrows: move   Space/Down: place   S: save   R: reset   P: test   Esc: menu"
        } else {
            &build_state.status_msg
        };
        text.0 = format!(
            "Level Editor  |  Blocks: {}\n{}",
            build_state.block_count, status
        );
    }
}
