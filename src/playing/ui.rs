use bevy::prelude::*;

use crate::blueprint::Blueprint;
use crate::constants::*;
use crate::editor::EditorTestPlay;
use crate::state::{GameState, LevelSequence, Score};
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

pub fn animate_score_popups(
    time: Res<Time>,
    mut build_state: ResMut<BuildState>,
    mut popup_query: Query<(&mut ScorePopup, &mut Transform, &mut TextColor)>,
    mut commands: Commands,
    mut score: ResMut<Score>,
    produced: Res<ProducedDimensions>,
    blueprint: Res<Blueprint>,
    sequence: Res<LevelSequence>,
    camera_query: Query<&Transform, (With<Camera2d>, Without<ScorePopup>)>,
) {
    let dt = time.delta_secs();

    // Always animate any existing popups (rise + fade), regardless of settle state
    for (mut popup, mut transform, mut color) in &mut popup_query {
        popup.age += dt;
        transform.translation.y += POPUP_FLOAT_SPEED * dt;
        let alpha = (1.0 - popup.age / POPUP_DURATION).max(0.0);
        color.0 = Color::srgba(popup.base_r, popup.base_g, popup.base_b, alpha);
    }

    // State transition still waits for global settle to complete
    if !build_state.showing_popups {
        return;
    }

    build_state.popup_timer += dt;
    if build_state.popup_timer >= POPUP_DURATION {
        build_state.showing_popups = false;

        // Record round score
        let n = produced.scores.len();
        let avg = if n > 0 {
            produced.scores.iter().sum::<f32>() / n as f32
        } else {
            0.0
        };
        score.total_score += avg;
        score.rounds_played += 1;

        // Spawn overlay (tagged PlayingEntity — cleaned up on level transition)
        let cam_y = camera_query
            .single()
            .map(|t| t.translation.y)
            .unwrap_or(0.0);
        let is_last = score.round + 1 >= sequence.entries.len();
        let msg = if is_last {
            format!("Level {} Complete!  \u{2605}", blueprint.level_number)
        } else {
            format!("Level {} Complete!", blueprint.level_number)
        };
        commands.spawn((
            PlayingEntity,
            LevelCompleteOverlay,
            Text2d::new(msg),
            TextFont {
                font_size: 52.0,
                ..default()
            },
            TextColor(Color::srgba(0.3, 0.95, 0.55, 0.0)),
            Transform::from_xyz(0.0, cam_y + 40.0, 10.0),
        ));

        build_state.showing_level_complete = true;
        build_state.level_complete_timer = 0.0;
    }
}

pub fn animate_level_complete(
    time: Res<Time>,
    mut build_state: ResMut<BuildState>,
    mut overlay_query: Query<(&mut TextColor, &mut Transform), With<LevelCompleteOverlay>>,
    camera_query: Query<&Transform, (With<Camera2d>, Without<LevelCompleteOverlay>)>,
    mut next_state: ResMut<NextState<GameState>>,
    mut score: ResMut<Score>,
    sequence: Res<LevelSequence>,
    testplay: Option<Res<EditorTestPlay>>,
) {
    if !build_state.showing_level_complete {
        return;
    }

    const FADE_IN: f32 = 0.3;
    const HOLD: f32 = 1.5;
    const FADE_OUT: f32 = 0.4;
    const TOTAL: f32 = FADE_IN + HOLD + FADE_OUT; // 2.2 s

    build_state.level_complete_timer += time.delta_secs();
    let t = build_state.level_complete_timer;

    if t >= TOTAL {
        build_state.showing_level_complete = false;
        if testplay.is_some() {
            next_state.set(GameState::Editor);
        } else {
            let next = score.round + 1;
            score.round = next;
            if next >= sequence.entries.len() {
                next_state.set(GameState::Stats);
            } else {
                next_state.set(GameState::Playing);
            }
        }
        return;
    }

    let alpha = if t < FADE_IN {
        t / FADE_IN
    } else if t < FADE_IN + HOLD {
        1.0
    } else {
        1.0 - (t - FADE_IN - HOLD) / FADE_OUT
    };

    let cam_y = camera_query
        .single()
        .map(|t| t.translation.y)
        .unwrap_or(0.0);
    for (mut color, mut transform) in &mut overlay_query {
        let c = color.0.to_srgba();
        color.0 = Color::srgba(c.red, c.green, c.blue, alpha);
        transform.translation.y = cam_y + 40.0;
    }
}
