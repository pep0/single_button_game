use bevy::prelude::*;

use crate::constants::*;
use super::components::*;
use super::resources::*;

pub fn setup_editor(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut camera_query: Query<&mut Transform, With<Camera2d>>,
    snapshot_res: Option<Res<EditorSnapshot>>,
) {
    // Clean up any resources left over from a test-play session.
    commands.remove_resource::<EditorTestPlay>();
    commands.remove_resource::<crate::blueprint::Blueprint>();
    commands.remove_resource::<crate::playing::ProducedDimensions>();

    if let Ok(mut cam_t) = camera_query.single_mut() {
        cam_t.translation.y = 0.0;
    }

    // Ground — visual only, no physics collider needed in editor
    let ground_mesh = meshes.add(Rectangle::new(GROUND_WIDTH, GROUND_HALF_HEIGHT * 2.0));
    let ground_mat = materials.add(ColorMaterial::from_color(GROUND_COLOR));
    commands.spawn((
        EditorEntity,
        Mesh2d(ground_mesh),
        MeshMaterial2d(ground_mat),
        Transform::from_xyz(0.0, GROUND_Y, 0.0),
    ));

    let slot_y = GROUND_Y + GROUND_HALF_HEIGHT + SPAWN_HEIGHT_ABOVE;

    // Slot indicator
    let slot_mesh = meshes.add(Rectangle::new(1.0, 1.0));
    let slot_mat = materials.add(ColorMaterial::from_color(SLOT_COLOR));
    commands.spawn((
        EditorEntity,
        EditorSlotIndicator,
        Mesh2d(slot_mesh),
        MeshMaterial2d(slot_mat),
        Transform::from_xyz(0.0, slot_y, 1.0)
            .with_scale(Vec3::new(SLOT_MIN_WIDTH, SLOT_HEIGHT, 1.0)),
    ));

    // HUD text
    commands.spawn((
        EditorEntity,
        EditorHudText,
        Text2d::new(
            "Level Editor  |  Blocks: 0  |  Arrows: move   Space/\u{2193}: place   S: save   R: reset   P: test   Esc: menu",
        ),
        TextFont {
            font_size: 16.0,
            ..default()
        },
        TextColor(TEXT_COLOR),
        Transform::from_xyz(0.0, GROUND_Y - 60.0, 1.0),
    ));

    let mut build_state = EditorBuildState::default();

    // Restore blocks from the snapshot (if returning from test-play).
    if let Some(snapshot) = snapshot_res {
        for (pos, width, height) in &snapshot.blocks {
            let mesh = meshes.add(Rectangle::new(*width, *height));
            let mat = materials.add(ColorMaterial::from_color(EDITOR_BLOCK_COLOR));
            commands.spawn((
                EditorEntity,
                EditorBlock { width: *width, height: *height },
                Mesh2d(mesh),
                MeshMaterial2d(mat),
                Transform::from_translation(*pos),
            ));
            build_state.block_count += 1;
        }
        commands.remove_resource::<EditorSnapshot>();
    }

    commands.insert_resource(EditorSlotState::default());
    commands.insert_resource(build_state);
    commands.insert_resource(EditorProductionState::default());
}

pub fn cleanup_editor(mut commands: Commands, query: Query<Entity, With<EditorEntity>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
    commands.remove_resource::<EditorSlotState>();
    commands.remove_resource::<EditorBuildState>();
    commands.remove_resource::<EditorProductionState>();
}
