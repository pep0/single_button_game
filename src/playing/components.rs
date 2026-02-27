use bevy::prelude::*;

#[derive(Component)]
pub struct PlayingEntity;

#[derive(Component)]
pub struct SlotIndicator;

#[derive(Component)]
pub struct ProductionRect;

#[derive(Component)]
pub struct TowerBlock(#[allow(dead_code)] pub usize);

#[derive(Component, Default)]
pub struct BlockSettleTimer {
    pub rest_secs: f32,
    pub popup_shown: bool,
    pub prev_speed: f32,
    pub collision_cooldown: f32, // seconds until next collision sound allowed
}

#[derive(Component)]
pub struct GhostBlock(pub usize);

#[derive(Component)]
pub struct HudText;

#[derive(Component)]
pub struct ScorePopup {
    pub age: f32,
    pub base_r: f32,
    pub base_g: f32,
    pub base_b: f32,
}

#[derive(Component)]
pub struct FrozenTowerBlock {
    pub height: f32,
}

#[derive(Component)]
pub struct TowerBlockDims {
    pub height: f32,
}

#[derive(Component)]
pub struct LevelCompleteOverlay;

#[derive(Component)]
pub struct HeartIcon(pub usize); // index 0..MAX_LIVES-1

#[derive(Component)]
pub struct EvaluatingText;
