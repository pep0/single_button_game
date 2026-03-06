use bevy::prelude::*;
use bevy::text::TextBounds;

use crate::blueprint::Blueprint;
use crate::constants::*;
use crate::editor::EditorTestPlay;
use crate::state::{GameState, LevelSequence, Score};
use super::components::*;
use super::resources::*;

pub fn hud_text(level_num: usize, blueprint: &Blueprint, block: usize, total: usize) -> String {
    match &blueprint.level_name {
        Some(name) => format!("Level {level_num} - {name}    Block: {block}/{total}"),
        None       => format!("Level {level_num}    Block: {block}/{total}"),
    }
}

pub fn camera_follow(
    time: Res<Time>,
    blueprint: Res<Blueprint>,
    mut shake: ResMut<ScreenShake>,
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
    let view_center = (GROUND_Y + target_top) / 2.0;

    // Only move camera up if the structure gets tall enough
    let target_y = if target_top - GROUND_Y > 500.0 {
        view_center
    } else {
        0.0
    };

    // Lerp the logical (non-shaken) base position
    shake.base_camera_y += (target_y - shake.base_camera_y) * 0.05;

    // Compute shake offset (linear intensity so small trauma is still visible)
    let t = time.elapsed_secs();
    let intensity = shake.trauma;
    let shake_x = intensity * 9.0 * (t * 11.0).sin();
    let shake_y = intensity * 7.0 * (t * 13.0).sin();

    if let Ok(mut cam_t) = camera_query.single_mut() {
        cam_t.translation.x = shake_x;
        cam_t.translation.y = shake.base_camera_y + shake_y;
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
                mat.color = Color::srgba(0.4, 0.5, 0.65, 0.10);
            } else {
                mat.color = GHOST_COLOR;
            }
        }
    }
}

pub fn update_hud(
    build_state: Res<BuildState>,
    blueprint: Res<Blueprint>,
    score: Res<Score>,
    mut hud_query: Query<&mut Text2d, With<HudText>>,
) {
    if let Ok(mut text) = hud_query.single_mut() {
        let block_num = (build_state.current_index + 1).min(blueprint.slots.len());
        text.0 = hud_text(score.round + 1, &blueprint, block_num, blueprint.slots.len());
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
    windows: Query<&Window>,
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
        let viewport_w = windows.single().map(|w| w.width()).unwrap_or(512.0);
        // Scale font down on narrow viewports; minimum 28pt to stay readable.
        let overlay_font_size = (52.0 * (viewport_w / 512.0)).clamp(28.0, 52.0);
        let overlay_wrap_w = (viewport_w - 32.0).max(200.0); // 16 px inset each side
        let level_num = score.round + 1;
        let is_last = level_num >= sequence.entries.len();
        let star = if is_last { "  *" } else { "" };
        let msg = match &blueprint.level_name {
            Some(name) => format!("Level {level_num} - {name}  Complete!{star}"),
            None       => format!("Level {level_num}  Complete!{star}"),
        };
        commands.spawn((
            PlayingEntity,
            LevelCompleteOverlay,
            Text2d::new(msg),
            TextFont {
                font_size: overlay_font_size,
                ..default()
            },
            TextColor(Color::srgba(0.42, 0.88, 0.62, 0.0)),
            TextBounds::new_horizontal(overlay_wrap_w),
            TextLayout::new(Justify::Center, LineBreak::WordBoundary),
            Transform::from_xyz(0.0, cam_y + 40.0, 10.0),
        ));

        build_state.showing_level_complete = true;
        build_state.level_complete_timer = 0.0;
    }
}

pub fn animate_level_complete(
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    touches: Res<Touches>,
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
    const HOLD: f32 = 0.8;
    const FADE_OUT: f32 = 0.4;
    const TOTAL: f32 = FADE_IN + HOLD + FADE_OUT; // 1.5 s

    build_state.level_complete_timer += time.delta_secs();
    let mut t = build_state.level_complete_timer;

    // Space or tap skips the remaining wait
    if keyboard.just_pressed(KeyCode::Space) || touches.any_just_pressed() {
        t = TOTAL;
        build_state.level_complete_timer = TOTAL;
    }

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

pub fn update_hearts(
    shake: Res<ScreenShake>,
    score: Res<Score>,
    windows: Query<&Window>,
    mut heart_query: Query<(&HeartIcon, &mut Transform, &MeshMaterial2d<ColorMaterial>)>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let half_w = windows.single().map(|w| w.width() / 2.0).unwrap_or(256.0);
    let half_h = windows.single().map(|w| w.height() / 2.0).unwrap_or(384.0);
    let heart_y = shake.base_camera_y + (half_h - 20.0).min(255.0);
    for (heart, mut transform, mat_handle) in &mut heart_query {
        transform.translation.x = -(half_w - 16.0) + heart.0 as f32 * 22.0;
        transform.translation.y = heart_y;
        if let Some(mat) = materials.get_mut(&mat_handle.0) {
            mat.color = if heart.0 < score.lives {
                Color::srgb(0.82, 0.30, 0.28)
            } else {
                Color::srgb(0.28, 0.26, 0.30)
            };
        }
    }
}

const BAR_MAX_H: f32 = 160.0;

pub fn update_score_bar(
    shake: Res<ScreenShake>,
    windows: Query<&Window>,
    mut level_score: ResMut<LevelScoreBar>,
    mut bg_query: Query<&mut Transform, With<ScoreBarBg>>,
    mut fill_query: Query<
        (&mut Transform, &MeshMaterial2d<ColorMaterial>),
        (With<ScoreBarFill>, Without<ScoreBarBg>),
    >,
    mut thresh_query: Query<
        &mut Transform,
        (With<ScoreBarThreshold>, Without<ScoreBarBg>, Without<ScoreBarFill>),
    >,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let bar_x = windows.single().map(|w| w.width() / 2.0 - 16.0).unwrap_or(234.0);
    let center_y = shake.base_camera_y;
    let bg_bottom_y = center_y - BAR_MAX_H / 2.0;

    // Reposition background
    if let Ok(mut t) = bg_query.single_mut() {
        t.translation.x = bar_x;
        t.translation.y = center_y;
    }

    // Reposition threshold line at top of background
    if let Ok(mut t) = thresh_query.single_mut() {
        t.translation.x = bar_x;
        t.translation.y = bg_bottom_y + BAR_MAX_H;
    }

    // Detect threshold crossing (once per level)
    if !level_score.threshold_reached && level_score.accumulated >= level_score.target {
        level_score.threshold_reached = true;
        // Burst at top of the filled bar
        let burst_y = bg_bottom_y + BAR_MAX_H;
        super::particles::spawn_celebration_burst(
            &mut commands,
            &mut meshes,
            &mut materials,
            bar_x,
            burst_y,
            level_score.accumulated as u32,
        );
    }

    // Scale and reposition fill
    let ratio = (level_score.accumulated as f32 / level_score.target as f32).clamp(0.0, 1.0);
    let fill_h = (ratio * BAR_MAX_H).max(1.0);
    if let Ok((mut t, mat_handle)) = fill_query.single_mut() {
        t.translation.x = bar_x;
        t.translation.y = bg_bottom_y + fill_h / 2.0;
        t.scale = Vec3::new(1.0, fill_h, 1.0);

        if let Some(mat) = materials.get_mut(&mat_handle.0) {
            mat.color = if level_score.threshold_reached {
                Color::srgb(0.38, 0.88, 0.55)
            } else {
                Color::srgb(0.85, 0.72, 0.22)
            };
        }
    }
}

pub fn update_streak_text(
    time: Res<Time>,
    shake: Res<ScreenShake>,
    windows: Query<&Window>,
    level_score: Res<LevelScoreBar>,
    mut query: Query<(&mut Text2d, &mut Transform, &mut TextColor), With<StreakText>>,
) {
    let Ok((mut text, mut transform, mut color)) = query.single_mut() else { return };
    let bar_x = windows.single().map(|w| w.width() / 2.0 - 16.0).unwrap_or(234.0);

    if level_score.streak >= 2 {
        text.0 = format!("x{} STREAK", level_score.streak);
        transform.translation.x = bar_x;
        transform.translation.y = shake.base_camera_y - BAR_MAX_H / 2.0 - 18.0;
        let pulse = (time.elapsed_secs() * 4.0).sin() * 0.15 + 0.85;
        color.0 = Color::srgba(1.0, 0.82, 0.20, pulse);
    } else {
        text.0.clear();
        color.0 = Color::srgba(1.0, 0.82, 0.20, 0.0);
    }
}

/// Score display in the top-right corner of the viewport.
pub fn update_score_text(
    shake: Res<ScreenShake>,
    level_score: Res<LevelScoreBar>,
    windows: Query<&Window>,
    mut text_query: Query<(&mut Text2d, &mut Transform), (With<ScoreText>, Without<ScoreTextShadow>)>,
    mut shadow_query: Query<(&mut Text2d, &mut Transform), (With<ScoreTextShadow>, Without<ScoreText>)>,
) {
    let half_w = windows.single().map(|w| w.width() / 2.0).unwrap_or(256.0);
    let half_h = windows.single().map(|w| w.height() / 2.0).unwrap_or(384.0);

    let label = format!("Score: {}/{}", level_score.accumulated, level_score.target);
    // Position anchor in top-right — with TOP_RIGHT anchor the transform is the top-right corner
    let text_x = half_w - 8.0;
    let text_y = shake.base_camera_y + (half_h - 8.0).min(252.0);

    if let Ok((mut text, mut transform)) = text_query.single_mut() {
        text.0 = label.clone();
        transform.translation.x = text_x;
        transform.translation.y = text_y;
        transform.translation.z = 2.0;
    }

    // Shadow offset by 1.5 px down-right
    if let Ok((mut text, mut transform)) = shadow_query.single_mut() {
        text.0 = label;
        transform.translation.x = text_x + 1.5;
        transform.translation.y = text_y - 1.5;
        transform.translation.z = 1.9;
    }
}

/// Pulsing "Evaluating..." text shown while the physics settle check is running.
pub fn update_evaluating_indicator(
    time: Res<Time>,
    build_state: Res<BuildState>,
    shake: Res<ScreenShake>,
    mut eval_query: Query<(&mut Text2d, &mut Transform, &mut TextColor), With<EvaluatingText>>,
) {
    let Ok((mut text, mut transform, mut color)) = eval_query.single_mut() else { return };

    if build_state.waiting_for_settle && !build_state.showing_popups {
        text.0 = "Evaluating...".to_string();
        transform.translation.x = 0.0;
        transform.translation.y = shake.base_camera_y - 80.0;
        transform.translation.z = 2.0;
        let pulse = (time.elapsed_secs() * 3.5).sin() * 0.25 + 0.75;
        color.0 = Color::srgba(0.92, 0.88, 0.42, pulse);
    } else {
        text.0.clear();
        color.0 = Color::srgba(0.92, 0.88, 0.42, 0.0);
    }
}
