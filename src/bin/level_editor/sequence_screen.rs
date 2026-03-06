use bevy::ecs::message::MessageReader;
use bevy::input::keyboard::{Key, KeyboardInput};
use bevy::input::ButtonState;
use bevy::prelude::*;
use bevy::text::TextBounds;

use crate::file_io;
use crate::state::{EditorScreen, SeqInput, SequenceEditorState};

// ── Colors ────────────────────────────────────────────────────────────────────
const NORMAL_COLOR: Color = Color::srgb(0.6, 0.6, 0.6);
const CURSOR_COLOR: Color = Color::srgb(0.2, 0.8, 0.9);
const GRABBED_COLOR: Color = Color::srgb(0.9, 0.7, 0.2);
const STATUS_COLOR: Color = Color::srgb(0.9, 0.9, 0.5);
const HINT_COLOR: Color = Color::srgb(0.45, 0.45, 0.45);
const INPUT_COLOR: Color = Color::srgb(0.4, 0.9, 0.5);

// ── Components ────────────────────────────────────────────────────────────────
#[derive(Component)]
pub struct SequenceEntity;

#[derive(Component)]
pub struct SeqRow(pub usize);

#[derive(Component)]
pub struct SeqStatusText;

#[derive(Component)]
pub struct SeqInputText;

// ── Plugin ────────────────────────────────────────────────────────────────────
pub struct SequenceScreenPlugin;

impl Plugin for SequenceScreenPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(EditorScreen::Sequence), setup_sequence_screen)
            .add_systems(OnExit(EditorScreen::Sequence), cleanup_sequence_screen)
            .add_systems(
                Update,
                (
                    sequence_keyboard,
                    sequence_update_rows,
                    sequence_tick_status,
                )
                    .chain()
                    .run_if(in_state(EditorScreen::Sequence)),
            );
    }
}

fn setup_sequence_screen(
    mut commands: Commands,
    mut state: ResMut<SequenceEditorState>,
    windows: Query<&Window>,
) {
    // Compute a safe wrap width with 16 px margin on each side.
    let viewport_w = windows.single().map(|w| w.width()).unwrap_or(1024.0);
    let hint_wrap_w = (viewport_w - 32.0).max(400.0);

    // Load entries the first time (guards against Bevy OnEnter/Startup ordering)
    if state.entries.is_empty() {
        let entries = file_io::load_sequence();
        let count = entries.len();
        state.entries = entries;
        state.blueprints = vec![None; count];
        for i in 0..count {
            let path = state.entries[i].clone();
            state.blueprints[i] = file_io::load_level_name(&path);
        }
    }
    // Header: app name (muted) + screen name (accent)
    commands.spawn((
        SequenceEntity,
        Text2d::new("TOWER STACKER"),
        TextFont { font_size: 11.0, ..default() },
        TextColor(HINT_COLOR),
        Transform::from_xyz(0.0, 352.0, 0.5),
    ));
    commands.spawn((
        SequenceEntity,
        Text2d::new("SEQUENCE EDITOR"),
        TextFont { font_size: 26.0, ..default() },
        TextColor(CURSOR_COLOR),
        Transform::from_xyz(0.0, 326.0, 0.5),
    ));

    // Controls hint — wrap so it stays inside the window on any desktop resolution.
    commands.spawn((
        SequenceEntity,
        Text2d::new(
            "↑/↓ Navigate  |  Enter Grab/Drop  |  Space/E Open  |  N New  |  A Add Path\n\
             Delete Remove  |  S Save  |  Q/Esc Quit",
        ),
        TextFont { font_size: 13.0, ..default() },
        TextColor(HINT_COLOR),
        TextBounds::new_horizontal(hint_wrap_w),
        TextLayout::new(Justify::Center, LineBreak::WordBoundary),
        Transform::from_xyz(0.0, -340.0, 0.5),
    ));

    // Status text
    commands.spawn((
        SequenceEntity,
        SeqStatusText,
        Text2d::new(state.status_msg.clone()),
        TextFont { font_size: 15.0, ..default() },
        TextColor(STATUS_COLOR),
        Transform::from_xyz(0.0, -310.0, 0.5),
    ));

    // Text-input line
    commands.spawn((
        SequenceEntity,
        SeqInputText,
        Text2d::new(""),
        TextFont { font_size: 15.0, ..default() },
        TextColor(INPUT_COLOR),
        Transform::from_xyz(0.0, -280.0, 0.5),
    ));

    // Row entities — spawn one per entry (at least 1 placeholder)
    let count = state.entries.len().max(1);
    for i in 0..count {
        commands.spawn((
            SequenceEntity,
            SeqRow(i),
            Text2d::new(""),
            TextFont { font_size: 16.0, ..default() },
            TextColor(NORMAL_COLOR),
            Transform::from_xyz(0.0, row_y(i), 0.5),
        ));
    }
}

fn cleanup_sequence_screen(
    mut commands: Commands,
    query: Query<Entity, With<SequenceEntity>>,
) {
    for e in &query {
        commands.entity(e).despawn();
    }
}

fn row_y(index: usize) -> f32 {
    290.0 - index as f32 * 22.0
}

// ── Row update ─────────────────────────────────────────────────────────────────

fn sequence_update_rows(
    state: Res<SequenceEditorState>,
    mut row_query: Query<(&SeqRow, &mut Text2d, &mut TextColor, &mut Transform)>,
    mut input_query: Query<(&mut Text2d, &mut TextColor), (With<SeqInputText>, Without<SeqRow>)>,
    mut status_query: Query<
        (&mut Text2d, &mut TextColor),
        (With<SeqStatusText>, Without<SeqRow>, Without<SeqInputText>),
    >,
) {
    for (row, mut text, mut color, mut transform) in &mut row_query {
        let i = row.0;
        if i >= state.entries.len() {
            text.0 = String::new();
            continue;
        }

        let path = &state.entries[i];
        let name_part = state
            .blueprints
            .get(i)
            .and_then(|n| n.as_ref())
            .map(|n| format!(" \"{n}\""))
            .unwrap_or_default();

        let prefix = if state.grabbed == Some(i) {
            "~ "
        } else if state.cursor == i {
            "> "
        } else {
            "  "
        };

        text.0 = format!("{prefix}{i}. {path}{name_part}");

        *color = if state.grabbed == Some(i) {
            TextColor(GRABBED_COLOR)
        } else if state.cursor == i {
            TextColor(CURSOR_COLOR)
        } else {
            TextColor(NORMAL_COLOR)
        };

        transform.translation.y = row_y(i);
    }

    // Input line
    if let Ok((mut text, mut color)) = input_query.single_mut() {
        match &state.text_input {
            Some(SeqInput::AddPath { buf }) => {
                text.0 = format!("Add path: {buf}_");
                *color = TextColor(INPUT_COLOR);
            }
            None => {
                text.0 = String::new();
            }
        }
    }

    // Status
    if let Ok((mut text, _color)) = status_query.single_mut() {
        text.0 = state.status_msg.clone();
    }
}

fn sequence_tick_status(mut state: ResMut<SequenceEditorState>, time: Res<Time>) {
    if state.status_timer > 0.0 {
        state.status_timer -= time.delta_secs();
        if state.status_timer <= 0.0 {
            state.status_timer = 0.0;
            state.status_msg = String::new();
        }
    }
}

// ── Keyboard ───────────────────────────────────────────────────────────────────

fn sequence_keyboard(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut key_events: MessageReader<KeyboardInput>,
    mut state: ResMut<SequenceEditorState>,
    mut next_screen: ResMut<NextState<EditorScreen>>,
    mut canvas_state: ResMut<crate::state::CanvasState>,
    mut commands: Commands,
    row_query: Query<Entity, With<SeqRow>>,
) {
    // ── Text input mode ──────────────────────────────────────────────────────
    if state.text_input.is_some() {
        for ev in key_events.read() {
            if ev.state != ButtonState::Pressed {
                continue;
            }
            match &ev.logical_key {
                Key::Escape => {
                    state.text_input = None;
                }
                Key::Enter => {
                    if let Some(SeqInput::AddPath { buf }) =
                        state.text_input.take()
                    {
                        let path = if buf.is_empty() {
                            return;
                        } else {
                            buf
                        };
                        let path = if path.ends_with(".json") {
                            path
                        } else {
                            format!("{path}.json")
                        };
                        let idx = state.cursor + 1;
                        state.entries.insert(idx, path);
                        state.blueprints.insert(idx, None);
                        state.cursor = idx;
                        // Eagerly load name
                        let name = file_io::load_level_name(&state.entries[idx]);
                        state.blueprints[idx] = name;
                        rebuild_rows(&mut commands, &row_query, state.entries.len());
                        state.status_msg = "Path added".to_string();
                        state.status_timer = 3.0;
                    }
                }
                Key::Backspace => {
                    if let Some(SeqInput::AddPath { buf }) = &mut state.text_input {
                        buf.pop();
                    }
                }
                Key::Character(s) => {
                    if let Some(SeqInput::AddPath { buf }) = &mut state.text_input {
                        for ch in s.chars() {
                            if !ch.is_control() {
                                buf.push(ch);
                            }
                        }
                    }
                }
                _ => {}
            }
        }
        return;
    }

    key_events.clear();

    let len = state.entries.len();

    // ── Q / Escape: quit ────────────────────────────────────────────────────
    if keyboard.just_pressed(KeyCode::KeyQ) || keyboard.just_pressed(KeyCode::Escape) {
        std::process::exit(0);
    }

    // ── Navigation ───────────────────────────────────────────────────────────
    if keyboard.just_pressed(KeyCode::ArrowUp) {
        if let Some(grabbed) = state.grabbed {
            // Reorder grabbed entry upward
            if grabbed > 0 {
                state.entries.swap(grabbed, grabbed - 1);
                state.blueprints.swap(grabbed, grabbed - 1);
                state.grabbed = Some(grabbed - 1);
                state.cursor = grabbed - 1;
            }
        } else if state.cursor > 0 {
            state.cursor -= 1;
        }
    }

    if keyboard.just_pressed(KeyCode::ArrowDown) {
        if let Some(grabbed) = state.grabbed {
            if grabbed + 1 < len {
                state.entries.swap(grabbed, grabbed + 1);
                state.blueprints.swap(grabbed, grabbed + 1);
                state.grabbed = Some(grabbed + 1);
                state.cursor = grabbed + 1;
            }
        } else if len > 0 && state.cursor < len - 1 {
            state.cursor += 1;
        }
    }

    // ── Enter: toggle grab ───────────────────────────────────────────────────
    if keyboard.just_pressed(KeyCode::Enter) {
        if len == 0 {
            return;
        }
        state.grabbed = match state.grabbed {
            Some(_) => None,
            None => Some(state.cursor),
        };
    }

    // ── Space / E: open level in canvas ─────────────────────────────────────
    if keyboard.just_pressed(KeyCode::Space) || keyboard.just_pressed(KeyCode::KeyE) {
        if len == 0 {
            return;
        }
        let idx = state.cursor;
        let path = state.entries[idx].clone();

        let full = format!("levels/{}", path);
        match std::fs::read_to_string(&full) {
            Ok(s) => match serde_json::from_str::<single_button_game::blueprint::Blueprint>(&s) {
                Ok(bp) => {
                    *canvas_state = crate::state::CanvasState {
                        filepath: Some(path),
                        level_name: bp.level_name,
                        slots: bp.slots,
                        sequence_index: Some(idx),
                        ..Default::default()
                    };
                    next_screen.set(EditorScreen::Canvas);
                }
                Err(e) => {
                    state.status_msg = format!("Parse error: {e}");
                    state.status_timer = 5.0;
                }
            },
            Err(e) => {
                state.status_msg = format!("Cannot read levels/{path}: {e}");
                state.status_timer = 5.0;
            }
        }
    }

    // ── N: new blank level ───────────────────────────────────────────────────
    if keyboard.just_pressed(KeyCode::KeyN) {
        *canvas_state = crate::state::CanvasState {
            sequence_index: if len > 0 { Some(state.cursor) } else { None },
            ..Default::default()
        };
        next_screen.set(EditorScreen::Canvas);
    }

    // ── A: add existing path ─────────────────────────────────────────────────
    if keyboard.just_pressed(KeyCode::KeyA) {
        state.text_input = Some(SeqInput::AddPath { buf: String::new() });
    }

    // ── Delete: remove entry ─────────────────────────────────────────────────
    if keyboard.just_pressed(KeyCode::Delete) || keyboard.just_pressed(KeyCode::Backspace) {
        if len > 0 {
            let cursor = state.cursor;
            state.entries.remove(cursor);
            state.blueprints.remove(cursor);
            if state.cursor >= state.entries.len() && state.cursor > 0 {
                state.cursor -= 1;
            }
            state.grabbed = None;
            let new_len = state.entries.len();
            rebuild_rows(&mut commands, &row_query, new_len);
        }
    }

    // ── S: save sequence ─────────────────────────────────────────────────────
    if keyboard.just_pressed(KeyCode::KeyS) {
        match file_io::save_sequence(&state.entries) {
            Ok(_) => {
                state.status_msg = "sequence.json saved".to_string();
                state.status_timer = 3.0;
            }
            Err(e) => {
                state.status_msg = format!("Save error: {e}");
                state.status_timer = 5.0;
            }
        }
    }
}

/// Spawn or despawn row entities so the count matches `count`.
fn rebuild_rows(
    commands: &mut Commands,
    row_query: &Query<Entity, With<SeqRow>>,
    count: usize,
) {
    let current = row_query.iter().count();
    if count > current {
        for i in current..count {
            commands.spawn((
                SequenceEntity,
                SeqRow(i),
                Text2d::new(""),
                TextFont { font_size: 16.0, ..default() },
                TextColor(NORMAL_COLOR),
                Transform::from_xyz(0.0, row_y(i), 0.5),
            ));
        }
    } else if count < current {
        // Remove excess rows from the end
        let mut entities: Vec<Entity> = row_query.iter().collect();
        // Sort by despawn order: keep indices 0..count, remove rest
        // We can't easily get the SeqRow index here without another query,
        // so just despawn all and respawn the right number.
        for e in entities.drain(..) {
            commands.entity(e).despawn();
        }
        for i in 0..count {
            commands.spawn((
                SequenceEntity,
                SeqRow(i),
                Text2d::new(""),
                TextFont { font_size: 16.0, ..default() },
                TextColor(NORMAL_COLOR),
                Transform::from_xyz(0.0, row_y(i), 0.5),
            ));
        }
    }
}
