use bevy::prelude::*;
use crate::constants::*;

#[derive(Resource)]
pub struct EditorSlotState {
    pub phase: f32,
    pub speed: f32,
    pub current_width: f32,
    /// Set when Space is held; clears the oscillation so the width stays fixed.
    pub locked_width: Option<f32>,
    /// Horizontal position, driven by arrow keys (clamped ±370 px).
    pub slot_x: f32,
    /// Vertical position, updated each frame to stay above the tallest block.
    pub slot_y: f32,
}

impl Default for EditorSlotState {
    fn default() -> Self {
        Self {
            phase: 0.0,
            speed: SLOT_SPEED,
            current_width: SLOT_MIN_WIDTH,
            locked_width: None,
            slot_x: 0.0,
            slot_y: GROUND_Y + GROUND_HALF_HEIGHT + SPAWN_HEIGHT_ABOVE,
        }
    }
}

#[derive(Resource)]
pub struct EditorBuildState {
    pub block_count: usize,
    pub status_msg: String,
    /// Seconds remaining before `status_msg` is auto-cleared; 0.0 = expired.
    pub status_timer: f32,
    /// `None` = inactive, `Some(buf)` = user is typing a filename.
    pub filename_input: Option<String>,
    /// Height of the last placed block (used by Arrow-Down instant-place).
    pub last_block_height: f32,
}

impl Default for EditorBuildState {
    fn default() -> Self {
        Self {
            block_count: 0,
            status_msg: String::new(),
            status_timer: 0.0,
            filename_input: None,
            last_block_height: 50.0,
        }
    }
}

#[derive(Resource, Default)]
pub struct EditorProductionState {
    pub is_producing: bool,
    pub current_height: f32,
}

/// Marker resource: Playing mode was entered from the editor (test-play).
#[derive(Resource)]
pub struct EditorTestPlay;

/// Snapshot of editor blocks, stored before entering test-play so the editor
/// can be restored when the player returns.
#[derive(Resource)]
pub struct EditorSnapshot {
    /// Each entry: (world position, width, height)
    pub blocks: Vec<(Vec3, f32, f32)>,
}
