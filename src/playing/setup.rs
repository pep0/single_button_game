use avian2d::prelude::*;
use bevy::prelude::*;

use crate::blueprint::{self, Blueprint};
use crate::constants::*;
use crate::editor::EditorTestPlay;
use crate::state::{GameState, LevelSequence, Score};
use super::components::*;
use super::resources::*;

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
        Mesh2d(ground_mesh),
        MeshMaterial2d(ground_material),
        Transform::from_xyz(0.0, GROUND_Y, 0.0),
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

    // Compute slot indicator Y: above the highest ghost block
    let max_ghost_y = blueprint
        .slots
        .iter()
        .map(|s| s.y + s.height / 2.0)
        .fold(f32::NEG_INFINITY, f32::max);
    let slot_y = max_ghost_y + SPAWN_HEIGHT_ABOVE;

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
        Text2d::new(format!("Level: {}    Block: 1/{}", blueprint.level_number, num_slots)),
        TextFont {
            font_size: 20.0,
            ..default()
        },
        TextColor(TEXT_COLOR),
        Transform::from_xyz(0.0, GROUND_Y - 60.0, 1.0),
    ));

    // Init resources — insert/replace Blueprint so Scoring/Failed/settle can read it.
    commands.insert_resource(SlotState::default());
    commands.insert_resource(ProductionState::default());
    commands.insert_resource(BuildState::default());
    commands.insert_resource(ProducedDimensions::default());
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
    // Keep Blueprint and ProducedDimensions alive for Scoring/Failed to read
    commands.remove_resource::<BuildState>();
    commands.remove_resource::<SlotState>();
    commands.remove_resource::<ProductionState>();
}
