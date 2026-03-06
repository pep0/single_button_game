use avian2d::prelude::*;
use bevy::prelude::*;
use bevy::sprite::Anchor;

use crate::blueprint::{self, Blueprint};
use crate::constants::*;
use crate::editor::EditorTestPlay;
use crate::state::{GameState, LevelSequence, Score};
use super::components::*;
use super::resources::*;
use super::ui::hud_text;

pub fn setup_playing(
    mut commands: Commands,
    score: Res<Score>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    testplay: Option<Res<EditorTestPlay>>,
    existing_blueprint: Option<Res<Blueprint>>,
    sequence: Res<LevelSequence>,
    mut next_state: ResMut<NextState<GameState>>,
    mut physics_time: ResMut<Time<Physics>>,
) {
    physics_time.unpause();

    // In test-play mode the Blueprint was pre-inserted by the editor's P handler; clone it.
    // In normal play, load from the sequence file.
    let blueprint = if testplay.is_some() {
        match existing_blueprint.map(|r| r.clone()) {
            Some(bp) => bp,
            None => {
                next_state.set(GameState::Menu);
                return;
            }
        }
    } else {
        let path = match sequence.entries.get(score.round) {
            Some(p) => p.clone(),
            None => {
                next_state.set(GameState::Menu);
                return;
            }
        };
        match blueprint::load_blueprint(&path) {
            Ok(bp) => bp,
            Err(e) => {
                bevy::log::error!("Failed to load blueprint {path}: {e}");
                next_state.set(GameState::Menu);
                return;
            }
        }
    };

    let num_slots = blueprint.slots.len();

    // Spawn ground as static rigid body (sized mesh, no scale — avian scales colliders with transform)
    let ground_mesh = meshes.add(Rectangle::new(GROUND_WIDTH, GROUND_HALF_HEIGHT * 2.0));
    let ground_material = materials.add(ColorMaterial::from_color(GROUND_COLOR));
    commands.spawn((
        PlayingEntity,
        RigidBody::Static,
        Collider::rectangle(GROUND_WIDTH, GROUND_HALF_HEIGHT * 2.0),
        CollisionEventsEnabled,
        Mesh2d(ground_mesh),
        MeshMaterial2d(ground_material),
        Transform::from_xyz(0.0, GROUND_Y - GROUND_HALF_HEIGHT * 2.0, 0.0),
    ));

    // Spawn ghost outlines for blueprint — unique material handle per ghost so highlights work correctly
    for (i, slot) in blueprint.slots.iter().enumerate() {
        let ghost_mesh = meshes.add(Rectangle::new(1.0, 1.0));
        let color = if i == 0 { GHOST_NEXT_COLOR } else { GHOST_COLOR };
        let mat = materials.add(ColorMaterial::from_color(color));
        commands.spawn((
            PlayingEntity,
            GhostBlock(i),
            Mesh2d(ghost_mesh),
            MeshMaterial2d(mat),
            Transform::from_xyz(slot.x, slot.y, 0.1)
                .with_scale(Vec3::new(slot.width, slot.height, 1.0)),
        ));
    }

    // Compute slot indicator Y: above the first (lowest) block; slot_oscillation updates it each frame
    let slot_y = blueprint.slots[0].y + blueprint.slots[0].height / 2.0 + SPAWN_HEIGHT_ABOVE;

    // Tint slot indicator warm orange as levels progress (level 1 = pale, level 10 = warm)
    let level_t = (score.round as f32 / 9.0).clamp(0.0, 1.0);
    let slot_color = {
        // SLOT_COLOR (0.88, 0.76, 0.48) → warm orange (0.95, 0.55, 0.22)
        let r = 0.88 + level_t * (0.95 - 0.88);
        let g = 0.76 + level_t * (0.55 - 0.76);
        let b = 0.48 + level_t * (0.22 - 0.48);
        Color::srgb(r, g, b)
    };

    // Spawn slot indicator
    let slot_mesh = meshes.add(Rectangle::new(1.0, 1.0));
    let slot_material = materials.add(ColorMaterial::from_color(slot_color));
    commands.spawn((
        PlayingEntity,
        SlotIndicator,
        Mesh2d(slot_mesh),
        MeshMaterial2d(slot_material),
        Transform::from_xyz(blueprint.slots[0].x, slot_y, 1.0)
            .with_scale(Vec3::new(SLOT_MIN_WIDTH, SLOT_HEIGHT, 1.0)),
    ));

    // HUD text (world-space, will be repositioned by camera_follow)
    commands.spawn((
        PlayingEntity,
        HudText,
        Text2d::new(hud_text(score.round + 1, &blueprint, 1, num_slots)),
        TextFont {
            font_size: 20.0,
            ..default()
        },
        TextColor(TEXT_COLOR),
        Transform::from_xyz(0.0, GROUND_Y - 60.0, 1.0),
    ));

    // Heart icons (lives display) — positioned each frame by update_hearts
    const MAX_LIVES: usize = 3;
    for i in 0..MAX_LIVES {
        let heart_mesh = meshes.add(Rectangle::new(18.0, 14.0));
        let alive = i < score.lives;
        let color = if alive {
            Color::srgb(0.82, 0.30, 0.28)
        } else {
            Color::srgb(0.28, 0.26, 0.30)
        };
        let mat = materials.add(ColorMaterial::from_color(color));
        commands.spawn((
            PlayingEntity,
            HeartIcon(i),
            Mesh2d(heart_mesh),
            MeshMaterial2d(mat),
            Transform::from_xyz(-230.0 + i as f32 * 22.0, 255.0, 1.5),
        ));
    }

    // "Evaluating..." indicator (hidden until settle phase)
    commands.spawn((
        PlayingEntity,
        EvaluatingText,
        Text2d::new(""),
        TextFont { font_size: 22.0, ..default() },
        TextColor(Color::srgba(0.9, 0.9, 0.3, 0.0)),
        Transform::from_xyz(0.0, 0.0, 2.0),
    ));

    // Score bar — background, fill, and threshold line
    const BAR_X: f32 = 234.0;
    const BAR_MAX_H: f32 = 160.0;
    const BAR_W: f32 = 10.0;
    let bg_mat = materials.add(ColorMaterial::from_color(Color::srgba(0.15, 0.15, 0.18, 0.80)));
    commands.spawn((
        PlayingEntity,
        ScoreBarBg,
        Mesh2d(meshes.add(Rectangle::new(BAR_W, BAR_MAX_H))),
        MeshMaterial2d(bg_mat),
        Transform::from_xyz(BAR_X, 0.0, 1.5),
    ));
    let fill_mat = materials.add(ColorMaterial::from_color(Color::srgb(0.85, 0.72, 0.22)));
    commands.spawn((
        PlayingEntity,
        ScoreBarFill,
        Mesh2d(meshes.add(Rectangle::new(BAR_W, 1.0))),
        MeshMaterial2d(fill_mat),
        Transform::from_xyz(BAR_X, 0.0, 1.6),
    ));
    // Threshold line at the very top of the background
    let thresh_mat = materials.add(ColorMaterial::from_color(Color::srgba(0.95, 0.95, 0.95, 0.70)));
    commands.spawn((
        PlayingEntity,
        ScoreBarThreshold,
        Mesh2d(meshes.add(Rectangle::new(BAR_W + 4.0, 2.0))),
        MeshMaterial2d(thresh_mat),
        Transform::from_xyz(BAR_X, 0.0, 1.7),
    ));

    // Streak badge — updated each frame by update_streak_text
    commands.spawn((
        PlayingEntity,
        StreakText,
        Text2d::new(""),
        TextFont { font_size: 14.0, ..default() },
        TextColor(Color::srgba(1.0, 0.82, 0.20, 0.0)),
        Transform::from_xyz(BAR_X, 0.0, 1.8),
    ));

    // Score display (top-right) — drop shadow first (lower z), then main text
    commands.spawn((
        PlayingEntity,
        ScoreTextShadow,
        Text2d::new("Score: 0"),
        TextFont { font_size: 22.0, ..default() },
        TextColor(Color::srgba(0.0, 0.0, 0.0, 0.75)),
        Transform::from_xyz(0.0, 0.0, 1.9),
        Anchor::TOP_RIGHT,
    ));
    commands.spawn((
        PlayingEntity,
        ScoreText,
        Text2d::new("Score: 0"),
        TextFont { font_size: 22.0, ..default() },
        TextColor(Color::srgba(1.0, 1.0, 1.0, 0.95)),
        Transform::from_xyz(0.0, 0.0, 2.0),
        Anchor::TOP_RIGHT,
    ));

    // Init resources — insert/replace Blueprint so Failed/settle can read it.
    let slot_speed = SLOT_SPEED * (1.0 + score.round as f32 * 0.06);
    commands.insert_resource(SlotState { speed: slot_speed, ..SlotState::default() });
    commands.insert_resource(ProductionState::default());
    commands.insert_resource(BuildState::default());
    commands.insert_resource(ProducedDimensions::default());
    commands.insert_resource(ScreenShake::default());
    commands.insert_resource(LevelScoreBar { accumulated: 0, target: num_slots as i32, threshold_reached: false, streak: 0 });
    commands.insert_resource(blueprint);
}

pub fn cleanup_playing(
    mut commands: Commands,
    query: Query<Entity, With<PlayingEntity>>,
    mut physics_time: ResMut<Time<Physics>>,
) {
    physics_time.pause();
    for entity in &query {
        commands.entity(entity).despawn();
    }
    // Keep Blueprint and ProducedDimensions alive for Failed to read
    commands.remove_resource::<BuildState>();
    commands.remove_resource::<SlotState>();
    commands.remove_resource::<ProductionState>();
    commands.remove_resource::<LevelScoreBar>();
}
