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

#[derive(Resource, Default)]
pub struct EditorBuildState {
    pub block_count: usize,
    pub status_msg: String,
}

#[derive(Resource, Default)]
pub struct EditorProductionState {
    pub is_producing: bool,
    pub current_height: f32,
}
