use avian2d::prelude::*;
use bevy::prelude::*;

use crate::blueprint::{self, Blueprint};
use crate::constants::*;
use crate::editor::EditorTestPlay;
use crate::state::{GameState, LevelSequence, Score, TowerModeActive};
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
    tower_mode: Option<Res<TowerModeActive>>,
    frozen_query: Query<(&Transform, &FrozenTowerBlock)>,
) {
    physics_time.unpause();
    // In test-play mode the Blueprint was pre-inserted by the editor's P handler; clone it.
    // In normal play, load from the sequence file.
    let mut blueprint = if testplay.is_some() {
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

    // In tower mode, offset the blueprint upward so it starts above the frozen stack
    if tower_mode.is_some() {
        let frozen_top = frozen_query
            .iter()
            .map(|(t, f)| t.translation.y + f.height / 2.0)
            .fold(GROUND_Y, f32::max);

        if frozen_top > GROUND_Y + 1.0 {
            let blueprint_bottom = blueprint
                .slots
                .iter()
                .map(|s| s.y - s.height / 2.0)
                .fold(f32::INFINITY, f32::min);
            let y_offset = frozen_top - blueprint_bottom + 10.0;
            for slot in blueprint.slots.iter_mut() {
                slot.y += y_offset;
            }
        }
    }

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

    // Spawn slot indicator
    let slot_mesh = meshes.add(Rectangle::new(1.0, 1.0));
    let slot_material = materials.add(ColorMaterial::from_color(SLOT_COLOR));
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
            Color::srgb(0.9, 0.2, 0.2)
        } else {
            Color::srgb(0.25, 0.25, 0.25)
        };
        let mat = materials.add(ColorMaterial::from_color(color));
        commands.spawn((
            PlayingEntity,
            HeartIcon(i),
            Mesh2d(heart_mesh),
            MeshMaterial2d(mat),
            Transform::from_xyz(-360.0 + i as f32 * 22.0, 255.0, 1.5),
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

    // Init resources — insert/replace Blueprint so Failed/settle can read it.
    commands.insert_resource(SlotState::default());
    commands.insert_resource(ProductionState::default());
    commands.insert_resource(BuildState::default());
    commands.insert_resource(ProducedDimensions::default());
    commands.insert_resource(ScreenShake::default());
    commands.insert_resource(blueprint);
}

pub fn cleanup_playing(
    mut commands: Commands,
    query: Query<Entity, With<PlayingEntity>>,
    mut physics_time: ResMut<Time<Physics>>,
    tower_mode: Option<Res<TowerModeActive>>,
    tower_block_query: Query<(Entity, &TowerBlockDims), With<TowerBlock>>,
) {
    physics_time.pause();
    for entity in &query {
        commands.entity(entity).despawn();
    }
    // In tower mode, convert dropped blocks into frozen static colliders for the next level
    if tower_mode.is_some() {
        for (entity, dims) in &tower_block_query {
            commands.entity(entity)
                .remove::<TowerBlock>()
                .remove::<TowerBlockDims>()
                .remove::<BlockSettleTimer>()
                .remove::<RigidBody>()
                .insert(RigidBody::Static)
                .insert(FrozenTowerBlock { height: dims.height });
        }
    }
    // Keep Blueprint and ProducedDimensions alive for Failed to read
    commands.remove_resource::<BuildState>();
    commands.remove_resource::<SlotState>();
    commands.remove_resource::<ProductionState>();
}
