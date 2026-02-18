use bevy::prelude::*;

/// Marker for all editor-owned entities — used for bulk cleanup on exit.
#[derive(Component)]
pub struct EditorEntity;

/// Placed (landed) block in the editor scene.
#[derive(Component)]
pub struct EditorBlock {
    pub width: f32,
    pub height: f32,
}

/// Block that is currently animating downward toward its snap target.
#[derive(Component)]
pub struct FallingBlock {
    pub target_y: f32,
    pub width: f32,
    pub height: f32,
}

#[derive(Component)]
pub struct EditorSlotIndicator;

#[derive(Component)]
pub struct EditorProductionRect;

#[derive(Component)]
pub struct EditorHudText;
