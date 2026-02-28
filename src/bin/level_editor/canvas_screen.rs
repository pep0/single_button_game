use bevy::ecs::message::MessageReader;
use bevy::input::keyboard::{Key, KeyboardInput};
use bevy::input::ButtonState;
use bevy::prelude::*;
use single_button_game::blueprint::BlockSlot;
use single_button_game::constants::{GROUND_Y, SLOT_MAX_WIDTH};

use crate::drag::{DragMode, DragState, HandlePosition};
use crate::file_io;
use crate::hit_test::{self, HitResult, HANDLE_HALF_SIZE};
use crate::state::{CanvasInput, CanvasState, EditorScreen, SequenceEditorState};

// ── Colors ────────────────────────────────────────────────────────────────────
const PREV_COLOR: Color = Color::srgba(1.0, 0.5, 0.1, 0.5);
const NEXT_COLOR: Color = Color::srgba(0.1, 0.8, 1.0, 0.5);
const EDIT_BLOCK_COLOR: Color = Color::srgb(0.4, 0.6, 0.9);
const EDIT_BLOCK_SELECTED: Color = Color::srgb(0.6, 0.8, 1.0);
const HANDLE_COLOR: Color = Color::srgb(1.0, 1.0, 1.0);
const HANDLE_HOVERED: Color = Color::srgb(1.0, 0.8, 0.2);
const PREVIEW_COLOR: Color = Color::srgba(0.4, 0.6, 0.9, 0.4);
const GROUND_COLOR: Color = Color::srgb(0.3, 0.3, 0.35);
const TEXT_COLOR: Color = Color::srgb(0.9, 0.9, 0.9);
const HINT_COLOR: Color = Color::srgb(0.45, 0.45, 0.45);
const STATUS_COLOR: Color = Color::srgb(0.9, 0.9, 0.5);
const INPUT_COLOR: Color = Color::srgb(0.4, 0.9, 0.5);

const GROUND_WIDTH: f32 = 1000.0;

fn snap(v: f32, grid: f32) -> f32 {
    (v / grid).round() * grid
}
const HANDLE_SIZE: f32 = 8.0;
const OVERLAY_GAP: f32 = 50.0;

// ── Components ────────────────────────────────────────────────────────────────
#[derive(Component)]
pub struct CanvasEntity;

#[derive(Component)]
struct GridLine;

#[derive(Component)]
pub struct EditBlock {
    #[allow(dead_code)]
    pub slot_index: usize,
}

#[derive(Component)]
pub struct ResizeHandle {
    pub position: HandlePosition,
    pub slot_index: usize,
}

#[derive(Component)]
pub struct OverlayBlock;

#[derive(Component)]
pub struct PreviewBlock;

#[derive(Component)]
pub struct CanvasHudText;

#[derive(Component)]
pub struct CanvasStatusText;

#[derive(Component)]
pub struct CanvasInputText;

#[derive(Component)]
pub struct ModalDimmer;

// ── Plugin ────────────────────────────────────────────────────────────────────
pub struct CanvasScreenPlugin;

impl Plugin for CanvasScreenPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DragState>()
            .add_systems(OnEnter(EditorScreen::Canvas), setup_canvas)
            .add_systems(OnExit(EditorScreen::Canvas), cleanup_canvas)
            .add_systems(
                Update,
                (
                    canvas_keyboard,
                    canvas_sync_grid,
                    canvas_mouse_input,
                    canvas_sync_to_ecs,
                    canvas_hover_system,
                    canvas_update_hud,
                    canvas_tick_status,
                )
                    .chain()
                    .run_if(in_state(EditorScreen::Canvas)),
            );
    }
}

// ── Setup ─────────────────────────────────────────────────────────────────────

fn setup_canvas(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    state: Res<CanvasState>,
    mut camera_query: Query<&mut Transform, With<Camera2d>>,
) {
    // Reset camera to origin so each level opens centred
    if let Ok(mut cam) = camera_query.single_mut() {
        cam.translation = Vec3::ZERO;
    }
    // Ground line
    commands.spawn((
        CanvasEntity,
        Mesh2d(meshes.add(Rectangle::new(GROUND_WIDTH, 4.0))),
        MeshMaterial2d(materials.add(ColorMaterial::from_color(GROUND_COLOR))),
        Transform::from_xyz(0.0, GROUND_Y, 0.0),
    ));

    // HUD text — top-left corner
    commands.spawn((
        CanvasEntity,
        CanvasHudText,
        Text2d::new(""),
        TextFont { font_size: 16.0, ..default() },
        TextColor(TEXT_COLOR),
        Transform::from_xyz(-520.0, 350.0, 0.5),
    ));

    // Controls hint
    commands.spawn((
        CanvasEntity,
        Text2d::new(
            "Drag empty=create  Drag block=move  Drag handle=resize  Del=delete  RMB=pan  G=grid\n\
             S=save  F2=rename  Esc=back to sequence",
        ),
        TextFont { font_size: 13.0, ..default() },
        TextColor(HINT_COLOR),
        Transform::from_xyz(0.0, -340.0, 0.5),
    ));

    // Status line
    commands.spawn((
        CanvasEntity,
        CanvasStatusText,
        Text2d::new(""),
        TextFont { font_size: 15.0, ..default() },
        TextColor(STATUS_COLOR),
        Transform::from_xyz(0.0, -315.0, 0.5),
    ));

    // Input text (hidden until active)
    commands.spawn((
        CanvasEntity,
        CanvasInputText,
        Text2d::new(""),
        TextFont { font_size: 18.0, ..default() },
        TextColor(INPUT_COLOR),
        Transform::from_xyz(0.0, 20.0, 9.5),
    ));

    // Modal dimmer (hidden until active)
    commands.spawn((
        CanvasEntity,
        ModalDimmer,
        Mesh2d(meshes.add(Rectangle::new(1200.0, 900.0))),
        MeshMaterial2d(materials.add(ColorMaterial::from_color(Color::NONE))),
        Transform::from_xyz(0.0, 0.0, 9.0),
        Visibility::Hidden,
    ));

    // Compute bounding box of current level (fall back to GROUND_Y if empty)
    let current_bottom = state.slots.iter()
        .map(|s| s.y - s.height / 2.0)
        .fold(GROUND_Y, f32::min);
    let current_top = state.slots.iter()
        .map(|s| s.y + s.height / 2.0)
        .fold(GROUND_Y, f32::max);

    // Prev level: align its top edge to (current_bottom - OVERLAY_GAP)
    let prev_y_offset = if state.prev_slots.is_empty() {
        0.0
    } else {
        let prev_top = state.prev_slots.iter()
            .map(|s| s.y + s.height / 2.0)
            .fold(f32::NEG_INFINITY, f32::max);
        current_bottom - OVERLAY_GAP - prev_top
    };

    // Next level: align its bottom edge to (current_top + OVERLAY_GAP)
    let next_y_offset = if state.next_slots.is_empty() {
        0.0
    } else {
        let next_bottom = state.next_slots.iter()
            .map(|s| s.y - s.height / 2.0)
            .fold(f32::INFINITY, f32::min);
        current_top + OVERLAY_GAP - next_bottom
    };

    // Prev-level overlays — positioned just below current level
    for slot in &state.prev_slots {
        commands.spawn((
            CanvasEntity,
            OverlayBlock,
            Mesh2d(meshes.add(Rectangle::new(slot.width, slot.height))),
            MeshMaterial2d(materials.add(ColorMaterial::from_color(PREV_COLOR))),
            Transform::from_xyz(slot.x, slot.y + prev_y_offset, 0.1),
        ));
    }

    // Next-level overlays — positioned just above current level
    for slot in &state.next_slots {
        commands.spawn((
            CanvasEntity,
            OverlayBlock,
            Mesh2d(meshes.add(Rectangle::new(slot.width, slot.height))),
            MeshMaterial2d(materials.add(ColorMaterial::from_color(NEXT_COLOR))),
            Transform::from_xyz(slot.x, slot.y + next_y_offset, 0.1),
        ));
    }

    // Initial edit blocks
    for (i, slot) in state.slots.iter().enumerate() {
        let selected = state.selected_block == Some(i);
        spawn_edit_block(&mut commands, &mut meshes, &mut materials, slot, i, selected);
    }
}

pub fn spawn_edit_block(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    slot: &BlockSlot,
    slot_index: usize,
    selected: bool,
) {
    let color = if selected { EDIT_BLOCK_SELECTED } else { EDIT_BLOCK_COLOR };

    let parent = commands
        .spawn((
            CanvasEntity,
            EditBlock { slot_index },
            Mesh2d(meshes.add(Rectangle::new(slot.width, slot.height))),
            MeshMaterial2d(materials.add(ColorMaterial::from_color(color))),
            Transform::from_xyz(slot.x, slot.y, 0.5),
        ))
        .id();

    // Block number label — child of block, centered
    commands.spawn((
        CanvasEntity,
        Text2d::new(format!("{}", slot_index + 1)),
        TextFont { font_size: 14.0, ..default() },
        TextColor(Color::srgb(0.0, 0.0, 0.0)),
        Transform::from_xyz(0.0, 0.0, 1.5),
        ChildOf(parent),
    ));

    // Resize handles — 8 children at edges/corners, only for selected block
    if selected {
        for handle_pos in HandlePosition::all() {
            let (sx, sy) = handle_pos.offset_signs();
            let hx = sx * slot.width / 2.0;
            let hy = sy * slot.height / 2.0;
            commands.spawn((
                CanvasEntity,
                ResizeHandle {
                    position: handle_pos,
                    slot_index,
                },
                Mesh2d(meshes.add(Rectangle::new(HANDLE_SIZE, HANDLE_SIZE))),
                MeshMaterial2d(materials.add(ColorMaterial::from_color(HANDLE_COLOR))),
                Transform::from_xyz(hx, hy, 1.0),
                ChildOf(parent),
            ));
        }
    }
}

fn cleanup_canvas(
    mut commands: Commands,
    query: Query<Entity, With<CanvasEntity>>,
    mut drag: ResMut<DragState>,
    mut canvas: ResMut<CanvasState>,
) {
    for e in &query {
        commands.entity(e).despawn();
    }
    drag.mode = DragMode::Idle;
    drag.pan = None;
    // Force grid rebuild on next canvas entry so lines are respawned if grid is active
    canvas.grid_needs_rebuild = true;
}

// ── Grid lines ────────────────────────────────────────────────────────────────

fn canvas_sync_grid(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut canvas: ResMut<CanvasState>,
    grid_query: Query<Entity, With<GridLine>>,
) {
    if !canvas.grid_needs_rebuild {
        return;
    }
    canvas.grid_needs_rebuild = false;

    // Despawn old lines
    for e in &grid_query {
        commands.entity(e).despawn();
    }

    let g = canvas.snap_grid;
    if g <= 0.0 {
        return;
    }

    let x_min = -800.0_f32;
    let x_max =  800.0_f32;
    let y_min = -500.0_f32;
    let y_max = 1500.0_f32;
    let w = x_max - x_min;
    let h = y_max - y_min;
    let color = Color::srgba(0.5, 0.5, 0.7, 0.2);

    // Vertical lines
    let mut x = x_min;
    while x <= x_max + 0.001 {
        commands.spawn((
            CanvasEntity, GridLine,
            Mesh2d(meshes.add(Rectangle::new(1.0, h))),
            MeshMaterial2d(materials.add(ColorMaterial::from_color(color))),
            Transform::from_xyz(x, (y_min + y_max) / 2.0, -0.2),
        ));
        x += g;
    }
    // Horizontal lines
    let mut y = y_min;
    while y <= y_max + 0.001 {
        commands.spawn((
            CanvasEntity, GridLine,
            Mesh2d(meshes.add(Rectangle::new(w, 1.0))),
            MeshMaterial2d(materials.add(ColorMaterial::from_color(color))),
            Transform::from_xyz((x_min + x_max) / 2.0, y, -0.2),
        ));
        y += g;
    }
}

// ── Sync ECS ──────────────────────────────────────────────────────────────────
// Always rebuilds block entities when sync is needed (simple & correct for an editor).

fn canvas_sync_to_ecs(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut canvas: ResMut<CanvasState>,
    edit_query: Query<Entity, With<EditBlock>>,
) {
    if !canvas.sync_needed {
        return;
    }
    canvas.sync_needed = false;

    // Despawn all existing block entities (handles + labels are children → auto-despawned)
    for e in &edit_query {
        commands.entity(e).despawn();
    }

    // Respawn from slots
    for (i, slot) in canvas.slots.iter().enumerate() {
        let selected = canvas.selected_block == Some(i);
        spawn_edit_block(&mut commands, &mut meshes, &mut materials, slot, i, selected);
    }
}

// ── Mouse input ───────────────────────────────────────────────────────────────

fn canvas_mouse_input(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mouse: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    mut canvas: ResMut<CanvasState>,
    mut drag: ResMut<DragState>,
    handle_query: Query<(&ResizeHandle, &GlobalTransform)>,
    mut preview_query: Query<&mut Transform, With<PreviewBlock>>,
    mut cam_transform_query: Query<&mut Transform, (With<Camera2d>, Without<PreviewBlock>)>,
) {
    if canvas.text_input.is_some() || canvas.showing_unsaved_warning {
        return;
    }

    let Ok(window) = windows.single() else { return };
    let Some(cursor_pos) = window.cursor_position() else { return };
    let Ok((camera, cam_gt)) = camera_query.single() else { return };
    let Ok(world_pos) = camera.viewport_to_world_2d(cam_gt, cursor_pos) else { return };
    // Snap to grid if enabled
    let world_pos = if canvas.snap_grid > 0.0 {
        Vec2::new(snap(world_pos.x, canvas.snap_grid), snap(world_pos.y, canvas.snap_grid))
    } else {
        world_pos
    };

    // Collect handle world positions for hit test
    let handle_positions: Vec<(usize, HandlePosition, Vec2)> = handle_query
        .iter()
        .map(|(h, gt)| (h.slot_index, h.position, gt.translation().truncate()))
        .collect();

    // ── Mouse press ──────────────────────────────────────────────────────────
    if mouse.just_pressed(MouseButton::Left) {
        let hit = hit_test::find_hit(
            world_pos,
            &canvas.slots,
            canvas.selected_block,
            &handle_positions,
        );

        match hit {
            HitResult::Handle { slot_index, position } => {
                canvas.selected_block = Some(slot_index);
                let slot = &canvas.slots[slot_index];
                let opp = position.opposite();
                let (sx, sy) = opp.offset_signs();
                let anchor = Vec2::new(
                    slot.x + sx * slot.width / 2.0,
                    slot.y + sy * slot.height / 2.0,
                );
                drag.mode = DragMode::ResizingBlock { slot_index, handle: position, anchor };
                canvas.sync_needed = true;
            }
            HitResult::Block { slot_index } => {
                let slot = &canvas.slots[slot_index];
                let grab_offset = Vec2::new(world_pos.x - slot.x, world_pos.y - slot.y);
                canvas.selected_block = Some(slot_index);
                drag.mode = DragMode::MovingBlock { slot_index, grab_offset };
                canvas.sync_needed = true;
            }
            HitResult::Empty => {
                let prev_sel = canvas.selected_block;
                canvas.selected_block = None;
                let preview_entity = commands
                    .spawn((
                        CanvasEntity,
                        PreviewBlock,
                        Mesh2d(meshes.add(Rectangle::new(1.0, 1.0))),
                        MeshMaterial2d(materials.add(ColorMaterial::from_color(PREVIEW_COLOR))),
                        Transform::from_xyz(world_pos.x, world_pos.y, 0.8)
                            .with_scale(Vec3::new(1.0, 1.0, 1.0)),
                    ))
                    .id();
                drag.mode = DragMode::DrawingNew { start: world_pos, preview_entity };
                if prev_sel.is_some() {
                    canvas.sync_needed = true;
                }
            }
        }
    }

    // ── Mouse held ───────────────────────────────────────────────────────────
    if mouse.pressed(MouseButton::Left) {
        match &drag.mode {
            DragMode::MovingBlock { slot_index, grab_offset } => {
                let si = *slot_index;
                let off = *grab_offset;
                if let Some(slot) = canvas.slots.get_mut(si) {
                    slot.x = world_pos.x - off.x;
                    slot.y = world_pos.y - off.y;
                    canvas.dirty = true;
                    canvas.sync_needed = true;
                }
            }
            DragMode::ResizingBlock { slot_index, handle, anchor } => {
                let si = *slot_index;
                let anch = *anchor;
                let hp = *handle;
                if let Some(slot) = canvas.slots.get_mut(si) {
                    let (min_x, max_x) = if hp.controls_x() {
                        (world_pos.x.min(anch.x), world_pos.x.max(anch.x))
                    } else {
                        (slot.x - slot.width / 2.0, slot.x + slot.width / 2.0)
                    };
                    let (min_y, max_y) = if hp.controls_y() {
                        (world_pos.y.min(anch.y), world_pos.y.max(anch.y))
                    } else {
                        (slot.y - slot.height / 2.0, slot.y + slot.height / 2.0)
                    };
                    slot.width = (max_x - min_x).max(10.0).min(SLOT_MAX_WIDTH);
                    slot.height = (max_y - min_y).max(10.0);
                    slot.x = (min_x + max_x) / 2.0;
                    slot.y = (min_y + max_y) / 2.0;
                    canvas.dirty = true;
                    canvas.sync_needed = true;
                }
            }
            DragMode::DrawingNew { start, preview_entity } => {
                let start = *start;
                let pe = *preview_entity;
                let min_x = world_pos.x.min(start.x);
                let max_x = world_pos.x.max(start.x);
                let min_y = world_pos.y.min(start.y);
                let max_y = world_pos.y.max(start.y);
                let w = (max_x - min_x).max(1.0).min(SLOT_MAX_WIDTH);
                let h = (max_y - min_y).max(1.0);
                if let Ok(mut t) = preview_query.get_mut(pe) {
                    t.translation = Vec3::new((min_x + max_x) / 2.0, (min_y + max_y) / 2.0, 0.8);
                    t.scale = Vec3::new(w, h, 1.0);
                }
            }
            DragMode::Idle => {}
        }
    }

    // ── Right-click pan ───────────────────────────────────────────────────────
    if mouse.just_pressed(MouseButton::Right) {
        if let Ok(cam) = cam_transform_query.single() {
            drag.pan = Some((cursor_pos, cam.translation));
        }
    }
    if mouse.pressed(MouseButton::Right) {
        if let Some((start_cursor, start_cam)) = drag.pan {
            if let Ok(mut cam) = cam_transform_query.single_mut() {
                let delta = cursor_pos - start_cursor;
                cam.translation = start_cam + Vec3::new(-delta.x, delta.y, 0.0);
            }
        }
    }
    if mouse.just_released(MouseButton::Right) {
        drag.pan = None;
    }

    // ── Mouse release ─────────────────────────────────────────────────────────
    if mouse.just_released(MouseButton::Left) {
        if let DragMode::DrawingNew { start, preview_entity } = &drag.mode {
            let start = *start;
            let pe = *preview_entity;
            let min_x = world_pos.x.min(start.x);
            let max_x = world_pos.x.max(start.x);
            let min_y = world_pos.y.min(start.y);
            let max_y = world_pos.y.max(start.y);
            let w = (max_x - min_x).min(SLOT_MAX_WIDTH);
            let h = max_y - min_y;

            commands.entity(pe).despawn();

            if w >= 10.0 && h >= 10.0 {
                let new_index = canvas.slots.len();
                canvas.slots.push(BlockSlot {
                    x: (min_x + max_x) / 2.0,
                    y: (min_y + max_y) / 2.0,
                    width: w,
                    height: h,
                });
                canvas.selected_block = Some(new_index);
                canvas.dirty = true;
                canvas.sync_needed = true;
            }
        }
        drag.mode = DragMode::Idle;
    }
}

// ── Hover highlight ───────────────────────────────────────────────────────────

fn canvas_hover_system(
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    handle_query: Query<(&ResizeHandle, &GlobalTransform, &MeshMaterial2d<ColorMaterial>)>,
    canvas: Res<CanvasState>,
    mut mat_assets: ResMut<Assets<ColorMaterial>>,
) {
    let Ok(window) = windows.single() else { return };
    let Some(cursor_pos) = window.cursor_position() else { return };
    let Ok((camera, cam_gt)) = camera_query.single() else { return };
    let Ok(world_pos) = camera.viewport_to_world_2d(cam_gt, cursor_pos) else { return };

    for (handle, gt, mat_handle) in &handle_query {
        let hp = gt.translation().truncate();
        let dist = (world_pos - hp).abs();
        let hovered = canvas.selected_block == Some(handle.slot_index)
            && dist.x <= HANDLE_HALF_SIZE + 2.0
            && dist.y <= HANDLE_HALF_SIZE + 2.0;
        if let Some(mat) = mat_assets.get_mut(&mat_handle.0) {
            mat.color = if hovered { HANDLE_HOVERED } else { HANDLE_COLOR };
        }
    }
}

// ── HUD update ────────────────────────────────────────────────────────────────

fn canvas_update_hud(
    canvas: Res<CanvasState>,
    mut hud_query: Query<
        &mut Text2d,
        (
            With<CanvasHudText>,
            Without<CanvasStatusText>,
            Without<CanvasInputText>,
        ),
    >,
    mut status_query: Query<
        &mut Text2d,
        (
            With<CanvasStatusText>,
            Without<CanvasHudText>,
            Without<CanvasInputText>,
        ),
    >,
    mut input_query: Query<
        &mut Text2d,
        (
            With<CanvasInputText>,
            Without<CanvasHudText>,
            Without<CanvasStatusText>,
        ),
    >,
    mut modal_query: Query<
        (&mut Visibility, &MeshMaterial2d<ColorMaterial>),
        With<ModalDimmer>,
    >,
    mut mat_assets: ResMut<Assets<ColorMaterial>>,
) {
    // HUD line
    if let Ok(mut text) = hud_query.single_mut() {
        let dirty_mark = if canvas.dirty { " *" } else { "" };
        let path_str = canvas.filepath.as_deref().unwrap_or("<unsaved>");
        let name_str = canvas
            .level_name
            .as_deref()
            .map(|n| format!(" \"{n}\""))
            .unwrap_or_default();
        let grid_label = if canvas.snap_grid > 0.0 {
            format!("  |  Grid: {}px", canvas.snap_grid as u32)
        } else {
            "  |  Grid: off".to_string()
        };
        text.0 = format!(
            "{path_str}{name_str}{dirty_mark}  |  {} blocks{grid_label}",
            canvas.slots.len()
        );
    }

    // Status
    if let Ok(mut text) = status_query.single_mut() {
        text.0 = canvas.status_msg.clone();
    }

    // Modal dimmer
    let modal_active = canvas.text_input.is_some() || canvas.showing_unsaved_warning;
    if let Ok((mut vis, mat_handle)) = modal_query.single_mut() {
        *vis = if modal_active {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
        if let Some(mat) = mat_assets.get_mut(&mat_handle.0) {
            mat.color = if modal_active {
                Color::srgba(0.0, 0.0, 0.0, 0.7)
            } else {
                Color::NONE
            };
        }
    }

    // Input text
    if let Ok(mut text) = input_query.single_mut() {
        match &canvas.text_input {
            Some(CanvasInput::FilenamePrompt { buf }) => {
                text.0 = format!(
                    "Save as: {buf}_\n(relative to levels/, e.g. custom/my_level.json)"
                );
            }
            Some(CanvasInput::LevelName { buf }) => {
                text.0 = format!("Level name: {buf}_");
            }
            None => {
                if canvas.showing_unsaved_warning {
                    text.0 =
                        "Unsaved changes! Press Esc again to discard, or S to save.".to_string();
                } else {
                    text.0 = String::new();
                }
            }
        }
    }
}

fn canvas_tick_status(mut canvas: ResMut<CanvasState>, time: Res<Time>) {
    if canvas.status_timer > 0.0 {
        canvas.status_timer -= time.delta_secs();
        if canvas.status_timer <= 0.0 {
            canvas.status_timer = 0.0;
            canvas.status_msg = String::new();
        }
    }
}

// ── Keyboard ───────────────────────────────────────────────────────────────────

fn canvas_keyboard(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut key_events: MessageReader<KeyboardInput>,
    mut canvas: ResMut<CanvasState>,
    mut next_screen: ResMut<NextState<EditorScreen>>,
    mut seq_state: ResMut<SequenceEditorState>,
) {
    // ── Text input active ────────────────────────────────────────────────────
    if canvas.text_input.is_some() {
        for ev in key_events.read() {
            if ev.state != ButtonState::Pressed {
                continue;
            }
            match &ev.logical_key {
                Key::Escape => {
                    canvas.text_input = None;
                }
                Key::Enter => match canvas.text_input.take() {
                    Some(CanvasInput::FilenamePrompt { buf }) => {
                        let path = if buf.is_empty() {
                            "custom/untitled.json".to_string()
                        } else if buf.ends_with(".json") {
                            buf
                        } else {
                            format!("{buf}.json")
                        };
                        perform_save(&mut canvas, &path, &mut seq_state);
                    }
                    Some(CanvasInput::LevelName { buf }) => {
                        canvas.level_name = if buf.is_empty() { None } else { Some(buf) };
                        canvas.dirty = true;
                    }
                    None => {}
                },
                Key::Backspace => match &mut canvas.text_input {
                    Some(CanvasInput::FilenamePrompt { buf })
                    | Some(CanvasInput::LevelName { buf }) => {
                        buf.pop();
                    }
                    None => {}
                },
                Key::Character(s) => match &mut canvas.text_input {
                    Some(CanvasInput::FilenamePrompt { buf }) => {
                        for ch in s.chars() {
                            if ch.is_alphanumeric() || matches!(ch, '-' | '_' | '.' | '/') {
                                buf.push(ch);
                            }
                        }
                    }
                    Some(CanvasInput::LevelName { buf }) => {
                        for ch in s.chars() {
                            if !ch.is_control() {
                                buf.push(ch);
                            }
                        }
                    }
                    None => {}
                },
                _ => {}
            }
        }
        return;
    }

    // ── Unsaved warning ───────────────────────────────────────────────────────
    if canvas.showing_unsaved_warning {
        key_events.clear();
        if keyboard.just_pressed(KeyCode::Escape) {
            canvas.showing_unsaved_warning = false;
            canvas.dirty = false;
            next_screen.set(EditorScreen::Sequence);
        } else if keyboard.just_pressed(KeyCode::KeyS) {
            canvas.showing_unsaved_warning = false;
            if let Some(path) = canvas.filepath.clone() {
                perform_save(&mut canvas, &path, &mut seq_state);
            } else {
                canvas.text_input = Some(CanvasInput::FilenamePrompt { buf: String::new() });
            }
        }
        return;
    }

    key_events.clear();

    // ── Delete selected block ────────────────────────────────────────────────
    if keyboard.just_pressed(KeyCode::Delete) || keyboard.just_pressed(KeyCode::Backspace) {
        if let Some(sel) = canvas.selected_block {
            if sel < canvas.slots.len() {
                canvas.slots.remove(sel);
                canvas.selected_block = if canvas.slots.is_empty() {
                    None
                } else {
                    Some(sel.saturating_sub(1))
                };
                canvas.dirty = true;
                canvas.sync_needed = true;
            }
        }
    }

    // ── S: save ──────────────────────────────────────────────────────────────
    if keyboard.just_pressed(KeyCode::KeyS) {
        if let Some(path) = canvas.filepath.clone() {
            perform_save(&mut canvas, &path, &mut seq_state);
        } else {
            canvas.text_input = Some(CanvasInput::FilenamePrompt { buf: String::new() });
        }
    }

    // ── F2: rename level ──────────────────────────────────────────────────────
    if keyboard.just_pressed(KeyCode::F2) {
        canvas.text_input = Some(CanvasInput::LevelName {
            buf: canvas.level_name.clone().unwrap_or_default(),
        });
    }

    // ── G: cycle snap grid ────────────────────────────────────────────────────
    if keyboard.just_pressed(KeyCode::KeyG) {
        canvas.snap_grid = match canvas.snap_grid as u32 {
            0  => 5.0,
            5  => 10.0,
            10 => 20.0,
            _  => 0.0,
        };
        canvas.grid_needs_rebuild = true;
    }

    // ── Escape: back to sequence ───────────────────────────────────────────────
    if keyboard.just_pressed(KeyCode::Escape) {
        if canvas.dirty {
            canvas.showing_unsaved_warning = true;
        } else {
            next_screen.set(EditorScreen::Sequence);
        }
    }
}

// ── Save helper ───────────────────────────────────────────────────────────────

fn perform_save(
    canvas: &mut CanvasState,
    path: &str,
    seq_state: &mut SequenceEditorState,
) {
    match file_io::save_blueprint(
        &canvas.slots,
        canvas.level_name.as_deref(),
        path,
    ) {
        Ok(()) => {
            let was_new = canvas.filepath.is_none();
            canvas.filepath = Some(path.to_string());
            canvas.dirty = false;
            canvas.status_msg = format!("Saved → levels/{path}");
            canvas.status_timer = 4.0;

            if was_new {
                // New level: add it after current sequence cursor
                let idx = seq_state.cursor + 1;
                seq_state.entries.insert(idx, path.to_string());
                seq_state.blueprints.insert(idx, canvas.level_name.clone());
                canvas.sequence_index = Some(idx);
                seq_state.cursor = idx;
            } else if let Some(si) = canvas.sequence_index {
                // Update cached name in sequence
                if let Some(cached) = seq_state.blueprints.get_mut(si) {
                    *cached = canvas.level_name.clone();
                }
            }
        }
        Err(e) => {
            canvas.status_msg = format!("Save error: {e}");
            canvas.status_timer = 5.0;
        }
    }
}
