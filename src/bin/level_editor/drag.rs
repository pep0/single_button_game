use bevy::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HandlePosition {
    TopLeft,
    TopCenter,
    TopRight,
    MiddleLeft,
    MiddleRight,
    BottomLeft,
    BottomCenter,
    BottomRight,
}

impl HandlePosition {
    /// Returns (x_sign, y_sign) for the offset from block center.
    pub fn offset_signs(&self) -> (f32, f32) {
        match self {
            HandlePosition::TopLeft => (-1.0, 1.0),
            HandlePosition::TopCenter => (0.0, 1.0),
            HandlePosition::TopRight => (1.0, 1.0),
            HandlePosition::MiddleLeft => (-1.0, 0.0),
            HandlePosition::MiddleRight => (1.0, 0.0),
            HandlePosition::BottomLeft => (-1.0, -1.0),
            HandlePosition::BottomCenter => (0.0, -1.0),
            HandlePosition::BottomRight => (1.0, -1.0),
        }
    }

    /// Returns the handle on the opposite side (for anchor during resize).
    pub fn opposite(&self) -> HandlePosition {
        match self {
            HandlePosition::TopLeft => HandlePosition::BottomRight,
            HandlePosition::TopCenter => HandlePosition::BottomCenter,
            HandlePosition::TopRight => HandlePosition::BottomLeft,
            HandlePosition::MiddleLeft => HandlePosition::MiddleRight,
            HandlePosition::MiddleRight => HandlePosition::MiddleLeft,
            HandlePosition::BottomLeft => HandlePosition::TopRight,
            HandlePosition::BottomCenter => HandlePosition::TopCenter,
            HandlePosition::BottomRight => HandlePosition::TopLeft,
        }
    }

    /// Returns true if this handle controls the X axis (width/horizontal position).
    pub fn controls_x(&self) -> bool {
        !matches!(self, HandlePosition::TopCenter | HandlePosition::BottomCenter)
    }

    /// Returns true if this handle controls the Y axis (height/vertical position).
    pub fn controls_y(&self) -> bool {
        !matches!(self, HandlePosition::MiddleLeft | HandlePosition::MiddleRight)
    }

    pub fn all() -> [HandlePosition; 8] {
        [
            HandlePosition::TopLeft,
            HandlePosition::TopCenter,
            HandlePosition::TopRight,
            HandlePosition::MiddleLeft,
            HandlePosition::MiddleRight,
            HandlePosition::BottomLeft,
            HandlePosition::BottomCenter,
            HandlePosition::BottomRight,
        ]
    }
}

pub enum DragMode {
    Idle,
    MovingBlock {
        slot_index: usize,
        grab_offset: Vec2,
    },
    ResizingBlock {
        slot_index: usize,
        handle: HandlePosition,
        anchor: Vec2, // world-space position that stays fixed
    },
    DrawingNew {
        start: Vec2,
        preview_entity: Entity,
    },
}

#[derive(Resource)]
pub struct DragState {
    pub mode: DragMode,
    pub pan: Option<(Vec2, Vec3)>, // (screen cursor at press, camera translation at press)
}

impl Default for DragState {
    fn default() -> Self {
        Self { mode: DragMode::Idle, pan: None }
    }
}
