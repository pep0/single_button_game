use bevy::prelude::*;

use crate::constants::*;
use super::components::*;
use super::resources::*;

/// Updates slot X (arrow keys) and width oscillation; also keeps slot_y above the tallest block.
/// Returns early without updating anything when the slot is locked (Space held).
pub fn editor_slot_oscillation(
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut slot_state: ResMut<EditorSlotState>,
    build_state: Res<EditorBuildState>,
    mut slot_query: Query<&mut Transform, With<EditorSlotIndicator>>,
    block_query: Query<(&Transform, &EditorBlock), Without<EditorSlotIndicator>>,
    falling_query: Query<(&Transform, &FallingBlock), Without<EditorSlotIndicator>>,
) {
    // Keep slot_y above the highest placed or falling block top edge.
    // This is always computed, even when locked, so slot_y reflects reality.
    let max_top = block_query
        .iter()
        .map(|(t, b)| t.translation.y + b.height / 2.0)
        .chain(
            falling_query
                .iter()
                .map(|(_, f)| f.target_y + f.height / 2.0),
        )
        .fold(GROUND_Y - GROUND_HALF_HEIGHT, f32::max);
    slot_state.slot_y = max_top + SPAWN_HEIGHT_ABOVE;

    // When locked (Space held), don't move the slot or oscillate width.
    if slot_state.locked_width.is_some() {
        return;
    }

    // Don't move slot while user is typing a filename.
    if build_state.filename_input.is_some() {
        return;
    }

    let dt = time.delta_secs();

    // Arrow keys move slot horizontally (left/right only — down is handled separately).
    if keyboard.pressed(KeyCode::ArrowLeft) {
        slot_state.slot_x = (slot_state.slot_x - EDITOR_SLOT_MOVE_SPEED * dt).max(-370.0);
    }
    if keyboard.pressed(KeyCode::ArrowRight) {
        slot_state.slot_x = (slot_state.slot_x + EDITOR_SLOT_MOVE_SPEED * dt).min(370.0);
    }

    // Width oscillates on a sine wave, identical to playing mode.
    slot_state.phase += slot_state.speed * dt;
    let t = (slot_state.phase.sin() + 1.0) / 2.0;
    slot_state.current_width = SLOT_MIN_WIDTH + t * (SLOT_MAX_WIDTH - SLOT_MIN_WIDTH);

    if let Ok(mut transform) = slot_query.single_mut() {
        transform.scale.x = slot_state.current_width;
        transform.translation.x = slot_state.slot_x;
        transform.translation.y = slot_state.slot_y;
    }
}

/// Handles Space: grow production rect, then release → spawn FallingBlock.
/// Also handles Arrow-Down: instant-place a block using last_block_height.
pub fn editor_production_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut slot_state: ResMut<EditorSlotState>,
    mut production: ResMut<EditorProductionState>,
    mut build_state: ResMut<EditorBuildState>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut prod_query: Query<(Entity, &mut Transform), With<EditorProductionRect>>,
    editor_blocks: Query<
        (&Transform, &EditorBlock),
        (Without<EditorProductionRect>, Without<EditorSlotIndicator>),
    >,
) {
    // All production input is blocked while typing a filename.
    if build_state.filename_input.is_some() {
        return;
    }

    let slot_x = slot_state.slot_x;
    let slot_y = slot_state.slot_y;
    let width = slot_state.locked_width.unwrap_or(slot_state.current_width);

    // ── Space pressed: lock width and start growing ──
    if keyboard.just_pressed(KeyCode::Space) && !production.is_producing {
        slot_state.locked_width = Some(slot_state.current_width);
        production.is_producing = true;
        production.current_height = 2.0;

        let mesh = meshes.add(Rectangle::new(1.0, 1.0));
        let mat = materials.add(ColorMaterial::from_color(PRODUCTION_COLOR));
        commands.spawn((
            EditorEntity,
            EditorProductionRect,
            Mesh2d(mesh),
            MeshMaterial2d(mat),
            Transform::from_xyz(slot_x, slot_y - production.current_height / 2.0, 0.5)
                .with_scale(Vec3::new(width, production.current_height, 1.0)),
        ));
    }

    // ── Space held: grow the rect ──
    if keyboard.pressed(KeyCode::Space) && production.is_producing {
        production.current_height =
            (production.current_height + GROW_SPEED * time.delta_secs()).min(MAX_HEIGHT);

        if let Ok((_entity, mut transform)) = prod_query.single_mut() {
            transform.scale.x = width;
            transform.scale.y = production.current_height;
            transform.translation.y = slot_y - production.current_height / 2.0;
            transform.translation.x = slot_x;
        }
    }

    // ── Space released: despawn rect, compute landing Y, spawn FallingBlock ──
    if keyboard.just_released(KeyCode::Space) && production.is_producing {
        production.is_producing = false;
        let produced_width = width;
        let produced_height = production.current_height;

        if let Ok((entity, _)) = prod_query.single_mut() {
            commands.entity(entity).despawn();
        }

        let landing_y = compute_landing_y(slot_x, produced_width, produced_height, &editor_blocks);

        // Spawn falling block starting at slot_y, falling to landing_y.
        let mesh = meshes.add(Rectangle::new(produced_width, produced_height));
        let mat = materials.add(ColorMaterial::from_color(EDITOR_BLOCK_COLOR));
        commands.spawn((
            EditorEntity,
            FallingBlock {
                target_y: landing_y,
                width: produced_width,
                height: produced_height,
            },
            Mesh2d(mesh),
            MeshMaterial2d(mat),
            Transform::from_xyz(slot_x, slot_y, 0.5),
        ));

        build_state.block_count += 1;
        build_state.last_block_height = produced_height;
        build_state.status_msg = String::new();

        // Unlock slot so oscillation resumes.
        slot_state.locked_width = None;
        production.current_height = 0.0;
    }

    // ── Arrow-Down: instant-place a block using last known height ──
    if keyboard.just_pressed(KeyCode::ArrowDown) && !production.is_producing {
        let instant_width = slot_state.current_width;
        let instant_height = build_state.last_block_height;

        let landing_y = compute_landing_y(slot_x, instant_width, instant_height, &editor_blocks);

        let mesh = meshes.add(Rectangle::new(instant_width, instant_height));
        let mat = materials.add(ColorMaterial::from_color(EDITOR_BLOCK_COLOR));
        commands.spawn((
            EditorEntity,
            FallingBlock {
                target_y: landing_y,
                width: instant_width,
                height: instant_height,
            },
            Mesh2d(mesh),
            MeshMaterial2d(mat),
            Transform::from_xyz(slot_x, slot_y, 0.5),
        ));

        build_state.block_count += 1;
        build_state.status_msg = String::new();
    }
}

/// Computes the Y coordinate at which a new block of `(produced_width × produced_height)`
/// dropped at `slot_x` would come to rest, accounting for already-placed blocks.
fn compute_landing_y(
    slot_x: f32,
    produced_width: f32,
    produced_height: f32,
    editor_blocks: &Query<
        (&Transform, &EditorBlock),
        (Without<EditorProductionRect>, Without<EditorSlotIndicator>),
    >,
) -> f32 {
    let half_h = produced_height / 2.0;
    let mut landing_y = GROUND_Y - GROUND_HALF_HEIGHT + half_h;

    for (block_t, block) in editor_blocks.iter() {
        let bx = block_t.translation.x;
        let by = block_t.translation.y;
        if (bx - slot_x).abs() < (block.width + produced_width) / 2.0 {
            let candidate = by + block.height / 2.0 + half_h;
            if candidate > landing_y {
                landing_y = candidate;
            }
        }
    }

    landing_y
}
