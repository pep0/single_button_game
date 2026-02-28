use bevy::prelude::*;
use crate::constants::*;

#[derive(Resource, Default)]
pub struct ScreenShake {
    /// Current trauma level 0.0–1.0; decays over time.
    pub trauma: f32,
    /// Logical camera Y (without shake offset), tracked here so shake doesn't drift the lerp.
    pub base_camera_y: f32,
}

#[derive(Resource)]
pub struct BuildState {
    pub current_index: usize,
    pub waiting_for_settle: bool,
    pub settle_timer: f32,
    pub stability_window: f32,
    pub showing_popups: bool,
    pub popup_timer: f32,
    pub showing_level_complete: bool,
    pub level_complete_timer: f32,
}

impl Default for BuildState {
    fn default() -> Self {
        Self {
            current_index: 0,
            waiting_for_settle: false,
            settle_timer: 0.0,
            stability_window: 0.0,
            showing_popups: false,
            popup_timer: 0.0,
            showing_level_complete: false,
            level_complete_timer: 0.0,
        }
    }
}

#[derive(Resource)]
pub struct SlotState {
    pub phase: f32,
    pub speed: f32,
    pub current_width: f32,
    pub locked_width: Option<f32>,
}

impl Default for SlotState {
    fn default() -> Self {
        Self {
            phase: 0.0,
            speed: SLOT_SPEED,
            current_width: SLOT_MIN_WIDTH,
            locked_width: None,
        }
    }
}

#[derive(Resource, Default)]
pub struct ProductionState {
    pub is_producing: bool,
    pub current_height: f32,
}

#[derive(Resource)]
pub struct ProducedDimensions {
    pub widths: Vec<f32>,
    pub heights: Vec<f32>,
    pub scores: Vec<f32>,
}

impl Default for ProducedDimensions {
    fn default() -> Self {
        Self {
            widths: Vec::new(),
            heights: Vec::new(),
            scores: Vec::new(),
        }
    }
}
