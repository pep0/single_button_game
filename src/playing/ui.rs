use bevy::prelude::*;

use crate::blueprint::Blueprint;
use crate::constants::*;
use super::components::*;
use super::resources::*;

pub fn camera_follow(
    blueprint: Res<Blueprint>,
    block_query: Query<&Transform, With<TowerBlock>>,
    mut camera_query: Query<&mut Transform, (With<Camera2d>, Without<TowerBlock>)>,
) {
    // Find the highest point we need to see
    let max_ghost_y = blueprint
        .slots
        .iter()
        .map(|s| s.y + s.height / 2.0)
        .fold(f32::NEG_INFINITY, f32::max);

    let max_block_y = block_query
        .iter()
        .map(|t| t.translation.y)
        .fold(f32::NEG_INFINITY, f32::max);

    let target_top = max_ghost_y.max(max_block_y) + SPAWN_HEIGHT_ABOVE + 50.0;
    // We want to center the view between ground and the top
    let view_center = (GROUND_Y + target_top) / 2.0;

    // Only move camera up if the structure gets tall enough
    let target_y = if target_top - GROUND_Y > 500.0 {
        view_center
    } else {
        0.0
    };

    if let Ok(mut cam_t) = camera_query.single_mut() {
        cam_t.translation.y += (target_y - cam_t.translation.y) * 0.05;
    }
}

pub fn update_ghost_highlights(
    build_state: Res<BuildState>,
    ghost_query: Query<(&GhostBlock, &MeshMaterial2d<ColorMaterial>)>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    for (ghost, mat_handle) in &ghost_query {
        if let Some(mat) = materials.get_mut(&mat_handle.0) {
            if ghost.0 == build_state.current_index {
                mat.color = GHOST_NEXT_COLOR;
            } else if ghost.0 < build_state.current_index {
                mat.color = Color::srgba(0.3, 0.5, 0.3, 0.08);
            } else {
                mat.color = GHOST_COLOR;
            }
        }
    }
}

pub fn update_hud(
    build_state: Res<BuildState>,
    blueprint: Res<Blueprint>,
    mut hud_query: Query<&mut Text2d, With<HudText>>,
) {
    if let Ok(mut text) = hud_query.single_mut() {
        let block_num = (build_state.current_index + 1).min(blueprint.slots.len());
        text.0 = format!(
            "Level: {}    Block: {}/{}",
            blueprint.level_number, block_num, blueprint.slots.len(),
        );
    }
}
