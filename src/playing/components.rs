use bevy::prelude::*;

#[derive(Component)]
pub struct PlayingEntity;

#[derive(Component)]
pub struct SlotIndicator;

#[derive(Component)]
pub struct ProductionRect;

#[derive(Component)]
pub struct TowerBlock(#[allow(dead_code)] pub usize);

#[derive(Component)]
pub struct GhostBlock(pub usize);

#[derive(Component)]
pub struct HudText;
