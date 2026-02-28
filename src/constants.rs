use bevy::prelude::*;

// === Color Palette ===
pub const BG_COLOR: Color = Color::srgb(0.30, 0.35, 0.48);
pub const SLOT_COLOR: Color = Color::srgb(0.88, 0.76, 0.48);
pub const PRODUCTION_COLOR: Color = Color::srgb(0.90, 0.60, 0.32);
pub const GROUND_COLOR: Color = Color::srgb(0.38, 0.32, 0.28);
pub const GHOST_COLOR: Color = Color::srgba(0.65, 0.72, 0.88, 0.18);
pub const GHOST_NEXT_COLOR: Color = Color::srgba(0.80, 0.88, 1.00, 0.45);
pub const TOWER_BLOCK_COLOR: Color = Color::srgb(0.88, 0.72, 0.40);
pub const FAIL_COLOR: Color = Color::srgb(0.82, 0.32, 0.25);
pub const TEXT_COLOR: Color = Color::srgb(0.92, 0.90, 0.88);

// === Layout Constants ===
pub const GROUND_Y: f32 = -200.0;
pub const GROUND_HALF_HEIGHT: f32 = 10.0;
pub const GROUND_WIDTH: f32 = 800.0;
pub const SPAWN_HEIGHT_ABOVE: f32 = 150.0;
pub const FAIL_Y_THRESHOLD: f32 = GROUND_Y - GROUND_HALF_HEIGHT * 2.0 - 10.0; // = -230

// === Slot Constants ===
pub const SLOT_MIN_WIDTH: f32 = 20.0;
pub const SLOT_MAX_WIDTH: f32 = 200.0;
pub const SLOT_SPEED: f32 = 2.5;
pub const SLOT_HEIGHT: f32 = 8.0;

// === Production Constants ===
pub const GROW_SPEED: f32 = 150.0;
pub const MAX_HEIGHT: f32 = 200.0;

// === Physics tuning ===
pub const GRAVITY_SCALE: f32 = 800.0;

// === Score Popup Constants ===
pub const POPUP_DURATION: f32 = 1.8;
pub const POPUP_FLOAT_SPEED: f32 = 40.0;

// === Editor Constants ===
pub const EDITOR_SLOT_MOVE_SPEED: f32 = 200.0;
pub const EDITOR_FALL_SPEED: f32 = 400.0;
pub const EDITOR_BLOCK_COLOR: Color = Color::srgb(0.52, 0.68, 0.88);
